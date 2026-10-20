mod read_game_login_player_name;
mod read_live_character_selection;

use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DETECTION_CACHE_DURATION: Duration = Duration::from_secs(5);

static DETECTION_CACHE: OnceLock<
    Mutex<HashMap<(PathBuf, Option<u32>), (Instant, DetectionReport)>>,
> = OnceLock::new();
static SAVED_NAME_CACHE: OnceLock<Mutex<HashMap<PathBuf, SavedNameEntry>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectionStatus {
    Detected,
    DetectedFromGameLog,
    GameNotRunning,
    MultipleGameProcesses,
    SelectionScanInProgress,
    SelectionNotFound,
    ProcessMemoryUnreadable,
    UnsupportedPlatform,
}

impl DetectionStatus {
    pub fn description(self) -> &'static str {
        match self {
            Self::Detected => "The selected character name was read from live game UI memory",
            Self::DetectedFromGameLog => {
                "The player name was read from the live Enshrouded login log"
            }
            Self::GameNotRunning => "The Enshrouded client is not running",
            Self::MultipleGameProcesses => {
                "More than one Enshrouded client is running, so the UI source is ambiguous"
            }
            Self::SelectionScanInProgress => {
                "The live game UI is being scanned for the selected character"
            }
            Self::SelectionNotFound => {
                "No player login name or supported character UI record was found"
            }
            Self::ProcessMemoryUnreadable => "The Enshrouded process memory could not be read",
            Self::UnsupportedPlatform => "Live character selection is supported only on Windows",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameSource {
    Live,
    GameLog,
    Saved,
    Fallback,
}

impl NameSource {
    pub fn key(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::GameLog => "live",
            Self::Saved => "saved",
            Self::Fallback => "fallback",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Live => "Live character name detected from game UI memory",
            Self::GameLog => "Live player name detected from Enshrouded login log",
            Self::Saved => "Previously detected character name used",
            Self::Fallback => "Configured fallback used",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedName {
    pub name: String,
    pub source: NameSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionReport {
    /// Name found in the current live game UI scan.
    pub name: Option<String>,
    /// Last name persisted after a successful live detection.
    pub saved_name: Option<String>,
    pub status: DetectionStatus,
    pub process_id: Option<u32>,
    /// Persistence errors do not hide a live or previously saved name.
    pub cache_error: Option<String>,
}

impl DetectionReport {
    pub fn resolve_name(&self, manual_mode: bool, fallback: &str) -> ResolvedName {
        if manual_mode {
            return ResolvedName {
                name: fallback.to_owned(),
                source: NameSource::Fallback,
            };
        }
        if let Some(name) = &self.name {
            return ResolvedName {
                name: name.clone(),
                source: if self.status == DetectionStatus::DetectedFromGameLog {
                    NameSource::GameLog
                } else {
                    NameSource::Live
                },
            };
        }
        if let Some(name) = &self.saved_name {
            return ResolvedName {
                name: name.clone(),
                source: NameSource::Saved,
            };
        }
        ResolvedName {
            name: fallback.to_owned(),
            source: NameSource::Fallback,
        }
    }
}

#[derive(Clone, Default)]
struct SavedNameEntry {
    loaded: bool,
    name: Option<String>,
    persisted_name: Option<String>,
    error: Option<String>,
}

pub fn detect_active_character(root: &Path) -> DetectionReport {
    detect_active_character_for_process(root, None)
}

pub fn detect_active_character_for_process(
    root: &Path,
    game_process_id: Option<u32>,
) -> DetectionReport {
    let mut report = read_live_with_short_cache(root, game_process_id);
    let mut saved = load_saved_name(root);

    if report.status == DetectionStatus::DetectedFromGameLog
        && let (Some(live_name), Some(saved_name)) = (report.name.as_deref(), saved.name.as_deref())
        && live_name.eq_ignore_ascii_case(saved_name)
    {
        report.name = Some(saved_name.to_owned());
    }

    if let Some(name) = report.name.as_deref() {
        saved = persist_live_name(root, name, saved);
    }

    report.saved_name = saved.name;
    report.cache_error = saved.error;
    report
}

fn read_live_with_short_cache(root: &Path, game_process_id: Option<u32>) -> DetectionReport {
    let key = (root.to_path_buf(), game_process_id);
    let cache = DETECTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        if let Some((sampled_at, report)) = cache.get(&key)
            && sampled_at.elapsed() < DETECTION_CACHE_DURATION
        {
            return report.clone();
        }

        let report = read_live(root, game_process_id);
        cache.insert(key, (Instant::now(), report.clone()));
        return report;
    }

    read_live(root, game_process_id)
}

fn read_live(root: &Path, game_process_id: Option<u32>) -> DetectionReport {
    if let Some(process_id) = game_process_id
        && let Some(name) = read_game_login_player_name::read_active_player_name(root)
    {
        return report_from_game_log(name, process_id);
    }

    let report = read_live_character_selection::read_live_character_selection();
    if report.name.is_some() {
        return report;
    }

    let process_id = game_process_id.or(report.process_id);
    if let Some(process_id) = process_id
        && let Some(name) = read_game_login_player_name::read_active_player_name(root)
    {
        return report_from_game_log(name, process_id);
    }

    report
}

fn report_from_game_log(name: String, process_id: u32) -> DetectionReport {
    DetectionReport {
        name: Some(name),
        saved_name: None,
        status: DetectionStatus::DetectedFromGameLog,
        process_id: Some(process_id),
        cache_error: None,
    }
}

fn load_saved_name(root: &Path) -> SavedNameEntry {
    let key = root.to_path_buf();
    let cache = SAVED_NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut cache) = cache.lock() else {
        return SavedNameEntry {
            error: Some("The in-memory player-name cache is unavailable".into()),
            ..SavedNameEntry::default()
        };
    };
    let entry = cache.entry(key).or_default();
    if !entry.loaded {
        match shroudforge_loader_config::config::read_state_section(root, "playerName") {
            Ok(section) => {
                let name = section.as_ref().and_then(cached_name_from_section);
                entry.name = name.clone();
                entry.persisted_name = name;
                entry.error = None;
                entry.loaded = true;
            }
            Err(error) => entry.error = Some(error),
        }
    }
    entry.clone()
}

fn persist_live_name(root: &Path, name: &str, mut saved: SavedNameEntry) -> SavedNameEntry {
    if saved.persisted_name.as_deref() == Some(name) {
        saved.name = Some(name.to_owned());
        saved.error = None;
        saved.loaded = true;
        update_saved_name_cache(root, &saved);
        return saved;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let section = json!({
        "lastDetectedName": name,
        "cachedAtUnixSeconds": timestamp,
    });
    match shroudforge_loader_config::config::update_state_section(root, "playerName", |_| {
        Ok(section)
    }) {
        Ok(()) => {
            saved.name = Some(name.to_owned());
            saved.persisted_name = Some(name.to_owned());
            saved.error = None;
            saved.loaded = true;
        }
        Err(error) => {
            saved.error = Some(error);
            saved.loaded = true;
        }
    }
    update_saved_name_cache(root, &saved);
    saved
}

fn update_saved_name_cache(root: &Path, entry: &SavedNameEntry) {
    let key = root.to_path_buf();
    let cache = SAVED_NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        cache.insert(key, entry.clone());
    }
}

fn cached_name_from_section(section: &Value) -> Option<String> {
    section
        .get("lastDetectedName")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{
        DetectionReport, DetectionStatus, NameSource, cached_name_from_section, load_saved_name,
        persist_live_name,
    };

    fn report(name: Option<&str>, saved_name: Option<&str>) -> DetectionReport {
        DetectionReport {
            name: name.map(str::to_owned),
            saved_name: saved_name.map(str::to_owned),
            status: if name.is_some() {
                DetectionStatus::Detected
            } else {
                DetectionStatus::SelectionNotFound
            },
            process_id: None,
            cache_error: None,
        }
    }

    #[test]
    fn live_name_has_precedence_over_saved_name() {
        let resolved =
            report(Some("BonsaiBauer"), Some("OldName")).resolve_name(false, "Flameborn");
        assert_eq!(resolved.name, "BonsaiBauer");
        assert_eq!(resolved.source, NameSource::Live);
    }

    #[test]
    fn game_log_name_is_reported_as_a_live_source() {
        let detected = DetectionReport {
            name: Some("bonsaibauer".into()),
            saved_name: None,
            status: DetectionStatus::DetectedFromGameLog,
            process_id: Some(31280),
            cache_error: None,
        };
        let resolved = detected.resolve_name(false, "Flameborn");
        assert_eq!(resolved.name, "bonsaibauer");
        assert_eq!(resolved.source, NameSource::GameLog);
        assert_eq!(resolved.source.key(), "live");
    }

    #[test]
    fn saved_name_is_used_when_live_detection_is_unavailable() {
        let resolved = report(None, Some("BonsaiBauer")).resolve_name(false, "Flameborn");
        assert_eq!(resolved.name, "BonsaiBauer");
        assert_eq!(resolved.source, NameSource::Saved);
    }

    #[test]
    fn fallback_is_used_until_a_name_has_been_detected() {
        let resolved = report(None, None).resolve_name(false, "Flameborn");
        assert_eq!(resolved.name, "Flameborn");
        assert_eq!(resolved.source, NameSource::Fallback);
    }

    #[test]
    fn manual_mode_always_uses_fallback() {
        let resolved = report(Some("BonsaiBauer"), Some("OldName")).resolve_name(true, "Flameborn");
        assert_eq!(resolved.name, "Flameborn");
        assert_eq!(resolved.source, NameSource::Fallback);
    }

    #[test]
    fn live_name_is_persisted_in_the_installation_state() {
        let installation = tempfile::tempdir().unwrap();
        let saved = persist_live_name(
            installation.path(),
            "BonsaiBauer",
            load_saved_name(installation.path()),
        );
        assert_eq!(saved.name.as_deref(), Some("BonsaiBauer"));
        assert_eq!(saved.persisted_name.as_deref(), Some("BonsaiBauer"));
        assert!(saved.error.is_none());

        let section = shroudforge_loader_config::config::read_state_section(
            installation.path(),
            "playerName",
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            cached_name_from_section(&section).as_deref(),
            Some("BonsaiBauer")
        );
        assert!(section["cachedAtUnixSeconds"].as_u64().is_some());

        let reopened_root = installation.path().join(".");
        let reloaded = load_saved_name(&reopened_root);
        assert_eq!(reloaded.name.as_deref(), Some("BonsaiBauer"));
    }
}
