---@meta

-- TODO: implement `Resource::original_data`

--- Represents a specific resource in the game, identified by its guid, type, and part.
---
--- @class Resource
--- @field guid Guid Unique identifier of this game resource.
--- @field type Type Reflected data layout used to interpret the resource data.
--- @field part u32 -- The part number of the asset, which is used for assets that are split into multiple parts.
--- @field source string `kfc-file`, `asset-transaction`, or `loaded-runtime`.
--- @field loaded boolean True when the resource came from the game's live ResourceService.
--- @field session_id integer? Creation world session for loaded-runtime snapshots, zero for menu snapshots, nil for file/transaction resources.
--- @field data unknown -- The data of the asset whose structure is defined by the `type` field.
--- @field original_data unknown -- A read-only reference to the original data of the asset.
local Resource = {}
