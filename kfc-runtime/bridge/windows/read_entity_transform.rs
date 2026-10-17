use super::load_native_provider::provider;
use super::native_abi::PropRecord;

pub fn world_entity_get_transform(handle: u32) -> Option<PropRecord> {
    let provider = provider()?;
    let mut prop = PropRecord::default();
    let ok = unsafe { (provider.world_entity_get_transform)(handle, &mut prop) };
    ok.then_some(prop)
}
