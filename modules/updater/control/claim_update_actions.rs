#[cfg(windows)]
use crate::update_control_action::HeadlessControlAction;

#[cfg(windows)]
pub(crate) fn claim_headless_control_actions(
    root: &std::path::Path,
) -> Result<Vec<HeadlessControlAction>, String> {
    shroudforge_loader_config::config::claim_control_actions(
        root,
        shroudforge_loader_config::config::ControlActionOwner::Updater,
        false,
    )
    .map(|actions| {
        actions
            .into_iter()
            .map(|action| HeadlessControlAction {
                name: action.name,
                request_id: action.request_id,
                parameters: action.parameters,
            })
            .collect()
    })
}
