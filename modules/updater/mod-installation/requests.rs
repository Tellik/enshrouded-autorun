use crate::*;
pub fn request_mod_install(
    root: &std::path::Path,
    project_id: &str,
    title: &str,
    provider: serde_json::Value,
) -> Result<(), String> {
    if !crate::update_check::updates_enabled(root) {
        return Err("Updates module is disabled".into());
    }
    if project_id.is_empty()
        || project_id.len() > 128
        || !project_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err("invalid mod project ID".into());
    }
    enqueue_update_item(
        root,
        serde_json::json!({"id":format!("mod-install:{project_id}"),"kind":"mod","operation":"install","title":title,"version":"","state":"queued","selected":true,"payload":{"projectId":project_id,"provider":provider}}),
    )
    .map(|_| ())
}

pub fn request_mod_update(
    root: &std::path::Path,
    project_id: &str,
    mod_id: &str,
    title: &str,
    provider: serde_json::Value,
) -> Result<(), String> {
    enqueue_mod_update(root, project_id, mod_id, title, provider).map(|_| ())
}

pub(crate) fn request_mod_update_if_needed(
    root: &std::path::Path,
    project_id: &str,
    mod_id: &str,
    title: &str,
    provider: serde_json::Value,
) -> Result<bool, String> {
    enqueue_mod_update(root, project_id, mod_id, title, provider)
}

fn enqueue_mod_update(
    root: &std::path::Path,
    project_id: &str,
    mod_id: &str,
    title: &str,
    provider: serde_json::Value,
) -> Result<bool, String> {
    if !crate::update_check::updates_enabled(root) {
        return Err("Updates module is disabled".into());
    }
    if project_id.is_empty()
        || project_id.len() > 128
        || !project_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        || mod_id.is_empty()
        || mod_id.len() > 80
        || !mod_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    {
        return Err("invalid mod project or package ID".into());
    }
    enqueue_update_item(
        root,
        serde_json::json!({"id":format!("mod-update:{project_id}:{mod_id}"),"kind":"mod","operation":"update","title":title,"version":"","state":"queued","selected":true,"payload":{"projectId":project_id,"modId":mod_id,"provider":provider}}),
    )
}
