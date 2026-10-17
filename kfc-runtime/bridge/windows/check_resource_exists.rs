use super::load_native_provider::provider;
use super::native_abi::RuntimeResourceId;
use super::resource_status;

pub fn resource_exists(resource: &kfc::guid::ResourceId) -> Result<bool, String> {
    let _span = shroudforge_runtime_diagnostics::measure_runtime_operations::span(
        "native.assets.resource_exists",
    );
    use kfc::container::StaticHash;
    let fetch = provider()
        .and_then(|value| value.resource_read)
        .ok_or("native provider lacks the ResourceService read extension")?;
    let native = RuntimeResourceId::from(resource);
    let mut size = 0usize;
    if unsafe {
        fetch(
            &native,
            resource.static_hash(),
            std::ptr::null_mut(),
            0,
            &mut size,
        )
    } {
        if size > 256 * 1024 * 1024 {
            return Err("live ResourceService resource exceeded the 256 MiB safety limit".into());
        }
        return Ok(true);
    }
    // A missing GUID and an unavailable owner share the existing native
    // read result. Preserve the Lua API's nil-versus-error distinction.
    if resource_status().is_some_and(|status| status["available"] == true) {
        Ok(false)
    } else {
        Err("live ResourceService resource is unavailable".into())
    }
}
