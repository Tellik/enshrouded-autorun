use super::load_native_provider::provider;
use super::native_abi::{GridSpec, GridSpecResult};
use std::ffi::CString;

pub fn world_grid_get_spec(id: &str) -> Option<GridSpecResult> {
    let provider = provider()?;
    let id = CString::new(id).ok()?;
    let mut spec = GridSpec::default();
    let ok = unsafe { (provider.world_grid_get_spec)(id.as_ptr(), &mut spec) };
    if !ok {
        return None;
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
    Some(GridSpecResult {
        id,
        origin: spec.origin,
        cell_size: spec.cell_size,
        maximum: spec.maximum,
    })
}
