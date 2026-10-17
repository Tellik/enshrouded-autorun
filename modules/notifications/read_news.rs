use std::path::Path;

pub fn read(root: &Path) -> Result<serde_json::Value, String> {
    let value: serde_json::Value = serde_json::from_str(include_str!("news.json"))
        .map_err(|e| format!("embedded news.json: {e}"))?;
    crate::config::validate_document(root, crate::documents::NEWS, &value)?;
    Ok(value)
}
