use shroudforge_loader_config::{Document, Storage};
pub const NEWS: Document = Document {
    name: "news",
    schema: include_str!("news-schema.json"),
    storage: Storage::Config("news/news.json"),
};
pub const EVENT: Document = Document {
    name: "event",
    schema: include_str!("event-schema.json"),
    storage: Storage::Config("event/event.json"),
};
pub const NEWS_STATE: Document = Document {
    name: "news-state",
    schema: include_str!("read-state-schema.json"),
    storage: Storage::State("news"),
};
pub const EVENTS_STATE: Document = Document {
    name: "events-state",
    schema: include_str!("events-state-schema.json"),
    storage: Storage::State("events"),
};
pub const MOD_STATE: Document = Document {
    name: "mod-state",
    schema: include_str!("mods.schema.json"),
    storage: Storage::State("mods"),
};
