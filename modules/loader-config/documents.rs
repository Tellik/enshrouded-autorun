use crate::{Document, Storage};
pub const LOADER: Document = Document {
    name: "shroudforge",
    schema: include_str!("loader.schema.json"),
    storage: Storage::Loader,
};
pub const WINDOW_STATE: Document = Document {
    name: "window-state",
    schema: include_str!("windows.schema.json"),
    storage: Storage::State("windows"),
};
