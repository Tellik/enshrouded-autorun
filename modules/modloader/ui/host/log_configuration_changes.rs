use std::path::Path;

pub(crate) fn log_configuration_check_transitions(
    root: &Path,
    configuration: &serde_json::Value,
    previous: &mut std::collections::BTreeMap<String, (String, String)>,
    previous_assets: &mut std::collections::BTreeMap<String, (String, String)>,
    previous_mods: &mut std::collections::BTreeMap<String, (String, String)>,
) {
    let checks = configuration
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|check| {
            let state = check.get("state")?.as_str()?;
            let id = check.get("id")?.as_str()?.to_owned();
            let group = check
                .get("group")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let detail = check
                .get("detail")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("No detail provided.");
            Some((
                id,
                (state.to_owned(), format!("group={group} detail={detail}")),
            ))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    log_status_issues(
        root,
        "compatibility-check",
        "check",
        checks,
        previous,
        |state| match state {
            "error" => Some('E'),
            "warning" => Some('W'),
            _ => None,
        },
        |state| state == "ok",
    );

    let assets = configuration
        .get("assets")
        .and_then(serde_json::Value::as_object)
        .and_then(|assets| {
            Some((
                "startup-preparation".to_owned(),
                (
                    assets.get("state")?.as_str()?.to_owned(),
                    assets
                        .get("detail")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("No detail provided.")
                        .to_owned(),
                ),
            ))
        })
        .into_iter()
        .collect();
    log_status_issues(
        root,
        "asset-status",
        "asset",
        assets,
        previous_assets,
        |state| match state {
            "failed" => Some('E'),
            "prepare-required" | "partial" => Some('W'),
            _ => None,
        },
        |state| matches!(state, "applied" | "not-required"),
    );

    let mods = configuration
        .get("modStates")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flat_map(|mods| mods.iter())
        .filter_map(|(id, value)| {
            Some((
                id.clone(),
                (
                    value.get("state")?.as_str()?.to_owned(),
                    value
                        .get("detail")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("No detail provided.")
                        .to_owned(),
                ),
            ))
        })
        .collect();
    log_status_issues(
        root,
        "mod-status",
        "mod",
        mods,
        previous_mods,
        |state| match state {
            "failed" => Some('E'),
            "blocked" | "restart-required" | "not-running" | "no-target" => Some('W'),
            _ => None,
        },
        |_| true,
    );
}

pub(crate) fn log_status_issues<F, R>(
    root: &Path,
    source: &str,
    label: &str,
    current: std::collections::BTreeMap<String, (String, String)>,
    previous: &mut std::collections::BTreeMap<String, (String, String)>,
    issue_level: F,
    is_resolved: R,
) where
    F: Fn(&str) -> Option<char>,
    R: Fn(&str) -> bool,
{
    for (id, (state, detail)) in &current {
        if let Some(level) = issue_level(state) {
            if !previous
                .get(id)
                .is_some_and(|(previous_state, _)| previous_state == state)
            {
                let _ = shroudforge_logging::append(
                    root,
                    level,
                    source,
                    &format!("{label}={id} state={state} detail={detail}"),
                );
            }
            previous.insert(id.clone(), (state.clone(), detail.clone()));
        } else if is_resolved(state)
            && let Some((previous_state, previous_detail)) = previous.remove(id)
        {
            let _ = shroudforge_logging::append(
                root,
                'I',
                source,
                &format!(
                    "{label}={id} state=resolved previous_state={previous_state} current_state={state} previous_detail={previous_detail} detail={detail}"
                ),
            );
        }
    }
}
