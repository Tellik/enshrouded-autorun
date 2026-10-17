use crate::runtime_resolution::attributes::read_header;
use crate::runtime_resolution::attributes::uint;
use kfc::guid::Guid;
use serde_json::Value;

/// Work on an owned snapshot. Reject relocated/external storage and mismatched
/// definitions instead of dereferencing any engine pointer from Lua.
pub(super) fn storage_range(
    descriptor: &Value,
    component: &Value,
    bytes: &[u8],
) -> Result<std::ops::Range<usize>, String> {
    if bytes.len() != uint(&component["size"])? as usize {
        return Err("attribute component size changed".into());
    }
    let header = &component["header"];
    if read_header(header, "rootId", bytes)? != uint(&descriptor["root_hash"])? {
        return Err("live attribute root ID differs from KFC definition".into());
    }
    if !component["definition_offset"].is_null() {
        let definition = uint(&component["definition_offset"])? as usize;
        let expected = Guid::parse(
            component["definition_guid"]
                .as_str()
                .ok_or("missing GUID")?,
        )
        .ok_or("invalid GUID")?;
        if bytes.get(definition..definition + 16) != Some(expected.data().as_slice()) {
            return Err("live attribute definition differs from KFC definition".into());
        }
    }
    let offset = read_header(header, "storageOffset", bytes)? as usize;
    // Both storageSize (KFC3) and storageCount (KFC4) count elements.
    let count = read_header(header, "storageCount", bytes)? as usize;
    let flags = read_header(header, "flags", bytes)?;
    // Current KFC4 native adders independently confirm the reflected bit with
    // test byte [component+6],1 at 0x21bdf5. Future layouts use their own enum.
    let initialized = uint(&header["flags"]["initialized_mask"])?;
    if initialized == 0
        || flags & initialized == 0
        || offset != uint(&component["storage_offset"])? as usize
        || count.checked_mul(4) != Some(uint(&component["storage_size"])? as usize)
    {
        return Err("attribute is uninitialized or uses an unverified storage layout".into());
    }
    let start = offset
        .checked_add(uint(&descriptor["index"])? as usize * 4)
        .ok_or("attribute offset overflow")?;
    if start + 4 > bytes.len() || start + 4 > offset + count * 4 {
        return Err("attribute element outside storage".into());
    }
    Ok(start..start + 4)
}
