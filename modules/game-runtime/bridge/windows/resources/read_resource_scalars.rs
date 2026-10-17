use super::resource_read;
use crate::windows::native_abi::{ResourceScalarRequest, RuntimeResourceId};
use crate::windows::provider::provider;
use crate::windows::world_context::world_session_id;

pub fn resource_scalar_batch(
    ids: &[kfc::guid::ResourceId],
    type_hash: u32,
    root_size: usize,
    offset: usize,
    session: u64,
) -> Result<Vec<[u8; 4]>, String> {
    let _span =
        shroudforge_runtime_diagnostics::operation_timings::span("native.assets.scalar_batch");
    use kfc::container::StaticHash;
    if ids.is_empty()
        || ids.len() > 64
        || offset % 4 != 0
        || offset.checked_add(4).is_none_or(|end| end > root_size)
        || ids.iter().any(|id| id.type_hash() != type_hash)
    {
        return Err("invalid resource scalar batch selection".into());
    }
    if world_session_id() != session {
        return Err("world session changed before resource scalar batch".into());
    }
    if let Some(fetch) = provider()
        .and_then(|value| value.resource_read_scalar_batch)
        .filter(|_| session != 0)
    {
        let requests: Vec<_> = ids
            .iter()
            .map(|id| ResourceScalarRequest {
                resource: RuntimeResourceId::from(id),
                static_hash: id.static_hash(),
            })
            .collect();
        let mut bits = vec![0u32; ids.len()];
        let mut actual = 0usize;
        let status = unsafe {
            fetch(
                requests.as_ptr(),
                requests.len(),
                type_hash,
                root_size,
                offset,
                session,
                bits.as_mut_ptr(),
                bits.len(),
                &mut actual,
            )
        };
        if world_session_id() != session {
            return Err("world session changed during resource scalar batch".into());
        }
        match status {
            1 if actual == ids.len() => {
                return Ok(bits.into_iter().map(u32::to_le_bytes).collect());
            }
            3 => {} // Older exact profiles retain the established read contract.
            4 => return Err("resource scalar batch context changed".into()),
            5 => {
                return Err(
                    "resource scalar batch incomplete: engine busy, unavailable or changed".into(),
                );
            }
            _ => return Err("resource scalar batch rejected invalid input or response".into()),
        }
    }
    let mut values = Vec::with_capacity(ids.len());
    for id in ids {
        let bytes = resource_read(id)?;
        if bytes.len() < root_size {
            return Err("resource scalar snapshot is smaller than reflected root".into());
        }
        values.push(
            bytes
                .get(offset..offset + 4)
                .ok_or("resource scalar outside current snapshot")?
                .try_into()
                .unwrap(),
        );
    }
    if world_session_id() != session {
        return Err("world session changed during resource scalar fallback".into());
    }
    Ok(values)
}
