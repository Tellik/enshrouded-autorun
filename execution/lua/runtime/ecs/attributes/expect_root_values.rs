use super::Catalog;
use super::encode_attribute_scalar;
use crate::runtime_resolution::attributes::uint;
use mlua::Table;
use mlua::Value as LuaValue;

/// Bind decisions from an earlier Lua read to this fresh root snapshot. The
/// native compare-exchange then guards the complete component until mutation.
pub(super) fn expect_root_values(
    catalog: &Catalog,
    root: u32,
    scalar: &str,
    values: &[u32],
    expected: &Table,
) -> Result<(), String> {
    let mut count = 0;
    for pair in expected.clone().pairs::<LuaValue, LuaValue>() {
        let (selector, value) = pair.map_err(|e| e.to_string())?;
        if !matches!(selector, LuaValue::String(_)) {
            return Err("expected_values keys must be original attribute names".into());
        }
        let descriptor = catalog.get(&selector)?;
        if uint(&descriptor["root_hash"])? != root {
            return Err("expected attribute belongs to a different root".into());
        }
        let index = uint(&descriptor["index"])? as usize;
        let expected_bits =
            u32::from_le_bytes(encode_attribute_scalar(descriptor, scalar, &value)?);
        if values.get(index) != Some(&expected_bits) {
            return Err(
                "Attribute snapshot changed before update. Read current values and retry".into(),
            );
        }
        count += 1;
        if count > values.len() {
            return Err("too many expected root attributes".into());
        }
    }
    if count == 0 {
        return Err("expected_values must contain at least one attribute".into());
    }
    Ok(())
}
