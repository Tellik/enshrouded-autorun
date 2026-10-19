use crate::job_definition::{
    DiagnosticJob, DiagnosticJobCatalog, MAX_JOB_FILE_BYTES, MAX_SCRIPT_BYTES, parse,
};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_JOB_FILES: usize = 128;
include!(concat!(env!("OUT_DIR"), "/diagnostic_jobs.rs"));

pub(super) fn bundled_files() -> &'static [(&'static str, &'static str)] {
    BUNDLED_FILES
}

pub(super) fn builtins() -> Result<Vec<DiagnosticJob>, String> {
    let mut jobs = Vec::new();
    let mut ids = HashSet::new();
    for (path, contents) in BUNDLED_FILES
        .iter()
        .filter(|(path, _)| path.ends_with(".json"))
    {
        let job = parse(contents.as_bytes(), |script| {
            BUNDLED_FILES
                .iter()
                .find(|(path, _)| *path == script)
                .map(|(_, source)| (*source).to_owned())
                .ok_or_else(|| format!("bundled diagnostic script was not found: {script}"))
        })
        .map_err(|error| format!("{path}: {error}"))?;
        if !ids.insert(job.id.clone()) {
            return Err(format!("{path}: duplicate bundled job id {}", job.id));
        }
        jobs.push(job);
    }
    Ok(jobs)
}

fn read_script(directory: &Path, script: &str) -> Result<String, String> {
    let directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let path = directory
        .join(script)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !path.starts_with(&directory) {
        return Err("diagnostic script resolves outside the jobs folder".into());
    }
    let file = fs::File::open(path).map_err(|error| error.to_string())?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_SCRIPT_BYTES as u64 {
        return Err("diagnostic script must be a regular file within the 1 MiB limit".into());
    }
    let mut source = String::new();
    file.take(MAX_SCRIPT_BYTES as u64 + 1)
        .read_to_string(&mut source)
        .map_err(|error| error.to_string())?;
    if source.len() > MAX_SCRIPT_BYTES {
        return Err("diagnostic script exceeds 1 MiB".into());
    }
    Ok(source)
}

fn collect_json_files(directory: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![directory.to_owned()];
    while let Some(path) = pending.pop() {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                errors.push(format!("{}: {error}", path.display()));
                continue;
            }
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            let entries = match fs::read_dir(&path) {
                Ok(entries) => entries,
                Err(error) => {
                    errors.push(format!("{}: {error}", path.display()));
                    continue;
                }
            };
            let mut children = Vec::new();
            for entry in entries {
                match entry {
                    Ok(entry) => children.push(entry.path()),
                    Err(error) => errors.push(format!("{}: {error}", path.display())),
                }
            }
            children.sort();
            pending.extend(children.into_iter().rev());
        } else if metadata.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            files.push(path);
            if files.len() > MAX_JOB_FILES {
                break;
            }
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

    let mut jobs = match builtins() {
        Ok(jobs) => jobs,
        Err(error) => {
            errors.push(error);
            Vec::new()
        }
    };
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
        if metadata.len() > MAX_JOB_FILE_BYTES as u64 {
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
        match parse(&bytes, |script| read_script(&directory, script)) {
            Ok(job) if !job.id.starts_with("user.") || job.id.len() <= 5 => errors.push(format!(
                "{}: custom diagnostic job id must start with user. and include a name",
                path.display()
            )),
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

pub fn catalog(root: &Path, settings: &Value) -> DiagnosticJobCatalog {
    discover(root, settings["jobId"].as_str())
}
