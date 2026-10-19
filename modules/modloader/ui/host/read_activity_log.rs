use super::ui_snapshot_types::Activity;
use std::path::Path;

pub(crate) fn read_activity(root: &Path) -> Result<Vec<Activity>, String> {
    shroudforge_logging::read_activity(root)
}

// Log check state transitions once instead of repeating the same status on every refresh.
