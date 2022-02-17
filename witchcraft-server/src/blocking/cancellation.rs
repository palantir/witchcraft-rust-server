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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A type tracking the cancellation state of a request.
///
/// This type will be added to the extensions of each request made to a blocking endpoint.
#[derive(Clone, Debug)]
pub struct Cancellation {
    cancelled: Arc<AtomicBool>,
}

impl Cancellation {
    pub(crate) fn new() -> (Cancellation, CancellationGuard) {
        let cancelled = Arc::new(AtomicBool::new(false));
        (
            Cancellation {
                cancelled: cancelled.clone(),
            },
            CancellationGuard { cancelled },
        )
    }

    /// Returns `true` if the client of a request has cancelled it.
    ///
    /// Long running blocking endpoint handlers should periodically check this to determine if they should continue
    /// working or not.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

pub struct CancellationGuard {
    cancelled: Arc<AtomicBool>,
}

impl Drop for CancellationGuard {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}
