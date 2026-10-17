use super::choose_screenshot_folder::choose_screenshot_folder;
use crate::blueprint_covers::ScreenshotList;
use crate::blueprint_covers::load_screenshot_list;
use crate::blueprint_covers::save_screenshot_folder;
use std::path::Path;

pub(crate) fn browse_screenshots(
    root: &Path,
    blueprint: &str,
) -> Result<Option<ScreenshotList>, Box<dyn std::error::Error>> {
    let folder = choose_screenshot_folder()?;
    save_screenshot_folder(root, &folder)?;
    load_screenshot_list(root, blueprint).map(Some)
}
