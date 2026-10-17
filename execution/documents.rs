use shroudforge_loader_config::{Document, Storage};
pub const API: Document = Document { name: "api", schema: include_str!("shroudforge/v1/runtime-operations.schema.json"), storage: Storage::Config("api/api.json") };
