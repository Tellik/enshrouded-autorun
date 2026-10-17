use super::load_native_provider::provider;

pub fn world_cursor_read() -> Option<(Vec<u8>, u64)> {
    let provider = provider()?;
    let mut bytes = vec![0u8; 0xa0];
    let mut sequence = 0u64;
    let read = provider.world_cursor_read?;
    let ok = unsafe { read(bytes.as_mut_ptr(), bytes.len(), &mut sequence) };
    (ok && sequence != 0).then_some((bytes, sequence))
}
