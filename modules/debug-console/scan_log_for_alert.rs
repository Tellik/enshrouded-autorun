use std::fs::File;
use std::io::SeekFrom;
use std::io::{Read, Seek};
use std::path::Path;

pub(crate) fn scan_log_for_alert(
    path: &Path,
    offset: &mut u64,
    popup_mode: &str,
    bootstrap_complete: &mut bool,
    bootstrap_popup_shown: &mut bool,
) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let Ok(end) = file.seek(SeekFrom::End(0)) else {
        return false;
    };
    if end < *offset {
        *offset = 0;
    }
    if end == *offset {
        return false;
    }
    let start = (*offset).min(end);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return false;
    }
    let count = (end - start).min(1024 * 1024);
    let mut bytes = Vec::with_capacity(count as usize);
    if file.take(count).read_to_end(&mut bytes).is_err() {
        return false;
    }
    *offset = start + bytes.len() as u64;
    let mut found = false;
    let text = String::from_utf8_lossy(&bytes);
    for line in text.lines() {
        if line.contains("[bootstrap] Startup stage: runtime-ready") {
            *bootstrap_complete = true;
        }
        let error = line.starts_with("[E ");
        let warning = line.starts_with("[W ");
        found |= match popup_mode {
            "all" => error,
            "errorsAndWarnings" => error || warning,
            "bootstrap"
                if error
                    && !*bootstrap_complete
                    && !*bootstrap_popup_shown
                    && line.contains("] [bootstrap] ") =>
            {
                *bootstrap_popup_shown = true;
                true
            }
            _ => false,
        };
    }
    found
}
