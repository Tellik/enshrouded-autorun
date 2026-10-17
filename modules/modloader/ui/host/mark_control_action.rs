use std::path::Path;

pub(crate) fn mark_control_action(root: &Path, request_id: &str, state: &str, result: &str) {
    if request_id.is_empty() {
        return;
    }
    let present = shroudforge_loader_config::config::read_loader(root)
        .ok()
        .and_then(|config| config.pointer("/control/results").cloned())
        .and_then(|results| results.as_object().cloned())
        .is_some_and(|results| {
            results
                .values()
                .any(|status| status["requestId"] == request_id)
        });
    if !present {
        return;
    }
    let _ = shroudforge_loader_config::config::update_loader(root, |config| {
        let results = config
            .pointer_mut("/control/results")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or("control.results must be an object")?;
        if let Some(status) = results
            .values_mut()
            .find(|status| status["requestId"] == request_id)
        {
            status["state"] = serde_json::Value::String(state.into());
            status["result"] = serde_json::Value::String(result.chars().take(2000).collect());
        }
        Ok(())
    });
}
