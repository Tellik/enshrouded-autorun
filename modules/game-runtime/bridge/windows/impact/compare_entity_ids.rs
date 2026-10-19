use crate::windows::{provider::provider, world_context::world_session_id};

/// Executes the original registered isSameEntity operation on owned VM buffers.
/// These are raw engine IDs, not ShroudForge handles. No entity lookup occurs.
pub fn impact_compare_entity_ids(session: u64, left: u32, right: u32) -> Result<bool, String> {
    if session == 0 || world_session_id() != session {
        return Err("Impact operation belongs to another or an inactive world session".into());
    }
    let call = provider()
        .and_then(|provider| provider.impact_compare_entity_ids)
        .ok_or("native provider lacks the registered IsSameEntity adapter")?;
    let mut equal = u8::MAX;
    let status = unsafe { call(session, left, right, &mut equal) };
    if status != 1 {
        return Err(match status {
            2 => "invalid owned Impact arguments",
            3 => "original Impact operation registry is unavailable or unstable",
            4 => "world session changed during Impact operation",
            5 => "original registered Impact operation has an unsupported contract",
            6 => "original registered Impact operation raised a native exception",
            7 => "original registered Impact operation returned an invalid result",
            _ => "native Impact provider returned an unknown result status",
        }
        .into());
    }
    if world_session_id() != session {
        return Err("world session changed during Impact operation".into());
    }
    match equal {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err("native Impact provider returned an invalid branch value".into()),
    }
}
