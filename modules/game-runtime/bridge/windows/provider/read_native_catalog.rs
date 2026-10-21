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
    // Registrations and status reports usually fit in one call. A size-only
    // probe would make the provider construct the same report twice.
    let mut buffer = vec![0u8; maximum.min(4096)];
    for attempt in 0..4 {
        let required = fetch(buffer.as_mut_ptr().cast(), buffer.len());
        if !(2..=maximum).contains(&required) {
            return Err(invalid_size.into());
        }
        if required > buffer.len() {
            if attempt == 3 {
                break;
            }
            buffer.resize(required, 0);
            continue;
        }
        if buffer[required - 1] != 0 {
            return Err(invalid_response.into());
        }
        return serde_json::from_slice(&buffer[..required - 1]).map_err(|e| e.to_string());
    }
    Err(repeated_change.into())
}
