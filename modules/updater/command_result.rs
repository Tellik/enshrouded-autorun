pub struct AsyncUiResult {
    pub request_id: String,
    pub source: String,
    pub action: String,
    pub success_message: String,
    pub result: Result<String, String>,
}
