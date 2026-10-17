use super::cached_inline_field;
use crate::lua::LuaValue;
use kfc::reflection::{TypeIndex, TypeRegistry};
use std::rc::Rc;

/// Check caller expectations through the exact inline field codec used by writes.
/// The subsequent native CAS still compares the complete fresh component snapshot.
pub(crate) fn expect_inline_fields(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    component: TypeIndex,
    before: &[u8],
    expected: &mlua::Table,
) -> Result<(), String> {
    let mut candidate = before.to_vec();
    let mut mask = vec![0; before.len()];
    let mut count = 0;
    for pair in expected.clone().pairs::<String, LuaValue>() {
        let (path, value) = pair.map_err(|error| error.to_string())?;
        count += 1;
        if count > 1024 {
            return Err("at most 1024 expected field paths may be selected".into());
        }
        cached_inline_field(lua, registry, component, &path)?
            .write_into(lua, &value, &mut candidate, &mut mask)
            .map_err(|error| format!("expected {path}: {error}"))?;
    }
    if count == 0 {
        return Err("expected fields must be nonempty when provided".into());
    }
    if mask
        .iter()
        .enumerate()
        .any(|(index, selected)| *selected != 0 && candidate[index] != before[index])
    {
        return Err(
            "Selected field snapshot changed before update. Read current fields and retry".into(),
        );
    }
    Ok(())
}
