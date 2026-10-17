#[cfg(windows)]
pub(crate) fn mark_headless_control_action(
    root: &std::path::Path,
    name: &str,
    request_id: &str,
    state: &str,
    message: &str,
) {
    let _ = shroudforge_loader_config::config::update_loader(root, |config| {
        let result = &mut config["control"]["results"][name];
        if result["requestId"].as_str() == Some(request_id) {
            result["state"] = serde_json::Value::String(state.into());
            result["result"] = serde_json::Value::String(message.into());
        }
        Ok(())
    });
}
