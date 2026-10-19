use std::path::Path;

/// Rewrite stored notification severity values from the previous event schema.
///
/// This is safe to call on startup. It writes only when a legacy value is
/// present and preserves every other event field and state section.
pub fn migrate_legacy_event_levels(root: &Path) -> Result<bool, String> {
    let current = crate::config::read_state_section(root, "events")?;
    let contains_legacy_levels = current
        .as_ref()
        .and_then(serde_json::Value::as_object)
        .is_some_and(|events| {
            events
                .values()
                .any(|event| crate::normalize_legacy_event_level(event.clone()) != event.clone())
        });
    if !contains_legacy_levels {
        return Ok(false);
    }

    crate::config::update_state_section(root, "events", |stored| {
        let mut events = stored.cloned().unwrap_or_else(|| serde_json::json!({}));
        if let Some(events) = events.as_object_mut() {
            for event in events.values_mut() {
                *event = crate::normalize_legacy_event_level(event.clone());
            }
        }
        Ok(events)
    })?;
    Ok(true)
}
