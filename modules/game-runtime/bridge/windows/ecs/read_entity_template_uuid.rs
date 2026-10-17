use crate::windows::provider::provider;

pub fn entity_template_uuid(entity: u32) -> Option<kfc::guid::Guid> {
    let fetch = provider()?.entity_template_uuid?;
    let mut bytes = [0u8; 16];
    unsafe { fetch(entity, bytes.as_mut_ptr()) }.then(|| kfc::guid::Guid::new(bytes))
}
