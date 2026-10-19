use super::IngameRuntime;
use crate::env::AppState;
use mlua::Function;
impl IngameRuntime {
    pub(super) fn update_mod_callbacks(&mut self, delta_seconds: f64) {
        for item in &mut self.lifecycle {
            if !item.active {
                continue;
            }
            if item.cleanup_pending {
                if std::time::Instant::now() < item.next_update {
                    continue;
                }
                self.diagnostics.release_mod_host(&item.id);
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
                    self.diagnostics.release_mod_host(&item.id);
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
