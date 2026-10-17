use super::IngameRuntime;
use crate::env::AppState;
use mlua::Function;

impl IngameRuntime {
    pub(super) fn restore_previous_runtime(&mut self, active_ids: &[String]) {
        for item in &mut self.lifecycle {
            if !active_ids.iter().any(|id| id == &item.id) || item.cleanup_pending {
                continue;
            }
            item.active = true;
            self.runner
                .lua
                .app_data_ref::<AppState>()
                .unwrap()
                .set_runtime_mod_active(&item.id, true);
            if let Some(key) = &item.load {
                if let Err(error) = self
                    .runner
                    .lua
                    .registry_value::<Function>(key)
                    .and_then(|callback| callback.call::<()>(()))
                {
                    tracing::error!(target:"shroudforge::runtime", mod_id=%item.id, "Could not restore runtime mod after refresh rollback: {error}");
                    if let Err(cleanup) = item.try_unload(&self.runner.lua) {
                        tracing::error!(target:"shroudforge::runtime", mod_id=%item.id,
                            "Refresh rollback cleanup is pending and will be retried: {cleanup}");
                    } else {
                        self.runner
                            .lua
                            .app_data_ref::<AppState>()
                            .unwrap()
                            .set_runtime_mod_active(&item.id, false);
                    }
                    continue;
                }
            }
            item.active = true;
        }
        self.publish_status(true);
    }
}
