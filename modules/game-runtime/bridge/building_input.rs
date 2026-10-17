pub fn cancel(id: u32) -> bool {
    if id == 0 {
        return false;
    }
    #[cfg(windows)]
    if let Some(address) = symbol(b"KfcRuntimeWorldBuildingInputCancel\0") {
        let function: unsafe extern "C" fn(u32) -> bool = unsafe { std::mem::transmute(address) };
        return unsafe { function(id) };
    }
    false
}

#[cfg(windows)]
fn symbol(name: &[u8]) -> Option<*const ()> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    unsafe {
        let module = GetModuleHandleW(
            "kfc-runtime.dll\0"
                .encode_utf16()
                .collect::<Vec<_>>()
                .as_ptr(),
        );
        if module.is_null() {
            return None;
        }
        GetProcAddress(module, name.as_ptr()).map(|value| value as *const ())
    }
}

pub fn submit(
    player: u32,
    kind: u32,
    item: u32,
    material: u32,
    slot: u32,
    target_entity_id: u32,
    position: [f64; 3],
    rotation: [f64; 4],
    scale: [f64; 3],
) -> u32 {
    #[cfg(windows)]
    if let Some(address) = symbol(b"KfcRuntimeWorldBuildingInput\0") {
        let function: unsafe extern "C" fn(
            u32,
            u32,
            u32,
            u32,
            u32,
            u32,
            *const f64,
            *const f64,
            *const f64,
        ) -> u32 = unsafe { std::mem::transmute(address) };
        return unsafe {
            function(
                player,
                kind,
                item,
                material,
                slot,
                target_entity_id,
                position.as_ptr(),
                rotation.as_ptr(),
                scale.as_ptr(),
            )
        };
    }
    let _ = (
        player,
        kind,
        item,
        material,
        slot,
        target_entity_id,
        position,
        rotation,
        scale,
    );
    0
}

pub fn status(id: u32) -> u32 {
    #[cfg(windows)]
    if let Some(address) = symbol(b"KfcRuntimeWorldBuildingInputStatus\0") {
        let function: unsafe extern "C" fn(u32) -> u32 = unsafe { std::mem::transmute(address) };
        return unsafe { function(id) };
    }
    let _ = id;
    0
}
