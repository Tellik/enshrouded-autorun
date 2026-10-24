use crate::windows::provider::provider;

pub fn world_autorun_available() -> bool {
    provider()
        .and_then(|value| value.world_autorun_available)
        .is_some_and(|available| unsafe { available() })
}

pub fn world_autorun_set(enabled: bool, session: u64) -> Result<(), String> {
    let set = provider()
        .and_then(|value| value.world_autorun_set)
        .ok_or("native autorun input is unavailable")?;
    if enabled && (!world_autorun_available() || session == 0) {
        return Err("native autorun input is not ready for this game session".into());
    }
    if unsafe { set(enabled, session) } {
        Ok(())
    } else {
        Err("native autorun input rejected this game session".into())
    }
}

pub fn world_autorun_status() -> Option<(u64, u64, u64, bool, f32, f32)> {
    let status = provider()?.world_autorun_status?;
    let (mut hits, mut local, mut writes, mut active, mut x, mut y) = (0, 0, 0, false, 0.0, 0.0);
    unsafe {
        status(
            &mut hits,
            &mut local,
            &mut writes,
            &mut active,
            &mut x,
            &mut y,
        )
    }
    .then_some((hits, local, writes, active, x, y))
}
