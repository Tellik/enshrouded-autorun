use crate::{ModNotification, Notice, NoticeAction, NoticeLink};
use std::path::Path;

pub fn read_notifications(root: &Path, mods: &[ModNotification<'_>]) -> Vec<Notice> {
    let events =
        crate::config::read_document(root, crate::documents::EVENTS_STATE).unwrap_or_default();
    let news = crate::read(root)
        .ok()
        .and_then(|value| value.get("messages").and_then(|v| v.as_array()).cloned())
        .unwrap_or_default();
    let mut notices = events
        .as_object()
        .into_iter()
        .flat_map(|values| values.values().cloned())
        .filter(|value| crate::validate_event(root, value).is_ok())
        .chain(news)
        .filter_map(|value| {
            let mod_id = value.get("modId").and_then(|value| value.as_str());
            Some(Notice {
                id: value.get("id")?.as_str()?.to_owned(),
                mod_id: mod_id.unwrap_or("shroudforge").to_owned(),
                ecosystem: value
                    .get("ecosystem")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
                    .or_else(|| {
                        let mod_id = mod_id?;
                        mods.iter()
                            .find(|item| item.id == mod_id)
                            .map(|item| item.ecosystem.to_owned())
                    }),
                title: value.get("title")?.as_str()?.chars().take(120).collect(),
                message: value
                    .get("message")?
                    .as_str()?
                    .chars()
                    .take(2_000)
                    .collect(),
                level: value
                    .get("level")
                    .and_then(|value| value.as_str())
                    .unwrap_or("info")
                    .to_owned(),
                action_url: value
                    .get("actionUrl")
                    .and_then(|value| value.as_str())
                    .filter(|url| url.starts_with("https://"))
                    .map(str::to_owned),
                action_label_key: value
                    .get("actionLabelKey")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned),
                links: value
                    .get("links")
                    .and_then(|value| value.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|link| {
                        let url = link.get("url")?.as_str()?;
                        let label_key = link.get("labelKey")?.as_str()?;
                        (url.starts_with("https://") && !label_key.is_empty()).then(|| NoticeLink {
                            label_key: label_key.chars().take(120).collect(),
                            url: url.to_owned(),
                        })
                    })
                    .take(4)
                    .collect(),
                actions: value
                    .get("actions")
                    .and_then(|value| value.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|action| {
                        let kind = action.get("kind")?.as_str()?;
                        if !["open-mod", "open-debug-console", "open-installed-mods"]
                            .contains(&kind)
                        {
                            return None;
                        }
                        Some(NoticeAction {
                            id: action
                                .get("id")
                                .and_then(|value| value.as_str())
                                .unwrap_or(kind)
                                .to_owned(),
                            kind: kind.to_owned(),
                            target: action
                                .get("target")
                                .and_then(|value| value.as_str())
                                .map(str::to_owned),
                        })
                    })
                    .take(4)
                    .collect(),
                updated_at: value
                    .get("updatedAt")
                    .and_then(|value| value.as_u64())
                    .unwrap_or_default(),
                kind: value
                    .get("kind")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned),
                values: value.get("values").cloned(),
                changelog: value
                    .get("changelog")
                    .and_then(|value| value.as_array())
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(|item| item.as_str().map(str::to_owned))
                            .take(50)
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();
    notices.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    notices.truncate(100);
    notices
}
