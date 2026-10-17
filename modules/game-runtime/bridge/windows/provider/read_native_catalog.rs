use std::ffi::c_char;

pub(in crate::windows) const CATALOG_ERRORS: [&str; 3] = [
    "invalid native catalog response size",
    "invalid component registry response",
    "component registry changed repeatedly during read",
];
pub(in crate::windows) const REGISTRATION_ERRORS: [&str; 3] = [
    "invalid native registration response size",
    "invalid native registration response",
    "native registration changed repeatedly during read",
];

pub(in crate::windows) fn fetch_json(
    mut fetch: impl FnMut(*mut c_char, usize) -> usize,
    maximum: usize,
    errors: [&str; 3],
) -> Result<serde_json::Value, String> {
    let [invalid_size, invalid_response, repeated_change] = errors;
    let mut size = fetch(std::ptr::null_mut(), 0);
    for _ in 0..3 {
        if !(2..=maximum).contains(&size) {
            return Err(invalid_size.into());
        }
        let mut buffer = vec![0u8; size];
        let required = fetch(buffer.as_mut_ptr().cast(), buffer.len());
        if required > size {
            size = required;
            continue;
        }
        if required < 2 || buffer[required - 1] != 0 {
            return Err(invalid_response.into());
        }
        return serde_json::from_slice(&buffer[..required - 1]).map_err(|e| e.to_string());
    }
    Err(repeated_change.into())
}
