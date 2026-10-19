use shroudforge_loader_config::{Document, Storage};
pub const STATUS: Document = Document {
    name: "diagnostics-status",
    schema: include_str!("diagnostics-status.schema.json"),
    storage: Storage::State("diagnostics"),
};

pub const MOD_STATUS: Document = Document {
    name: "mod-status",
    schema: include_str!("runtime-mod.schema.json"),
    storage: Storage::State("runtime"),
};
