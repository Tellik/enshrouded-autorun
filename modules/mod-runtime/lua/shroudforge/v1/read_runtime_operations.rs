use serde_json::Value;
use std::sync::LazyLock;
static CATALOG: LazyLock<Result<Value, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("runtime-operations.json")).map_err(|error| error.to_string())
});
pub(crate) fn catalog() -> Result<&'static Value, String> {
    CATALOG.as_ref().map_err(Clone::clone)
}
pub(crate) fn operations() -> Result<Vec<String>, String> {
    serde_json::from_value(catalog()?["runtimeOperations"].clone())
        .map_err(|error| error.to_string())
}
