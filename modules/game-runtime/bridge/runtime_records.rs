//! Shared runtime snapshot records for supported and unsupported platforms.

#[derive(Clone, Copy)]
pub struct Component {
    pub size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PropRecord {
    pub entity_handle: u32,
    pub item_id: u32,
    pub position: [i64; 3],
    pub orientation: [f32; 4],
    pub scale: [f32; 3],
    pub template_uuid: [u64; 2],
    pub entity_id: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PropRecipe {
    pub item_id: u32,
    pub bounds: [f32; 6],
    pub feedback: u32,
}

pub struct GridSpecResult {
    pub id: String,
    pub origin: [f64; 3],
    pub cell_size: [f64; 3],
    pub maximum: [u64; 3],
}
