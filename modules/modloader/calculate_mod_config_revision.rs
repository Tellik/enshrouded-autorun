pub fn config_revision(value: &serde_json::Value) -> String {
    crate::config::revision(&serde_json::to_vec(value).expect("JSON value is serializable"))
}
