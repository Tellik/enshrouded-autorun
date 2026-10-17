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
        _ => format!("{operation} timed out. The game thread may still have consumed the request"),
    }
}
