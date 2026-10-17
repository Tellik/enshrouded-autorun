use crate::diagnostics_jobs::{DiagnosticJob, mod_callbacks::Measurement};
use crate::refresh_session_settings::defaults;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Instant,
};

pub struct Session {
    pub(super) root: PathBuf,
    pub(super) settings: Value,
    pub(super) active: bool,
    pub(super) started: Instant,
    pub(super) next_config: Instant,
    pub(super) next_sample: Instant,
    pub(super) last_report: Value,
    pub(super) last_signature: Value,
    pub(super) last_error: String,
    pub(super) reason: &'static str,
    pub(super) measurements: BTreeMap<String, Measurement>,
    pub(super) job: Option<DiagnosticJob>,
    pub(super) last_sample: u64,
}

impl Session {
    pub fn new(root: &Path) -> Self {
        let now = Instant::now();
        let mut session = Self {
            root: root.into(),
            settings: defaults(),
            active: false,
            started: now,
            next_config: now,
            next_sample: now,
            last_report: Value::Null,
            last_signature: Value::Null,
            last_error: String::new(),
            reason: "disabled",
            measurements: BTreeMap::new(),
            job: None,
            last_sample: 0,
        };
        session.refresh();
        session.publish();
        session
    }
}
