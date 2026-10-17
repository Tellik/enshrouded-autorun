#[cfg(windows)]
#[derive(Debug)]
pub(crate) struct HeadlessControlAction {
    pub(crate) name: String,
    pub(crate) request_id: String,
    pub(crate) parameters: serde_json::Value,
}
