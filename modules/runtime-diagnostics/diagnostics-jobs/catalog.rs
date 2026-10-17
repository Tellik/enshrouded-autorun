use super::definition::{DiagnosticJob, DiagnosticJobCatalog, parse};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

const MAX_JOB_FILES: usize = 128;
const MAX_JOB_FILE_BYTES: u64 = 64 * 1024;

fn builtin(id: &str, name: &str, description: &str, capabilities: &[&str]) -> DiagnosticJob {
    DiagnosticJob {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        capabilities: capabilities
            .iter()
            .map(|capability| (*capability).into())
            .collect(),
    }
}

fn builtins() -> Vec<DiagnosticJob> {
    vec![
        builtin(
            "runtime.overview",
            "Runtime overview",
            "Runtime health, queue activity, mod callbacks and operation timings.",
            &["runtime", "queue", "mods", "operations"],
        ),
        builtin(
            "runtime.health",
            "Runtime health",
            "Native runtime provider and world health snapshot.",
            &["runtime"],
        ),
        builtin(
            "runtime.queue",
            "Queue activity",
            "Dispatcher and runtime queue observations.",
            &["queue"],
        ),
        builtin(
            "runtime.callbacks",
            "Mod callbacks",
            "Lua mod callback duration, errors and slow calls.",
            &["mods"],
        ),
        builtin(
            "runtime.operations",
            "Runtime operation timings",
            "Timing samples for instrumented native and Lua operations.",
            &["operations"],
        ),
    ]
}

fn collect_json_files(directory: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!("{}: {error}", directory.display()));
            return Vec::new();
        }
    };
    let mut files = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("{}: {error}", directory.display()));
                continue;
            }
        };
        let path = entry.path();
        match entry.file_type() {
            Ok(kind)
                if kind.is_file()
                    && path
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("json")) =>
            {
                files.push(path)
            }
            Ok(_) => {}
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    files.sort();
    if files.len() > MAX_JOB_FILES {
        errors.push(format!(
            "job folder contains more than {MAX_JOB_FILES} JSON files"
        ));
        files.truncate(MAX_JOB_FILES);
    }
    files
}

pub fn discover(root: &Path, selected_id: Option<&str>) -> DiagnosticJobCatalog {
    let directory = shroudforge_loader_config::paths::diagnostic_jobs_dir(root);
    let mut errors = Vec::new();
    if let Err(error) = fs::create_dir_all(&directory) {
        errors.push(format!("{}: {error}", directory.display()));
    }

    let mut jobs = builtins();
    let files = collect_json_files(&directory, &mut errors);
    let mut ids = jobs
        .iter()
        .map(|job| job.id.clone())
        .collect::<HashSet<_>>();
    for path in files.iter().take(MAX_JOB_FILES) {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        if metadata.len() > MAX_JOB_FILE_BYTES {
            errors.push(format!("{}: file exceeds 64 KiB", path.display()));
            continue;
        }
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        match parse(&bytes) {
            Ok(job) if ids.insert(job.id.clone()) => jobs.push(job),
            Ok(job) => errors.push(format!("{}: duplicate job id {}", path.display(), job.id)),
            Err(error) => errors.push(format!("{}: {error}", path.display())),
        }
    }
    let selected_job_id = if let Some(selected_id) = selected_id {
        if !jobs.iter().any(|job| job.id == selected_id) {
            errors.push(format!(
                "selected diagnostic job is unavailable: {selected_id}"
            ));
        }
        selected_id.to_owned()
    } else {
        "runtime.overview".to_owned()
    };
    jobs.sort_by(|left, right| left.id.cmp(&right.id));
    DiagnosticJobCatalog {
        jobs,
        errors,
        directory: directory.display().to_string(),
        selected_job_id,
    }
}

pub fn resolve(root: &Path, settings: &Value, id: Option<&str>) -> Result<DiagnosticJob, String> {
    let requested = id
        .or_else(|| settings["jobId"].as_str())
        .unwrap_or("runtime.overview");
    let catalog = discover(root, Some(requested));
    catalog
        .jobs
        .into_iter()
        .find(|job| job.id == requested)
        .ok_or_else(|| format!("diagnostic job was not found: {requested}"))
}
