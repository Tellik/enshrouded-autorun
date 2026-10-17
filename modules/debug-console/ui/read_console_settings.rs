use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_F10;

pub(crate) const MAX_TAIL: u64 = 1024 * 1024;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    #[serde(default = "default_toggle_key")]
    pub(crate) toggle_key: u32,
    #[serde(default = "default_refresh")]
    pub(crate) refresh_milliseconds: u64,
    #[serde(default = "default_tail")]
    pub(crate) tail_bytes: u64,
}

pub(crate) fn default_toggle_key() -> u32 {
    VK_F10 as u32
}

pub(crate) fn default_refresh() -> u64 {
    500
}

pub(crate) fn default_tail() -> u64 {
    MAX_TAIL
}
