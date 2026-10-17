use crate::{Document, Storage};
use std::path::{Path, PathBuf};
pub fn document_path(root: &Path, document: Document) -> PathBuf {
    match document.storage {
        Storage::Loader => crate::paths::loader_config(root),
        Storage::State(_) => crate::paths::state_file(root),
        Storage::Config(relative) => crate::paths::config_dir(root).join(relative),
        Storage::Package => crate::paths::config_dir(root).join("mod/mod.json"),
    }
}
