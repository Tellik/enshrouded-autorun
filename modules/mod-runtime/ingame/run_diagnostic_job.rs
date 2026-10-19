#[path = "../../runtime-diagnostics/diagnostics-jobs/game_inspection/inspect/inspect_system_dependencies.rs"]
mod system_dependency_inspection;

use crate::{env::AppState, lua::convert_json::lua_to_json, runner::LuaModRunner};
use mlua::{Function, Lua, Table, Value};
use serde_json::{Value as JsonValue, json};
use shroudforge_runtime_diagnostics::{
    DiagnosticExecution, DiagnosticHost, JobExecutionRequest, JobExports, JobOutput, RunningJob,
};
use std::{
    cell::RefCell,
    path::Path,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    thread::{self, JoinHandle},
};

struct ActiveDiagnosticMods(Vec<String>);

pub(super) struct RuntimeDiagnosticHost<'a> {
    runner: &'a LuaModRunner,
}

impl<'a> RuntimeDiagnosticHost<'a> {
    pub fn new(runner: &'a LuaModRunner, active_mods: Vec<String>) -> Self {
        runner.lua.set_app_data(ActiveDiagnosticMods(active_mods));
        Self { runner }
    }
}

#[derive(Default)]
struct ReportState {
    report: serde_json::Value,
    finished: bool,
}

struct LuaJob {
    lua: Lua,
    callbacks: Option<Table>,
    state: Rc<RefCell<ReportState>>,
    owner: String,
    interval: f64,
    accumulated: f64,
}

fn current_owner(lua: &Lua, owner: &str) -> bool {
    lua.app_data_ref::<ActiveDiagnosticMods>()
        .is_some_and(|mods| mods.0.iter().any(|id| id == owner))
        && lua
            .app_data_ref::<AppState>()
            .is_some_and(|state| state.runtime_mod_is_active(owner))
}

fn encoded(value: Value) -> mlua::Result<(serde_json::Value, String)> {
    let report = lua_to_json(value)?;
    let text = serde_json::to_string(&report).map_err(mlua::Error::external)?;
    if text.len() > shroudforge_runtime_diagnostics::MAX_EXPORT_BYTES {
        return Err(mlua::Error::runtime("diagnostic report exceeds 4 MiB"));
    }
    Ok((report, text))
}

fn encoded_report(value: Value) -> mlua::Result<(serde_json::Value, String)> {
    let result = encoded(value)?;
    if !result.0.is_object() {
        return Err(mlua::Error::runtime("diagnostic report must be an object"));
    }
    Ok(result)
}

impl DiagnosticHost for RuntimeDiagnosticHost<'_> {
    fn start_job(&mut self, request: JobExecutionRequest) -> Result<Box<dyn RunningJob>, String> {
        let execution = request
            .job
            .execution
            .as_ref()
            .ok_or("diagnostic job has no execution definition")?;
        if let DiagnosticExecution::Rust { action } = execution {
            let action = action.clone();
            return match action.as_str() {
                "inspect-system-dependencies" => start_system_dependency_job(request),
                _ => Err(format!(
                    "unsupported built-in Rust diagnostic action '{action}'"
                )),
            };
        }
        let lua: &Lua = &self.runner.lua;
        let owner = execution
            .mod_id()
            .ok_or("this diagnostic adapter does not use mod-runtime")?;
        if !current_owner(lua, owner) {
            return Err(format!("diagnostic job requires active mod '{}'", owner));
        }
        if let DiagnosticExecution::ModAction {
            mod_id,
            action,
            value,
        } = execution
        {
            let active = lua
                .app_data_ref::<ActiveDiagnosticMods>()
                .map(|mods| mods.0.clone())
                .unwrap_or_default();
            let mut report = serde_json::json!({"state":"calling", "modId":mod_id,"action":action});
            let persist = |report: &serde_json::Value| -> Result<(), String> {
                let text = serde_json::to_string(report).map_err(|error| error.to_string())?;
                request.exports.write("report.json", &text)
            };
            persist(&report)?;
            let result = crate::env::shroudforge::invoke_mod_action(
                lua,
                &active,
                mod_id,
                action,
                value.as_deref(),
            );
            report["state"] = serde_json::json!(if result.is_ok() { "complete" } else { "failed" });
            if let Err(error) = &result {
                report["error"] = serde_json::json!(error);
            }
            persist(&report)?;
            result?;
            return Ok(Box::new(CompletedAction { report }));
        }
        let DiagnosticExecution::Lua { source, .. } = execution else {
            unreachable!()
        };
        let environment = self
            .runner
            .diagnostic_environment(owner)
            .map_err(|e| e.to_string())?;
        let state = Rc::new(RefCell::new(ReportState::default()));
        let api = lua.create_table().map_err(|e| e.to_string())?;
        let attach = || -> mlua::Result<()> {
            let published = state.clone();
            let exports = request.exports.clone();
            api.set(
                "publish",
                lua.create_function(move |_, value: Value| {
                    let (report, text) = encoded_report(value)?;
                    exports
                        .write("report.json", &text)
                        .map_err(mlua::Error::runtime)?;
                    published.borrow_mut().report = report;
                    Ok(())
                })?,
            )?;
            api.set(
                "encode",
                lua.create_function(|_, value: Value| Ok(encoded(value)?.1))?,
            )?;
            let exports = request.exports.clone();
            api.set(
                "write_file",
                lua.create_function(move |_, (name, text): (String, String)| {
                    exports.write(&name, &text).map_err(mlua::Error::runtime)
                })?,
            )?;
            let exports = request.exports.clone();
            api.set(
                "read_file",
                lua.create_function(move |_, name: String| {
                    exports.read(&name).map_err(mlua::Error::runtime)
                })?,
            )?;
            let exports = request.exports.clone();
            api.set(
                "exists",
                lua.create_function(move |_, name: String| {
                    exports.exists(&name).map_err(mlua::Error::runtime)
                })?,
            )?;
            let finished = state.clone();
            api.set(
                "finish",
                lua.create_function(move |_, value: Option<Value>| {
                    let mut state = finished.borrow_mut();
                    if let Some(value) = value {
                        state.report = encoded_report(value)?.0;
                    }
                    state.finished = true;
                    Ok(())
                })?,
            )?;
            api.set(
                "call_mod",
                lua.create_function(
                    |lua, (mod_id, action, payload): (String, String, Option<String>)| {
                        let active = lua
                            .app_data_ref::<ActiveDiagnosticMods>()
                            .map(|mods| mods.0.clone())
                            .unwrap_or_default();
                        let result = crate::env::shroudforge::invoke_mod_action(
                            lua,
                            &active,
                            &mod_id,
                            &action,
                            payload.as_deref(),
                        );
                        Ok(match result {
                            Ok(()) => (true, None),
                            Err(error) => (false, Some(error)),
                        })
                    },
                )?,
            )?;
            environment.set("diagnostic", api)?;
            Ok(())
        };
        attach().map_err(|e| e.to_string())?;
        let callbacks: Table = lua
            .load(source)
            .set_name(&request.job.id)
            .set_environment(environment)
            .eval()
            .map_err(|e| e.to_string())?;
        let interval = callbacks
            .get::<Option<f64>>("update_interval_ms")
            .map_err(|e| e.to_string())?
            .unwrap_or(100.0);
        if !interval.is_finite() || !(16.0..=60_000.0).contains(&interval) {
            return Err("diagnostic update_interval_ms must be between 16 and 60000".into());
        }
        let job = LuaJob {
            lua: lua.clone(),
            callbacks: Some(callbacks),
            state,
            owner: owner.into(),
            interval: interval / 1000.0,
            accumulated: 0.0,
        };
        if let Some(load) = job
            .callbacks
            .as_ref()
            .unwrap()
            .get::<Option<Function>>("on_load")
            .map_err(|e| e.to_string())?
        {
            load.call::<()>(()).map_err(|e| e.to_string())?;
        }
        Ok(Box::new(job))
    }
}

fn start_system_dependency_job(
    request: JobExecutionRequest,
) -> Result<Box<dyn RunningJob>, String> {
    let capture = shroudforge_loader_config::paths::export_dir(&request.game_root)
        .join("diagnostics/capture.runtime");
    for name in ["functions.json", "components.json"] {
        if !capture.join(name).is_file() {
            return Err(format!(
                "run capture.runtime first. Missing {}",
                capture.join(name).display()
            ));
        }
    }
    let report = json!({"state":"starting","action":"inspect-system-dependencies"});
    request.exports.write("report.json", &report.to_string())?;
    let cancelled = Arc::new(AtomicBool::new(false));
    let thread_cancelled = cancelled.clone();
    let game_root = request.game_root;
    let (sender, receiver) = mpsc::channel::<Result<JsonValue, String>>();
    let worker = thread::Builder::new()
        .name("diagnostic-system-dependencies".into())
        .spawn(move || {
            let result = system_dependency_inspection::inspect_system_dependencies(
                Path::new(&game_root),
                &thread_cancelled,
            )
            .map_err(|error| error.to_string());
            let _ = sender.send(result);
        })
        .map_err(|error| format!("cannot start system dependency inspection: {error}"))?;
    let mut report = report;
    report["state"] = json!("running");
    if let Err(error) = request.exports.write("report.json", &report.to_string()) {
        cancelled.store(true, Ordering::Release);
        return Err(error);
    }
    Ok(Box::new(SystemDependencyJob {
        receiver: Some(receiver),
        worker: Some(worker),
        cancelled,
        exports: request.exports,
        report,
        finished: false,
    }))
}

struct SystemDependencyJob {
    receiver: Option<Receiver<Result<JsonValue, String>>>,
    worker: Option<JoinHandle<()>>,
    cancelled: Arc<AtomicBool>,
    exports: JobExports,
    report: JsonValue,
    finished: bool,
}

impl SystemDependencyJob {
    fn publish(&self) -> Result<(), String> {
        self.exports.write("report.json", &self.report.to_string())
    }

    fn output(&self) -> JobOutput {
        JobOutput {
            report: self.report.clone(),
            finished: self.finished,
        }
    }

    fn finish_worker(&mut self) {
        if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        } else {
            self.worker.take();
        }
    }
}

impl RunningJob for SystemDependencyJob {
    fn update(&mut self, _: f64) -> Result<JobOutput, String> {
        if self.finished {
            return Ok(self.output());
        }
        let received = match self.receiver.as_ref().map(Receiver::try_recv) {
            Some(Ok(result)) => Some(result),
            Some(Err(TryRecvError::Empty)) => None,
            Some(Err(TryRecvError::Disconnected)) => {
                return Err("system dependency inspection worker stopped unexpectedly".into());
            }
            None => None,
        };
        let Some(result) = received else {
            return Ok(self.output());
        };
        self.receiver.take();
        self.finish_worker();
        match result {
            Ok(inspection) => {
                if let Err(error) = self
                    .exports
                    .write("system-dependencies.json", &inspection.to_string())
                {
                    self.report = json!({"state":"failed","action":"inspect-system-dependencies","error":error});
                    self.finished = true;
                    let _ = self.publish();
                    return Err(error);
                }
                self.report = json!({"state":"complete","action":"inspect-system-dependencies",
                    "output":"system-dependencies.json","coverage":inspection["coverage"]});
                self.finished = true;
                self.publish()?;
                Ok(self.output())
            }
            Err(error) => {
                self.report =
                    json!({"state":"failed","action":"inspect-system-dependencies","error":error});
                self.finished = true;
                self.publish()?;
                Err(error)
            }
        }
    }

    fn stop(&mut self, reason: &str) -> Result<JobOutput, String> {
        if self.finished {
            return Ok(self.output());
        }
        self.cancelled.store(true, Ordering::Release);
        self.receiver.take();
        self.report =
            json!({"state":"stopped","action":"inspect-system-dependencies","reason":reason});
        self.finished = true;
        self.publish()?;
        self.finish_worker();
        Ok(self.output())
    }
}

impl Drop for SystemDependencyJob {
    fn drop(&mut self) {
        if !self.finished {
            if let Err(error) = self.stop("host-dropped") {
                tracing::warn!(target:"shroudforge::diagnostics", %error, "Native diagnostic job cleanup failed");
            }
        }
    }
}

struct CompletedAction {
    report: serde_json::Value,
}

impl RunningJob for CompletedAction {
    fn update(&mut self, _: f64) -> Result<JobOutput, String> {
        Ok(JobOutput {
            report: self.report.clone(),
            finished: true,
        })
    }

    fn stop(&mut self, _: &str) -> Result<JobOutput, String> {
        self.update(0.0)
    }
}

impl LuaJob {
    fn output(&self) -> JobOutput {
        let state = self.state.borrow();
        JobOutput {
            report: state.report.clone(),
            finished: state.finished,
        }
    }
}

impl RunningJob for LuaJob {
    fn update(&mut self, delta_seconds: f64) -> Result<JobOutput, String> {
        if self.state.borrow().finished {
            return Ok(self.output());
        }
        if !current_owner(&self.lua, &self.owner) {
            return Err(format!(
                "diagnostic API owner '{}' is no longer active",
                self.owner
            ));
        }
        if delta_seconds.is_finite() && delta_seconds >= 0.0 {
            self.accumulated += delta_seconds;
        }
        if self.accumulated >= self.interval {
            let delta = self.accumulated;
            self.accumulated = 0.0;
            if let Some(callback) = self
                .callbacks
                .as_ref()
                .unwrap()
                .get::<Option<Function>>("on_update")
                .map_err(|e| e.to_string())?
            {
                callback.call::<()>(delta).map_err(|e| e.to_string())?;
            }
        }
        Ok(self.output())
    }

    fn stop(&mut self, reason: &str) -> Result<JobOutput, String> {
        if let Some(callbacks) = self.callbacks.take() {
            if let Some(callback) = callbacks
                .get::<Option<Function>>("on_unload")
                .map_err(|e| e.to_string())?
            {
                callback.call::<()>(reason).map_err(|e| e.to_string())?;
            }
        }
        Ok(self.output())
    }
}

impl Drop for LuaJob {
    fn drop(&mut self) {
        if let Err(error) = self.stop("host-dropped") {
            tracing::warn!(target:"shroudforge::diagnostics", %error, "Diagnostic Lua job cleanup failed");
        }
    }
}
