use crate::valid_news_id;
use std::path::Path;

pub fn mark_unread(root: &Path, ids: &[String]) -> Result<(), String> {
    crate::config::update_state_section(root, "news", |stored| {
        let mut value = stored
            .cloned()
            .unwrap_or(serde_json::json!({"read": [], "readAt": {}}));
        let requested: std::collections::HashSet<&str> = ids.iter().map(String::as_str).collect();
        let read = value["read"]
            .as_array_mut()
            .ok_or("news read must be an array")?;
        read.retain(|id| id.as_str().map_or(true, |id| !requested.contains(id)));
        if let Some(timestamps) = value["readAt"].as_object_mut() {
            timestamps.retain(|id, _| !requested.contains(id.as_str()));
        }
        crate::config::validate_document(root, crate::documents::NEWS_STATE, &value)?;
        Ok(value)
    })
}

pub fn mark_news_unread(root: &Path, ids: &[String]) -> Result<(), String> {
    let ids = ids
        .iter()
        .filter(|id| valid_news_id(id))
        .cloned()
        .collect::<Vec<_>>();
    crate::mark_unread(root, &ids)
}
