use super::IngameRuntime;
use crate::env::AppState;
use mlua::Function;

impl IngameRuntime {
    pub fn update(&mut self, delta_seconds: f64) {
        self.diagnostics.tick();
        crate::env::runtime_metrics::set_enabled(
            self.diagnostics.enabled("runtime") || self.diagnostics.enabled("mods"),
        );
        self.runner
            .lua
            .app_data_ref::<AppState>()
            .unwrap()
            .poll_native_dll_loads();
        self.process_runtime_reload_request();
        if std::time::Instant::now() >= self.next_status {
            let (authorized_clients, restrict_network_clients) =
                shroudforge_loader_config::config::read_loader(&self.root)
                    .map(|config| shroudforge_steam_networking::read_network_allowlist(&config))
                    .unwrap_or_else(|_| (Vec::new(), true));
            shroudforge_steam_networking::report_network_health(
                &authorized_clients,
                restrict_network_clients,
                || {
                    self.runner
                        .lua
                        .app_data_ref::<AppState>()
                        .unwrap()
                        .is_server()
                },
            );
            self.refresh_configuration();
            self.publish_status(true);
            self.next_status = std::time::Instant::now() + std::time::Duration::from_secs(1);
        }
        crate::env::shroudforge::dispatch_ui_actions(&self.runner.lua, &self.active_mod_ids());
        self.update_mod_callbacks(delta_seconds);
        let state = self.runner.lua.app_data_ref::<AppState>().unwrap();
        state.advance_loaded_resource_indexes();
    }

    fn update_mod_callbacks(&mut self, delta_seconds: f64) {
        for item in &mut self.lifecycle {
            if !item.active {
                continue;
            }
            if item.cleanup_pending {
                if std::time::Instant::now() < item.next_update {
                    continue;
                }
                if item.try_unload(&self.runner.lua).is_ok() {
                    self.runner
                        .lua
                        .app_data_ref::<AppState>()
                        .unwrap()
                        .set_runtime_mod_active(&item.id, false);
                    tracing::info!(target: "shroudforge::runtime", mod_id = %item.id,
                        "Pending runtime mod cleanup completed");
                }
                item.next_update = std::time::Instant::now() + item.update_interval;
                continue;
            }
            if let Some(key) = &item.update {
                if std::time::Instant::now() < item.next_update {
                    continue;
                }
                let started = self
                    .diagnostics
                    .enabled("mods")
                    .then(std::time::Instant::now);
                let result = self
                    .runner
                    .lua
                    .registry_value::<Function>(key)
                    .and_then(|callback| callback.call::<()>(delta_seconds));
                if let Some(started) = started {
                    self.diagnostics.measure(
                        "mods",
                        &item.id,
                        "on_update",
                        started.elapsed(),
                        result.is_err(),
                    );
                }
                // Advance the planned phase and skip every missed period.
                // Never replay old callbacks in a burst after a slow operation.
                let completed = std::time::Instant::now();
                let periods = completed
                    .saturating_duration_since(item.next_update)
                    .as_nanos()
                    / item.update_interval.as_nanos()
                    + 1;
                item.next_update = u32::try_from(periods)
                    .ok()
                    .and_then(|periods| {
                        item.next_update
                            .checked_add(item.update_interval.saturating_mul(periods))
                    })
                    .filter(|next| *next > completed)
                    .unwrap_or(completed + item.update_interval);
                if let Err(error) = result {
                    tracing::error!(target: "shroudforge::runtime", mod_id = %item.id,
                        "on_update failed: {error}");
                    self.errors
                        .insert(item.id.clone(), format!("on_update failed: {error}").into());
                    if let Err(cleanup) = item.try_unload(&self.runner.lua) {
                        tracing::error!(target: "shroudforge::runtime", mod_id = %item.id,
                            "Update rollback is pending and will be retried: {cleanup}");
                    } else {
                        self.runner
                            .lua
                            .app_data_ref::<AppState>()
                            .unwrap()
                            .set_runtime_mod_active(&item.id, false);
                    }
                }
            }
        }
    }
}
