use crate::{ModNotification, write_event_notice};
use shroudforge_logging::LogLevel;
use std::path::Path;

pub fn sync_mod_events(root: &Path, mods: &[ModNotification<'_>]) {
    let current = mods
        .iter()
        .map(|item| {
            (
                item.id.to_owned(),
                serde_json::json!({
                    "name": item.name,
                    "version": item.version,
                    "ecosystem": item.ecosystem,
                    "changelog": item.changelog
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let previous = crate::config::read_document(root, crate::documents::MOD_STATE)
        .ok()
        .and_then(|value| value.as_object().cloned());
    let Some(previous) = previous else {
        let _ = crate::config::update_state_section(root, "mods", |_| {
            Ok(serde_json::Value::Object(current))
        });
        return;
    };
    for (id, item) in &current {
        let name = item
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or(id);
        let version = item
            .get("version")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        match previous.get(id) {
            None => write_event_notice(
                root,
                &format!("mod-install-{id}-{version}"),
                id,
                "mod.install.success",
                LogLevel::Info,
                name,
                version,
                item.get("ecosystem").and_then(|value| value.as_str()),
                None,
                item.get("changelog"),
            ),
            Some(old) if old.get("version") != item.get("version") => {
                let old_version = old
                    .get("version")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown");
                write_event_notice(
                    root,
                    &format!("mod-update-{id}-{version}"),
                    id,
                    "mod.update.success",
                    LogLevel::Info,
                    name,
                    version,
                    item.get("ecosystem").and_then(|value| value.as_str()),
                    Some(old_version),
                    item.get("changelog"),
                );
            }
            _ => {}
        }
    }
    for (id, item) in &previous {
        if !current.contains_key(id) {
            let name = item
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or(id);
            let version = item
                .get("version")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            write_event_notice(
                root,
                &format!("mod-remove-{id}-{version}"),
                id,
                "mod.remove.success",
                LogLevel::Info,
                name,
                version,
                item.get("ecosystem").and_then(|value| value.as_str()),
                None,
                None,
            );
        }
    }
    let _ = crate::config::update_state_section(root, "mods", |_| {
        Ok(serde_json::Value::Object(current))
    });
}
