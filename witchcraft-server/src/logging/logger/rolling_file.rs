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
use crate::logging::logger::byte_buffer::BufBytesSink;
use async_compression::tokio::write::GzipEncoder;
use bytes::{Buf, Bytes};
use conjure_error::Error;
use conjure_object::chrono::{Date, TimeZone};
use conjure_object::Utc;
use futures_sink::Sink;
use futures_util::ready;
use pin_project::pin_project;
use regex::Regex;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{self, AsyncWrite, AsyncWriteExt};
use tokio::task;

const MAX_LOG_SIZE: u64 = 1024 * 1024 * 1024;

struct CurrentFile {
    file: BufBytesSink<FileBytesSink>,
    len: u64,
    date: Date<Utc>,
}

impl CurrentFile {
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.file).poll_ready(cx)
    }

    fn start_send(&mut self, item: Bytes) -> io::Result<()> {
        self.len += item.remaining() as u64;
        Pin::new(&mut self.file).start_send(item)
    }

    fn poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.file).poll_flush(cx)
    }

    fn poll_close(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.file).poll_close(cx)
    }
}

enum State {
    Live(CurrentFile),
    Rotating(Pin<Box<dyn Future<Output = io::Result<File>> + Sync + Send>>),
}

pub struct RollingFileAppender {
    state: State,
    next_archive_index: u32,
    name: &'static str,
    max_archive_size: u64,
    max_archive_days: u32,
    archive_locator: Arc<ArchiveLocator>,
}

impl RollingFileAppender {
    pub async fn new(
        name: &'static str,
        size_limit_gb: u32,
        max_archive_days: u32,
    ) -> Result<Self, Error> {
        let max_archive_size = u64::from(size_limit_gb) * 1024 * 1024 * 1024;

        let dir = log_dir();
        fs::create_dir_all(&dir)
            .await
            .map_err(Error::internal_safe)?;
        let file_path = log_path(dir, name);
        let file = open_log(&file_path).await.map_err(Error::internal_safe)?;
        let len = file.metadata().await.map_err(Error::internal_safe)?.len();

        let archive_locator = ArchiveLocator::new(name);
        let date = Utc::now().date();

        let next_archive_index = archive_locator
            .archived_logs(dir)
            .await
            .map_err(Error::internal_safe)?
            .iter()
            .chain(
                archive_locator
                    .uncompressed_logs(dir)
                    .await
                    .map_err(Error::internal_safe)?
                    .iter(),
            )
            .filter(|l| l.date == date)
            .map(|l| l.number)
            .max()
            .map_or(0, |n| n + 1);

        clear_old_archives(
            dir,
            date,
            max_archive_size,
            max_archive_days,
            &archive_locator,
        )
        .await
        .map_err(Error::internal_safe)?;

        clear_tmp_files(dir, &archive_locator)
            .await
            .map_err(Error::internal_safe)?;
        restart_compression(dir, name, &archive_locator)
            .await
            .map_err(Error::internal_safe)?;

        Ok(RollingFileAppender {
            state: State::Live(CurrentFile {
                file: BufBytesSink::new(FileBytesSink::new(file)),
                len,
                date,
            }),
            next_archive_index,
            name,
            max_archive_size,
            max_archive_days,
            archive_locator: Arc::new(archive_locator),
        })
    }
}

impl Sink<Bytes> for RollingFileAppender {
    type Error = io::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        loop {
            let this = &mut *self;
            match &mut this.state {
                State::Live(file) => {
                    let date = Utc::now().date();
                    if file.len < MAX_LOG_SIZE && date <= file.date {
                        return file.poll_ready(cx);
                    }

                    ready!(file.poll_close(cx))?;

                    let number = this.next_archive_index;
                    if date > file.date {
                        this.next_archive_index = 0;
                    } else {
                        this.next_archive_index += 1;
                    }

                    this.state = State::Rotating(Box::pin(rotate(
                        log_dir(),
                        this.name,
                        file.date,
                        number,
                        this.max_archive_size,
                        this.max_archive_days,
                        this.archive_locator.clone(),
                    )));
                }
                State::Rotating(future) => match ready!(future.as_mut().poll(cx)) {
                    Ok(file) => {
                        self.state = State::Live(CurrentFile {
                            file: BufBytesSink::new(FileBytesSink::new(file)),
                            len: 0,
                            date: Utc::now().date(),
                        });
                    }
                    Err(e) => {
                        let path = log_path(log_dir(), this.name);
                        this.state =
                            State::Rotating(Box::pin(async move { open_log(&path).await }));
                        return Poll::Ready(Err(e));
                    }
                },
            }
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        match &mut self.state {
            State::Live(file) => file.start_send(item),
            State::Rotating(_) => panic!("start_send called without poll_ready"),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut self.state {
            State::Live(file) => file.poll_flush(cx),
            State::Rotating(_) => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut self.state {
            State::Live(file) => file.poll_close(cx),
            State::Rotating(_) => Poll::Ready(Ok(())),
        }
    }
}

async fn open_log(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)
        .await
}

fn log_dir() -> &'static Path {
    Path::new("var/log")
}

fn log_path(dir: &Path, name: &str) -> PathBuf {
    let mut path = dir.to_path_buf();
    path.push(format!("{}.log", name));
    path
}

fn archive_path(dir: &Path, name: &str, date: Date<Utc>, number: u32) -> PathBuf {
    let mut path = dir.to_path_buf();
    path.push(format!("{}-{}-{}.log", name, date.naive_utc(), number,));
    path
}

fn archive_gz_tmp_path(dir: &Path, name: &str, date: Date<Utc>, number: u32) -> PathBuf {
    let mut path = dir.to_path_buf();
    path.push(format!(
        "{}-{}-{}.log.gz.tmp",
        name,
        date.naive_utc(),
        number,
    ));
    path
}

fn archive_gz_path(dir: &Path, name: &str, date: Date<Utc>, number: u32) -> PathBuf {
    let mut path = dir.to_path_buf();
    path.push(format!("{}-{}-{}.log.gz", name, date.naive_utc(), number,));
    path
}

async fn clear_old_archives(
    dir: &Path,
    date: Date<Utc>,
    max_archive_size: u64,
    max_archive_days: u32,
    archive_locator: &ArchiveLocator,
) -> io::Result<()> {
    let logs = archive_locator.archived_logs(dir).await?;
    clear_old_archives_inner(date, max_archive_size, max_archive_days, logs).await
}

// split out for testing
async fn clear_old_archives_inner(
    date: Date<Utc>,
    max_archive_size: u64,
    max_archive_days: u32,
    mut logs: Vec<ArchivedLog>,
) -> io::Result<()> {
    logs.sort_by_key(|l| (l.date, l.number));

    let mut total_size = logs.iter().map(|l| l.len).sum::<u64>();

    let mut date_cutoff = date;
    // do a silly loop to make sure we're correct WRT leap things
    for _ in 0..max_archive_days {
        date_cutoff = date_cutoff.pred();
    }

    for log in logs {
        if log.date >= date_cutoff && total_size < max_archive_size {
            break;
        }

        // management infrastructure could be cleaning these up concurrently, so an error is ok
        let _ = fs::remove_file(&log.path).await;
        total_size -= log.len;
    }

    Ok(())
}

async fn clear_tmp_files(dir: &Path, archive_locator: &ArchiveLocator) -> io::Result<()> {
    for log in archive_locator.tmp_files(dir).await? {
        fs::remove_file(&log.path).await?;
    }

    Ok(())
}

async fn restart_compression(
    dir: &Path,
    name: &str,
    archive_locator: &ArchiveLocator,
) -> io::Result<()> {
    for log in archive_locator.uncompressed_logs(dir).await? {
        let dir = dir.to_path_buf();
        let name = name.to_string();
        task::spawn(async move {
            let _ = compress(&dir, &name, log.date, log.number).await;
        });
    }

    Ok(())
}

async fn compress(dir: &Path, name: &str, date: Date<Utc>, number: u32) -> io::Result<()> {
    let source_path = archive_path(dir, name, date, number);
    let mut source = File::open(&source_path).await?;

    let tmp_path = archive_gz_tmp_path(dir, name, date, number);
    let target = File::create(&tmp_path).await?;
    let mut target = GzipEncoder::new(target);

    io::copy(&mut source, &mut target).await?;
    target.shutdown().await?;

    let path = archive_gz_path(dir, name, date, number);
    fs::rename(&tmp_path, &path).await?;

    fs::remove_file(&source_path).await?;

    Ok(())
}

async fn rotate(
    dir: &Path,
    name: &'static str,
    date: Date<Utc>,
    number: u32,
    max_archive_size: u64,
    max_archive_days: u32,
    archive_locator: Arc<ArchiveLocator>,
) -> io::Result<File> {
    let log_path = log_path(dir, name);
    let tmp_path = archive_path(dir, name, date, number);

    fs::rename(&log_path, &tmp_path).await?;

    let dir = dir.to_path_buf();
    task::spawn(async move {
        let _ = compress(&dir, name, date, number).await;
        // clear archives based on the current date rather than the date of the log being archived.
        let _ = clear_old_archives(
            &dir,
            Utc::now().date(),
            max_archive_size,
            max_archive_days,
            &archive_locator,
        )
        .await;
    });

    open_log(&log_path).await
}

struct ArchiveLocator {
    gz_regex: Regex,
    gz_tmp_regex: Regex,
    raw_regex: Regex,
}

impl ArchiveLocator {
    fn new(name: &str) -> ArchiveLocator {
        let gz_regex = format!(
            r"^{}-(\d{{4}})-(\d{{2}})-(\d{{2}})-(\d+)\.log\.gz$",
            regex::escape(name)
        );
        let gz_tmp_regex = format!(
            r"^{}-(\d{{4}})-(\d{{2}})-(\d{{2}})-(\d+)\.log\.gz\.tmp$",
            regex::escape(name)
        );
        let raw_regex = format!(
            r"^{}-(\d{{4}})-(\d{{2}})-(\d{{2}})-(\d+)\.log$",
            regex::escape(name)
        );
        ArchiveLocator {
            gz_regex: Regex::new(&gz_regex).unwrap(),
            gz_tmp_regex: Regex::new(&gz_tmp_regex).unwrap(),
            raw_regex: Regex::new(&raw_regex).unwrap(),
        }
    }

    async fn uncompressed_logs(&self, dir: &Path) -> io::Result<Vec<ArchivedLog>> {
        self.get_logs(&self.raw_regex, dir).await
    }

    async fn tmp_files(&self, dir: &Path) -> io::Result<Vec<ArchivedLog>> {
        self.get_logs(&self.gz_tmp_regex, dir).await
    }

    async fn archived_logs(&self, dir: &Path) -> io::Result<Vec<ArchivedLog>> {
        self.get_logs(&self.gz_regex, dir).await
    }

    async fn get_logs(&self, regex: &Regex, dir: &Path) -> io::Result<Vec<ArchivedLog>> {
        let mut logs = vec![];
        let mut files = fs::read_dir(dir).await?;
        while let Some(file) = files.next_entry().await? {
            let name = file.file_name();
            let name = match name.to_str() {
                Some(name) => name,
                None => continue,
            };

            let captures = match regex.captures(name) {
                Some(captures) => captures,
                None => continue,
            };

            let year = captures[1].parse().unwrap();
            let month = captures[2].parse().unwrap();
            let day = captures[3].parse().unwrap();
            let date = match Utc.ymd_opt(year, month, day).single() {
                Some(date) => date,
                None => continue,
            };

            let number = match captures[4].parse() {
                Ok(number) => number,
                Err(_) => continue,
            };

            let len = file.metadata().await?.len();

            let log = ArchivedLog {
                path: file.path(),
                date,
                number,
                len,
            };
            logs.push(log);
        }
        Ok(logs)
    }
}

#[derive(Debug, PartialEq)]
struct ArchivedLog {
    path: PathBuf,
    date: Date<Utc>,
    number: u32,
    len: u64,
}

#[pin_project]
struct FileBytesSink {
    #[pin]
    file: File,
    pending: Bytes,
}

impl FileBytesSink {
    fn new(file: File) -> Self {
        FileBytesSink {
            file,
            pending: Bytes::new(),
        }
    }
}

impl Sink<Bytes> for FileBytesSink {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        let this = self.project();
        debug_assert!(this.pending.is_empty());
        *this.pending = item;

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        while !this.pending.is_empty() {
            let nwritten = ready!(this.file.as_mut().poll_write(cx, this.pending))?;
            this.pending.advance(nwritten);
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use async_compression::tokio::bufread::GzipDecoder;
    use tokio::io::AsyncReadExt;

    #[test]
    fn log_path_format() {
        let name = "service";

        assert_eq!(log_path(log_dir(), name), Path::new("var/log/service.log"));
    }

    #[test]
    fn archive_tmp_path_format() {
        let name = "service";
        let date = Utc.ymd(2017, 4, 20);
        let number = 3;

        assert_eq!(
            archive_path(log_dir(), name, date, number),
            Path::new("var/log/service-2017-04-20-3.log"),
        );
    }

    #[test]
    fn archive_path_format() {
        let name = "service";
        let date = Utc.ymd(2017, 4, 20);
        let number = 3;

        assert_eq!(
            archive_gz_path(log_dir(), name, date, number),
            Path::new("var/log/service-2017-04-20-3.log.gz"),
        );
    }

    #[tokio::test]
    async fn compress_validity() {
        let dir = tempfile::tempdir().unwrap();

        let name = "service";
        let date = Utc.ymd(2017, 4, 20);
        let number = 3;

        let tmp_path = archive_path(dir.path(), name, date, number);
        let mut file = open_log(&tmp_path).await.unwrap();
        file.write_all(b"hello world").await.unwrap();
        file.flush().await.unwrap();

        compress(dir.path(), name, date, number).await.unwrap();

        let archive_path = archive_gz_path(dir.path(), name, date, number);
        let file = fs::read(&archive_path).await.unwrap();
        let mut buf = vec![];
        GzipDecoder::new(&mut &file[..])
            .read_to_end(&mut buf)
            .await
            .unwrap();

        assert_eq!(buf, b"hello world");
    }

    #[tokio::test]
    async fn archive_locator() {
        let dir = tempfile::tempdir().unwrap();

        let service_path = log_path(dir.path(), "service");
        let file = File::create(&service_path).await.unwrap();
        file.set_len(1).await.unwrap();

        let requests_path = log_path(dir.path(), "requests");
        let file = File::create(&requests_path).await.unwrap();
        file.set_len(2).await.unwrap();

        let day1 = Utc.ymd(2017, 4, 20);
        let service_archive_1_0_path = archive_gz_path(dir.path(), "service", day1, 0);
        let file = File::create(&service_archive_1_0_path).await.unwrap();
        file.set_len(3).await.unwrap();

        let service_archive_1_1_path = archive_gz_path(dir.path(), "service", day1, 1);
        let file = File::create(&service_archive_1_1_path).await.unwrap();
        file.set_len(4).await.unwrap();

        let day2 = Utc.ymd(2017, 4, 21);
        let service_archive_2_0_path = archive_gz_path(dir.path(), "service", day2, 0);
        let file = File::create(&service_archive_2_0_path).await.unwrap();
        file.set_len(5).await.unwrap();

        let service_archive_2_1_path = archive_gz_path(dir.path(), "service", day2, 1);
        let file = File::create(&service_archive_2_1_path).await.unwrap();
        file.set_len(6).await.unwrap();

        let requests_archive_1_0_path = archive_gz_path(dir.path(), "requests", day1, 0);
        let file = File::create(&requests_archive_1_0_path).await.unwrap();
        file.set_len(7).await.unwrap();

        let locator = ArchiveLocator::new("service");

        let mut logs = locator.archived_logs(dir.path()).await.unwrap();
        logs.sort_by_key(|l| (l.date, l.number));

        let expected = [
            ArchivedLog {
                path: service_archive_1_0_path,
                date: day1,
                number: 0,
                len: 3,
            },
            ArchivedLog {
                path: service_archive_1_1_path,
                date: day1,
                number: 1,
                len: 4,
            },
            ArchivedLog {
                path: service_archive_2_0_path,
                date: day2,
                number: 0,
                len: 5,
            },
            ArchivedLog {
                path: service_archive_2_1_path,
                date: day2,
                number: 1,
                len: 6,
            },
        ];

        assert_eq!(logs, expected);
    }

    #[tokio::test]
    async fn clear_old_archives_always_deletes_old_logs() {
        let dir = tempfile::tempdir().unwrap();

        let day1 = Utc.ymd(2017, 4, 20);
        let service_archive_1_0_path = archive_gz_path(dir.path(), "service", day1, 0);
        File::create(&service_archive_1_0_path).await.unwrap();

        let service_archive_1_1_path = archive_gz_path(dir.path(), "service", day1, 1);
        File::create(&service_archive_1_1_path).await.unwrap();

        let day2 = Utc.ymd(2017, 4, 21);
        let service_archive_2_0_path = archive_gz_path(dir.path(), "service", day2, 0);
        File::create(&service_archive_2_0_path).await.unwrap();

        let service_archive_2_1_tmp_path = archive_path(dir.path(), "service", day2, 1);
        File::create(&service_archive_2_1_tmp_path).await.unwrap();

        let logs = vec![
            ArchivedLog {
                path: service_archive_1_0_path.clone(),
                date: day1,
                number: 0,
                len: 0,
            },
            ArchivedLog {
                path: service_archive_1_1_path.clone(),
                date: day1,
                number: 1,
                len: 0,
            },
            ArchivedLog {
                path: service_archive_2_0_path.clone(),
                date: day2,
                number: 0,
                len: 0,
            },
            ArchivedLog {
                path: service_archive_2_1_tmp_path.clone(),
                date: day2,
                number: 1,
                len: 0,
            },
        ];

        let date = Utc.ymd(2017, 5, 21);
        clear_old_archives_inner(date, 1024 * 1024 * 1024, 30, logs)
            .await
            .unwrap();

        assert!(!service_archive_1_0_path.exists());
        assert!(!service_archive_1_1_path.exists());
        assert!(service_archive_2_0_path.exists());
        assert!(service_archive_2_1_tmp_path.exists());
    }

    #[tokio::test]
    async fn clear_old_archives_deletes_to_save_space() {
        let dir = tempfile::tempdir().unwrap();

        let day1 = Utc.ymd(2017, 4, 20);
        let service_archive_1_0_path = archive_gz_path(dir.path(), "service", day1, 0);
        File::create(&service_archive_1_0_path).await.unwrap();

        let service_archive_1_1_path = archive_gz_path(dir.path(), "service", day1, 1);
        File::create(&service_archive_1_1_path).await.unwrap();

        let day2 = Utc.ymd(2017, 4, 21);
        let service_archive_2_0_path = archive_gz_path(dir.path(), "service", day2, 0);
        File::create(&service_archive_2_0_path).await.unwrap();

        let service_archive_2_1_tmp_path = archive_gz_tmp_path(dir.path(), "service", day2, 1);
        File::create(&service_archive_2_1_tmp_path).await.unwrap();

        let logs = vec![
            ArchivedLog {
                path: service_archive_1_0_path.clone(),
                date: day1,
                number: 0,
                len: 4,
            },
            ArchivedLog {
                path: service_archive_1_1_path.clone(),
                date: day1,
                number: 1,
                len: 5,
            },
            ArchivedLog {
                path: service_archive_2_0_path.clone(),
                date: day2,
                number: 0,
                len: 1,
            },
            ArchivedLog {
                path: service_archive_2_1_tmp_path.clone(),
                date: day2,
                number: 1,
                len: 1023,
            },
        ];

        let date = Utc.ymd(2017, 4, 21);
        clear_old_archives_inner(date, 1024, 30, logs)
            .await
            .unwrap();

        assert!(!service_archive_1_0_path.exists());
        assert!(!service_archive_1_1_path.exists());
        assert!(!service_archive_2_0_path.exists());
        assert!(service_archive_2_1_tmp_path.exists());
    }

    #[tokio::test]
    async fn clear_old_archives_ignores_missing_files() {
        let dir = tempfile::tempdir().unwrap();

        let day1 = Utc.ymd(2017, 4, 20);
        let service_archive_1_0_path = archive_gz_path(dir.path(), "service", day1, 0);
        // not actually making this

        let service_archive_1_1_path = archive_gz_path(dir.path(), "service", day1, 1);
        File::create(&service_archive_1_1_path).await.unwrap();

        let day2 = Utc.ymd(2017, 4, 21);
        let service_archive_2_0_path = archive_gz_path(dir.path(), "service", day2, 0);
        File::create(&service_archive_2_0_path).await.unwrap();

        let service_archive_2_1_tmp_path = archive_gz_tmp_path(dir.path(), "service", day2, 1);
        File::create(&service_archive_2_1_tmp_path).await.unwrap();

        let logs = vec![
            ArchivedLog {
                path: service_archive_1_0_path.clone(),
                date: day1,
                number: 0,
                len: 4,
            },
            ArchivedLog {
                path: service_archive_1_1_path.clone(),
                date: day1,
                number: 1,
                len: 5,
            },
            ArchivedLog {
                path: service_archive_2_0_path.clone(),
                date: day2,
                number: 0,
                len: 1,
            },
            ArchivedLog {
                path: service_archive_2_1_tmp_path.clone(),
                date: day2,
                number: 1,
                len: 1023,
            },
        ];

        let date = Utc.ymd(2017, 4, 21);
        clear_old_archives_inner(date, 1024, 30, logs)
            .await
            .unwrap();

        assert!(!service_archive_1_0_path.exists());
        assert!(!service_archive_1_1_path.exists());
        assert!(!service_archive_2_0_path.exists());
        assert!(service_archive_2_1_tmp_path.exists());
    }
}
