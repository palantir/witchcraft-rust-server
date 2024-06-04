// Copyright 2022 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use conjure_error::Error;
use refreshable::{RefreshHandle, Refreshable};
use serde::de::DeserializeOwned;
use serde_encrypted_value::{Key, ReadOnly};
use sha2::digest::Output;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{fs, io};
use tokio::runtime::Handle;
use tokio::{task, time};
use witchcraft_log::{error, info};

const RELOAD_INTERVAL: Duration = Duration::from_secs(3);
const INSTALL_YML: &str = "var/conf/install.yml";
const RUNTIME_YML: &str = "var/conf/runtime.yml";
const ENCRYPTED_CONFIG_VALUE_KEY: &str = "var/conf/encrypted-config-value.key";

pub fn load_install<T>() -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let key = load_key()?;
    let bytes = load_file(INSTALL_YML)?;
    parse(&bytes, key.as_ref()).0
}

pub fn load_runtime<T>(
    runtime: &Handle,
    config_ok: &Arc<AtomicBool>,
) -> Result<Refreshable<T, Error>, Error>
where
    T: DeserializeOwned + PartialEq + 'static + Sync + Send,
{
    let key = load_key()?;
    let bytes = load_file(RUNTIME_YML)?;
    let (value, files) = parse(&bytes, key.as_ref());
    let value = value?;

    let (refreshable, handle) = Refreshable::new(value);

    runtime.spawn(runtime_reload(files, key, handle, config_ok.clone()));

    Ok(refreshable)
}

struct ConfigFiles {
    root_hash: Output<Sha256>,
    ok_files: HashMap<PathBuf, Output<Sha256>>,
    err_files: HashSet<PathBuf>,
}

impl ConfigFiles {
    fn up_to_date(&self, root_bytes: &[u8]) -> bool {
        if self.root_hash != Sha256::digest(root_bytes) {
            return false;
        }

        for (path, hash) in &self.ok_files {
            match fs::read(path) {
                Ok(bytes) => {
                    if *hash != Sha256::digest(&bytes) {
                        return false;
                    }
                }
                Err(_) => return false,
            }
        }

        for path in &self.err_files {
            if fs::read(path).is_ok() {
                return false;
            }
        }

        true
    }

    fn add(&mut self, path: &Path, r: &io::Result<Vec<u8>>) {
        match r {
            Ok(bytes) => {
                self.ok_files
                    .insert(path.to_path_buf(), Sha256::digest(bytes));
            }
            Err(_) => {
                self.err_files.insert(path.to_path_buf());
            }
        }
    }
}

fn load_key() -> Result<Option<Key<ReadOnly>>, Error> {
    Key::from_file(ENCRYPTED_CONFIG_VALUE_KEY).map_err(Error::internal_safe)
}

fn load_file(path: &str) -> Result<Vec<u8>, Error> {
    fs::read(path).map_err(|e| Error::internal_safe(e).with_safe_param("path", path))
}

fn parse<T>(raw: &[u8], key: Option<&Key<ReadOnly>>) -> (Result<T, Error>, ConfigFiles)
where
    T: DeserializeOwned,
{
    let mut files = ConfigFiles {
        root_hash: Sha256::digest(raw),
        ok_files: HashMap::new(),
        err_files: HashSet::new(),
    };
    let mut callback = |path: &Path, r: &io::Result<Vec<u8>>| files.add(path, r);

    let de = serde_yaml::Deserializer::from_slice(raw);
    let de = serde_encrypted_value::Deserializer::new(de, key);
    let de = serde_file_value::Deserializer::new(de, &mut callback);

    let value = T::deserialize(de).map_err(Error::internal);
    (value, files)
}

async fn runtime_reload<T>(
    mut files: ConfigFiles,
    key: Option<Key<ReadOnly>>,
    mut handle: RefreshHandle<T, Error>,
    config_ok: Arc<AtomicBool>,
) where
    T: DeserializeOwned + PartialEq + 'static + Sync + Send,
{
    loop {
        time::sleep(RELOAD_INTERVAL).await;

        // it's okay to use block_in_place here since we know this future is running as its own task.
        task::block_in_place(|| {
            let new_bytes = match fs::read(RUNTIME_YML) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!(
                        "error reading runtime config",
                        error: Error::internal_safe(e)
                    );
                    config_ok.store(false, Ordering::Relaxed);
                    return;
                }
            };

            if files.up_to_date(&new_bytes) {
                return;
            }

            let (value, new_files) = parse(&new_bytes, key.as_ref());
            files = new_files;
            let value = match value {
                Ok(value) => value,
                Err(e) => {
                    error!("error parsing runtime config", error: e);
                    config_ok.store(false, Ordering::Relaxed);
                    return;
                }
            };

            match handle.refresh(value) {
                Ok(()) => config_ok.store(true, Ordering::Relaxed),
                Err(errors) => {
                    for error in errors {
                        error!("error reloading runtime config", error: error);
                    }
                    config_ok.store(false, Ordering::Relaxed);
                }
            }

            info!("reloaded runtime config");
        });
    }
}
