pub(crate) fn notify_command_result(
    webview: &wry::WebView,
    request_id: &str,
    result: Result<(), String>,
    success_message: &str,
    batch_key: Option<&str>,
    restore_settings: bool,
    ecosystem: Option<&str>,
) {
    let (success, message) = match result {
        Ok(()) => (true, success_message.to_owned()),
        Err(error) => (false, error),
    };
    let payload = serde_json::json!({"requestId":request_id,"success":success,"message":message,"batchKey":batch_key,"restoreSettings":restore_settings,"ecosystem":ecosystem});
    if let Ok(payload) = serde_json::to_string(&payload) {
        let _ = webview.evaluate_script(&format!("window.__shroudforgeCommandResult({payload});"));
    }
}
