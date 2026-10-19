//! Python adapter for the same diagnostic job lifecycle used by Lua.
use crate::{DiagnosticExecution, JobExecutionRequest, JobExports, JobOutput, RunningJob};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
};

pub(crate) fn start(
    root: &Path,
    request: JobExecutionRequest,
) -> Result<Box<dyn RunningJob>, String> {
    let Some(DiagnosticExecution::Python {
        script,
        args,
        interpreter,
        source,
    }) = request.job.execution
    else {
        return Err("diagnostic job does not declare Python execution".into());
    };
    fs::create_dir_all(request.exports.directory()).map_err(|error| error.to_string())?;
    let cache = shroudforge_loader_config::paths::cache_dir(root).join("diagnostics");
    fs::create_dir_all(&cache).map_err(|error| error.to_string())?;
    let files = tempfile::Builder::new()
        .prefix("python-")
        .tempdir_in(cache)
        .map_err(|error| error.to_string())?;
    for (name, source) in crate::job_catalog::bundled_files()
        .iter()
        .filter(|(name, _)| name.starts_with("game_inspection/"))
    {
        let path = files.path().join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
        fs::write(path, source).map_err(|error| error.to_string())?;
    }
    let path = files.path().join(&script);
    fs::create_dir_all(path.parent().unwrap()).map_err(|error| error.to_string())?;
    fs::write(&path, source).map_err(|error| error.to_string())?;
    let jobs = shroudforge_loader_config::paths::diagnostic_jobs_dir(root);
    let diagnostic_exports = shroudforge_loader_config::paths::export_dir(root).join("diagnostics");
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let game_exe = if executable
        .file_name()
        .is_some_and(|name| name == "enshrouded.exe" || name == "enshrouded_server.exe")
    {
        executable
    } else if root.join("enshrouded_server.exe").is_file() && !root.join("enshrouded.exe").is_file()
    {
        root.join("enshrouded_server.exe")
    } else {
        root.join("enshrouded.exe")
    };
    let substitutions = [
        ("{pid}", std::process::id().to_string()),
        ("{gameExe}", game_exe.display().to_string()),
        ("{gameRoot}", root.display().to_string()),
        ("{jobs}", jobs.display().to_string()),
        (
            "{diagnosticExports}",
            diagnostic_exports.display().to_string(),
        ),
        (
            "{exports}",
            request.exports.directory().display().to_string(),
        ),
    ];
    let args: Vec<String> = args
        .into_iter()
        .map(|mut arg| {
            for (name, value) in &substitutions {
                arg = arg.replace(name, value);
            }
            arg
        })
        .collect();
    let interpreter = interpreter
        .or_else(|| std::env::var("SHROUDFORGE_DIAGNOSTICS_PYTHON").ok())
        .unwrap_or_else(|| "python".into());
    let mut python_paths = vec![files.path().to_owned(), jobs];
    if let Some(existing) = std::env::var_os("PYTHONPATH") {
        python_paths.extend(std::env::split_paths(&existing));
    }
    let mut command = Command::new(&interpreter);
    command
        .arg(path)
        .args(args)
        .current_dir(files.path())
        .env(
            "PYTHONPATH",
            std::env::join_paths(python_paths).map_err(|error| error.to_string())?,
        )
        .stdin(Stdio::null())
        .stdout(
            fs::File::create(request.exports.directory().join("stdout.log"))
                .map_err(|error| error.to_string())?,
        )
        .stderr(
            fs::File::create(request.exports.directory().join("stderr.log"))
                .map_err(|error| error.to_string())?,
        );
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut job = PythonJob {
        child: None,
        _files: files,
        exports: request.exports,
        report: json!({"state":"starting","script":script,"stdout":"stdout.log","stderr":"stderr.log"}),
    };
    job.publish()?;
    job.child = Some(command.spawn().map_err(|error| {
        format!("cannot start diagnostic Python interpreter '{interpreter}': {error}")
    })?);
    job.report["pid"] = json!(job.child.as_ref().unwrap().id());
    job.report["state"] = json!("running");
    job.publish()?;
    Ok(Box::new(job))
}

struct PythonJob {
    child: Option<Child>,
    _files: tempfile::TempDir,
    exports: JobExports,
    report: Value,
}

impl PythonJob {
    fn publish(&self) -> Result<(), String> {
        self.exports.write("report.json", &self.report.to_string())
    }
    fn output(&self) -> JobOutput {
        JobOutput {
            report: self.report.clone(),
            finished: self.child.is_none(),
        }
    }
}

impl RunningJob for PythonJob {
    fn update(&mut self, _: f64) -> Result<JobOutput, String> {
        if let Some(child) = &mut self.child {
            if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                self.child.take();
                self.report["exitCode"] = json!(status.code());
                self.report["state"] = json!(if status.success() {
                    "complete"
                } else {
                    "failed"
                });
                self.publish()?;
                if !status.success() {
                    return Err(format!(
                        "diagnostic Python exited with {status}, see stderr.log"
                    ));
                }
            }
        }
        Ok(self.output())
    }

    fn stop(&mut self, reason: &str) -> Result<JobOutput, String> {
        if let Some(child) = self.child.as_mut() {
            if child.try_wait().ok().flatten().is_none() {
                if let Err(error) = child.kill() {
                    if child
                        .try_wait()
                        .map_err(|error| error.to_string())?
                        .is_none()
                    {
                        return Err(error.to_string());
                    }
                }
            }
            let status = child.wait().map_err(|error| error.to_string())?;
            self.child.take();
            self.report["exitCode"] = json!(status.code());
            self.report["state"] = json!("stopped");
            self.report["reason"] = json!(reason);
            self.publish()?;
        }
        Ok(self.output())
    }
}

impl Drop for PythonJob {
    fn drop(&mut self) {
        if let Err(error) = self.stop("process-stopped") {
            tracing::warn!(target:"shroudforge::diagnostics", %error, "Diagnostic Python cleanup failed");
        }
    }
}
