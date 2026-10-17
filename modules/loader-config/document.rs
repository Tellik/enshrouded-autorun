/// The schema and location belong to the feature that owns this document.
#[derive(Clone, Copy)]
pub struct Document {
    pub name: &'static str,
    pub schema: &'static str,
    pub storage: Storage,
}
#[derive(Clone, Copy)]
pub enum Storage {
    Package,
    Loader,
    State(&'static str),
    Config(&'static str),
}

impl Document {
    pub const fn state_section(self) -> Option<&'static str> {
        match self.storage {
            Storage::State(section) => Some(section),
            _ => None,
        }
    }
}
