use crate::now;
use crate::valid_news_id;
use std::path::Path;

pub fn mark_read(root: &Path, ids: &[String]) -> Result<(), String> {
    crate::config::update_state_section(root, "news", |stored| {
        let mut value = stored
            .cloned()
            .unwrap_or(serde_json::json!({"read": [], "readAt": {}}));
        for id in ids {
            if !value["read"]
                .as_array()
                .is_some_and(|read| read.iter().any(|old| old == id))
            {
                value["read"]
                    .as_array_mut()
                    .ok_or("news read must be an array")?
                    .push(serde_json::json!(id));
            }
            value["readAt"][id] = serde_json::json!(now());
        }
        crate::config::validate_document(root, crate::documents::NEWS_STATE, &value)?;
        Ok(value)
    })
}

pub fn mark_news_read(root: &Path, ids: &[String]) -> Result<(), String> {
    let ids = ids
        .iter()
        .filter(|id| valid_news_id(id))
        .cloned()
        .collect::<Vec<_>>();
    crate::mark_read(root, &ids)
}
