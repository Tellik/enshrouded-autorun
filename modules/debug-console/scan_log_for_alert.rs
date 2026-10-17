use std::io::{Read, Seek};
use std::fs::File;
use std::io::SeekFrom;
use std::path::Path;

pub(crate) fn scan_log_for_alert(path: &Path, offset: &mut u64) -> bool {
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
    String::from_utf8_lossy(&bytes)
        .lines()
        .any(|line| line.starts_with("[E "))
}
