use crate::lua::LuaValue;
/// JSON objects are detached Lua tables, never mutable access to provider state.
pub(crate) fn json_to_lua(lua: &mlua::Lua, value: &serde_json::Value) -> mlua::Result<LuaValue> {
    use mlua::IntoLua;
    match value {
        serde_json::Value::Null => Ok(LuaValue::Nil),
        serde_json::Value::Bool(value) => value.into_lua(lua),
        serde_json::Value::String(value) => value.as_str().into_lua(lua),
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                value.into_lua(lua)
            } else if let Some(value) = value.as_u64() {
                Ok(LuaValue::Integer(value as i64))
            } else {
                value.as_f64().into_lua(lua)
            }
        }
        serde_json::Value::Array(values) => {
            let table = lua.create_table_with_capacity(values.len(), 0)?;
            for (i, value) in values.iter().enumerate() {
                table.raw_set(i + 1, json_to_lua(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        serde_json::Value::Object(values) => {
            let table = lua.create_table_with_capacity(0, values.len())?;
            for (key, value) in values {
                table.raw_set(key.as_str(), json_to_lua(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}
