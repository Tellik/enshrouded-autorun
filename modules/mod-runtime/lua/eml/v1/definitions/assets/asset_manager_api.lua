--- @meta

--- The asset processor is used to access and modify game assets before the game starts.
---
---@class AssetManager
local AssetManager = {}

--- Returns an resource by its guid, type, and part.
---
--- @param guid Guid -- The unique identifier of the asset.
--- @param type string|Type -- The qualified type name of the asset, such as `keen::RenderModel`.
--- @param part u32? -- The part number of the asset, which is used for assets that are split into multiple parts. If not specified, defaults to 0.
--- @return Resource? -- The resource containing information and data about the asset.
function AssetManager.get_resource(guid, type, part) end

--- Returns a list of all resource's parts with the given guid and type.
---
--- @param guid Guid -- The unique identifier of the asset.
--- @param type string|Type -- The qualified type name of the asset, such as `keen::RenderModel`.
--- @return Resource[] -- A sorted list of resources for each part of the asset.
function AssetManager.get_resource_parts(guid, type) end

--- Returns a list of all resources with the given type.
---
--- @param type string|Type -- The qualified type name of the asset, such as `keen::RenderModel`.
--- @return Resource[] -- A list of resources for the specified type.
function AssetManager.get_resources_by_type(type) end

--- Returns one resource from the game's live ResourceService by guid, type, and part.
--- This function is available in the ingame phase and returns read-only resource data.
--- The complete checked snapshot is captured by this call. Reading data decodes those same owned bytes.
--- The returned session_id binds writes to the creation world. Menu snapshots have session_id=0 and cannot be written.
---
--- @param guid Guid -- The unique identifier of the resource.
--- @param type string|Type -- The qualified type name of the resource.
--- @param part u32? -- The part number, defaulting to 0.
--- @return Resource? -- The matching live resource, if it is loaded.
function AssetManager.get_loaded_resource(guid, type, part) end

--- Returns live resources from the game's ResourceService that match the given type.
--- This function is available in the ingame phase and returns read-only resource data.
---
--- @param type string|Type -- The qualified type name of the resources.
--- @return Resource[] -- The matching live resources.
function AssetManager.get_loaded_resources_by_type(type) end

---@class LoadedResourceIndexProgress
---@field session_id integer Captured world session, including zero for read-only menu lookup.
---@field scanned integer Resource scalar keys indexed so far.
---@field total integer ResourceIds in the captured type listing.
---@field payload_reads integer Full candidate snapshots read by this lookup. Discovery work belongs to the shared queue.
---@field complete boolean Whether the whole captured listing has been indexed.
---@field phase 'indexing'|'confirming'|'complete' Retry indexing and confirming from a later update.

--- Resolve a unique loaded resource by one reflected scalar field, without a per-mod index.
--- Ingame phase only. Menu session zero is readable and never authorizes writes.
--- The field must be fixed inline storage, aligned and exactly 32 bits: integer, finite float or enum.
--- Nested structs, typedefs and one-based static array elements are supported. Blob/DS traversal and references are rejected.
--- Values use the existing typed codec, including enum names and exact Float32 bits. Positive and negative zero are distinct.
--- The first call registers a job. The runtime advances it independently of later Lua calls.
--- Shared maintenance uses up to 64 scalar keys per native batch, with checked payload fallback where needed.
--- All jobs share a cooperative 1 ms and 1024-key budget per runtime round. Synchronous listing can overrun it.
--- No candidate is returned before the entire captured type listing has been indexed.
--- Retry while progress.phase is indexing or confirming. Complete missing and ambiguous results return nil and a reason.
--- All mods share at most 16 indexes keyed by session, type and path. Confirmed writes invalidate overlapping key fields only.
--- A conservative 16 MiB admission limit bounds the index jobs. Capacity errors preserve other active jobs.
--- World changes and uncertain writes invalidate conservatively. Live pointers are never retained by the index.
--- Missing and ambiguous indexes refresh after two seconds. A changed candidate discards its index immediately.
--- A successful lookup rereads the candidate and returns a Resource preserving those confirmed bytes and captured session.
--- Uniqueness refers to the completed index snapshot. Concurrent resource additions or changes elsewhere are not a transaction.
---@param type string|Type Qualified resource type name or reflected Type.
---@param field_path string Fixed inline scalar path, at most 1024 bytes.
---@param value integer|number|string Scalar value, or an original enum name.
---@return Resource? resource Freshly confirmed unique candidate.
---@return string? reason Pending, missing, ambiguous or unavailable explanation.
---@return LoadedResourceIndexProgress? progress Absent when validation or resource access fails.
function AssetManager.get_loaded_resource_by_field(type, field_path, value) end

---Register a shared field index before an action needs its first lookup.
---The existing runtime loop advances it without repeated Lua calls, including after a world becomes ready.
---Registrations belong to the active mod. Matching type/path requests share one bounded job.
---World changes reset indexed values and retain declarations. Unloading a mod removes only its own registration.
---A false ready value may mean normal pending preparation. It does not require repeated calls to advance work.
---Ready describes the captured read snapshot. Session zero never authorizes writes.
---@param type string|Type
---@param field_path string Fixed inline aligned 32-bit scalar path.
---@return boolean ready
---@return string? reason
---@return LoadedResourceIndexProgress? progress
function AssetManager.prepare_loaded_index(type, field_path) end

---Resolve a canonical entry in an independently validated original engine registry.
---Currently supports g38_items::ItemRegistry with a nonzero unsigned 32-bit ItemId key.
---This selects the original resource-owner registry entry, not global field uniqueness.
---Requires the current nonzero authoritative session and an exact validated provider contract.
---Returns a normal immutable Resource snapshot using the common writers and claims.
---@param registry_name string
---@param key integer
---@return Resource? resource
---@return string? reason
function AssetManager.resolve_loaded_resource(registry_name, key) end

---Read one aligned 32-bit scalar and its reflected type from an immutable loaded resource snapshot.
---Uses the exact field codec as write_loaded_field, including owned BlobArray indices and BlobVariant.value.
---This performs no new process read. In particular, it can inspect the writer's confirmed returned snapshot.
---@param resource Resource Loaded-runtime snapshot.
---@param field_path string
---@return unknown? value May be false or zero.
---@return string? reason
---@return Type? type Exact reflected scalar type for encode/decode.
function AssetManager.read_loaded_field(resource, field_path) end

--- Atomically update one reflected 32-bit scalar in a loaded resource.
--- Requires runtime.assets.write, runtime.ecs.write authority and an active ingame mod with Runtime capability.
--- The loaded Resource must originate from the same nonzero current world session. Menu and prior-world snapshots cannot write.
--- Paths use reflected field names, one-based owned BlobArray indices and BlobVariant.value.
--- Only one aligned float, integer or 32-bit enum is supported. References and container descriptors are rejected.
--- The current scalar is compared again before writing. Without expected, this Resource's complete immutable source snapshot is required to match.
--- An explicit expected value checks caller intent instead of that source scalar.
--- The native original engine gates also compare the complete fresh resource snapshot, preserving the selected variant and descriptor layout.
--- Success includes an independent complete native readback under the original engine gates. The returned Resource preserves those verified bytes.
--- Existing Resource snapshots remain unchanged.
--- This does not publish notifications or guarantee that the engine consumes this field.
--- A genuinely uncertain native outcome blocks public writes in the original session. Stop retries and restoration.
--- A later independent observation or world change cannot turn that confirmed native result into a failed write.
---@param resource Resource A loaded-runtime Resource from the current ResourceService.
---@param field_path string
---@param value unknown
---@param expected unknown? Optional expected current scalar.
---@return Resource? updated New Resource preserving the independently verified snapshot.
---@return string? reason
function AssetManager.write_loaded_field(resource, field_path, value, expected) end

--- Returns a list of all resources in the game.
---
--- @return Resource[] -- A list of all resources in the game.
function AssetManager.get_all_resources() end

--- Returns a list of all resource types in the game.
---
--- @return Type[] -- A list of all resource types in the game.
function AssetManager.get_resource_types() end

--- Creates a new resource with the specified value and type.
---
--- The returned resource will contain a newly generated guid which can be used
--- to reference this resource in other assets.
--- The part number of the new resource will be 0.
---
--- ### Errors
--- - When the specified type is not registered in the game.
--- - When the provided value is not compatible with the specified type.
---
--- @param value any -- The value to be stored in the resource. This must be compatible with the specified type.
--- @param type string|Type -- The qualified type name of the asset, such as `keen::RenderModel`.
--- @return Resource
function AssetManager.create_resource(value, type) end

--- Creates a new resource with the specified value, type, part, and guid.
---
--- This method is primarily used for creating resources which are split into multiple parts.
--- You first create a new resource with `AssetManager.create_resource` to get a new guid,
--- then use that guid to create additional parts of the same resource.
---
--- ### Errors
--- - When the specified type is not registered in the game.
--- - When the provided value is not compatible with the specified type.
--- - When the provided guid and part is not unique and is already used by another asset.
---
--- @param value any -- The value to be stored in the resource. This must be compatible with the specified type.
--- @param type string|Type -- The qualified type name of the asset, such as `keen::RenderModel`.
--- @param guid Guid -- The guid to assign to the new resource.
--- @param part u32 -- The part number of the asset.
--- @return Resource
function AssetManager.create_resource(value, type, guid, part) end

--- Returns a content by its guid.
---
--- @param guid Guid|keen.ContentHash -- The unique identifier of the asset.
--- @return Content? -- A content resource containing information about and the raw data of the asset.
function AssetManager.get_content(guid) end

--- Returns a list of all contents.
---
--- @return Content[] -- A list of all content resources in the game.
function AssetManager.get_all_contents() end

--- Creates a new content with the specified data.
---
--- The returned content will contain a newly generated guid which can be used
--- to reference this content in other assets.
---
--- @param data Buffer -- The raw binary data to be stored in the content.
--- @return Content
function AssetManager.create_content(data) end
