use crate::windows::native_abi::RuntimeResourceId;
use crate::windows::provider::provider;

pub fn resources_by_type(type_hash: u32) -> Result<Vec<kfc::guid::ResourceId>, String> {
    let _span = shroudforge_runtime_diagnostics::operation_timings::span("native.assets.list_type");
    use kfc::guid::{Guid, ResourceId};
    let fetch = provider()
        .and_then(|value| value.resources_by_type)
        .ok_or("native provider lacks the ResourceService listing extension")?;
    let mut count = 0usize;
    if !unsafe { fetch(type_hash, std::ptr::null_mut(), 0, &mut count) } {
        return Err("live ResourceService listing is unavailable".into());
    }
    if count > 1 << 20 {
        return Err("live ResourceService listing exceeded the resource safety limit".into());
    }
    let mut native = vec![RuntimeResourceId::default(); count];
    if count != 0 {
        let mut actual = 0usize;
        if !unsafe { fetch(type_hash, native.as_mut_ptr(), native.len(), &mut actual) }
            || actual != count
        {
            return Err("live ResourceService listing changed during retrieval".into());
        }
    }
    Ok(native
        .into_iter()
        .map(|resource| {
            ResourceId::new(
                Guid::new(resource.guid),
                resource.type_hash,
                resource.part_index,
            )
        })
        .collect())
}
