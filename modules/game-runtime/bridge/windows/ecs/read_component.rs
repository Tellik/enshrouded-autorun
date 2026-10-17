use crate::windows::provider::provider;
use std::ffi::CString;

pub fn read(entity: u32, name: &str, size: u32) -> Option<Vec<u8>> {
    let _span =
        shroudforge_runtime_diagnostics::operation_timings::span("native.ecs.read_component");
    let provider = provider()?;
    let name = CString::new(name).ok()?;
    let mut bytes = vec![0; size as usize];
    unsafe {
        (provider.read)(
            entity,
            name.as_ptr(),
            bytes.as_mut_ptr().cast(),
            bytes.len(),
        )
    }
    .then_some(bytes)
}
