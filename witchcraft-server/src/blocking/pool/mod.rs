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
use crate::blocking::pool::job_queue::JobQueue;
use conjure_error::Error;
use parking_lot::Mutex;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use witchcraft_log::error;
use witchcraft_metrics::MetricRegistry;
use witchcraft_server_config::install::InstallConfig;

mod job_queue;

struct State {
    threads: usize,
    idle_threads: usize,
    next_id: usize,
}

impl State {
    fn active(&self) -> usize {
        self.threads - self.idle_threads
    }
}

struct Shared {
    min_threads: usize,
    max_threads: usize,
    idle_timeout: Duration,
    queue: JobQueue<Box<dyn FnOnce() + Send>>,
    state: Mutex<State>,
}

impl Shared {
    fn max(&self) -> usize {
        self.max_threads
    }

    fn active(&self) -> usize {
        self.state.lock().active()
    }

    fn utilization_max(&self) -> f64 {
        let used = self.state.lock().active();
        let available = self.max_threads;
        used as f64 / available as f64
    }

    fn worker_loop(&self) {
        while let Some(job) = self.get_job() {
            let _ = panic::catch_unwind(AssertUnwindSafe(job));
        }
    }

    fn get_job(&self) -> Option<Box<dyn FnOnce() + Send>> {
        // fast path if there's a job ready
        if let Some(job) = self.queue.try_pop() {
            return Some(job);
        }

        let mut timeout = Instant::now() + self.idle_timeout;
        let mut state = self.state.lock();
        loop {
            state.idle_threads += 1;
            drop(state);
            let r = self.queue.pop_until(timeout);
            state = self.state.lock();
            state.idle_threads -= 1;

            match r {
                Some(job) => return Some(job),
                None => {
                    if state.threads > self.min_threads {
                        state.threads -= 1;
                        return None;
                    } else {
                        timeout += self.idle_timeout;
                    }
                }
            }
        }
    }
}

pub struct ThreadPool {
    shared: Arc<Shared>,
}

impl ThreadPool {
    pub fn new(config: &InstallConfig, metrics: &MetricRegistry) -> Self {
        let pool = ThreadPool {
            shared: Arc::new(Shared {
                min_threads: config.server().min_threads(),
                max_threads: config.server().max_threads(),
                idle_timeout: config.server().idle_thread_timeout(),
                queue: JobQueue::new(),
                state: Mutex::new(State {
                    threads: 0,
                    idle_threads: 0,
                    next_id: 0,
                }),
            }),
        };

        metrics.gauge("server.worker.max", {
            let shared = pool.shared.clone();
            move || shared.max()
        });
        metrics.gauge("server.worker.active", {
            let shared = pool.shared.clone();
            move || shared.active()
        });
        metrics.gauge("server.worker.utilization-max", {
            let shared = pool.shared.clone();
            move || shared.utilization_max()
        });

        let mut state = pool.shared.state.lock();
        for _ in 0..config.server().min_threads() {
            pool.add_thread(&mut state);
        }
        drop(state);

        pool
    }

    fn add_thread(&self, state: &mut State) {
        if state.threads >= self.shared.max_threads {
            return;
        }

        let id = state.next_id;
        state.next_id += 1;
        let r = thread::Builder::new().name(format!("server-{id}")).spawn({
            let shared = self.shared.clone();
            move || shared.worker_loop()
        });

        if let Err(e) = r {
            error!(
                "failed to spawn new worker thread",
                error: Error::internal_safe(e)
            );
        }
    }

    pub fn try_execute<F>(&self, f: F) -> Result<(), F>
    where
        F: FnOnce() + 'static + Send,
    {
        let mut state = self.shared.state.lock();
        let current_jobs = self.shared.queue.len() + state.active();
        if current_jobs >= self.shared.max_threads {
            return Err(f);
        }

        self.shared.queue.push(Box::new(f));

        if self.shared.queue.len() > state.idle_threads {
            self.add_thread(&mut state);
        }

        Ok(())
    }
}
