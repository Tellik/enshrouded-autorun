pub(crate) const MAX_TAIL: u64 = 1024 * 1024;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    #[serde(default = "default_toggle_key")]
    pub(crate) toggle_key: String,
    #[serde(default = "default_refresh")]
    pub(crate) refresh_milliseconds: u64,
    #[serde(default = "default_tail")]
    pub(crate) tail_bytes: u64,
}

pub(crate) fn default_toggle_key() -> String {
    shroudforge_loader_config::config::read_loader_defaults()
        .expect("embedded loader defaults must be valid")["modules"]["debugConsole"]["toggleKey"]
        .as_str()
        .expect("module hotkey default must be an SDL binding string")
        .to_owned()
}

pub(crate) fn default_refresh() -> u64 {
    500
}

pub(crate) fn default_tail() -> u64 {
    MAX_TAIL
}
