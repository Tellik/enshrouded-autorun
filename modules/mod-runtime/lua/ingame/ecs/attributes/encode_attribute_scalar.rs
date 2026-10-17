use mlua::Value as LuaValue;
use serde_json::Value;

pub(super) fn encode_scalar(kind: &str, value: &LuaValue) -> Result<[u8; 4], String> {
    match (kind, value) {
        ("keen::sint32", LuaValue::Integer(v)) => Ok(i32::try_from(*v)
            .map_err(|_| "value outside sint32")?
            .to_le_bytes()),
        ("keen::uint32", LuaValue::Integer(v)) => Ok(u32::try_from(*v)
            .map_err(|_| "value outside uint32")?
            .to_le_bytes()),
        ("keen::float32", LuaValue::Number(v)) if v.is_finite() && (*v as f32).is_finite() => {
            Ok((*v as f32).to_le_bytes())
        }
        ("keen::float32", LuaValue::Integer(v)) => Ok((*v as f32).to_le_bytes()),
        _ => Err(format!("expected finite scalar of type {kind}")),
    }
}

pub(super) fn encode_attribute_scalar(
    descriptor: &Value,
    kind: &str,
    value: &LuaValue,
) -> Result<[u8; 4], String> {
    if descriptor["write_value_domain"] == "integer-common-range-0-to-2147483647"
        && !matches!(value, LuaValue::Integer(v) if (0..=i32::MAX as i64).contains(v))
    {
        return Err(
            "conflicting signedness: only the common integer range 0..2147483647 is validated"
                .into(),
        );
    }
    encode_scalar(kind, value)
}
