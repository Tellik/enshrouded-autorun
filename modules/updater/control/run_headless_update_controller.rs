#[cfg(windows)]
use crate::claim_update_actions::claim_headless_control_actions;
#[cfg(windows)]
use crate::complete_update_action::mark_headless_control_action;
#[cfg(windows)]
use crate::execute_update_action::execute_headless_control_action;

#[cfg(windows)]
pub fn start_headless_control_worker(root: &std::path::Path) -> Result<(), String> {
    use std::sync::{Mutex, OnceLock};

    static STARTED_ROOTS: OnceLock<Mutex<std::collections::HashSet<std::path::PathBuf>>> =
        OnceLock::new();
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not resolve game root for config controller: {error}"))?;
    let started = STARTED_ROOTS.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    {
        let mut roots = started
            .lock()
            .map_err(|_| "headless config controller registry is unavailable")?;
        if !roots.insert(root.clone()) {
            return Ok(());
        }
    }

    std::thread::Builder::new()
        .name("shroudforge-config-controller".into())
        .spawn(move || {
            let mut next_auto_update_check = std::time::Instant::now();
            loop {
                if std::time::Instant::now() >= next_auto_update_check {
                    next_auto_update_check =
                        crate::check_due_system_update::check_due_system_update(&root);
                }
                match claim_headless_control_actions(&root) {
                    Ok(actions) => {
                        for action in actions {
                            let result = execute_headless_control_action(&root, &action);
                            let (state, message) = match result {
                                Ok(message) => ("succeeded", message),
                                Err(error) => ("failed", error),
                            };
                            mark_headless_control_action(
                                &root,
                                &action.name,
                                &action.request_id,
                                state,
                                &message,
                            );
                        }
                    }
                    Err(error) => {
                        let _ = shroudforge_logging::append(
                            &root,
                            'E',
                            "config-controller",
                            &format!("Could not read pending server actions: {error}"),
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        })
        .map(|_| ())
        .map_err(|error| format!("could not start headless config controller: {error}"))
}

#[cfg(not(windows))]
pub fn start_headless_control_worker(_: &std::path::Path) -> Result<(), String> {
    Ok(())
}
