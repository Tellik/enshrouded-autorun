use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

const MAX_JOB_FILES: usize = 64;
const MAX_JOB_FILE_BYTES: u64 = 16 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JobDefinition {
    schema_version: u32,
    id: String,
    revision: u32,
    name: String,
    description: String,
    trigger: Trigger,
    repeatable: bool,
    action: Action,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Trigger {
    BeforeUpdate,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Action {
    MigrateLegacyBackups,
    MigrateLegacyUpdateFiles,
    MigrateLegacyMods,
    PurgeExpiredRecycleBin,
}

impl Action {
    fn is_migration(self) -> bool {
        matches!(
            self,
            Self::MigrateLegacyBackups | Self::MigrateLegacyUpdateFiles | Self::MigrateLegacyMods
        )
    }
}

const REQUIRED_MIGRATIONS: [Action; 3] = [
    Action::MigrateLegacyBackups,
    Action::MigrateLegacyUpdateFiles,
    Action::MigrateLegacyMods,
];

const CURRENT_MOD_IDS: &[&str] = &[
    "sf-crafting-essentials",
    "sf-survival",
    "sf-unlimited-flight",
    "sf-unlock-blueprints",
    "world-editor",
];

const LEGACY_SHROUDFORGE_MODS: &[(&str, &str)] = &[
    ("flight", "mod.flight"),
    ("unlock-blueprints", "mod.unlock-blueprints"),
    ("sf-world-editor", "sf-world-editor"),
    ("no-stamina-loss", "mod.no-stamina-loss"),
    ("sf-no-stamina-loss", "sf-no-stamina-loss"),
    ("sf-auto-stamina-refill", "sf-auto-stamina-refill"),
    ("infinite-item-split", "mod.infinite-item-split"),
    ("sf-infinite-item-split", "sf-infinite-item-split"),
    ("infinite-item-use", "mod.infinite-item-use"),
    ("sf-infinite-item-use", "sf-infinite-item-use"),
    ("no-fall-damage", "mod.no-fall-damage"),
    ("sf-no-fall-damage", "sf-no-fall-damage"),
    ("no-resource-cost", "mod.no-resource-cost"),
    ("sf-no-resource-cost", "sf-no-resource-cost"),
    ("sf-production-time", "sf-production-time"),
];

const DEFAULT_JOBS: &[(&str, &str)] = &[
    (
        "migrate-legacy-update-files.json",
        include_str!("migrate-legacy-update-files.json"),
    ),
    (
        "migrate-legacy-backups.json",
        include_str!("migrate-legacy-backups.json"),
    ),
    (
        "migrate-legacy-mods.json",
        include_str!("migrate-legacy-mods.json"),
    ),
    (
        "purge-expired-recycle-bin.json",
        include_str!("purge-expired-recycle-bin.json"),
    ),
];

struct JobCatalog {
    jobs: Vec<JobDefinition>,
    errors: Vec<String>,
}

pub(crate) fn enqueue_before_update(root: &Path) -> Result<(), String> {
    let catalog = discover(root);
    for error in catalog.errors {
        let _ = shroudforge_logging::append(root, 'W', "updater", &error);
    }
    if REQUIRED_MIGRATIONS.iter().any(|required| {
        !catalog
            .jobs
            .iter()
            .any(|job| job.trigger == Trigger::BeforeUpdate && job.action == *required)
    }) {
        return Err("required updater migration jobs are missing or invalid".into());
    }
    crate::update_queue_storage::update_update_queue(root, |queue| {
        if queue["run"]["state"] == "running" {
            return Err("the update queue is already running".into());
        }
        let items = queue["items"]
            .as_array_mut()
            .ok_or("update queue has an invalid items list")?;
        for job in catalog
            .jobs
            .into_iter()
            .filter(|job| job.trigger == Trigger::BeforeUpdate)
        {
            let id = format!("maintenance:{}:{}", job.id, job.revision);
            let existing_index = items.iter().position(|item| item["id"] == id);
            let Some(index) = existing_index else {
                items.push(queue_item(&id, job));
                continue;
            };
            if (items[index]["state"] == "complete" && job.repeatable)
                || matches!(items[index]["state"].as_str(), Some("error" | "cancelled"))
            {
                items[index] = queue_item(&id, job);
            }
        }
        Ok(())
    })
}

pub(crate) fn execute(root: &Path, item: &serde_json::Value) -> Result<(), String> {
    let action = action_from_item(item)?;
    match action {
        Action::MigrateLegacyBackups => {
            crate::recycle_bin::migrate_legacy_backups(root)?;
        }
        Action::MigrateLegacyUpdateFiles => {
            #[cfg(windows)]
            migrate_legacy_update_files(root)?;
        }
        Action::MigrateLegacyMods => migrate_legacy_mods(root)?,
        Action::PurgeExpiredRecycleBin => {
            if crate::recycle_bin::automatic_cleanup_enabled(root) {
                crate::recycle_bin::purge_expired(root)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn is_required_migration(item: &serde_json::Value) -> bool {
    action_from_item(item).is_ok_and(Action::is_migration)
}

fn action_from_item(item: &serde_json::Value) -> Result<Action, String> {
    let id = item["id"]
        .as_str()
        .ok_or("maintenance queue item is missing its ID")?;
    match item["operation"].as_str() {
        Some("MigrateLegacyBackups") => Ok(Action::MigrateLegacyBackups),
        Some("MigrateLegacyUpdateFiles") => Ok(Action::MigrateLegacyUpdateFiles),
        Some("MigrateLegacyMods") => Ok(Action::MigrateLegacyMods),
        Some("PurgeExpiredRecycleBin") => Ok(Action::PurgeExpiredRecycleBin),
        _ => Err(format!("unsupported maintenance job in queue item {id}")),
    }
}

fn migrate_legacy_mods(root: &Path) -> Result<(), String> {
    let mods = shroudforge_loader_config::paths::mods_dir(root);
    let mut moved = 0;
    for (folder, expected_id) in LEGACY_SHROUDFORGE_MODS {
        if CURRENT_MOD_IDS.iter().any(|current| {
            current.eq_ignore_ascii_case(folder) || current.eq_ignore_ascii_case(expected_id)
        }) {
            return Err(format!(
                "legacy mod migration list contains a current mod ID: {folder} ({expected_id})"
            ));
        }
        let source = mods.join(folder);
        let metadata = match fs::symlink_metadata(&source) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "cannot inspect legacy mod folder {}: {error}",
                    source.display()
                ));
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(format!(
                "legacy mod path is not a regular folder: {}",
                source.display()
            ));
        }
        let manifest_path = source.join("mod.json");
        let manifest_metadata = fs::symlink_metadata(&manifest_path).map_err(|error| {
            format!(
                "cannot inspect legacy mod manifest {}: {error}",
                manifest_path.display()
            )
        })?;
        if manifest_metadata.file_type().is_symlink() || !manifest_metadata.is_file() {
            return Err(format!(
                "legacy mod manifest is not a regular file: {}",
                manifest_path.display()
            ));
        }
        let manifest = read_json(&manifest_path)?;
        let actual_id = manifest["id"]
            .as_str()
            .ok_or_else(|| format!("legacy mod manifest has no ID: {}", manifest_path.display()))?;
        if CURRENT_MOD_IDS
            .iter()
            .any(|current| current.eq_ignore_ascii_case(actual_id))
        {
            return Err(format!(
                "legacy mod folder contains current mod ID {actual_id}: {}",
                source.display()
            ));
        }
        if actual_id != *expected_id {
            return Err(format!(
                "legacy mod ID mismatch in {}: expected {}, found {actual_id}",
                source.display(),
                expected_id
            ));
        }
        crate::recycle_bin::recycle_mod_directory(
            root,
            &source,
            expected_id,
            "Removed legacy ShroudForge mod",
        )?;
        shroudforge_logging::write_activity(
            root,
            "Updates",
            "Legacy mod migration",
            "Moved to recycle bin",
            Some(&format!("{folder} ({expected_id})")),
            "info",
        );
        moved += 1;
    }
    if moved == 0 {
        shroudforge_logging::write_activity(
            root,
            "Updates",
            "Legacy mod migration",
            "No legacy mods found",
            None,
            "info",
        );
    }
    Ok(())
}

#[cfg(windows)]
fn migrate_legacy_update_files(root: &Path) -> Result<usize, String> {
    use std::fs;

    let updates = shroudforge_loader_config::paths::updates_dir(root);
    let mut migrated = usize::from(remove_legacy_updater_task(root)?);
    let stage_request = updates.join("system-stage-request.json");
    let mut install_staged_update = false;
    if stage_request.is_file() {
        let request = normalize_legacy_stage_request(read_json(&stage_request)?)?;
        crate::system_stage::stage_system_update(root, &request)?;
        // Older stage requests created a worker-queue request after a successful
        // download. Complete that same install here because the new stager does
        // not create the legacy worker file.
        install_staged_update = true;
    }

    let mod_request = updates.join("mod-install-queue.json");
    if mod_request.is_file() {
        let request = read_json(&mod_request)?;
        let queue_item_id = request["queueItemId"]
            .as_str()
            .unwrap_or("migration.legacy-mod-install");
        let project_id = request["projectId"]
            .as_str()
            .ok_or("legacy mod install request has no projectId")?;
        let provider = serde_json::from_value(request["provider"].clone())
            .map_err(|error| format!("invalid legacy mod provider settings: {error}"))?;
        let update_mod_id = request["modId"].as_str();
        crate::mod_installation::install::install_mod_from_catalog(
            root,
            &provider,
            project_id,
            update_mod_id,
            queue_item_id,
        )?;
        fs::remove_file(&mod_request).map_err(|error| {
            format!("legacy mod request completed but could not be removed: {error}")
        })?;
        migrated += 1;
    }

    let worker_request = updates.join("worker-queue.json");
    if worker_request.is_file() {
        let request = read_json(&worker_request)?;
        if request["operation"] != "installPending" {
            return Err("unsupported legacy worker queue operation".into());
        }
        let pending = updates.join("pending");
        if pending.is_dir() {
            install_staged_update = true;
        }
    }

    if install_staged_update {
        let pending = updates.join("pending");
        if !pending.is_dir() {
            return Err("legacy system update request produced no staged package".into());
        }
        crate::system_install::install_staged(root, &pending)?;
    }

    if stage_request.is_file() {
        fs::remove_file(&stage_request).map_err(|error| {
            format!("legacy system stage request completed but could not be removed: {error}")
        })?;
        migrated += 1;
    }
    if worker_request.is_file() {
        fs::remove_file(&worker_request).map_err(|error| {
            format!("legacy worker request completed but could not be removed: {error}")
        })?;
        migrated += 1;
    }
    Ok(migrated)
}

fn normalize_legacy_stage_request(
    mut request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let version = request["version"]
        .as_str()
        .ok_or("legacy system stage request has no version")?;
    let (base_version, build) = crate::parse_release_tag(version)?;
    request["baseVersion"] = serde_json::Value::String(base_version);
    request["build"] = serde_json::json!(build);
    Ok(request)
}

#[cfg(windows)]
fn remove_legacy_updater_task(root: &Path) -> Result<bool, String> {
    use std::process::Command;

    let canonical = root.canonicalize().map_err(|error| error.to_string())?;
    let mut hash = 0xcbf29ce484222325u64;
    for byte in canonical.to_string_lossy().to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let name = format!("ShroudForgeUpdater_{hash:016x}");
    let query = Command::new("schtasks.exe")
        .args(["/Query", "/TN", &name])
        .output()
        .map_err(|error| format!("could not inspect legacy updater task: {error}"))?;
    if !query.status.success() {
        return Ok(false);
    }
    let deletion = Command::new("schtasks.exe")
        .args(["/Delete", "/TN", &name, "/F"])
        .output()
        .map_err(|error| format!("could not remove legacy updater task: {error}"))?;
    if !deletion.status.success() {
        return Err(format!(
            "could not remove legacy updater task {}: {}",
            name,
            String::from_utf8_lossy(&deletion.stderr).trim()
        ));
    }
    Ok(true)
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let bytes =
        std::fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid {}: {error}", path.display()))
}

fn queue_item(id: &str, job: JobDefinition) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "kind": "maintenance",
        "operation": format!("{:?}", job.action),
        "title": job.name,
        "version": job.revision,
        "state": "queued",
        "selected": true,
        "required": job.action.is_migration(),
        "message": job.description
    })
}

fn discover(root: &Path) -> JobCatalog {
    let directory = shroudforge_loader_config::paths::updater_jobs_dir(root);
    let mut errors = Vec::new();
    if let Err(error) = fs::create_dir_all(&directory) {
        errors.push(format!(
            "Cannot read maintenance jobs folder {}: {error}",
            directory.display()
        ));
        return JobCatalog {
            jobs: Vec::new(),
            errors,
        };
    }
    for (name, contents) in DEFAULT_JOBS {
        let path = directory.join(name);
        if path.exists() {
            continue;
        }
        if let Err(error) = fs::write(&path, contents) {
            errors.push(format!(
                "Cannot install default maintenance job {}: {error}",
                path.display()
            ));
        }
    }
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(format!(
                "Cannot read maintenance jobs folder {}: {error}",
                directory.display()
            ));
            return JobCatalog {
                jobs: Vec::new(),
                errors,
            };
        }
    };
    let mut files = entries
        .filter_map(|entry| match entry {
            Ok(entry)
                if entry.file_type().is_ok_and(|kind| kind.is_file())
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("json")) =>
            {
                Some(entry.path())
            }
            Ok(_) => None,
            Err(error) => {
                errors.push(format!("Cannot read maintenance job entry: {error}"));
                None
            }
        })
        .collect::<Vec<_>>();
    files.sort();
    if files.len() > MAX_JOB_FILES {
        errors.push(format!(
            "Maintenance jobs folder contains more than {MAX_JOB_FILES} JSON files"
        ));
        files.truncate(MAX_JOB_FILES);
    }
    let mut ids = HashSet::new();
    let mut jobs = Vec::new();
    for path in files {
        match fs::metadata(&path) {
            Ok(metadata) if metadata.len() <= MAX_JOB_FILE_BYTES => {}
            Ok(_) => {
                errors.push(format!(
                    "Maintenance job exceeds 16 KiB: {}",
                    path.display()
                ));
                continue;
            }
            Err(error) => {
                errors.push(format!(
                    "Cannot inspect maintenance job {}: {error}",
                    path.display()
                ));
                continue;
            }
        }
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                errors.push(format!(
                    "Cannot read maintenance job {}: {error}",
                    path.display()
                ));
                continue;
            }
        };
        let job: JobDefinition = match serde_json::from_slice(&bytes) {
            Ok(job) => job,
            Err(error) => {
                errors.push(format!(
                    "Invalid maintenance job {}: {error}",
                    path.display()
                ));
                continue;
            }
        };
        if let Err(error) = validate(&job) {
            errors.push(format!(
                "Invalid maintenance job {}: {error}",
                path.display()
            ));
            continue;
        }
        if !ids.insert(job.id.clone()) {
            errors.push(format!("Duplicate maintenance job ID: {}", job.id));
            continue;
        }
        jobs.push(job);
    }
    JobCatalog { jobs, errors }
}

fn validate(job: &JobDefinition) -> Result<(), String> {
    if job.schema_version != 1 {
        return Err("schemaVersion must be 1".into());
    }
    if !(job.id.starts_with("maintenance.") || job.id.starts_with("migration."))
        || job.id.len() > 100
        || !job.id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
    {
        return Err("id must use the maintenance. or migration. namespace".into());
    }
    if job.revision == 0 {
        return Err("revision must be greater than zero".into());
    }
    if job.name.trim().is_empty() || job.name.chars().count() > 100 {
        return Err("name must contain 1 to 100 characters".into());
    }
    if job.description.chars().count() > 500 {
        return Err("description must contain at most 500 characters".into());
    }
    Ok(())
}
