use crate::windows::{provider::provider, world_context::world_session_id};

pub fn world_voxel_read(
    origin: [i32; 3],
    dimensions: [u32; 3],
    count: usize,
) -> Result<Vec<u16>, String> {
    let expected = dimensions
        .iter()
        .try_fold(1usize, |total, value| total.checked_mul(*value as usize));
    if count == 0 || count > 65_536 || expected != Some(count) {
        return Err(
            "voxel read must contain between 1 and 65,536 cells with matching dimensions".into(),
        );
    }
    let session = world_session_id();
    if session == 0 {
        return Err("voxel read requires an active world session".into());
    }
    let provider = provider().ok_or("native voxel provider is unavailable")?;
    let mut values = vec![0u16; count];
    let mut actual = 0usize;
    let read = provider
        .world_voxel_read_for_session
        .ok_or("native registered voxel reader is unavailable")?;
    let status = unsafe {
        read(
            session,
            origin.as_ptr(),
            dimensions.as_ptr(),
            values.as_mut_ptr(),
            count,
            &mut actual,
        )
    };
    if world_session_id() != session {
        return Err("world-session-changed".into());
    }
    match status {
        1 if actual == count => Ok(values),
        1 => Err("native voxel reader returned an incomplete result".into()),
        2 => Err("voxel-read-pending".into()),
        3 => Err("voxel read rejected its region or buffer contract".into()),
        4 => Err("world-session-changed".into()),
        5 => Err("original registered voxel read contract is unavailable".into()),
        6 => Err("native voxel reader is faulted for this process".into()),
        _ => Err("native voxel reader returned an unknown status".into()),
    }
}
