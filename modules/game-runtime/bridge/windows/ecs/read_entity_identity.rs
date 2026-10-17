use crate::windows::provider::provider;

pub fn world_entity_identity(handle: u32) -> Option<u32> {
    let provider = provider()?;
    let mut entity_id = 0;
    let ok = unsafe { (provider.world_entity_identity)(handle, &mut entity_id) };
    (ok && entity_id != 0).then_some(entity_id)
}
