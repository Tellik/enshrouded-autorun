//! Bounded, opt-in instrumentation hosted by the loader. No scans or game writes.
mod capture_runtime_snapshot;
mod create_diagnostic_session;
pub mod documents;
mod is_measurement_enabled;
mod measure_callback;
pub mod measure_runtime_operations;
mod publish_diagnostic_status;
mod read_diagnostic_status;
mod refresh_session_settings;
mod request_diagnostic_action;
mod run_diagnostic_command;
mod start_callback_measurement;
mod stop_callback_measurement;
mod tick_diagnostic_session;

pub use create_diagnostic_session::Session;
pub use read_diagnostic_status::status;
pub use request_diagnostic_action::request;
pub use run_diagnostic_command::run_module;

use std::time::{SystemTime, UNIX_EPOCH};

fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests;
