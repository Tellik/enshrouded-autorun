use crate::*;
#[cfg(windows)]
pub fn request_mod_install(
    root: &std::path::Path,
    project_id: &str,
    title: &str,
    provider: serde_json::Value,
) -> Result<(), String> {
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
}


#[cfg(windows)]
pub fn request_mod_update(
    root: &std::path::Path,
    project_id: &str,
    mod_id: &str,
    title: &str,
    provider: serde_json::Value,
) -> Result<(), String> {
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


#[cfg(not(windows))]
pub fn request_mod_update(
    _: &std::path::Path,
    _: &str,
    _: &str,
    _: &str,
    _: serde_json::Value,
) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}


#[cfg(not(windows))]
pub fn request_mod_install(
    _: &std::path::Path,
    _: &str,
    _: &str,
    _: serde_json::Value,
) -> Result<(), String> {
    Err("scheduled updater is available on Windows only".into())
}
