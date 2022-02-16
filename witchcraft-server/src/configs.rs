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
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
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
    parse(&bytes, key.as_ref())
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
    let value = parse(&bytes, key.as_ref())?;

    let (refreshable, handle) = Refreshable::new(value);

    runtime.spawn(runtime_reload(bytes, key, handle, config_ok.clone()));

    Ok(refreshable)
}

fn load_key() -> Result<Option<Key<ReadOnly>>, Error> {
    Key::from_file(ENCRYPTED_CONFIG_VALUE_KEY).map_err(Error::internal_safe)
}

fn load_file(path: &str) -> Result<Vec<u8>, Error> {
    fs::read(path).map_err(|e| Error::internal_safe(e).with_safe_param("path", path))
}

fn parse<T>(raw: &[u8], key: Option<&Key<ReadOnly>>) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let de = serde_yaml::Deserializer::from_slice(raw);
    let de = serde_encrypted_value::Deserializer::new(de, key);

    T::deserialize(de).map_err(Error::internal)
}

async fn runtime_reload<T>(
    mut bytes: Vec<u8>,
    key: Option<Key<ReadOnly>>,
    mut handle: RefreshHandle<T, Error>,
    config_ok: Arc<AtomicBool>,
) where
    T: DeserializeOwned + PartialEq + 'static + Sync + Send,
{
    loop {
        time::sleep(RELOAD_INTERVAL).await;

        let new_bytes = match tokio::fs::read(RUNTIME_YML).await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!(
                    "error reading runtime config",
                    error: Error::internal_safe(e)
                );
                config_ok.store(false, Ordering::Relaxed);
                continue;
            }
        };

        if bytes == new_bytes {
            continue;
        }
        bytes = new_bytes;

        let value = match parse(&bytes, key.as_ref()) {
            Ok(value) => value,
            Err(e) => {
                error!("error parsing runtime config", error: e);
                config_ok.store(false, Ordering::Relaxed);
                continue;
            }
        };

        // it's okay to use block_in_place here since we know this future is running as its own task.
        match task::block_in_place(|| handle.refresh(value)) {
            Ok(()) => config_ok.store(true, Ordering::Relaxed),
            Err(errors) => {
                for error in errors {
                    error!("error reloading runtime config", error: error);
                }
                config_ok.store(false, Ordering::Relaxed);
            }
        }

        info!("reloaded runtime config");
    }
}
