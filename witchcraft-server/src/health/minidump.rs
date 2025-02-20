use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::time::{Duration, Instant};

use super::{HealthCheck, HealthCheckResult, HealthState};

/// A health check which reports an error state if minidump initialization has not completed successfully.
pub struct MinidumpHealthCheck {
    minidump_ok: Arc<AtomicBool>,
    first_unhealthy_time: Arc<Mutex<Option<Instant>>>,
}

impl MinidumpHealthCheck {
    pub fn new(minidump_ok: Arc<AtomicBool>) -> Self {
        MinidumpHealthCheck { minidump_ok,
        first_unhealthy_time: Arc::new(Mutex::new(None)) }
    }
}

impl HealthCheck for MinidumpHealthCheck {
    fn type_(&self) -> &str {
        "MINIDUMP"
    }

    fn result(&self) -> HealthCheckResult {
        if self.minidump_ok.load(Ordering::Relaxed) {
            let mut start_time = self.first_unhealthy_time.lock().unwrap();
            *start_time = None;
            return HealthCheckResult::builder().state(HealthState::Healthy).build();
        }

        let mut start_time = self.first_unhealthy_time.lock().unwrap();

        let elapsed = start_time.get_or_insert_with(Instant::now).elapsed();

        if elapsed > Duration::from_secs(300) {
            HealthCheckResult::builder()
                .state(HealthState::Error)
                .message("minidump client could not connect to server for over 5 minutes".to_string())
                .build()
        } else {
            HealthCheckResult::builder()
                .state(HealthState::Warning)
                .message("minidump client has not connected to server".to_string())
                .build()
        }
    }
}
