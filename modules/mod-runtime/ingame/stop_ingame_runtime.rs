use super::IngameRuntime;

impl Drop for IngameRuntime {
    fn drop(&mut self) {
        for item in self.lifecycle.iter_mut().rev() {
            if !item.active {
                continue;
            }
            let started = std::time::Instant::now();
            let result = item.try_unload(&self.runner.lua);
            self.diagnostics.measure(
                "mods",
                &item.id,
                "on_unload",
                started.elapsed(),
                result.is_err(),
            );
            if let Err(error) = result {
                tracing::error!(target: "shroudforge::runtime", mod_id = %item.id,
                    "on_unload failed: {error}");
            }
        }
        if self.publish_on_drop {
            self.publish_status(false);
        }
    }
}
