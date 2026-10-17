use super::encode_attribute_scalar;
use mlua::Value as LuaValue;
use serde_json::Value;

pub(super) fn validate_attribute_bits(
    descriptor: &Value,
    scalar: &str,
    bits: u32,
) -> Result<(), String> {
    let value = match scalar {
        "keen::sint32" => LuaValue::Integer(i64::from(bits as i32)),
        "keen::uint32" => LuaValue::Integer(i64::from(bits)),
        "keen::float32" => LuaValue::Number(f64::from(f32::from_bits(bits))),
        _ => return Err("unsupported attribute scalar".into()),
    };
    encode_attribute_scalar(descriptor, scalar, &value).map(|_| ())
}
