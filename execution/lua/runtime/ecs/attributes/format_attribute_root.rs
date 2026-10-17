use crate::env::runtime_metrics;
use serde_json::Value;
use serde_json::json;

pub(super) fn root_result(
    entries: &[&Value],
    scalar: &str,
    values: &[u32],
    trace: Vec<Value>,
) -> Result<Value, String> {
    let _timing = runtime_metrics::span("attributes.root_result");
    let mut named = serde_json::Map::new();
    for (entry, bits) in entries.iter().zip(values) {
        let value = match scalar {
            "keen::sint32" => json!(*bits as i32),
            "keen::uint32" => json!(*bits),
            "keen::float32" => {
                let value = f32::from_bits(*bits);
                if !value.is_finite() {
                    return Err("non-finite attribute value".into());
                }
                json!(value)
            }
            _ => return Err("unresolved attribute scalar".into()),
        };
        named.insert(entry["name"].as_str().unwrap().to_owned(), value);
    }
    Ok(
        json!({"root_hash":entries[0]["root_hash"],"values":named,"trace":trace,
        "calculation_model":"attribute-command-v1","scalar_type":scalar}),
    )
}
