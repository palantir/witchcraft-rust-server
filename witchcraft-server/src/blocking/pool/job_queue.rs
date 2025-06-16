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
use parking_lot::{Condvar, Mutex};
use pin_list::{Node, PinList};
use std::collections::VecDeque;
use std::pin::pin;
use std::time::Instant;

type PinListTypes = dyn pin_list::Types<
    Id = pin_list::id::Checked,
    Protected = (),
    Removed = (),
    Unprotected = Condvar,
>;

struct State<T> {
    waiters: PinList<PinListTypes>,
    jobs: VecDeque<T>,
}

/// A blocking queue that is "maximally unfair" to waiters.
///
/// That is, while jobs are processed FIFO, waiters are processed LIFO. This allows us to keep the number of threads in
/// the pool to the minimum number required to keep up with the current request volume.
///
/// To make this happen, we need to use a manual queueing implementation rather than a simple Mutex + Condvar.
pub struct JobQueue<T> {
    state: Mutex<State<T>>,
}

impl<T> JobQueue<T> {
    pub fn new() -> Self {
        JobQueue {
            state: Mutex::new(State {
                waiters: PinList::new(pin_list::id::Checked::new()),
                jobs: VecDeque::new(),
            }),
        }
    }

    pub fn len(&self) -> usize {
        self.state.lock().jobs.len()
    }

    pub fn push(&self, job: T) {
        let mut state = self.state.lock();

        state.jobs.push_back(job);

        let mut cursor = state.waiters.cursor_back_mut();
        if let Some(cvar) = cursor.unprotected() {
            cvar.notify_one();
            cursor.remove_current(()).expect("cursor at node");
        }
    }

    pub fn try_pop(&self) -> Option<T> {
        self.state.lock().jobs.pop_front()
    }

    pub fn pop_until(&self, timeout: Instant) -> Option<T> {
        let mut state = self.state.lock();

        loop {
            if let Some(job) = state.jobs.pop_front() {
                return Some(job);
            }

            let node = pin!(Node::new());
            let node = state.waiters.push_back(node, (), Condvar::new());

            let result = node.unprotected().wait_until(&mut state, timeout);
            // We may or may not have been removed from the list, but don't actually care which.
            node.reset(&mut state.waiters);

            if result.timed_out() {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn single_threaded() {
        let queue = JobQueue::new();

        queue.push(0);
        assert_eq!(queue.try_pop(), Some(0));
        assert_eq!(queue.try_pop(), None);

        queue.push(1);
        let start = Instant::now();
        assert_eq!(queue.pop_until(start - Duration::from_millis(10)), Some(1));
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(10));

        let start = Instant::now();
        assert_eq!(queue.pop_until(start + Duration::from_millis(10)), None);
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(10) && elapsed < Duration::from_millis(20));
    }

    #[test]
    fn wake_lifo() {
        let queue = Arc::new(JobQueue::<i32>::new());

        let handle1 = thread::spawn({
            let queue = queue.clone();
            move || queue.pop_until(Instant::now() + Duration::from_millis(1500))
        });

        let handle2 = thread::spawn({
            let queue = queue.clone();
            move || {
                thread::sleep(Duration::from_millis(100));
                queue.pop_until(Instant::now() + Duration::from_millis(1500))
            }
        });

        // wait for threads to get set up
        thread::sleep(Duration::from_secs(1));
        queue.push(0);

        assert_eq!(handle1.join().unwrap(), None);
        assert_eq!(handle2.join().unwrap(), Some(0));
    }
}
