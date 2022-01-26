// Copyright 2021 Palantir Technologies, Inc.
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
use serde_path_to_error::Track;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::{task, time};
use witchcraft_log::{error, info, warn};

pub fn load_install<T>() -> Result<ParsedConfig<T>, Error>
where
    T: DeserializeOwned,
{
    let dir = Path::new("var/conf");

    let key =
        Key::from_file(dir.join("encryped-config-value.key")).map_err(Error::internal_safe)?;

    let path = dir.join("install.yml");
    let bytes =
        fs::read(&path).map_err(|e| Error::internal_safe(e).with_safe_param("file", &path))?;

    parse(&path, &bytes, key.as_ref())
}

pub fn load_runtime<T>(runtime: &Runtime) -> Result<ParsedConfig<Refreshable<T, Error>>, Error>
where
    T: DeserializeOwned + PartialEq + 'static + Sync + Send,
{
    let dir = Path::new("var/conf");

    let key =
        Key::from_file(dir.join("encryped-config-value.key")).map_err(Error::internal_safe)?;

    let path = dir.join("runtime.yml");
    let bytes =
        fs::read(&path).map_err(|e| Error::internal_safe(e).with_safe_param("file", &path))?;

    let config = parse(&path, &bytes, key.as_ref())?;
    let (value, handle) = Refreshable::new(config.value);

    runtime.spawn(runtime_reload(path, bytes, key, handle));

    Ok(ParsedConfig {
        value,
        ignored: config.ignored,
    })
}

pub struct ParsedConfig<T> {
    pub value: T,
    pub ignored: IgnoredFields,
}

pub struct IgnoredFields {
    file: PathBuf,
    ignored: Vec<String>,
}

impl IgnoredFields {
    pub fn log(&self) {
        if !self.ignored.is_empty() {
            warn!("unknown field(s) set in configuration file", safe: { fields: self.ignored, file: self.file });
        }
    }
}

fn parse<T>(path: &Path, raw: &[u8], key: Option<&Key<ReadOnly>>) -> Result<ParsedConfig<T>, Error>
where
    T: DeserializeOwned,
{
    let de = serde_yaml::Deserializer::from_slice(raw);
    let de = serde_encrypted_value::Deserializer::new(de, key);
    let mut track = Track::new();
    let de = serde_path_to_error::Deserializer::new(de, &mut track);
    let mut ignored = vec![];

    let value = serde_ignored::deserialize(de, |path| ignored.push(path.to_string()))
        .map_err(|e| Error::internal(e).with_safe_param("path", track.path().to_string()))?;

    Ok(ParsedConfig {
        value,
        ignored: IgnoredFields {
            file: path.to_path_buf(),
            ignored,
        },
    })
}

// FIXME(sfackler) health check
async fn runtime_reload<T>(
    path: PathBuf,
    mut bytes: Vec<u8>,
    key: Option<Key<ReadOnly>>,
    mut handle: RefreshHandle<T, Error>,
) where
    T: DeserializeOwned + PartialEq + 'static + Sync + Send,
{
    loop {
        time::sleep(Duration::from_secs(3)).await;

        let new_bytes = match tokio::fs::read(&path).await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("error reading runtime config", safe: { file: path }, error: Error::internal_safe(e));
                continue;
            }
        };

        if bytes == new_bytes {
            continue;
        }
        bytes = new_bytes;

        let value = match parse(&path, &bytes, key.as_ref()) {
            Ok(value) => value,
            Err(e) => {
                error!("error parsing runtime config", safe: { file: path }, error: e);
                continue;
            }
        };

        value.ignored.log();

        // it's okay to use block_in_place here since we know this future is running as its own task.
        if let Err(errors) = task::block_in_place(|| handle.refresh(value.value)) {
            for error in errors {
                error!("error reloading runtime config", safe: { file: path }, error: error);
            }
        }

        info!("reloaded runtime config");
    }
}
