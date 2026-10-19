use shroudforge_loader_config::{Document, Storage};
pub const CATALOG_STATE: Document = Document {
    name: "catalog-state",
    schema: include_str!("catalog.schema.json"),
    storage: Storage::State("catalog"),
};
pub const STATE: Document = Document {
    name: "state",
    schema: include_str!("updates.schema.json"),
    storage: Storage::State("updates"),
};
