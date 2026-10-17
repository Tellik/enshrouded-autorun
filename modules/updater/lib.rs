pub use mod_loader_bridge::missing_required_files;

#[path = "system-updates/notes.rs"]
mod system_updates_notes;
pub use system_updates_notes::*;

#[path = "system-updates/repair.rs"]
mod system_updates_repair;
pub use system_updates_repair::*;

#[path = "system-updates/headless_check.rs"]
mod system_updates_headless_check;
pub use system_updates_headless_check::*;

#[path = "system-updates/requests.rs"]
mod system_updates_requests;
pub use system_updates_requests::*;

#[path = "mod-installation/requests.rs"]
mod mod_installation_requests;
pub use mod_installation_requests::*;

#[path = "windows/entry.rs"]
mod windows_entry;
pub use windows_entry::*;

pub use shroudforge_game_assets::request_gamefiles_restore;

#[cfg(windows)]
#[path = "windows/worker.rs"]
mod windows;

#[cfg(windows)]
#[path = "windows/restore_gamefiles.rs"]
mod restore_worker;

#[cfg(windows)]
#[path = "system-updates/install_system_update.rs"]
mod system_install;

#[cfg(windows)]
#[path = "system-updates/stage_system_update.rs"]
mod system_stage;

#[cfg(windows)]
#[path = "windows/scheduler.rs"]
mod scheduled;

#[path = "system-updates/check.rs"]
pub mod configured_check;
#[path = "mod-installation/mod.rs"]
pub mod mod_installation;
pub mod ui;

pub mod documents;
#[path = "system-updates/check_lock.rs"]
pub mod update_check;

pub mod catalog_item;
pub mod provider_config;

pub mod command_result;

pub mod release_state;

pub mod release;

pub mod catalog_search_state;

pub mod search_mod_catalog;
pub use search_mod_catalog::start_catalog_search;

pub mod queue_mod_installation;
pub use queue_mod_installation::start_mod_install;

pub mod queue_mod_update;
pub use queue_mod_update::start_mod_update;

pub mod restore_checked_release;
pub use restore_checked_release::restore_persisted_update_result;

pub mod check_system_update;
pub use check_system_update::start_update_check;

pub mod queue_system_update_download;
pub use queue_system_update_download::start_staging;

#[path = "system-updates/read_installed_release.rs"]
mod read_installed_release;
pub use read_installed_release::installed_release;
pub use read_installed_release::installed_version;

#[path = "system-updates/fetch_system_release.rs"]
mod fetch_system_release;
pub use fetch_system_release::fetch_latest;
pub use fetch_system_release::fetch_latest_github;

#[path = "system-updates/compare_release_versions.rs"]
mod compare_release_versions;
pub use compare_release_versions::is_update_available;
pub use compare_release_versions::parse_release_tag;

#[path = "system-updates/github_release.rs"]
mod github_release;

#[path = "queue/update_queue_storage.rs"]
mod update_queue_storage;
pub(crate) use update_queue_storage::read_update_queue_value;

#[path = "queue/read_update_queue.rs"]
mod read_update_queue;
pub use read_update_queue::read_update_queue;

#[path = "queue/set_update_item_state.rs"]
mod set_update_item_state;
pub use set_update_item_state::set_active_queue_item_state;

#[path = "queue/add_update_item.rs"]
mod add_update_item;
#[cfg(windows)]
pub(crate) use add_update_item::enqueue_update_item;

#[path = "queue/select_update_items.rs"]
mod select_update_items;
pub use select_update_items::select_update_queue_item;
pub use select_update_items::select_update_queue_items;

#[path = "queue/remove_update_item.rs"]
mod remove_update_item;
pub use remove_update_item::remove_update_queue_item;

#[path = "queue/clear_update_queue.rs"]
mod clear_update_queue;
pub use clear_update_queue::clear_update_queue;

#[path = "queue/cancel_update_queue.rs"]
mod cancel_update_queue;
pub use cancel_update_queue::cancel_update_queue;

#[path = "queue/start_update_queue.rs"]
mod start_update_queue;
pub use start_update_queue::start_update_queue;

#[path = "queue/run_update_queue.rs"]
mod run_update_queue;
#[cfg(windows)]
pub(crate) use run_update_queue::run_update_queue;

#[path = "queue/finish_cancelled_queue.rs"]
mod finish_cancelled_queue;

#[path = "control/request_runtime_mod_action.rs"]
mod request_runtime_mod_action;
pub use request_runtime_mod_action::request_runtime_mod_reload;
pub use request_runtime_mod_action::request_runtime_mod_unload;

#[path = "control/run_headless_update_controller.rs"]
mod run_headless_update_controller;
pub use run_headless_update_controller::start_headless_control_worker;

#[path = "control/claim_update_actions.rs"]
mod claim_update_actions;

#[path = "control/execute_update_action.rs"]
mod execute_update_action;

#[path = "control/complete_update_action.rs"]
mod complete_update_action;

#[path = "control/update_control_action.rs"]
mod update_control_action;

#[cfg(windows)]
#[path = "system-updates/write_update_status.rs"]
mod write_update_status;

#[cfg(windows)]
#[path = "system-updates/check_update_cancelled.rs"]
mod check_update_cancelled;

#[cfg(windows)]
#[path = "system-updates/clear_staged_update.rs"]
mod clear_staged_update;

#[cfg(windows)]
#[path = "system-updates/read_install_arguments.rs"]
mod read_install_arguments;

#[cfg(windows)]
#[path = "system-updates/validate_update_paths.rs"]
mod validate_update_paths;

#[cfg(windows)]
#[path = "system-updates/read_managed_release_files.rs"]
mod read_managed_release_files;

#[cfg(windows)]
#[path = "system-updates/apply_release_files.rs"]
mod apply_release_files;

#[cfg(windows)]
#[path = "system-updates/restore_release_files.rs"]
mod restore_release_files;

#[cfg(windows)]
#[path = "system-updates/remove_release_files.rs"]
mod remove_release_files;

#[path = "control/queue_stored_system_update.rs"]
mod queue_stored_system_update;

#[path = "control/stage_stored_system_update.rs"]
mod stage_stored_system_update;

#[path = "control/start_selected_updates.rs"]
mod start_selected_updates;

#[path = "control/capture_original_game_files.rs"]
mod capture_original_game_files;

#[path = "control/queue_requested_mod_installation.rs"]
mod queue_requested_mod_installation;

#[path = "control/queue_requested_mod_update.rs"]
mod queue_requested_mod_update;

#[path = "control/read_action_parameters.rs"]
mod read_action_parameters;

#[path = "control/check_due_system_update.rs"]
mod check_due_system_update;

#[path = "mod-installation/run_mod_installation_worker.rs"]
mod run_mod_installation_worker;

#[path = "mod-installation/install_catalog_mod.rs"]
mod install_catalog_mod;

#[path = "mod-installation/download_mod_package.rs"]
mod download_mod_package;

#[path = "mod-installation/replace_installed_mod.rs"]
mod replace_installed_mod;

#[path = "mod-installation/verify_mod_checksum.rs"]
mod verify_mod_checksum;

#[path = "mod-installation/extract_mod_archive.rs"]
mod extract_mod_archive;

#[path = "mod-installation/mod_package.rs"]
mod mod_package;
