#include "loaded-resources/loaded_resource_state.h"

using namespace ResourceImplementation;

namespace ResourceImplementation {
bool snapshot_arrays(const Layout& layout, const View& view,
    std::vector<unsigned char>& keys, std::vector<unsigned char>& values,
    std::vector<unsigned char>& chunks){
    const auto key_bytes = static_cast<std::size_t>(view.key_count) * layout.key_stride;
    const auto value_bytes = static_cast<std::size_t>(view.value_count) * layout.value_stride;
    const auto chunk_bytes = static_cast<std::size_t>(view.chunk_count) * layout.chunk_stride;
    keys.resize(key_bytes);
    values.resize(value_bytes);
    chunks.resize(chunk_bytes);
    return copy_memory(view.keys, keys.data(), keys.size()) &&
        copy_memory(view.values, values.data(), values.size()) &&
        copy_memory(view.chunks, chunks.data(), chunks.size());
}

bool list_type(const Layout& layout, const View& view, std::uint32_t type_hash,
    std::vector<KfcRuntimeResourceId>& result){
    std::vector<unsigned char> keys, values, chunks;
    if (!snapshot_arrays(layout, view, keys, values, chunks)) return false;
    std::set<std::array<unsigned char, 24>> identities;
    // Reuse page-region observations only within this stable listing. Check
    // every selected span, so holes elsewhere in a chunk cannot hide a readable
    // resource. Actual reads copy/recheck payloads with RPM again.
    RegionCache regions;
    result.clear();
    for (std::size_t index = 0; index < view.key_count; ++index) {
        const auto* key = keys.data() + index * layout.key_stride;
        std::uint32_t key_type{};
        std::memcpy(&key_type, key + 16, sizeof(key_type));
        if (key_type != type_hash) continue;
        std::array<unsigned char, 24> identity{};
        std::memcpy(identity.data(), key, identity.size());
        if (!identities.insert(identity).second) return false;
        std::uint32_t part{};
        std::memcpy(&part, key + 20, sizeof(part));
        const auto* entry = values.data() + index * layout.value_stride;
        // Validate the indexed span without calling into the game. The later
        // ResourceRead copies and revalidates the selected payload itself.
        std::uint32_t offset{}, packed{};
        std::memcpy(&offset, entry + layout.resource_offset_offset, sizeof(offset));
        std::memcpy(&packed, entry + layout.packed_size_offset, sizeof(packed));
        const auto chunk_id = entry[layout.chunk_index_offset];
        const auto size = static_cast<std::size_t>(packed & static_cast<std::uint32_t>(layout.size_mask));
        if (size > layout.maximum_resource_bytes || chunk_id >= view.chunk_count) continue;
        const auto* chunk = chunks.data() + static_cast<std::size_t>(chunk_id) * layout.chunk_stride;
        std::uintptr_t chunk_base{};
        std::uint64_t chunk_size{};
        std::memcpy(&chunk_base, chunk + layout.chunk_base_pointer_offset, sizeof(chunk_base));
        std::memcpy(&chunk_size, chunk + layout.chunk_size_offset, sizeof(chunk_size));
        if (!chunk_base || offset > chunk_size || size > chunk_size - offset) continue;
        std::uintptr_t data{};
        if (!add(chunk_base, offset, data) || size > UINTPTR_MAX - data) continue;
        if (!readable(data, size, regions)) continue;
        KfcRuntimeResourceId id{};
        std::memcpy(id.guid, key, sizeof(id.guid));
        id.type_hash = key_type;
        id.part_index = part;
        result.push_back(id);
    }
    View after{};
    std::vector<unsigned char> keys_after, values_after, chunks_after;
    return resolve_view(layout, after) && same_view(view, after) &&
        snapshot_arrays(layout, after, keys_after, values_after, chunks_after) &&
        keys == keys_after && values == values_after && chunks == chunks_after;
}
}

namespace ResourceRuntime {
bool ResourcesByType(std::uint32_t type_hash, KfcRuntimeResourceId* resources,
    std::size_t capacity, std::size_t* count) {
    if (!count || (!resources && capacity)) return false;
    *count = 0;
    Layout layout;
    {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded) return false;
        layout = active_layout;
    }
    View view{};
    if (!resolve_view(layout, view) || !registry_is_bound(layout, view)) return false;
    std::vector<KfcRuntimeResourceId> result;
    if (!list_type(layout, view, type_hash, result)) return false;
    *count = result.size();
    if (!resources && capacity == 0) return true;
    if (capacity < result.size()) return false;
    std::copy(result.begin(), result.end(), resources);
    return true;
}
}

extern "C" bool __cdecl KfcRuntimeResourcesByType(std::uint32_t type_hash,
    KfcRuntimeResourceId* resources, std::size_t capacity, std::size_t* count) {
    try { return ResourceRuntime::ResourcesByType(type_hash, resources, capacity, count); }
    catch (...) { if (count) *count = 0; return false; }
}
