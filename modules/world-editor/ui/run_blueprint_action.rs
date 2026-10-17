use super::browse_screenshots::browse_screenshots;
use super::read_editor_arguments::Arguments;
use crate::blueprint_covers::apply_screenshot;
use crate::blueprint_covers::load_screenshot_list;
use crate::blueprint_covers::undo_screenshot;
use crate::log_editor_action::log_world_editor;
use crate::queue_editor_action::dispatch_action;
use std::time::Instant;

pub(crate) fn run_blueprint_action(
    action: String,
    value: String,
    arguments: &Arguments,
    webview: &wry::WebView,
    next_refresh: &mut Instant,
) {
    if matches!(
        action.as_str(),
        "browseScreenshots" | "refreshScreenshots" | "applyScreenshot" | "undoScreenshot"
    ) {
        log_world_editor(
            &arguments.root,
            'D',
            "input",
            &format!("action=\"{action}\" value=\"{value}\" source=world-editor-window"),
        );
    }
    let result = match action.as_str() {
        "browseScreenshots" => browse_screenshots(&arguments.root, &value),
        "refreshScreenshots" => load_screenshot_list(&arguments.root, &value).map(Some),
        "applyScreenshot" => apply_screenshot(&arguments.root, &value).map(|()| None),
        "undoScreenshot" => undo_screenshot(&arguments.root, &value).map(|()| None),
        _ => dispatch_action(&arguments.root, &action, &value).map(|()| None),
    };
    match result {
        Ok(Some(list)) => {
            if let Ok(payload) = serde_json::to_string(&list) {
                let _ = webview
                    .evaluate_script(&format!("window.__worldEditorScreenshots({payload});"));
            }
        }
        Ok(None) => {
            if action == "applyScreenshot" {
                let name = value
                    .split_once('\t')
                    .map(|(name, _)| name)
                    .unwrap_or("unknown");
                log_world_editor(
                    &arguments.root,
                    'I',
                    "blueprint",
                    &format!("Screenshot cover saved for blueprint \"{name}\"."),
                );
                let _ = webview.evaluate_script("window.__worldEditorScreenshotApplied(false);");
                *next_refresh = Instant::now();
            } else if action == "undoScreenshot" {
                log_world_editor(
                    &arguments.root,
                    'I',
                    "blueprint",
                    &format!("Previous screenshot cover restored for blueprint \"{value}\"."),
                );
                let _ = webview.evaluate_script("window.__worldEditorScreenshotApplied(true);");
                *next_refresh = Instant::now();
            }
        }
        Err(error) => {
            log_world_editor(
                &arguments.root,
                'E',
                "aktion",
                &format!("Action \"{action}\" failed: {error}"),
            );
            let _ = webview.evaluate_script(&format!(
                "window.__worldEditorScreenshotError({});",
                js_string(&error.to_string())
            ));
        }
    }
}

pub(crate) fn js_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into())
}
