use mlua::Value;
pub(crate) fn set_settings(
    lua: &mlua::Lua,
    id: &str,
    values: &serde_json::Value,
) -> mlua::Result<()> {
    let settings = values
        .as_object()
        .ok_or_else(|| mlua::Error::runtime("mod settings must be an object"))?;
    let table = lua.create_table_with_capacity(0, settings.len())?;
    for (key, value) in settings {
        table.raw_set(key.as_str(), json_to_lua(lua, value)?)?;
    }
    lua.set_named_registry_value(&format!("shroudforge.settings.{id}"), table)
}

fn json_to_lua<'lua>(lua: &'lua mlua::Lua, value: &serde_json::Value) -> mlua::Result<Value> {
    Ok(match value {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(value) => Value::Boolean(*value),
        serde_json::Value::Number(value) if value.is_i64() => {
            Value::Integer(value.as_i64().unwrap())
        }
        serde_json::Value::Number(value) => Value::Number(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => Value::String(lua.create_string(value)?),
        serde_json::Value::Array(values) => {
            let table = lua.create_table_with_capacity(values.len(), 0)?;
            for (index, value) in values.iter().enumerate() {
                table.raw_set(index + 1, json_to_lua(lua, value)?)?;
            }
            Value::Table(table)
        }
        _ => {
            return Err(mlua::Error::runtime(
                "setting value must be an array, string, number, boolean or null",
            ));
        }
    })
}

#[cfg(test)]
mod tests {
    use mlua::Value;

    use super::json_to_lua;

    #[test]
    fn converts_json_arrays_to_lua_sequences() {
        let lua = mlua::Lua::new();
        let input = serde_json::json!(["One", "Half", "CustomAmount"]);
        let Value::Table(table) = json_to_lua(&lua, &input).unwrap() else {
            panic!("array setting did not become a Lua table");
        };

        assert_eq!(table.raw_get::<String>(1).unwrap(), "One");
        assert_eq!(table.raw_get::<String>(2).unwrap(), "Half");
        assert_eq!(table.raw_get::<String>(3).unwrap(), "CustomAmount");
    }
}
