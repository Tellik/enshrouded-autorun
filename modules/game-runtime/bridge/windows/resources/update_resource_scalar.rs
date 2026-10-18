use crate::windows::native_abi::RuntimeResourceId;
use crate::windows::provider::provider;
use crate::windows::world_context::world_session_id;

pub fn resource_compare_exchange_scalar_for_session(
    resource: &kfc::guid::ResourceId,
    resource_size: usize,
    offset: usize,
    expected: &[u8],
    desired: &[u8],
    expected_session: u64,
) -> Result<(), String> {
    use kfc::container::StaticHash;
    if expected.len() != resource_size {
        return Err(
            "session-bound resource writes require the complete expected resource snapshot".into(),
        );
    }
    let end = offset
        .checked_add(desired.len())
        .filter(|end| *end <= expected.len())
        .ok_or("resource scalar lies outside the complete expected snapshot")?;
    validate_resource_scalar(resource_size, offset, &expected[offset..end], desired)?;
    if expected_session == 0 || world_session_id() != expected_session {
        return Err(
            "resource scalar update belongs to another or an inactive world session".into(),
        );
    }
    let apply = provider()
        .and_then(|value| value.resource_compare_exchange_scalar_for_session)
        .ok_or("native provider lacks the session-bound ResourceService scalar write extension")?;
    let native = RuntimeResourceId::from(resource);
    resource_scalar_result(unsafe {
        apply(
            &native,
            resource.static_hash(),
            resource_size,
            offset,
            expected.as_ptr().cast(),
            desired.as_ptr().cast(),
            desired.len(),
            expected_session,
        )
    })
}

fn validate_resource_scalar(
    resource_size: usize,
    offset: usize,
    expected: &[u8],
    desired: &[u8],
) -> Result<(), String> {
    let width = desired.len();
    if !matches!(width, 1 | 2 | 4 | 8)
        || expected.len() != width
        || offset % width != 0
        || offset
            .checked_add(width)
            .is_none_or(|end| end > resource_size)
    {
        return Err(
            "resource writes require one naturally aligned 1/2/4/8-byte scalar inside the snapshot"
                .into(),
        );
    }
    Ok(())
}

fn resource_scalar_result(status: u32) -> Result<(), String> {
    match status {
        1 => Ok(()),
        2 => Err("Resource scalar snapshot changed before update. Read current values and retry".into()),
        3 => Err("Resource write outcome is uncertain because lifetime or original data-page protection could not be verified. Do not retry blindly".into()),
        _ => Err("guarded resource scalar update failed".into()),
    }
}
