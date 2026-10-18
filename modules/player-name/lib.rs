mod read_live_character_selection;

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const DETECTION_CACHE_DURATION: Duration = Duration::from_secs(5);

static DETECTION_CACHE: OnceLock<Mutex<Option<(Instant, DetectionReport)>>> = OnceLock::new();

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
                "No character-selection UI record matching the observed format was found"
            }
            Self::ProcessMemoryUnreadable => "The Enshrouded process memory could not be read",
            Self::UnsupportedPlatform => "Live character selection is supported only on Windows",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionReport {
    pub name: Option<String>,
    pub status: DetectionStatus,
    pub process_id: Option<u32>,
}

pub fn detect_active_character() -> DetectionReport {
    let cache = DETECTION_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut cache) = cache.lock() {
        if let Some((sampled_at, report)) = cache.as_ref()
            && sampled_at.elapsed() < DETECTION_CACHE_DURATION
        {
            return report.clone();
        }

        let report = read_live_character_selection::read_live_character_selection();
        *cache = Some((Instant::now(), report.clone()));
        return report;
    }

    read_live_character_selection::read_live_character_selection()
}
