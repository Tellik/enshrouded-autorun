use crate::seconds;
use serde_json::{Value, json};
use std::path::Path;

pub fn status(root: &Path) -> Value {
    let result = shroudforge_loader_config::config::read_document(root, crate::documents::STATUS);
    match result {
        Ok(mut value) => {
            let fresh = value["updatedAt"]
                .as_u64()
                .is_some_and(|time| time <= seconds() && seconds() - time <= 65);
            value["fresh"] = fresh.into();
            if !fresh {
                value["active"] = false.into();
                value["reason"] = "process-not-reporting".into();
            }
            value
        }
        Err(error) => {
            json!({"active":false,"fresh":false,"reason":"no-runtime-report","detail":error})
        }
    }
}
