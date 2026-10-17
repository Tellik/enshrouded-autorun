use crate::windows::provider::provider;
use crate::windows::world_context::world_session_id;
use std::ffi::CString;

pub fn service_compare_exchange(
    key: &str,
    type_hash: u32,
    size: u32,
    session: u64,
    expected: &[u8],
    mask: &[u8],
    desired: &[u8],
) -> Result<(), String> {
    if size == 0
        || size > 256 * 1024 * 1024
        || expected.len() != size as usize
        || mask.len() != size as usize
        || desired.len() != size as usize
        || mask.iter().any(|byte| *byte > 1)
    {
        return Err("service snapshot, mask and reflected layout sizes disagree".into());
    }
    if session == 0 || world_session_id() != session {
        return Err("service update belongs to another or an inactive world session".into());
    }
    let key = CString::new(key).map_err(|_| "invalid service key")?;
    let apply = provider()
        .and_then(|provider| provider.service_compare_exchange)
        .ok_or("native provider lacks typed service writes")?;
    match unsafe {
        apply(
            key.as_ptr(),
            type_hash,
            size,
            session,
            expected.as_ptr().cast(),
            mask.as_ptr().cast(),
            desired.as_ptr().cast(),
        )
    } {
        1 => Ok(()),
        2 => Err("Service snapshot changed before update. Read current values and retry".into()),
        3 => Err("Service write outcome is uncertain. Stop writes and do not retry blindly".into()),
        _ => Err("guarded service update failed".into()),
    }
}
