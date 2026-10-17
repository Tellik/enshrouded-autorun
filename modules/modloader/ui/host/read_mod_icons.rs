use base64::Engine as _;
use shroudforge_package::package_files::read_package_file;
use std::path::Path;

pub(crate) fn read_mod_assets(
    package: &Path,
    icon: Option<&str>,
) -> std::collections::HashMap<String, String> {
    let mut paths = Vec::new();
    if let Some(icon) = icon {
        paths.push(icon.to_owned());
    }
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .filter_map(|relative| {
            if relative.contains("..") || relative.contains('\\') {
                return None;
            }
            let bytes = read_package_file(package, &relative)
                .ok()
                .filter(|bytes| bytes.len() <= 2 * 1024 * 1024)?;
            let mime = match Path::new(&relative)
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str()
            {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                _ => return None,
            };
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            Some((relative, format!("data:{mime};base64,{encoded}")))
        })
        .collect()
}
