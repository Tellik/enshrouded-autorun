use crate::windows::provider::provider;

pub fn world_voxel_read(origin: [i32; 3], dimensions: [u32; 3], count: usize) -> Option<Vec<u16>> {
    let provider = provider()?;
    let mut values = vec![0u16; count];
    let mut actual = 0usize;
    let ok = unsafe {
        (provider.world_voxel_read)(
            origin.as_ptr(),
            dimensions.as_ptr(),
            values.as_mut_ptr(),
            count,
            &mut actual,
        )
    };
    (ok && actual == count).then_some(values)
}
