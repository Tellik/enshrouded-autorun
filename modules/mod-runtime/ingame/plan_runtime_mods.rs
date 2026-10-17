use crate::API_VERSION;
use mod_loader::ModEnvironment;
use std::path::Path;

pub(super) fn plan_runtime_mods<'a>(
    env: &'a ModEnvironment,
    root: &Path,
    is_server: bool,
) -> (
    Vec<&'a mod_loader::Mod>,
    serde_json::Map<String, serde_json::Value>,
) {
    tracing::debug!(target: "shroudforge::runtime", stage = "runtime-plan", "Runtime startup stage started");
    let (mut plan, plan_errors) = env.runtime_plan_report_detailed(is_server, API_VERSION);
    tracing::debug!(target: "shroudforge::runtime", stage = "runtime-plan", planned_mods = plan.len(), "Runtime startup stage completed");
    let mut errors = serde_json::Map::new();
    for issue in plan_errors {
        tracing::warn!(target: "shroudforge::runtime", mod_id = %issue.mod_id,
        code = %issue.code, detail = %issue.detail, "Mod is blocked by its execution plan");
        errors.insert(
            issue.mod_id,
            format!("plan-blocked: {}", issue.detail).into(),
        );
    }
    // Candidate plans contain disabled mods so the UI can toggle them live.
    // Enabled mods still require their non-optional dependencies to be
    // enabled and available before their startup callbacks may run.
    for item in &plan {
        if !item.info().enabled || errors.contains_key(&item.info().id) {
            continue;
        }
        let blocked_dependency = item.info().dependencies.iter().find_map(|dependency| {
            if dependency.id == "shroudforge-api" || dependency.optional.unwrap_or(false) {
                return None;
            }
            let candidate = env.mod_registry().get(&dependency.id);
            if candidate.is_none_or(|candidate| !candidate.info().enabled) {
                Some(format!(
                    "required dependency is disabled or unavailable: {}",
                    dependency.id
                ))
            } else {
                errors
                    .get(&dependency.id)
                    .map(|_| format!("required dependency is blocked: {}", dependency.id))
            }
        });
        if let Some(detail) = blocked_dependency {
            tracing::warn!(target: "shroudforge::runtime", mod_id = %item.info().id,
            detail = %detail, "Runtime mod activation skipped");
            errors.insert(
                item.info().id.clone(),
                format!("dependency-blocked: {detail}").into(),
            );
        }
    }
    let needs_assets = plan.iter().any(|item| {
        item.info().enabled
            && item
                .info()
                .capabilities
                .contains(&mod_loader::Capability::Patch)
    });
    if needs_assets {
        let ready = mod_loader::prepared::fingerprint(env, is_server, API_VERSION)
            .map(|fingerprint| mod_loader::prepared::matches(&root, &fingerprint))
            .unwrap_or(false);
        if !ready {
            let mut blocked: std::collections::HashSet<String> = plan
                .iter()
                .filter(|item| {
                    item.info().enabled
                        && item
                            .info()
                            .capabilities
                            .contains(&mod_loader::Capability::Patch)
                })
                .map(|item| item.info().id.clone())
                .collect();
            loop {
                let before = blocked.len();
                for item in &plan {
                    if item.info().dependencies.iter().any(|dependency| {
                        !dependency.optional.unwrap_or(false) && blocked.contains(&dependency.id)
                    }) {
                        blocked.insert(item.info().id.clone());
                    }
                }
                if before == blocked.len() {
                    break;
                }
            }
            plan.retain(|item| {
            if blocked.contains(&item.info().id) { errors.insert(item.info().id.clone(), "startup-asset-apply-required: the early startup transaction did not publish this asset configuration".into()); false } else {true}
        });
        }
    }

    (plan, errors)
}
