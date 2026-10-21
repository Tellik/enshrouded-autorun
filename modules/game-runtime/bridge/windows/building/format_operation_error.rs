pub(super) fn operation_error(operation: &str, outcome: u32) -> String {
    match outcome {
        30..=61 => {
            let flags = outcome - 30;
            format!(
                "{operation} was not consumed (hooks_ready={}, actor_hook_entered={}, actor_saw_request={}, building_dispatch_entered={}, building_dispatch_saw_request={})",
                flags & 16 != 0,
                flags & 1 != 0,
                flags & 2 != 0,
                flags & 4 != 0,
                flags & 8 != 0,
            )
        }
        1 => format!(
            "{operation} was rejected before dispatch (profile, arguments, or live context unavailable)"
        ),
        2 => format!("{operation} failed in the engine call"),
        3 => format!(
            "{operation} was queued, but its native game hook did not consume it within 3 seconds"
        ),
        4 => format!(
            "{operation} was dispatched, but the requested live ECS state change was not observed. The final world state is uncertain"
        ),
        5 => format!(
            "{operation} could not be confirmed because the world session or entity handle changed. No retry was sent"
        ),
        6 => format!(
            "{operation} has an uncertain native command outcome. Further native writes are blocked for this world session"
        ),
        _ => format!("{operation} timed out. The game thread may still have consumed the request"),
    }
}

#[cfg(test)]
mod tests {
    use super::operation_error;

    #[test]
    fn registered_command_failures_preserve_their_meaning() {
        let stale = operation_error("runtime.world.entity.destroy", 5);
        assert!(stale.contains("session or entity handle changed"));
        assert!(stale.contains("No retry was sent"));
        let uncertain = operation_error("runtime.world.entity.destroy", 6);
        assert!(uncertain.contains("uncertain native command outcome"));
        assert!(uncertain.contains("writes are blocked"));
        assert!(!stale.contains("hook"));
        assert!(!uncertain.contains("hook"));
        assert!(operation_error("runtime.world.entity.destroy", 4).contains("was not observed"));
    }
}
