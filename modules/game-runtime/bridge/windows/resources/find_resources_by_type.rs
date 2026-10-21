use crate::windows::native_abi::RuntimeResourceId;
use crate::windows::provider::provider;

pub fn resources_by_type(type_hash: u32) -> Result<Vec<kfc::guid::ResourceId>, String> {
    let _span = shroudforge_runtime_diagnostics::operation_timings::span("native.assets.list_type");
    use kfc::guid::{Guid, ResourceId};
    let fetch = provider()
        .and_then(|value| value.resources_by_type)
        .ok_or("native provider lacks the ResourceService listing extension")?;
    // Listing validates the complete resource index on every native call.
    // Small lists need no separate full scan just to discover their size.
    let mut native = vec![RuntimeResourceId::default(); 256];
    for attempt in 0..3 {
        let mut actual = 0usize;
        let success = unsafe { fetch(type_hash, native.as_mut_ptr(), native.len(), &mut actual) };
        if actual > 1 << 20 {
            return Err("live ResourceService listing exceeded the resource safety limit".into());
        }
        if success {
            if actual > native.len() {
                return Err("live ResourceService listing changed during retrieval".into());
            }
            native.truncate(actual);
            return Ok(native
                .into_iter()
                .map(|resource| {
                    ResourceId::new(
                        Guid::new(resource.guid),
                        resource.type_hash,
                        resource.part_index,
                    )
                })
                .collect());
        }
        if actual <= native.len() {
            return Err(if attempt == 0 {
                "live ResourceService listing is unavailable".into()
            } else {
                "live ResourceService listing changed during retrieval".into()
            });
        }
        if attempt == 2 {
            return Err("live ResourceService listing changed during retrieval".into());
        }
        native.resize(actual, RuntimeResourceId::default());
    }
    Err("live ResourceService listing changed during retrieval".into())
}
