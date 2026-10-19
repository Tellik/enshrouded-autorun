use super::{DetectionReport, DetectionStatus};

#[cfg(windows)]
mod windows {
    use super::{DetectionReport, DetectionStatus};
    use std::mem::size_of;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::{
                Debug::ReadProcessMemory,
                ToolHelp::{
                    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
                    TH32CS_SNAPPROCESS,
                },
            },
            Memory::{
                MEM_COMMIT, MEM_PRIVATE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ,
                PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_NOACCESS,
                PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY, VirtualQueryEx,
            },
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    };

    const CHUNK_SIZE: usize = 4 * 1024 * 1024;
    const MAX_RECORD_SIZE: usize = 512;
    const SCAN_RETRY_DELAY: Duration = Duration::from_secs(60);
    const UI_PLATFORM_PREFIX: &[u8] = b"[Steam]";
    const LEVEL_LABELS: [&[u8]; 2] = [b" Stufe ", b" Level "];
    const MAX_PLAYER_NAME_LENGTH: usize = 40;

    static SCAN_STATE: OnceLock<Mutex<ScanState>> = OnceLock::new();

    #[derive(Default)]
    struct ScanState {
        process_id: Option<u32>,
        address: Option<usize>,
        scanning: bool,
        last_scan_finished: Option<Instant>,
        last_status: Option<DetectionStatus>,
    }

    struct OwnedHandle(windows_sys::Win32::Foundation::HANDLE);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    pub(super) fn read_live_character_selection() -> DetectionReport {
        let process_ids = match find_game_processes() {
            Ok(process_ids) => process_ids,
            Err(()) => return report(None, None, DetectionStatus::ProcessMemoryUnreadable),
        };
        let process_id = match process_ids.as_slice() {
            [] => {
                reset_scan_state(None);
                return report(None, None, DetectionStatus::GameNotRunning);
            }
            [process_id] => *process_id,
            _ => {
                reset_scan_state(None);
                return report(None, None, DetectionStatus::MultipleGameProcesses);
            }
        };

        let cache = SCAN_STATE.get_or_init(|| Mutex::new(ScanState::default()));
        let (cached_address, currently_scanning, last_scan_finished, last_status) = {
            let Ok(mut state) = cache.lock() else {
                return report(
                    None,
                    Some(process_id),
                    DetectionStatus::ProcessMemoryUnreadable,
                );
            };
            if state.process_id != Some(process_id) {
                *state = ScanState {
                    process_id: Some(process_id),
                    ..ScanState::default()
                };
            }
            (
                state.address,
                state.scanning,
                state.last_scan_finished,
                state.last_status,
            )
        };

        if let Some(address) = cached_address {
            match read_selection_at(process_id, address) {
                Ok(name) => return report(Some(name), Some(process_id), DetectionStatus::Detected),
                Err(()) => {
                    if let Ok(mut state) = cache.lock()
                        && state.process_id == Some(process_id)
                        && state.address == Some(address)
                    {
                        state.address = None;
                        state.last_scan_finished = Some(Instant::now());
                        state.last_status = Some(DetectionStatus::SelectionNotFound);
                    }
                }
            }
        }

        if currently_scanning {
            return report(
                None,
                Some(process_id),
                DetectionStatus::SelectionScanInProgress,
            );
        }
        if let Some(last_scan_finished) = last_scan_finished
            && last_scan_finished.elapsed() < SCAN_RETRY_DELAY
        {
            return report(
                None,
                Some(process_id),
                last_status.unwrap_or(DetectionStatus::SelectionNotFound),
            );
        }

        if start_scan(process_id).is_err() {
            return report(
                None,
                Some(process_id),
                DetectionStatus::ProcessMemoryUnreadable,
            );
        }
        report(
            None,
            Some(process_id),
            DetectionStatus::SelectionScanInProgress,
        )
    }

    fn report(
        name: Option<String>,
        process_id: Option<u32>,
        status: DetectionStatus,
    ) -> DetectionReport {
        DetectionReport {
            name,
            saved_name: None,
            status,
            process_id,
            cache_error: None,
        }
    }

    fn reset_scan_state(process_id: Option<u32>) {
        let cache = SCAN_STATE.get_or_init(|| Mutex::new(ScanState::default()));
        if let Ok(mut state) = cache.lock() {
            if state.process_id != process_id {
                *state = ScanState {
                    process_id,
                    ..ScanState::default()
                };
            }
        }
    }

    fn start_scan(process_id: u32) -> Result<(), ()> {
        let cache = SCAN_STATE.get_or_init(|| Mutex::new(ScanState::default()));
        {
            let mut state = cache.lock().map_err(|_| ())?;
            if state.process_id != Some(process_id) {
                *state = ScanState {
                    process_id: Some(process_id),
                    ..ScanState::default()
                };
            }
            state.scanning = true;
            state.last_scan_finished = None;
            state.last_status = Some(DetectionStatus::SelectionScanInProgress);
        }

        let spawn = std::thread::Builder::new()
            .name("shroudforge-player-name-scan".into())
            .spawn(move || {
                let result = scan_process_for_selection(process_id);
                let cache = SCAN_STATE.get_or_init(|| Mutex::new(ScanState::default()));
                let Ok(mut state) = cache.lock() else {
                    return;
                };
                if state.process_id != Some(process_id) {
                    return;
                }
                state.scanning = false;
                state.last_scan_finished = Some(Instant::now());
                match result {
                    Ok(address) => {
                        state.address = Some(address);
                        state.last_status = Some(DetectionStatus::Detected);
                    }
                    Err(status) => {
                        state.address = None;
                        state.last_status = Some(status);
                    }
                }
            });

        match spawn {
            Ok(_) => Ok(()),
            Err(_) => {
                if let Ok(mut state) = cache.lock()
                    && state.process_id == Some(process_id)
                {
                    state.scanning = false;
                    state.last_scan_finished = Some(Instant::now());
                    state.last_status = Some(DetectionStatus::ProcessMemoryUnreadable);
                }
                Err(())
            }
        }
    }

    fn find_game_processes() -> Result<Vec<u32>, ()> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(());
        }
        let snapshot = OwnedHandle(snapshot);
        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut process_ids = Vec::new();
        let mut has_entry = unsafe { Process32FirstW(snapshot.0, &mut entry) } != 0;
        while has_entry {
            let length = entry
                .szExeFile
                .iter()
                .position(|character| *character == 0)
                .unwrap_or(entry.szExeFile.len());
            let executable = String::from_utf16_lossy(&entry.szExeFile[..length]);
            if executable.eq_ignore_ascii_case("enshrouded.exe") {
                process_ids.push(entry.th32ProcessID);
            }
            has_entry = unsafe { Process32NextW(snapshot.0, &mut entry) } != 0;
        }
        process_ids.sort_unstable();
        process_ids.dedup();
        Ok(process_ids)
    }

    fn open_process(process_id: u32) -> Result<OwnedHandle, ()> {
        let handle =
            unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id) };
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            Err(())
        } else {
            Ok(OwnedHandle(handle))
        }
    }

    fn read_selection_at(process_id: u32, address: usize) -> Result<String, ()> {
        let process = open_process(process_id)?;
        let region = query_region(process.0, address).ok_or(())?;
        if !is_readable_private_region(&region) {
            return Err(());
        }
        let region_base = region.BaseAddress as usize;
        let region_end = region_base.checked_add(region.RegionSize).ok_or(())?;
        let available = region_end.checked_sub(address).ok_or(())?;
        if available == 0 {
            return Err(());
        }
        let mut bytes = vec![0; available.min(MAX_RECORD_SIZE)];
        let mut bytes_read = 0usize;
        let read = unsafe {
            ReadProcessMemory(
                process.0,
                address as *const _,
                bytes.as_mut_ptr().cast(),
                bytes.len(),
                &mut bytes_read,
            )
        };
        if read == 0 || bytes_read == 0 {
            return Err(());
        }
        bytes.truncate(bytes_read);
        parse_ui_record(&bytes).ok_or(())
    }

    fn scan_process_for_selection(process_id: u32) -> Result<usize, DetectionStatus> {
        let process =
            open_process(process_id).map_err(|_| DetectionStatus::ProcessMemoryUnreadable)?;
        let mut cursor = 0usize;
        let mut carry = Vec::new();

        loop {
            let Some(region) = query_region(process.0, cursor) else {
                break;
            };
            let base = region.BaseAddress as usize;
            let Some(end) = base.checked_add(region.RegionSize) else {
                break;
            };
            if end <= cursor {
                break;
            }

            carry.clear();
            if is_readable_private_region(&region) {
                let mut offset = 0usize;
                while offset < region.RegionSize {
                    let address = base + offset;
                    let length = (region.RegionSize - offset).min(CHUNK_SIZE);
                    let mut bytes = vec![0; length];
                    let mut bytes_read = 0usize;
                    let read = unsafe {
                        ReadProcessMemory(
                            process.0,
                            address as *const _,
                            bytes.as_mut_ptr().cast(),
                            length,
                            &mut bytes_read,
                        )
                    };
                    if read != 0 && bytes_read > 0 {
                        bytes.truncate(bytes_read);
                        let combined_address = address.saturating_sub(carry.len());
                        let mut combined = std::mem::take(&mut carry);
                        combined.extend_from_slice(&bytes);
                        // VirtualQueryEx walks upward by address. The first matching row was the
                        // row that changed in the live character comparison. The parser accepts
                        // the observed platform and level text without case sensitivity.
                        if let Some((record_offset, _name)) = find_first_ui_record(&combined) {
                            let found_address = combined_address
                                .checked_add(record_offset)
                                .ok_or(DetectionStatus::ProcessMemoryUnreadable)?;
                            return Ok(found_address);
                        }
                        let carry_length = combined.len().min(MAX_RECORD_SIZE);
                        carry.extend_from_slice(&combined[combined.len() - carry_length..]);
                    } else {
                        carry.clear();
                    }
                    offset = offset.saturating_add(length);
                }
            }
            cursor = end;
        }

        Err(DetectionStatus::SelectionNotFound)
    }

    fn query_region(
        process: windows_sys::Win32::Foundation::HANDLE,
        address: usize,
    ) -> Option<MEMORY_BASIC_INFORMATION> {
        let mut region = MEMORY_BASIC_INFORMATION::default();
        let queried = unsafe {
            VirtualQueryEx(
                process,
                address as *const _,
                &mut region,
                size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };
        (queried == size_of::<MEMORY_BASIC_INFORMATION>()).then_some(region)
    }

    fn is_readable_private_region(region: &MEMORY_BASIC_INFORMATION) -> bool {
        if region.State != MEM_COMMIT
            || region.Type != MEM_PRIVATE
            || region.Protect & (PAGE_GUARD | PAGE_NOACCESS) != 0
        {
            return false;
        }
        matches!(
            region.Protect & 0xff,
            PAGE_READONLY
                | PAGE_READWRITE
                | PAGE_WRITECOPY
                | PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY
        )
    }

    fn find_first_ui_record(bytes: &[u8]) -> Option<(usize, String)> {
        let mut cursor = 0usize;
        while cursor < bytes.len() {
            let utf8 = find_ascii_case_insensitive(&bytes[cursor..], UI_PLATFORM_PREFIX)
                .map(|relative| cursor + relative);
            let utf16 = find_utf16_ascii_prefix(&bytes[cursor..], UI_PLATFORM_PREFIX)
                .map(|relative| cursor + relative);
            let Some(start) = [utf8, utf16].into_iter().flatten().min() else {
                break;
            };
            if let Some(name) = parse_ui_record(&bytes[start..]) {
                return Some((start, name));
            }
            cursor = start.checked_add(1)?;
        }
        None
    }

    fn parse_ui_record(bytes: &[u8]) -> Option<String> {
        if starts_with_ascii_case_insensitive(bytes, UI_PLATFORM_PREFIX) {
            return parse_ui_utf8_record(bytes);
        }
        if starts_with_utf16_ascii_prefix(bytes, UI_PLATFORM_PREFIX) {
            let utf16: Vec<u16> = bytes
                .get(..bytes.len().min(MAX_RECORD_SIZE))?
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            let text = String::from_utf16_lossy(&utf16);
            return parse_ui_text_record(&text);
        }
        None
    }

    fn parse_ui_utf8_record(bytes: &[u8]) -> Option<String> {
        let name_start = UI_PLATFORM_PREFIX.len();
        let search_end = bytes.len().min(name_start.checked_add(160)?);
        let name_area = bytes.get(name_start..search_end)?;
        let (label_offset, label) = find_level_label(name_area)?;
        let name_bytes = name_area.get(..label_offset)?;
        let name = std::str::from_utf8(name_bytes)
            .ok()?
            .trim_matches('\0')
            .trim();
        validate_name(name)?;

        let level_start = name_start
            .checked_add(label_offset)?
            .checked_add(label.len())?;
        validate_level(bytes, level_start)?;
        Some(name.to_owned())
    }

    fn parse_ui_text_record(text: &str) -> Option<String> {
        if !text
            .as_bytes()
            .get(..UI_PLATFORM_PREFIX.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(UI_PLATFORM_PREFIX))
        {
            return None;
        }
        let bytes = text.as_bytes();
        let name_start = UI_PLATFORM_PREFIX.len();
        let search_end = bytes.len().min(name_start.checked_add(160)?);
        let name_area = bytes.get(name_start..search_end)?;
        let (label_offset, label) = find_level_label(name_area)?;
        let name_bytes = name_area.get(..label_offset)?;
        let name = std::str::from_utf8(name_bytes)
            .ok()?
            .trim_matches('\0')
            .trim();
        validate_name(name)?;

        let level_start = name_start
            .checked_add(label_offset)?
            .checked_add(label.len())?;
        validate_level(bytes, level_start)?;
        Some(name.to_owned())
    }

    fn find_level_label(name_area: &[u8]) -> Option<(usize, &[u8])> {
        LEVEL_LABELS
            .iter()
            .filter_map(|label| {
                find_ascii_case_insensitive(name_area, label).map(|offset| (offset, *label))
            })
            .min_by_key(|(offset, _)| *offset)
            .filter(|(offset, _)| *offset > 0)
    }

    fn validate_name(name: &str) -> Option<()> {
        if name.is_empty()
            || name.chars().count() > MAX_PLAYER_NAME_LENGTH
            || name.chars().any(char::is_control)
        {
            return None;
        }
        Some(())
    }

    fn validate_level(bytes: &[u8], level_start: usize) -> Option<()> {
        let mut position = level_start;
        let level_digit_start = position;
        while bytes.get(position).is_some_and(u8::is_ascii_digit) {
            position += 1;
        }
        if position == level_digit_start || position - level_digit_start > 3 {
            return None;
        }
        Some(())
    }

    fn starts_with_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .get(..needle.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(needle))
    }

    fn find_ascii_case_insensitive(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || needle.len() > haystack.len() {
            return None;
        }
        haystack
            .windows(needle.len())
            .position(|window| window.eq_ignore_ascii_case(needle))
    }

    fn starts_with_utf16_ascii_prefix(haystack: &[u8], needle: &[u8]) -> bool {
        let Some(prefix) = haystack.get(..needle.len().saturating_mul(2)) else {
            return false;
        };
        needle.iter().enumerate().all(|(index, expected)| {
            prefix[index * 2].eq_ignore_ascii_case(expected) && prefix[index * 2 + 1] == 0
        })
    }

    fn find_utf16_ascii_prefix(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        let length = needle.len().checked_mul(2)?;
        if length > haystack.len() {
            return None;
        }
        haystack.windows(length).position(|window| {
            needle.iter().enumerate().all(|(index, expected)| {
                window[index * 2].eq_ignore_ascii_case(expected) && window[index * 2 + 1] == 0
            })
        })
    }

    #[cfg(test)]
    mod tests {
        use super::parse_ui_record;

        #[test]
        fn accepts_uppercase_platform_and_level_without_nearby_build_label() {
            assert_eq!(
                parse_ui_record(b"[STEAM]BonsaiBauer LEVEL 37"),
                Some("BonsaiBauer".into())
            );
        }

        #[test]
        fn accepts_previous_label_casing_and_optional_build_label() {
            assert_eq!(
                parse_ui_record(b"[Steam]BonsaiBauer Stufe 37 Build-Version: 1269579"),
                Some("BonsaiBauer".into())
            );
        }

        #[test]
        fn accepts_utf16le_menu_text() {
            let bytes: Vec<u8> = "[STEAM]BonsaiBauer Level 37"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect();
            assert_eq!(parse_ui_record(&bytes), Some("BonsaiBauer".into()));
        }

        #[test]
        fn rejects_records_without_a_level_or_with_a_too_long_name() {
            assert_eq!(parse_ui_record(b"[STEAM]BonsaiBauer"), None);
            let long_name = format!("[STEAM]{} Level 37", "A".repeat(41));
            assert_eq!(parse_ui_record(long_name.as_bytes()), None);
        }
    }
}

#[cfg(windows)]
pub(super) fn read_live_character_selection() -> DetectionReport {
    windows::read_live_character_selection()
}

#[cfg(not(windows))]
pub(super) fn read_live_character_selection() -> DetectionReport {
    DetectionReport {
        name: None,
        saved_name: None,
        process_id: None,
        status: DetectionStatus::UnsupportedPlatform,
        cache_error: None,
    }
}
