use super::load_native_provider::provider;
use super::native_abi::RuntimeResourceId;
use super::world_session_id;

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
        .checked_add(4)
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
            4,
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
    if expected.len() != 4
        || desired.len() != 4
        || offset % 4 != 0
        || offset.checked_add(4).is_none_or(|end| end > resource_size)
    {
        return Err("resource writes require one aligned 32-bit scalar inside the snapshot".into());
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
