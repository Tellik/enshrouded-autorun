pub fn rgba() -> Result<(Vec<u8>, u32, u32), image::ImageError> {
    const SIZE: u32 = 256;
    let icon = image::load_from_memory_with_format(
        include_bytes!("../../../../assets/shroudedit-hammer.png"),
        image::ImageFormat::Png,
    )?
    .resize_exact(SIZE, SIZE, image::imageops::FilterType::Lanczos3)
    .into_rgba8();
    Ok((icon.into_raw(), SIZE, SIZE))
}
