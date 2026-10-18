use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const KSC_HEADER_SIZE: usize = 24;
const KSC_ENTRY_SIZE: usize = 12;
const MAX_KSC_ENTRIES: usize = 4096;
const MAX_CHARACTER_BLOB_SIZE: usize = 16 * 1024 * 1024;
const DETECTION_CACHE_DURATION: Duration = Duration::from_secs(5);
const CHARACTER_INDEX_FILE_NAME: &str = "characters-index";

static DETECTION_CACHE: OnceLock<Mutex<Option<(Instant, DetectionReport)>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectionStatus {
    Detected,
    NoCharacterIndex,
    IndexUnreadable,
    InvalidCharacterIndex,
    DeletedCharacterSave,
    UnsupportedCharacterSlot,
    CharacterSaveUnavailable,
    CharacterSaveChanged,
    InvalidCharacterSave,
    NoCharacterFound,
    AmbiguousCharacter,
}

impl DetectionStatus {
    pub fn description(self) -> &'static str {
        match self {
            Self::Detected => "A character name was read from a save file",
            Self::NoCharacterIndex => {
                "No characters-index file was found under the known save folders"
            }
            Self::IndexUnreadable => "A characters-index file could not be read",
            Self::InvalidCharacterIndex => "A characters-index file has an invalid format",
            Self::DeletedCharacterSave => "The characters-index marks the current save as deleted",
            Self::UnsupportedCharacterSlot => {
                "The characters-index points to an unsupported save slot"
            }
            Self::CharacterSaveUnavailable => {
                "The save file named by characters-index could not be read"
            }
            Self::CharacterSaveChanged => "The index or save file changed while it was being read",
            Self::InvalidCharacterSave => {
                "The current save file has an unsupported or invalid format"
            }
            Self::NoCharacterFound => "The current save contains no readable character name",
            Self::AmbiguousCharacter => {
                "The current save data does not identify one unambiguous character"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectionReport {
    pub name: Option<String>,
    pub status: DetectionStatus,
    pub directories_scanned: usize,
    pub indexes_found: usize,
    pub inaccessible_directories: usize,
}

#[derive(Debug)]
struct CharacterName {
    name: String,
    last_play_time: Option<u32>,
}

#[derive(Default)]
struct IndexSearch {
    paths: Vec<PathBuf>,
    directories_scanned: usize,
    inaccessible_directories: usize,
}

pub fn detect_active_character() -> DetectionReport {
    let cache = DETECTION_CACHE.get_or_init(|| Mutex::new(None));
    let Ok(mut cache) = cache.lock() else {
        return detect_active_character_from_saves();
    };
    if let Some((sampled_at, report)) = cache.as_ref()
        && sampled_at.elapsed() < DETECTION_CACHE_DURATION
    {
        return report.clone();
    }

    let report = detect_active_character_from_saves();
    *cache = Some((Instant::now(), report.clone()));
    report
}

fn detect_active_character_from_saves() -> DetectionReport {
    let search = find_character_indexes(character_save_roots());
    let indexes_found = search.paths.len();
    if indexes_found == 0 {
        return DetectionReport {
            name: None,
            status: DetectionStatus::NoCharacterIndex,
            directories_scanned: search.directories_scanned,
            indexes_found,
            inaccessible_directories: search.inaccessible_directories,
        };
    }

    let mut candidates = Vec::new();
    let mut first_failure = None;
    for index_path in search.paths {
        match read_latest_character(&index_path) {
            Ok(character) => candidates.push(character),
            Err(status) if first_failure.is_none() => first_failure = Some(status),
            Err(_) => {}
        }
    }

    let selection = if candidates.is_empty() {
        Err(first_failure.unwrap_or(DetectionStatus::InvalidCharacterIndex))
    } else {
        select_most_recent_character(candidates)
    };
    let (name, status) = match selection {
        Ok(character) => (Some(character.name), DetectionStatus::Detected),
        Err(status) => (None, status),
    };

    DetectionReport {
        name,
        status,
        directories_scanned: search.directories_scanned,
        indexes_found,
        inaccessible_directories: search.inaccessible_directories,
    }
}

fn character_save_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        roots.push(PathBuf::from(profile).join("Saved Games/Enshrouded"));
    }

    let mut steam_roots = Vec::new();
    for variable in ["PROGRAMFILES(X86)", "PROGRAMFILES"] {
        if let Some(value) = std::env::var_os(variable) {
            steam_roots.push(PathBuf::from(value).join("Steam"));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        steam_roots.push(PathBuf::from(local).join("Programs/Steam"));
    }
    if let Some(steam) = std::env::var_os("STEAM_COMPAT_CLIENT_INSTALL_PATH") {
        steam_roots.push(PathBuf::from(steam));
    }
    steam_roots.sort();
    steam_roots.dedup();

    for steam in steam_roots {
        let Ok(accounts) = fs::read_dir(steam.join("userdata")) else {
            continue;
        };
        for account in accounts.flatten() {
            if account
                .file_type()
                .is_ok_and(|file_type| file_type.is_dir())
            {
                roots.push(account.path().join("1203620/remote"));
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

fn find_character_indexes(roots: Vec<PathBuf>) -> IndexSearch {
    let mut search = IndexSearch::default();
    let mut pending = roots;
    let mut visited = HashSet::new();

    while let Some(directory) = pending.pop() {
        if !visited.insert(directory.clone()) {
            continue;
        }
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => {
                search.inaccessible_directories += 1;
                continue;
            }
        };
        search.directories_scanned += 1;

        for entry in entries {
            let Ok(entry) = entry else {
                search.inaccessible_directories += 1;
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                search.inaccessible_directories += 1;
                continue;
            };
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() && entry.file_name() == CHARACTER_INDEX_FILE_NAME {
                search.paths.push(entry.path());
            }
        }
    }

    search.paths.sort();
    search.paths.dedup();
    search
}

fn read_latest_character(index_path: &Path) -> Result<CharacterName, DetectionStatus> {
    let index_before = fs::read(index_path).map_err(|_| DetectionStatus::IndexUnreadable)?;
    let index: serde_json::Value = serde_json::from_slice(&index_before)
        .map_err(|_| DetectionStatus::InvalidCharacterIndex)?;
    let latest = index
        .get("latest")
        .and_then(serde_json::Value::as_u64)
        .ok_or(DetectionStatus::InvalidCharacterIndex)?;
    if latest > 9 {
        return Err(DetectionStatus::UnsupportedCharacterSlot);
    }
    if index.get("deleted").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(DetectionStatus::DeletedCharacterSave);
    }

    let directory = index_path
        .parent()
        .ok_or(DetectionStatus::InvalidCharacterIndex)?;
    let save_path = if latest == 0 {
        directory.join("characters")
    } else {
        directory.join(format!("characters-{latest}"))
    };
    let save_before =
        fs::metadata(&save_path).map_err(|_| DetectionStatus::CharacterSaveUnavailable)?;
    let bytes = fs::read(&save_path).map_err(|_| DetectionStatus::CharacterSaveUnavailable)?;
    let save_after =
        fs::metadata(&save_path).map_err(|_| DetectionStatus::CharacterSaveUnavailable)?;
    let index_after = fs::read(index_path).map_err(|_| DetectionStatus::IndexUnreadable)?;
    if index_before != index_after
        || save_before.len() != save_after.len()
        || save_after.len() != bytes.len() as u64
        || save_before.modified().ok() != save_after.modified().ok()
    {
        return Err(DetectionStatus::CharacterSaveChanged);
    }

    let characters = parse_ksc1_characters(&bytes).ok_or(DetectionStatus::InvalidCharacterSave)?;
    if characters.is_empty() {
        return Err(DetectionStatus::NoCharacterFound);
    }
    select_most_recent_character(characters)
}

fn parse_ksc1_characters(bytes: &[u8]) -> Option<Vec<CharacterName>> {
    if bytes.len() < KSC_HEADER_SIZE || &bytes[..4] != b"KSC1" {
        return None;
    }
    let entry_count = read_u32(bytes, 4)? as usize;
    if entry_count == 0 || entry_count > MAX_KSC_ENTRIES {
        return None;
    }
    let table_end = KSC_HEADER_SIZE.checked_add(entry_count.checked_mul(KSC_ENTRY_SIZE)?)?;
    if table_end > bytes.len() {
        return None;
    }

    let mut data_offset = table_end;
    let mut characters = Vec::new();
    for index in 0..entry_count {
        let entry_offset = KSC_HEADER_SIZE + index * KSC_ENTRY_SIZE;
        let owner_id = read_u32(bytes, entry_offset)?;
        let blob_type = bytes.get(entry_offset + 4..entry_offset + 8)?;
        let compressed_size = read_u32(bytes, entry_offset + 8)? as usize;
        let data_end = data_offset.checked_add(compressed_size)?;
        let compressed = bytes.get(data_offset..data_end)?;
        data_offset = data_end;

        if blob_type != b"CHAR" || owner_id == 0 || compressed.is_empty() {
            continue;
        }
        let raw = zstd::bulk::decompress(compressed, MAX_CHARACTER_BLOB_SIZE).ok()?;
        if raw.get(..4)? != b"BDB1" {
            continue;
        }
        if let Some(name) = extract_character_name(&raw) {
            characters.push(CharacterName {
                name,
                last_play_time: extract_last_play_time(&raw),
            });
        }
    }
    (data_offset == bytes.len()).then_some(characters)
}

fn extract_character_name(data: &[u8]) -> Option<String> {
    for offset in 2..data.len().saturating_sub(8) {
        if read_u16(data, offset - 2) != Some(4) || data.get(offset..offset + 4)? != b"name" {
            continue;
        }
        let length = read_u16(data, offset + 4)? as usize;
        if length == 0 || length > 128 {
            continue;
        }
        let end = offset.checked_add(6)?.checked_add(length)?;
        let Ok(name) = std::str::from_utf8(data.get(offset + 6..end)?) else {
            continue;
        };
        let name = name.trim();
        if name.is_empty() || name.chars().any(char::is_control) {
            continue;
        }
        return Some(name.to_owned());
    }
    None
}

fn extract_last_play_time(data: &[u8]) -> Option<u32> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let maximum = u32::try_from(now.saturating_add(24 * 60 * 60)).ok()?;
    for offset in (0..data.len().saturating_sub(31)).step_by(4) {
        if read_u32(data, offset) != Some(12)
            || read_u32(data, offset + 4) != Some(1)
            || read_u32(data, offset + 8) != Some(1)
            || read_u32(data, offset + 12) != Some(13)
            || read_u32(data, offset + 20) != Some(0)
            || read_u32(data, offset + 24) != Some(1)
            || read_u32(data, offset + 28) != Some(14)
        {
            continue;
        }
        let timestamp = read_u32(data, offset + 16)?;
        if (1_500_000_000..=maximum).contains(&timestamp) {
            return Some(timestamp);
        }
    }
    None
}

fn select_most_recent_character(
    mut characters: Vec<CharacterName>,
) -> Result<CharacterName, DetectionStatus> {
    if characters.len() == 1 {
        return characters.pop().ok_or(DetectionStatus::NoCharacterFound);
    }
    let newest_time = characters
        .iter()
        .filter_map(|character| character.last_play_time)
        .max()
        .ok_or(DetectionStatus::AmbiguousCharacter)?;
    let mut newest = characters
        .into_iter()
        .filter(|character| character.last_play_time == Some(newest_time));
    let selected = newest.next().ok_or(DetectionStatus::AmbiguousCharacter)?;
    if newest.all(|character| character.name == selected.name) {
        Ok(selected)
    } else {
        Err(DetectionStatus::AmbiguousCharacter)
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
