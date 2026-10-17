use std::borrow::Cow;

use kfc::guid::{ContentHash, Guid};
use mlua::ObjectLike;

use crate::{
    env::game::{Content, Resource},
    lua::{CastLua, CastLuaExt, LuaValue},
};

impl<'a> CastLua<'a> for Guid {
    type Output = Self;

    fn try_cast_lua(value: &'a LuaValue) -> mlua::Result<Option<Self::Output>> {
        match value {
            LuaValue::String(s) => {
                let s = s.to_str()?;
                let guid = match Self::parse(&s) {
                    Some(v) => v,
                    None => return Ok(None),
                };

                Ok(Some(guid))
            }
            LuaValue::UserData(ud) => {
                if ud.is::<Content>() {
                    let content = ud.borrow::<Content>()?;

                    return Ok(Some(content.guid().into_guid()));
                } else if ud.is::<Resource>() {
                    let resource = ud.borrow::<Resource>()?;

                    return Ok(Some(resource.info().resource_id.guid()));
                }

                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "guid".into()
    }
}

impl<'a> CastLua<'a> for ContentHash {
    type Output = Self;

    fn try_cast_lua(value: &'a LuaValue) -> mlua::Result<Option<Self::Output>> {
        macro_rules! some_or_return {
            ($e:expr) => {
                match $e {
                    Some(v) => v,
                    None => return Ok(None),
                }
            };
        }

        match value {
            LuaValue::UserData(ud) => {
                let size = ud.get::<LuaValue>("size")?.try_cast::<u32>()?;
                let hash0 = ud.get::<LuaValue>("hash0")?.try_cast::<u32>()?;
                let hash1 = ud.get::<LuaValue>("hash1")?.try_cast::<u32>()?;
                let hash2 = ud.get::<LuaValue>("hash2")?.try_cast::<u32>()?;

                let size = some_or_return!(size);
                let hash0 = some_or_return!(hash0);
                let hash1 = some_or_return!(hash1);
                let hash2 = some_or_return!(hash2);

                Ok(Some(Self::new(size, hash0, hash1, hash2)))
            }
            LuaValue::Table(table) => {
                let size = table.get::<LuaValue>("size")?.try_cast::<u32>()?;
                let hash0 = table.get::<LuaValue>("hash0")?.try_cast::<u32>()?;
                let hash1 = table.get::<LuaValue>("hash1")?.try_cast::<u32>()?;
                let hash2 = table.get::<LuaValue>("hash2")?.try_cast::<u32>()?;

                let size = some_or_return!(size);
                let hash0 = some_or_return!(hash0);
                let hash1 = some_or_return!(hash1);
                let hash2 = some_or_return!(hash2);

                Ok(Some(Self::new(size, hash0, hash1, hash2)))
            }
            LuaValue::String(s) => {
                let s = s.to_str()?;
                let guid = Self::parse(&s);
                let guid = some_or_return!(guid);

                Ok(Some(guid))
            }
            _ => Ok(None),
        }
    }

    fn name() -> Cow<'static, str> {
        "keen::ContentHash or guid".into()
    }
}
