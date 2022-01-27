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
use std::path::Path;
use std::time::{Duration, SystemTime};
use tokio::fs;
use witchcraft_log::{error, info};

// Archived logs are retained for 30 days, so anything more than 31 days old is definitely eligible for cleanup
const MAX_AGE: Duration = Duration::from_secs(31 * 24 * 60 * 60);

pub async fn cleanup_logs() {
    let path = Path::new("var/log");

    if let Err(e) = cleanup_logs_inner(path, SystemTime::now()).await {
        error!("error cleaning up log directory", safe: { directory: path }, error: e);
    }
}

async fn cleanup_logs_inner(path: &Path, now: SystemTime) -> Result<(), Error> {
    fs::create_dir_all(path)
        .await
        .map_err(Error::internal_safe)?;

    let mut dir = fs::read_dir(path).await.map_err(Error::internal_safe)?;

    while let Some(entry) = dir.next_entry().await.map_err(Error::internal_safe)? {
        let metadata = match entry.metadata().await {
            Ok(metadata) => metadata,
            Err(e) => {
                error!(
                    "error statting log file",
                    safe: {
                        directory: path,
                    },
                    unsafe: {
                        file: entry.file_name()
                    },
                    error: Error::internal_safe(e),
                );
                continue;
            }
        };

        if metadata.is_dir() {
            continue;
        }

        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(e) => {
                info!(
                    "file modification times are not supported by the filesystem, skipping log cleanup",
                    safe: {
                        directory: path,
                    },
                    error: Error::internal_safe(e),
                );
                return Ok(());
            }
        };

        let age = match now.duration_since(modified) {
            Ok(age) => age,
            Err(_) => continue,
        };

        if age < MAX_AGE {
            continue;
        }

        match fs::remove_file(entry.path()).await {
            Ok(()) => {
                info!(
                    "deleted file more than 30 days old in the log directory",
                    safe: {
                        directory: path,
                        size: metadata.len(),
                        age: format_args!("{:?}", age),
                    },
                    unsafe: {
                        file: entry.file_name(),
                    },
                );
            }
            Err(e) => {
                error!(
                    "error deleting file more than 30 days old from log directory",
                    safe: {
                        directory: path,
                        size: metadata.len(),
                        age: format_args!("{:?}", age),
                    },
                    unsafe: {
                        file: entry.file_name(),
                    },
                    error: Error::internal_safe(e),
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn delete_old_files() {
        let dir = tempfile::tempdir().unwrap();

        let file1 = dir.path().join("file1");
        let file2 = dir.path().join("file2");
        fs::write(&file1, &[]).await.unwrap();
        fs::write(&file2, &[]).await.unwrap();

        let now = SystemTime::now() + MAX_AGE + Duration::from_secs(10);
        cleanup_logs_inner(dir.path(), now).await.unwrap();

        assert!(!file1.exists());
        assert!(!file2.exists());
    }

    #[tokio::test]
    async fn preserve_new_files() {
        let dir = tempfile::tempdir().unwrap();

        let file1 = dir.path().join("file1");
        let file2 = dir.path().join("file2");
        fs::write(&file1, &[]).await.unwrap();
        fs::write(&file2, &[]).await.unwrap();

        let now = SystemTime::now() + MAX_AGE - Duration::from_secs(10);
        cleanup_logs_inner(dir.path(), now).await.unwrap();

        assert!(file1.exists());
        assert!(file2.exists());
    }
}
