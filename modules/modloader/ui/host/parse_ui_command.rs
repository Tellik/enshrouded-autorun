use super::loader_settings_input::UiSettings;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UiCommand {
    #[serde(default)]
    pub(crate) revision: Option<String>,
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) settings: Option<UiSettings>,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(default)]
    pub(crate) mod_id: Option<String>,
    #[serde(default)]
    pub(crate) values: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) query: Option<String>,
    #[serde(default)]
    pub(crate) locale: Option<String>,
    #[serde(default)]
    pub(crate) action: Option<String>,
    #[serde(default)]
    pub(crate) job_id: Option<String>,
    #[serde(default)]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    pub(crate) ids: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) selected: Option<bool>,
    #[serde(default)]
    pub(crate) read: Option<bool>,
    #[serde(default)]
    pub(crate) project_id: Option<String>,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) message: Option<String>,
    #[serde(default)]
    pub(crate) scope: Option<String>,
    #[serde(default)]
    pub(crate) module: Option<String>,
    #[serde(default)]
    pub(crate) key: Option<String>,
    #[serde(default)]
    pub(crate) value: serde_json::Value,
    #[serde(default)]
    pub(crate) request_id: Option<String>,
    #[serde(default)]
    pub(crate) visible: Option<bool>,
    #[serde(default)]
    pub(crate) wait_for_game: Option<bool>,
}

pub(crate) enum Command {
    Hide(String),
    Drag(String),
    MinimizeWindow,
    ToggleMaximizeWindow,
    ResetWindowGeometry(String),
    Refresh(Option<String>),
    CopyLogs(String),
    RefreshMod(String, String),
    ReloadSettings(String),
    CheckUpdates(String),
    StageUpdate(String, bool),
    QueueSystemUpdate(String),
    SelectUpdateQueueItems(Vec<String>, bool, String),
    RemoveUpdateQueueItem(String, String),
    StartUpdateQueue(Vec<String>, bool, bool, String),
    ClearUpdateQueue(String),
    CancelUpdate(String),
    RestoreGamefiles(String),
    CaptureGamefileOriginals(String),
    OpenUrl(String, String),
    SetModEnabled(String, bool, String, String),
    SaveModSettings(String, serde_json::Value, String, String),
    SaveSettings(UiSettings, String),
    SaveSetting(String, Option<String>, String, serde_json::Value, String),
    BrowsePath(String, String),
    OpenPath(String, String),
    SetWindowVisibility(String, bool, String),
    SearchCatalog(String, String),
    InstallMod(String, String, String),
    UpdateMod(String, String, String),
    SaveLanguage(String, String),
    RunModAction(String, String, String),
    RemoveMod(String, String),
    SetNewsRead(Vec<String>, bool, String),
    Diagnostics(String, Option<String>, String),
    UiReady,
    UiError(String),
}

pub(crate) fn parse_ui_command(
    value: UiCommand,
    desktop: bool,
    updater_window: bool,
    server: bool,
) -> Option<Command> {
    Some(match value.command.as_str() {
        "diagnostics" => Command::Diagnostics(
            value.action.unwrap_or_default(),
            value.job_id,
            value.request_id.unwrap_or_default(),
        ),
        "hide" => Command::Hide(value.request_id.unwrap_or_default()),
        "drag" => Command::Drag(value.request_id.unwrap_or_default()),
        "minimize-window" => Command::MinimizeWindow,
        "toggle-maximize-window" => Command::ToggleMaximizeWindow,
        "reset-window-geometry" => {
            Command::ResetWindowGeometry(value.request_id.unwrap_or_default())
        }
        "refresh" => Command::Refresh(value.request_id),
        "copy-logs" => Command::CopyLogs(value.request_id.unwrap_or_default()),
        "refresh-mod" => match value.mod_id {
            Some(id) => Command::RefreshMod(id, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "reload-settings" => Command::ReloadSettings(value.request_id.unwrap_or_default()),
        "check-updates" => Command::CheckUpdates(value.request_id.unwrap_or_default()),
        "stage-update" => Command::StageUpdate(
            value.request_id.unwrap_or_default(),
            value.wait_for_game.unwrap_or(!desktop),
        ),
        "queue-system-update" => Command::QueueSystemUpdate(value.request_id.unwrap_or_default()),
        "select-update-queue-items" => match value.selected {
            Some(selected) => Command::SelectUpdateQueueItems(
                value.ids.unwrap_or_default(),
                selected,
                value.request_id.unwrap_or_default(),
            ),
            _ => return None,
        },
        "remove-update-queue-item" => match value.key {
            Some(id) => Command::RemoveUpdateQueueItem(id, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "start-update-queue" => Command::StartUpdateQueue(
            value.ids.unwrap_or_default(),
            value.wait_for_game.unwrap_or(!desktop),
            !server && !updater_window,
            value.request_id.unwrap_or_default(),
        ),
        "clear-update-queue" => Command::ClearUpdateQueue(value.request_id.unwrap_or_default()),
        "cancel-update" => Command::CancelUpdate(value.request_id.unwrap_or_default()),
        "restore-gamefiles" => Command::RestoreGamefiles(value.request_id.unwrap_or_default()),
        "capture-gamefile-originals" => {
            Command::CaptureGamefileOriginals(value.request_id.unwrap_or_default())
        }
        "search-catalog" => match value.query {
            Some(query) => Command::SearchCatalog(query, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "install-mod" => match value.project_id {
            Some(project_id) => {
                let title = value.title.unwrap_or_else(|| project_id.clone());
                Command::InstallMod(project_id, title, value.request_id.unwrap_or_default())
            }
            None => return None,
        },
        "update-mod" => match value.mod_id {
            Some(mod_id) => {
                let title = value.title.unwrap_or_else(|| mod_id.clone());
                Command::UpdateMod(mod_id, title, value.request_id.unwrap_or_default())
            }
            None => return None,
        },
        "save-language" => match value.locale {
            Some(locale) => Command::SaveLanguage(locale, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "open-url" => match value.url {
            Some(url) => Command::OpenUrl(url, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "save-mod-settings" => match (value.mod_id, value.values) {
            (Some(id), Some(values)) => Command::SaveModSettings(
                id,
                values,
                value.revision.unwrap_or_default(),
                value.request_id.unwrap_or_default(),
            ),
            _ => return None,
        },
        "set-mod-enabled" => match (value.mod_id, value.enabled) {
            (Some(id), Some(enabled)) => Command::SetModEnabled(
                id,
                enabled,
                value.revision.unwrap_or_default(),
                value.request_id.unwrap_or_default(),
            ),
            _ => return None,
        },
        "save-settings" => match value.settings {
            Some(settings) => Command::SaveSettings(settings, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "save-setting" => match (value.scope, value.key) {
            (Some(scope), Some(key)) => Command::SaveSetting(
                scope,
                value.module,
                key,
                value.value,
                value.request_id.unwrap_or_default(),
            ),
            _ => return None,
        },
        "browse-path" => match value.key {
            Some(key) => Command::BrowsePath(key, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "open-path" => match value.key {
            Some(key) => Command::OpenPath(key, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "set-window-visibility" => match (value.module, value.visible) {
            (Some(module), Some(visible)) => {
                Command::SetWindowVisibility(module, visible, value.request_id.unwrap_or_default())
            }
            _ => return None,
        },
        "run-mod-action" => match (value.mod_id, value.action) {
            (Some(id), Some(action)) => {
                Command::RunModAction(id, action, value.request_id.unwrap_or_default())
            }
            _ => return None,
        },
        "remove-mod" => match value.mod_id {
            Some(id) => Command::RemoveMod(id, value.request_id.unwrap_or_default()),
            None => return None,
        },
        "set-news-read" => Command::SetNewsRead(
            value.ids.unwrap_or_default(),
            value.read.unwrap_or(true),
            value.request_id.unwrap_or_default(),
        ),
        "ui-ready" => Command::UiReady,
        "ui-error" => Command::UiError(
            value
                .message
                .unwrap_or_else(|| "Unknown WebView error".into()),
        ),
        _ => return None,
    })
}
