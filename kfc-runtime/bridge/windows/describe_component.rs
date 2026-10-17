use super::load_native_provider::provider;
use super::native_abi::Component;
use std::ffi::CString;

pub fn resolve(name: &str) -> Option<Component> {
    let _span = shroudforge_runtime_diagnostics::measure_runtime_operations::span(
        "native.ecs.resolve_type",
    );
    let provider = provider()?;
    let name = CString::new(name).ok()?;
    let mut size = 0;
    unsafe { (provider.describe)(name.as_ptr(), &mut size) }.then_some(Component { size })
}
