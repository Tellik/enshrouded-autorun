mod read_live_character_selection;

use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DETECTION_CACHE_DURATION: Duration = Duration::from_secs(5);

static DETECTION_CACHE: OnceLock<Mutex<HashMap<PathBuf, (Instant, DetectionReport)>>> =
    OnceLock::new();
static SAVED_NAME_CACHE: OnceLock<Mutex<HashMap<PathBuf, SavedNameEntry>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectionStatus {
    Detected,
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
            Self::Detected => "The selected character name was read from the live game UI",
            Self::GameNotRunning => "The Enshrouded client is not running",
            Self::MultipleGameProcesses => {
                "More than one Enshrouded client is running, so the UI source is ambiguous"
            }
            Self::SelectionScanInProgress => {
                "The live game UI is being scanned for the selected character"
            }
            Self::SelectionNotFound => {
                "No character UI record matching the supported text pattern was found"
            }
            Self::ProcessMemoryUnreadable => "The Enshrouded process memory could not be read",
            Self::UnsupportedPlatform => "Live character selection is supported only on Windows",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameSource {
    Live,
    Saved,
    Fallback,
}

impl NameSource {
    pub fn key(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Saved => "saved",
            Self::Fallback => "fallback",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Live => "Live character name detected",
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
                source: NameSource::Live,
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
    let mut report = read_live_with_short_cache(root);
    let mut saved = load_saved_name(root);

    if let Some(name) = report.name.as_deref() {
        saved = persist_live_name(root, name, saved);
    }

    report.saved_name = saved.name;
    report.cache_error = saved.error;
    report
}

fn read_live_with_short_cache(root: &Path) -> DetectionReport {
    let key = root.to_path_buf();
    let cache = DETECTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        if let Some((sampled_at, report)) = cache.get(&key)
            && sampled_at.elapsed() < DETECTION_CACHE_DURATION
        {
            return report.clone();
        }

        let report = read_live_character_selection::read_live_character_selection();
        cache.insert(key, (Instant::now(), report.clone()));
        return report;
    }

    read_live_character_selection::read_live_character_selection()
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
