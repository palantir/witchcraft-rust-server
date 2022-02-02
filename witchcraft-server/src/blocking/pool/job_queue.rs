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
use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::ptr;
use std::time::Instant;

struct Node {
    cvar: Condvar,
    next: *mut Node,
    prev: *mut Node,
}

struct State<T> {
    head: *mut Node,
    jobs: VecDeque<T>,
}

unsafe impl<T> Sync for State<T> where T: Sync {}
unsafe impl<T> Send for State<T> where T: Send {}

/// A blocking queue that is "maximally unfair" to waiters.
///
/// That is, while jobs are processed FIFO, waiters are processed LIFO. This allows us to keep the number of threads in
/// the pool to the minimum number required to keep up with the current request volume.
///
/// To make this happen, we unfortunately need to use a manual queueing implementation with intrusive lists rather than
/// a simple Mutex + Condvar.
pub struct JobQueue<T> {
    state: Mutex<State<T>>,
}

impl<T> JobQueue<T> {
    pub fn new() -> Self {
        JobQueue {
            state: Mutex::new(State {
                head: ptr::null_mut(),
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

        unsafe {
            if !state.head.is_null() {
                let woken = state.head;
                state.head = (*woken).next;
                if !state.head.is_null() {
                    (*state.head).prev = ptr::null_mut();
                }

                (*woken).next = ptr::null_mut();
                (*woken).cvar.notify_one();
            }
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

            let node = UnsafeCell::new(Node {
                cvar: Condvar::new(),
                next: state.head,
                prev: ptr::null_mut(),
            });

            unsafe {
                if !state.head.is_null() {
                    (*state.head).prev = node.get();
                }
                state.head = node.get();

                let result = (*node.get()).cvar.wait_until(&mut state, timeout);

                if !(*node.get()).next.is_null() {
                    (*(*node.get()).next).prev = (*node.get()).prev;
                }

                if !(*node.get()).prev.is_null() {
                    (*(*node.get()).prev).next = (*node.get()).next;
                } else if state.head == node.get() {
                    state.head = (*node.get()).next;
                }

                if result.timed_out() {
                    return None;
                }
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
