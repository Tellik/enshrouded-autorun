use crate::windows::native_abi::PropRecord;
use crate::windows::provider::provider;

pub fn world_entity_get_transform(handle: u32) -> Option<PropRecord> {
    let provider = provider()?;
    let mut prop = PropRecord::default();
    let ok = unsafe { (provider.world_entity_get_transform)(handle, &mut prop) };
    ok.then_some(prop)
}

/// None means confirmed absence. An old bool-only provider can prove presence,
/// but its failure cannot distinguish absence from an unreadable snapshot.
pub fn world_entity_read_transform(handle: u32) -> Result<Option<PropRecord>, String> {
    let provider = provider().ok_or("native-prop-provider-unavailable")?;
    let Some(read) = provider.world_entity_read_transform_for_session else {
        return world_entity_get_transform(handle)
            .map(Some)
            .ok_or_else(|| "entity-state-unavailable".into());
    };
    let session = provider
        .world_session_id
        .map(|read| unsafe { read() })
        .unwrap_or(0);
    if session == 0 {
        return Err("world-session-unavailable".into());
    }
    let mut prop = PropRecord::default();
    match unsafe { read(handle, session, &mut prop) } {
        1 => Ok(Some(prop)),
        6 => Ok(None),
        2 => Err("invalid-entity-handle".into()),
        3 => world_entity_get_transform(handle)
            .map(Some)
            .ok_or_else(|| "entity-state-unavailable".into()),
        4 => Err("world-session-or-entity-handle-stale".into()),
        5 => Err("entity-state-unavailable-retry".into()),
        7 => Err("world-write-outcome-uncertain".into()),
        _ => Err("unknown-native-prop-read-status".into()),
    }
}
