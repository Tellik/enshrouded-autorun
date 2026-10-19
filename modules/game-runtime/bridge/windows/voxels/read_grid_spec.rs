use crate::windows::native_abi::{GridSpec, GridSpecResult};
use crate::windows::provider::provider;
use crate::windows::world_context::world_session_id;
use std::ffi::CString;

pub fn world_grid_get_spec(id: &str) -> Result<GridSpecResult, String> {
    let provider = provider().ok_or("native voxel provider is unavailable")?;
    let read = provider
        .world_grid_get_spec_for_session
        .ok_or("native registered grid metadata is unavailable")?;
    let requested_id = id;
    let id = CString::new(id).map_err(|_| "voxel grid id contains a null byte")?;
    let session = world_session_id();
    if session == 0 {
        return Err("world-session-changed".into());
    }
    let mut spec = GridSpec::default();
    let status = unsafe { read(session, id.as_ptr(), &mut spec) };
    if world_session_id() != session {
        return Err("world-session-changed".into());
    }
    match status {
        1 => {}
        2 => return Err("voxel-grid-pending".into()),
        3 => return Err("voxel grid rejected its id or buffer contract".into()),
        4 => return Err("world-session-changed".into()),
        5 => return Err("original registered voxel grid contract is unavailable".into()),
        6 => return Err("native voxel reader is faulted for this process".into()),
        _ => return Err("native voxel grid returned an unknown status".into()),
    }
    let id_end = spec
        .id
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(spec.id.len());
    let id = String::from_utf8_lossy(unsafe {
        std::slice::from_raw_parts(spec.id.as_ptr().cast::<u8>(), id_end)
    })
    .into_owned();
    if id_end == spec.id.len()
        || id != requested_id
        || spec.origin.iter().any(|value| !value.is_finite())
        || spec
            .cell_size
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.)
        || spec.maximum.contains(&0)
    {
        return Err("native voxel grid returned invalid metadata".into());
    }
    Ok(GridSpecResult {
        id,
        origin: spec.origin,
        cell_size: spec.cell_size,
        maximum: spec.maximum,
    })
}
