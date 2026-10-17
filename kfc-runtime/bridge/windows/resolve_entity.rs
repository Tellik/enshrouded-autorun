use super::load_native_provider::provider;

pub fn resolve_entity(entity_id: u32) -> Option<u32> {
    let provider = provider()?;
    let handle = unsafe { (provider.resolve_entity)(entity_id) };
    (handle != 0).then_some(handle)
}
