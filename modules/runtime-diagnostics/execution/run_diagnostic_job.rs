use crate::{DiagnosticJob, Session};
use serde_json::{Value, json};

pub struct JobExecutionRequest {
    pub job: DiagnosticJob,
    pub game_root: std::path::PathBuf,
    pub exports: crate::write_job_exports::JobExports,
}

#[derive(Default)]
pub struct JobOutput {
    pub report: Value,
    pub finished: bool,
}

/// The existing mod-runtime owns execution. Diagnostics owns its lifetime.
pub trait DiagnosticHost {
    fn start_job(&mut self, request: JobExecutionRequest) -> Result<Box<dyn RunningJob>, String>;
}

pub trait RunningJob {
    fn update(&mut self, delta_seconds: f64) -> Result<JobOutput, String>;
    fn stop(&mut self, reason: &str) -> Result<JobOutput, String>;
}

pub(crate) struct UnavailableHost;
impl DiagnosticHost for UnavailableHost {
    fn start_job(&mut self, _: JobExecutionRequest) -> Result<Box<dyn RunningJob>, String> {
        Err("this diagnostic job requires the active mod-runtime host".into())
    }
}

impl Session {
    pub(crate) fn advance_job_execution(
        &mut self,
        host: &mut dyn DiagnosticHost,
    ) -> Result<bool, String> {
        let Some(job) = self.job.as_ref().filter(|job| job.execution.is_some()) else {
            return Ok(false);
        };
        if self.job_execution.is_none() {
            let exports = crate::write_job_exports::JobExports::new(&self.root, &job.id);
            self.job_exports = Some(exports.directory().display().to_string());
            tracing::info!(target:"shroudforge::diagnostics", job_id=%job.id,
                exports_directory=%exports.directory().display(), "Diagnostic job execution starting");
            let request = JobExecutionRequest {
                job: job.clone(),
                game_root: self.root.clone(),
                exports,
            };
            self.job_execution = Some(
                if matches!(
                    request.job.execution,
                    Some(crate::DiagnosticExecution::Python { .. })
                ) {
                    crate::run_python_diagnostic_job::start(&self.root, request)?
                } else {
                    host.start_job(request)?
                },
            );
            self.job_last_tick = std::time::Instant::now();
        }
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.job_last_tick).as_secs_f64();
        self.job_last_tick = now;
        let output = self.job_execution.as_mut().unwrap().update(delta)?;
        if !output.report.is_null() {
            self.job_report = output.report;
        }
        Ok(output.finished)
    }

    pub(crate) fn stop_job_execution(&mut self, reason: &str) {
        if let Some(mut execution) = self.job_execution.take() {
            match execution.stop(reason) {
                Ok(output) if !output.report.is_null() => self.job_report = output.report,
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(target:"shroudforge::diagnostics", %error, "Diagnostic job cleanup failed");
                    if !self.job_report.is_object() {
                        self.job_report = json!({});
                    }
                    self.job_report["cleanupError"] = json!(error);
                }
            }
        }
    }

    /// End before the Lua host is unloaded. Never restart implicitly.
    pub fn release_runtime_host(&mut self) {
        if self.active && self.job.as_ref().is_some_and(|job| job.execution.is_some()) {
            self.finish_diagnostic_job("runtime-host-stopped");
            self.publish();
        }
    }

    pub fn release_mod_host(&mut self, mod_id: &str) {
        if self
            .job
            .as_ref()
            .and_then(|job| job.execution.as_ref())
            .is_some_and(|execution| execution.mod_id() == Some(mod_id))
        {
            self.release_runtime_host();
        }
    }
}
