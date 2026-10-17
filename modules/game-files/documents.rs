use shroudforge_loader_config::{Document, Storage};
pub const APPLIED: Document = Document {
    name: "applied",
    schema: include_str!("asset-preparation.schema.json"),
    storage: Storage::State("assets"),
};
pub const PARSER_STATUS: Document = Document {
    name: "parser-status",
    schema: include_str!("parser-status.schema.json"),
    storage: Storage::State("parser"),
};
