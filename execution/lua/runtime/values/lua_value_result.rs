use crate::lua::LuaValue;

pub(super) fn outcome(result: Result<LuaValue, String>) -> (Option<LuaValue>, Option<String>) {
    match result {
        Ok(value) => (Some(value), None),
        Err(reason) => (None, Some(reason)),
    }
}
