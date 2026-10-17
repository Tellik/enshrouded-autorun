use crate::blueprint_library::valid_blueprint_name;
use base64::Engine as _;
use image::DynamicImage;
use image::ImageReader;
use image::RgbaImage;
use serde::Serialize;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScreenshotCard {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) modified: u64,
    pub(crate) thumbnail: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScreenshotList {
    pub(crate) folder: String,
    pub(crate) screenshots: Vec<ScreenshotCard>,
    pub(crate) truncated: bool,
    pub(crate) can_undo: bool,
}

pub(crate) fn screenshot_folder_file(root: &Path) -> PathBuf {
    shroudforge_loader_config::paths::export_dir(root)
        .join("world-editor")
        .join("screenshot-source.txt")
}

pub(crate) fn save_screenshot_folder(
    root: &Path,
    folder: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !folder.is_dir() {
        return Err("The selected screenshot folder is unavailable".into());
    }
    let path = screenshot_folder_file(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, folder.to_string_lossy().as_bytes())?;
    Ok(())
}

pub(crate) fn current_screenshot_folder(
    root: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let saved = screenshot_folder_file(root);
    if let Ok(value) = fs::read_to_string(&saved) {
        let folder = PathBuf::from(value.trim());
        if folder.is_dir() {
            return Ok(folder);
        }
    }
    if let Some(folder) = discover_screenshot_folder() {
        save_screenshot_folder(root, &folder)?;
        return Ok(folder);
    }
    Err("Choose the folder where your screenshots are stored".into())
}

pub(crate) fn discover_screenshot_folder() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        candidates.push(PathBuf::from(profile).join("Pictures/Screenshots/Enshrouded"));
    }
    let mut steam_roots = Vec::new();
    for variable in ["PROGRAMFILES(X86)", "PROGRAMFILES"] {
        if let Some(value) = std::env::var_os(variable) {
            steam_roots.push(PathBuf::from(value).join("Steam"));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        steam_roots.push(PathBuf::from(local).join("Programs/Steam"));
    }
    for steam in steam_roots {
        let userdata = steam.join("userdata");
        let Ok(accounts) = fs::read_dir(userdata) else {
            continue;
        };
        for account in accounts.flatten() {
            candidates.push(account.path().join("760/remote/1203620/screenshots"));
        }
    }
    candidates.into_iter().find(|path| {
        path.is_dir()
            && fs::read_dir(path).ok().is_some_and(|entries| {
                entries.flatten().any(|entry| {
                    entry.file_type().is_ok_and(|kind| kind.is_file())
                        && is_supported_image(&entry.path())
                })
            })
    })
}

pub(crate) fn load_screenshot_list(
    root: &Path,
    blueprint: &str,
) -> Result<ScreenshotList, Box<dyn std::error::Error>> {
    const MAX_SCREENSHOTS: usize = 120;
    if !valid_blueprint_name(blueprint) {
        return Err("Select a blueprint before choosing a screenshot".into());
    }
    let folder = current_screenshot_folder(root)?;
    let mut files = fs::read_dir(&folder)?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !entry.file_type().ok()?.is_file() || !is_supported_image(&path) {
                return None;
            }
            let modified = entry
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Some((path, modified))
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| right.1.cmp(&left.1));
    let truncated = files.len() > MAX_SCREENSHOTS;
    files.truncate(MAX_SCREENSHOTS);
    let mut screenshots = Vec::with_capacity(files.len());
    for (path, modified) in files {
        let Ok(image) = load_screenshot_image(&path, 320) else {
            continue;
        };
        let thumbnail = image.thumbnail(320, 180);
        let mut bytes = Cursor::new(Vec::new());
        thumbnail.write_to(&mut bytes, image::ImageFormat::Png)?;
        screenshots.push(ScreenshotCard {
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Screenshot")
                .to_owned(),
            path: path.to_string_lossy().into_owned(),
            modified,
            thumbnail: format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())
            ),
        });
    }
    Ok(ScreenshotList {
        folder: folder.to_string_lossy().into_owned(),
        screenshots,
        truncated,
        can_undo: shroudforge_loader_config::paths::export_dir(root)
            .join("world-editor/blueprints")
            .join(format!("{blueprint}.cover-undo.json"))
            .is_file(),
    })
}

pub(crate) fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "svg" | "png" | "jpg" | "jpeg"
            )
        })
}

pub(crate) fn load_screenshot_image(
    path: &Path,
    max_dimension: u32,
) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
    const MAX_RASTER_DIMENSION: u32 = 8192;
    const MAX_RASTER_PIXELS: u64 = 32 * 1024 * 1024;
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err("Image file is not a supported size".into());
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "svg" {
        let data = fs::read(path)?;
        let mut options = resvg::usvg::Options::default();
        options.fontdb_mut().load_system_fonts();
        let tree = resvg::usvg::Tree::from_data(&data, &options)?;
        let size = tree.size();
        let source_width = size.width();
        let source_height = size.height();
        let longest = source_width.max(source_height);
        if !longest.is_finite() || longest <= 0.0 {
            return Err("SVG has invalid dimensions".into());
        }
        let scale = (max_dimension.max(1) as f32 / longest).min(1.0);
        let width = (source_width * scale).ceil().max(1.0) as u32;
        let height = (source_height * scale).ceil().max(1.0) as u32;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
            .ok_or("Could not allocate the SVG preview")?;
        let transform = resvg::tiny_skia::Transform::from_scale(
            width as f32 / source_width,
            height as f32 / source_height,
        );
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        let pixels = RgbaImage::from_raw(width, height, pixmap.take())
            .ok_or("Could not read the rendered SVG pixels")?;
        return Ok(DynamicImage::ImageRgba8(pixels));
    }

    let reader = ImageReader::open(path)?.with_guessed_format()?;
    let format = reader
        .format()
        .ok_or("Image format could not be identified")?;
    let expected_format = match extension.as_str() {
        "png" => image::ImageFormat::Png,
        "jpg" | "jpeg" => image::ImageFormat::Jpeg,
        _ => return Err("Choose an SVG, PNG, JPG, or JPEG image".into()),
    };
    if format != expected_format {
        return Err("Image contents do not match the file extension".into());
    }
    let (width, height) = reader.into_dimensions()?;
    if width == 0
        || height == 0
        || width > MAX_RASTER_DIMENSION
        || height > MAX_RASTER_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_RASTER_PIXELS
    {
        return Err("Image dimensions exceed the 8192 by 8192 limit".into());
    }
    Ok(image::open(path)?)
}

pub(crate) fn apply_screenshot(root: &Path, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (name, source) = value
        .split_once('\t')
        .ok_or("Screenshot selection is incomplete")?;
    if !valid_blueprint_name(name) {
        return Err("The selected blueprint name is invalid".into());
    }
    let exports = shroudforge_loader_config::paths::export_dir(root);
    let blueprint_dir = exports.join("world-editor/blueprints");
    if !blueprint_dir.join(format!("{name}.sfbp")).is_file() {
        return Err("The selected blueprint no longer exists".into());
    }
    let folder = current_screenshot_folder(root)?.canonicalize()?;
    let source = PathBuf::from(source).canonicalize()?;
    if !source.starts_with(&folder) || !source.is_file() || !is_supported_image(&source) {
        return Err("Choose an image from the selected screenshot folder".into());
    }
    let image = load_screenshot_image(&source, 4096)?;
    fs::create_dir_all(&blueprint_dir)?;
    backup_cover(&blueprint_dir, name)?;
    let mut full = Cursor::new(Vec::new());
    image.write_to(&mut full, image::ImageFormat::Png)?;
    fs::write(blueprint_dir.join(format!("{name}.png")), full.into_inner())?;
    let thumbnail = image.thumbnail(960, 540);
    let mut thumb = Cursor::new(Vec::new());
    thumbnail.write_to(&mut thumb, image::ImageFormat::Png)?;
    fs::write(
        blueprint_dir.join(format!("{name}.thumb.png")),
        thumb.into_inner(),
    )?;
    Ok(())
}

pub(crate) fn backup_cover(directory: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cover = directory.join(format!("{name}.png"));
    let thumbnail = directory.join(format!("{name}.thumb.png"));
    let cover_backup = directory.join(format!("{name}.cover-undo.png"));
    let thumbnail_backup = directory.join(format!("{name}.cover-undo.thumb.png"));
    let had_cover = cover.is_file();
    let had_thumbnail = thumbnail.is_file();
    if had_cover {
        fs::copy(&cover, &cover_backup)?;
    } else {
        let _ = fs::remove_file(&cover_backup);
    }
    if had_thumbnail {
        fs::copy(&thumbnail, &thumbnail_backup)?;
    } else {
        let _ = fs::remove_file(&thumbnail_backup);
    }
    fs::write(
        directory.join(format!("{name}.cover-undo.json")),
        serde_json::to_vec(&serde_json::json!({
            "hadCover": had_cover,
            "hadThumbnail": had_thumbnail,
        }))?,
    )?;
    Ok(())
}

pub(crate) fn undo_screenshot(root: &Path, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !valid_blueprint_name(name) {
        return Err("The selected blueprint name is invalid".into());
    }
    let directory =
        shroudforge_loader_config::paths::export_dir(root).join("world-editor/blueprints");
    if !directory.join(format!("{name}.sfbp")).is_file() {
        return Err("The selected blueprint no longer exists".into());
    }
    let undo_path = directory.join(format!("{name}.cover-undo.json"));
    let undo: serde_json::Value = serde_json::from_slice(&fs::read(&undo_path)?)?;
    for (key, current_name, backup_name) in [
        (
            "hadCover",
            format!("{name}.png"),
            format!("{name}.cover-undo.png"),
        ),
        (
            "hadThumbnail",
            format!("{name}.thumb.png"),
            format!("{name}.cover-undo.thumb.png"),
        ),
    ] {
        let current = directory.join(current_name);
        let backup = directory.join(backup_name);
        if undo[key].as_bool().unwrap_or(false) {
            if !backup.is_file() {
                return Err("The previous cover backup is missing".into());
            }
            fs::copy(&backup, &current)?;
        } else if current.exists() {
            fs::remove_file(current)?;
        }
        let _ = fs::remove_file(backup);
    }
    fs::remove_file(undo_path)?;
    Ok(())
}
