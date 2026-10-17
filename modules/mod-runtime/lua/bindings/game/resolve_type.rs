//! Registry identity and traversal shared by reflection, value and ECS APIs.
use super::Type;
use crate::lua::LuaValue;
use kfc::reflection::{TypeIndex, TypeRegistry};

pub(crate) fn resolve(registry: &TypeRegistry, selector: &LuaValue) -> Result<TypeIndex, String> {
    let by_hash = |hash, kind| {
        by_hash(registry, hash, kind)?
            .map(|ty| ty.index)
            .ok_or_else(|| format!("type hash not found: 0x{hash:08x}"))
    };
    match selector {
        LuaValue::String(name) => {
            let name = name.to_str().map_err(|e| e.to_string())?;
            registry
                .iter()
                .find(|ty| ty.qualified_name == name.as_ref())
                .map(|ty| ty.index)
                .ok_or_else(|| format!("type not found: {name}"))
        }
        LuaValue::Integer(hash) if (0..=u32::MAX as i64).contains(hash) => {
            by_hash(*hash as u32, HashKind::Qualified)
        }
        LuaValue::Table(value) => {
            let hash = value.get::<u32>("hash").map_err(|e| e.to_string())?;
            let domain = value.get::<String>("domain").map_err(|e| e.to_string())?;
            by_hash(hash, HashKind::parse(&domain)?)
        }
        LuaValue::UserData(value) if value.is::<Type>() => {
            let ty = value.borrow::<Type>().map_err(|e| e.to_string())?;
            registry
                .get(ty.index)
                .filter(|current| {
                    current.qualified_hash == ty.qualified_hash
                        && current.qualified_name == ty.qualified_name
                })
                .map(|current| current.index)
                .ok_or_else(|| "type belongs to another registry".into())
        }
        _ => Err(
            "expected a qualified name, unsigned qualified hash, Type, or {hash, domain}".into(),
        ),
    }
}

pub(crate) use crate::runtime_resolution::types::{fields, inheritance};

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn fixture() -> TypeRegistry {
        let mut types = Vec::new();
        for (i, (name, kind, size, inner, flags)) in [
            ("U32", "UINT32", 4, None, ""),
            ("Base", "STRUCT", 4, None, ""),
            ("Child", "STRUCT", 8, Some(1), ""),
            ("Array", "STATIC_ARRAY", 8, Some(0), ""),
            ("DsArray", "DS_ARRAY", 24, Some(0), "HAS_DS"),
            ("HasDs", "STRUCT", 24, None, "HAS_DS"),
        ]
        .into_iter()
        .enumerate()
        {
            let qualified = format!("test::{name}");
            types.push(serde_json::json!({
                "index":i,"name":name,"qualifiedName":qualified,"impactName":name,
                "size":size,"alignment":4,"elementAlignment":4,"fieldCount":if i == 3 {2} else {0},
                "primitiveType":kind,"innerType":inner,"flags":flags,
                "nameHash":kfc::hash::fnv(name),"qualifiedHash":kfc::hash::fnv(&qualified),
                "impactHash":kfc::hash::fnv(name),"internalHash":99,
                "defaultValue":vec![0;size]
            }));
        }
        types[1]["structFields"] = serde_json::json!({"x":{"name":"x","type":0,"dataOffset":0}});
        types[2]["structFields"] = serde_json::json!({"y":{"name":"y","type":0,"dataOffset":4}});
        types[5]["structFields"] =
            serde_json::json!({"items":{"name":"items","type":4,"dataOffset":0}});
        types[2]["defaultValue"] = serde_json::json!([17, 0, 0, 0, 23, 0, 0, 0]);
        serde_json::from_value(serde_json::json!({"version":"fixture", "types":types})).unwrap()
    }

    #[test]
    fn inheritance_is_not_element_containment_and_fields_keep_owners() {
        let registry = fixture();
        assert_eq!(
            inheritance(&registry, TypeIndex::new(2)).unwrap(),
            [TypeIndex::new(2), TypeIndex::new(1)]
        );
        assert_eq!(
            inheritance(&registry, TypeIndex::new(3)).unwrap(),
            [TypeIndex::new(3)]
        );
        let fields = fields(&registry, TypeIndex::new(2)).unwrap();
        assert_eq!(
            fields
                .iter()
                .map(|(owner, f)| (owner.as_usize(), f.name.as_str(), f.data_offset))
                .collect::<Vec<_>>(),
            [(1, "x", 0), (2, "y", 4)]
        );
    }

    #[test]
    fn selectors_reject_collisions_wrong_domains_and_invalid_numbers() {
        let registry = fixture();
        let lua = mlua::Lua::new();
        let selector = lua.create_table().unwrap();
        selector.set("hash", 99).unwrap();
        selector.set("domain", "internal").unwrap();
        assert!(
            resolve(&registry, &LuaValue::Table(selector.clone()))
                .unwrap_err()
                .to_ascii_lowercase()
                .contains("ambiguous")
        );
        selector.set("domain", "address").unwrap();
        assert!(resolve(&registry, &LuaValue::Table(selector)).is_err());
        assert!(resolve(&registry, &LuaValue::Integer(-1)).is_err());
        assert!(resolve(&registry, &LuaValue::Integer(u32::MAX as i64 + 1)).is_err());
        let hash = kfc::hash::fnv("test::Child");
        assert_eq!(
            resolve(&registry, &LuaValue::Integer(hash as i64))
                .unwrap()
                .as_usize(),
            2
        );
    }

    #[test]
    fn cycles_are_reported_instead_of_looping() {
        let mut data = serde_json::to_value(fixture()).unwrap();
        data["types"][1]["innerType"] = serde_json::json!(2);
        let registry: TypeRegistry = serde_json::from_value(data).unwrap();
        assert!(
            inheritance(&registry, TypeIndex::new(2))
                .unwrap_err()
                .contains("cyclic")
        );
    }
}

use std::borrow::Borrow;

use kfc::reflection::{LookupKey, TypeHandle, TypeMetadata};

use crate::lua::{Args, Either, LuaError};

pub fn get_type<T>(args: &impl Args, index: usize, type_registry: T) -> mlua::Result<TypeHandle<T>>
where
    T: Borrow<TypeRegistry> + Clone,
{
    let r#type = args.get::<Either<String, &Type>>(index)?;
    let r#type = match r#type {
        Either::A(type_name) => type_registry
            .borrow()
            .get_by_name(LookupKey::Qualified(&type_name))
            .map(|t| TypeHandle::new(type_registry.clone(), t.index))
            .ok_or_else(|| LuaError::generic(format!("type '{type_name}' not found")))?,
        Either::B(type_data) => TypeHandle::new(type_registry, type_data.index),
    };

    Ok(r#type)
}

#[derive(Clone, Copy)]
pub(crate) enum HashKind {
    Qualified,
    Internal,
    Name,
    Impact,
}

impl HashKind {
    pub(crate) fn parse(name: &str) -> Result<Self, String> {
        match name {
            "qualified" => Ok(Self::Qualified),
            "internal" => Ok(Self::Internal),
            "name" => Ok(Self::Name),
            "impact" => Ok(Self::Impact),
            _ => Err(format!(
                "Unknown hash domain '{name}'. Expected qualified, internal, name or impact"
            )),
        }
    }

    pub(crate) fn value(self, ty: &TypeMetadata) -> u32 {
        match self {
            Self::Qualified => ty.qualified_hash,
            Self::Internal => ty.internal_hash,
            Self::Name => ty.name_hash,
            Self::Impact => ty.impact_hash,
        }
    }
}

/// Internal/layout hashes are frequently shared. Never choose the first collision.
pub(crate) fn by_hash(
    registry: &TypeRegistry,
    hash: u32,
    kind: HashKind,
) -> Result<Option<&TypeMetadata>, String> {
    unique_match(registry.iter().filter(|ty| kind.value(ty) == hash)).map_err(|()| {
        format!("Ambiguous type hash 0x{hash:08x}. Use a qualified name or qualified hash")
    })
}

fn unique_match<T>(mut values: impl Iterator<Item = T>) -> Result<Option<T>, ()> {
    let first = values.next();
    if values.next().is_some() {
        Err(())
    } else {
        Ok(first)
    }
}

#[cfg(test)]
mod lookup_tests {
    use super::*;

    #[test]
    fn hash_collision_never_selects_an_arbitrary_type() {
        assert_eq!(unique_match(["client", "server"].into_iter()), Err(()));
        assert_eq!(unique_match(["exact"].into_iter()), Ok(Some("exact")));
        assert_eq!(unique_match(std::iter::empty::<&str>()), Ok(None));
    }

    #[test]
    fn qualified_hash_resolves_while_shared_internal_hash_is_rejected() {
        let types: Vec<_> = (0..2)
            .map(|index| {
                serde_json::json!({
                    "index": index, "name": format!("Type{index}"),
                    "impactName": format!("ecs.Type{index}"),
                    "qualifiedName": format!("keen::ecs::Type{index}"),
                    "size": 4, "alignment": 4, "elementAlignment": 4,
                    "fieldCount": 0, "primitiveType": "STRUCT", "flags": "",
                    "nameHash": 100 + index, "impactHash": 200 + index,
                    "qualifiedHash": 300 + index, "internalHash": 777
                })
            })
            .collect();
        let registry: TypeRegistry = serde_json::from_value(serde_json::json!({
            "version": "test", "types": types
        }))
        .unwrap();
        assert_eq!(
            by_hash(&registry, 301, HashKind::Qualified)
                .unwrap()
                .unwrap()
                .qualified_name,
            "keen::ecs::Type1"
        );
        assert!(
            by_hash(&registry, 302, HashKind::Qualified)
                .unwrap()
                .is_none()
        );
        assert!(
            by_hash(&registry, 777, HashKind::Internal)
                .unwrap_err()
                .to_ascii_lowercase()
                .contains("ambiguous")
        );
    }
}
