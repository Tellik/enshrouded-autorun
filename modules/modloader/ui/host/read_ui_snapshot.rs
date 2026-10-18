use super::read_activity_log::read_activity;
use super::read_game_version::read_game_version;
use super::read_host_arguments::Arguments;
use super::read_release_status::read_release_status;
use super::read_runtime_profiles::active_runtime_profile;
use super::read_runtime_profiles::runtime_profile_ids;
use super::read_storage_paths::storage_paths;
use super::read_ui_configuration::load_user_config;
use super::save_language::is_valid_locale;
use super::ui_snapshot_types::ModInfo;
use super::ui_snapshot_types::Snapshot;
use super::ui_snapshot_types::SnapshotSettings;
use super::validate_message_name::default_message_name_fallback;
use super::validate_message_name::normalize_message_name;
use shroudforge_loader_config::runtime_status::runtime_is_active;
use shroudforge_package::config_revision;
use shroudforge_updater::catalog_search_state::CatalogState;
use shroudforge_updater::installed_version;
use shroudforge_updater::provider_config::CatalogProviderConfig;
use shroudforge_updater::release_state::ReleaseState;
use std::sync::Arc;
use std::sync::Mutex;

pub(crate) fn read_ui_snapshot(
    arguments: &Arguments,
    provider: &CatalogProviderConfig,
    release: &Arc<Mutex<ReleaseState>>,
    catalog: &Arc<Mutex<CatalogState>>,
    central: serde_json::Value,
    mods: Vec<ModInfo>,
    player_name_detection: &shroudforge_player_name::DetectionReport,
) -> Snapshot {
    let logging_level = central
        .pointer("/logging/minimumLevel")
        .and_then(|v| v.as_str())
        .unwrap_or("INFO")
        .to_owned();
    let export_enabled = central
        .pointer("/exports/enabled")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let release_snapshot =
        read_release_status(&arguments.root, arguments.updater_window, &central, release);
    let username_mode = central
        .pointer("/general/usernameMode")
        .and_then(|value| value.as_str())
        .filter(|value| matches!(*value, "auto" | "manual"))
        .unwrap_or("auto")
        .to_owned();
    let fallback_username = central
        .pointer("/general/messageNameFallback")
        .and_then(|value| value.as_str())
        .and_then(|value| normalize_message_name(value).ok())
        .unwrap_or_else(default_message_name_fallback);
    let detected_username = player_name_detection.name.clone();
    let diagnostic_jobs = shroudforge_runtime_diagnostics::diagnostic_job_catalog(
        &arguments.root,
        &central["modules"]["runtimeDiagnostics"],
    );
    let notification_mods: Vec<_> = mods
        .iter()
        .map(|item| shroudforge_news::ModNotification {
            id: &item.id,
            name: &item.name,
            version: &item.version,
            ecosystem: item.ecosystem,
            changelog: &item.changelog,
        })
        .collect();
    Snapshot {
        diagnostics: shroudforge_runtime_diagnostics::status(&arguments.root),
        windows: shroudforge_loader_config::config::window_state(&arguments.root),
        configuration: super::read_configuration_status::configuration(
            &arguments.root,
            arguments.server,
            env!("CARGO_PKG_VERSION"),
        ),
        connected: !arguments.desktop && runtime_is_active(&arguments.root),
        mode: if arguments.desktop {
            if arguments.updater_window {
                "UPDATER"
            } else {
                "DESKTOP"
            }
        } else if !arguments.server {
            "CLIENT"
        } else {
            "SERVER"
        },
        updater_window: arguments.updater_window,
        game_version: read_game_version(&arguments.root),
        version: installed_version(&arguments.root),
        notices: shroudforge_news::read_notifications(&arguments.root, &notification_mods),
        mods,
        activity: read_activity(&arguments.root).unwrap_or_default(),
        news_templates: shroudforge_news::read(&arguments.root)
            .ok()
            .and_then(|value| value.get("templates").cloned())
            .unwrap_or_else(|| serde_json::json!({})),
        read_notice_ids: shroudforge_news::read_news_state(&arguments.root),
        release: release_snapshot,
        update_queue: shroudforge_updater::read_update_queue(&arguments.root),
        backup_management: shroudforge_game_assets::originals::management_status(
            &arguments.root,
            if arguments.server { "server" } else { "client" },
        ),
        settings: SnapshotSettings {
            config_revision: config_revision(&central),
            module_preferences: central["modules"].clone(),
            compact_mode: central
                .pointer("/general/compactMode")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            reduced_motion: central
                .pointer("/general/reducedMotion")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            username_mode,
            detected_username,
            message_name_fallback: fallback_username,
            log_level: logging_level,
            export_enabled,
            runtime_profile_id: central
                .pointer("/runtime/profileId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            active_runtime_profile: active_runtime_profile(&arguments.root),
            runtime_profiles: runtime_profile_ids(&arguments.root),
            runtime_profile_directory: shroudforge_loader_config::paths::runtime_profiles_dir(
                &arguments.root,
            )
            .display()
            .to_string(),
            diagnostic_jobs: diagnostic_jobs.jobs,
            diagnostic_job_errors: diagnostic_jobs.errors,
            diagnostic_job_directory: diagnostic_jobs.directory,
            diagnostic_selected_job_id: diagnostic_jobs.selected_job_id,
            base_url: provider.base_url.clone(),
            project_id: provider.project_id.clone(),
            paths: storage_paths(&arguments.root),
        },
        catalog: catalog
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default(),
        locale: load_user_config(&arguments.root)
            .locale
            .filter(|locale| is_valid_locale(locale))
            .unwrap_or_else(|| "de".into()),
    }
}
