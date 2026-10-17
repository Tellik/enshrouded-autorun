use crate::windows::native_abi::RuntimeResourceId;
use crate::windows::provider::provider;
use crate::windows::world_context::world_session_id;

pub fn resource_resolve_item(
    item_id: u32,
    type_hash: u32,
    root_size: usize,
    item_id_offset: usize,
    guid_offset: usize,
    session: u64,
) -> Result<Option<kfc::guid::ResourceId>, String> {
    let _span =
        shroudforge_runtime_diagnostics::operation_timings::span("native.assets.resolve_item");
    let fetch = provider()
        .and_then(|value| value.resource_resolve_item)
        .ok_or("provider lacks the original ItemRegistry resolver")?;
    if session == 0 || world_session_id() != session {
        return Err("original ItemRegistry needs the current authoritative world".into());
    }
    let mut id = RuntimeResourceId::default();
    let status = unsafe {
        fetch(
            item_id,
            type_hash,
            root_size,
            item_id_offset,
            guid_offset,
            session,
            &mut id,
        )
    };
    if world_session_id() != session {
        return Err("world changed during original ItemRegistry resolution".into());
    }
    match status {
        1 if id.type_hash == type_hash && id.guid != [0; 16] => Ok(Some(
            kfc::guid::ResourceId::new(kfc::guid::Guid::new(id.guid), id.type_hash, id.part_index),
        )),
        6 => Ok(None),
        3 => Err("original ItemRegistry contract is unavailable for this provider/profile".into()),
        4 => Err("original ItemRegistry world or owner changed".into()),
        5 => Err("original ItemRegistry is busy, unavailable or changed. Retry later".into()),
        _ => Err("original ItemRegistry rejected the reflected key/layout contract".into()),
    }
}
