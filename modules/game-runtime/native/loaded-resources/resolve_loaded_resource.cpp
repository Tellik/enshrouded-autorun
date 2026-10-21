#include "loaded-resources/loaded_resource_state.h"

using namespace ResourceImplementation;

namespace ResourceImplementation {
bool readable(std::uintptr_t address, std::size_t size, RegionCache& regions){
    if (!size) return true;
    if (!address || size > std::numeric_limits<std::uintptr_t>::max() - address) return false;
    const auto end = address + size;
    auto cursor = address;
    while (cursor < end) {
        MEMORY_BASIC_INFORMATION info{};
        auto known = regions.upper_bound(cursor);
        bool cached{};
        if (known != regions.begin()) {
            --known;
            const auto begin = known->first;
            if (known->second.RegionSize <= UINTPTR_MAX - begin && cursor < begin + known->second.RegionSize) {
                info = known->second;
                cached = true;
            }
        }
        if (!cached) {
            if (!VirtualQuery(reinterpret_cast<const void*>(cursor), &info, sizeof(info))) return false;
            regions.insert_or_assign(reinterpret_cast<std::uintptr_t>(info.BaseAddress), info);
        }
        if (info.State != MEM_COMMIT || (info.Protect & (PAGE_GUARD | PAGE_NOACCESS)) ||
        !([&] {
            const auto protection = info.Protect & 0xff;
            return protection == PAGE_READONLY || protection == PAGE_READWRITE ||
                protection == PAGE_WRITECOPY || protection == PAGE_EXECUTE_READ ||
                protection == PAGE_EXECUTE_READWRITE || protection == PAGE_EXECUTE_WRITECOPY;
        })()) return false;
        const auto region = reinterpret_cast<std::uintptr_t>(info.BaseAddress);
        if (info.RegionSize > std::numeric_limits<std::uintptr_t>::max() - region) return false;
        const auto region_end = region + info.RegionSize;
        if (cursor < region || cursor >= region_end) return false;
        cursor = (std::min)(end, region_end);
    }
    return true;
}

bool copy_memory(std::uintptr_t address, void* destination, std::size_t size){
    if (!size) return true;
    // ReadProcessMemory already rejects unreadable/unmapped ranges. Repeating
    // VirtualQuery for each scalar walks large engine allocations at high cost.
    if (!address || !destination || size > std::numeric_limits<std::uintptr_t>::max() - address) return false;
    SIZE_T actual{};
    return ReadProcessMemory(GetCurrentProcess(), reinterpret_cast<const void*>(address),
        destination, size, &actual) && actual == size;
}

// Caller buffers are copied only before or after original engine gates. The
// count is published last, so failed output copies never publish partial rows.
bool set_batch_count(std::size_t* actual, std::size_t count) noexcept{
    __try { *actual = count; return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool publish_scalar_batch(const std::uint32_t* values, std::size_t count,
    std::uint32_t* output, std::size_t* actual) noexcept{
    __try {
        std::memcpy(output, values, count * sizeof(std::uint32_t));
        *actual = count;
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool publish_resource_id(const KfcRuntimeResourceId& value, KfcRuntimeResourceId* output) noexcept{
    __try { std::memcpy(output, &value, sizeof(value)); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& result){
    if (!base || offset > std::numeric_limits<std::uintptr_t>::max() - base) return false;
    result = base + offset;
    return true;
}

bool resolve_view(const Layout& layout, View& view, std::string* reason,
    const EngineContext::Owners* guarded_owners){
    auto fail = [reason](const char* text) {
        if (reason) *reason = text;
        return false;
    };
    EngineContext::Owners owners;
    if (guarded_owners) owners = *guarded_owners;
    else if (!EngineContext::ResolveOwners(owners, reason)) return false;
    if (owners.dedicated_server != layout.dedicated_server) return fail("resource-owner-target-mismatch");
    const auto image = owners.image;
    view = {};
    view.image = image;
    view.root = owners.root;
    view.client = owners.client;
    view.controller = owners.controller;
    view.main_owner = owners.main_owner;
    // MainOwner and the context are allocator-backed data objects, not proven
    // polymorphic classes. Their first word is not a vtable contract.
    std::uintptr_t active_context_address{};
    const auto resource_owner = layout.dedicated_server ? owners.world_context : view.main_owner;
    if (!add(resource_owner, layout.main_active_context_offset, active_context_address) ||
        !read_value(active_context_address, view.context))
        return fail("active-context-unavailable");
    std::uintptr_t registry_address{};
    if (!add(view.context, layout.context_item_registry_offset, registry_address) ||
        !read_value(registry_address, view.registry) || !view.registry)
        return fail("item-registry-pointer-unavailable");

    std::uintptr_t service_address{};
    std::uintptr_t registry_resource_address{};
    std::uint32_t registry_handle{};
    if (!add(view.registry, layout.item_registry_service_offset, service_address) ||
        !read_value(service_address, view.service) ||
        !add(view.registry, layout.item_registry_handle_offset, service_address) ||
        !read_value(service_address, registry_handle) || !registry_handle ||
        !add(view.registry, layout.item_registry_resource_pointer_offset, registry_resource_address))
        return fail("item-registry-service-unavailable");
    if (!read_value(registry_resource_address, view.registry_resource) || !view.registry_resource)
        return fail("item-registry-resource-unavailable");
    // The original constructor installs these two callbacks in ResourceService.
    // Validate the service by its actual contract without invoking either one.
    std::uintptr_t callback_slot{}, callback{};
    if (!add(view.service, layout.resource_service_id_callback_offset, callback_slot) ||
        !read_value(callback_slot, callback) || callback != image + layout.resource_service_id_callback_rva ||
        !add(view.service, layout.resource_service_handle_callback_offset, callback_slot) ||
        !read_value(callback_slot, callback) || callback != image + layout.resource_service_handle_callback_rva)
        return fail("resource-service-callback-mismatch");

    std::uintptr_t reader_address{};
    if (!add(view.service, layout.resource_service_reader_offset, reader_address) ||
        !read_value(reader_address, view.reader)) return fail("resource-reader-unavailable");
    std::uintptr_t metadata_address{};
    if (!add(view.reader, layout.reader_metadata_offset, metadata_address) ||
        !read_value(metadata_address, view.metadata) ||
        !add(view.metadata, layout.metadata_map_offset, view.map))
        return fail("resource-index-metadata-unavailable");
    if (layout.map_header_size > UINTPTR_MAX - view.map) return fail("resource-index-header-range-overflow");

    std::uintptr_t field{};
    if (!add(view.map, layout.bucket_rel32_offset, field) || !relative_pointer(field, view.buckets) ||
        !add(view.map, layout.keys_rel32_offset, field) || !relative_pointer(field, view.keys) ||
        !add(view.map, layout.values_rel32_offset, field) || !relative_pointer(field, view.values) ||
        !read_value(view.map + layout.bucket_count_offset, view.bucket_count) ||
        !read_value(view.map + layout.key_count_offset, view.key_count) ||
        !read_value(view.map + layout.value_count_offset, view.value_count))
        return fail("resource-index-header-invalid");
    if (!view.buckets || !view.keys || !view.values || !view.bucket_count ||
        (view.bucket_count & (view.bucket_count - 1)) ||
        view.bucket_count > layout.maximum_entries || !view.key_count ||
        view.key_count != view.value_count || view.key_count > layout.maximum_entries)
        return fail("resource-index-counts-or-relative-pointers-invalid");
    view.registry_handle = registry_handle;
    const auto bucket_bytes = static_cast<std::size_t>(view.bucket_count) * layout.bucket_stride;
    const auto key_bytes = static_cast<std::size_t>(view.key_count) * layout.key_stride;
    const auto value_bytes = static_cast<std::size_t>(view.value_count) * layout.value_stride;
    // The selected records are copied with RPM below. A single-resource lookup
    // does not need to query every page of the complete resource index.
    if (bucket_bytes > UINTPTR_MAX - view.buckets || key_bytes > UINTPTR_MAX - view.keys ||
        value_bytes > UINTPTR_MAX - view.values) return fail("resource-index-arrays-range-overflow");

    std::uintptr_t chunk_table_address{};
    if (!add(view.service, layout.resource_service_chunk_table_offset, chunk_table_address) ||
        !read_value(chunk_table_address, view.chunks) ||
        !read_value(view.service + layout.resource_service_chunk_count_offset, view.chunk_count) ||
        !view.chunks || !view.chunk_count || view.chunk_count > layout.maximum_chunks ||
        static_cast<std::size_t>(view.chunk_count) * layout.chunk_stride > UINTPTR_MAX - view.chunks)
        return fail("resource-data-chunks-unavailable");
    return true;
}

bool same_view(const View& left, const View& right){
    return left.image == right.image && left.root == right.root && left.client == right.client &&
        left.controller == right.controller && left.main_owner == right.main_owner &&
        left.context == right.context && left.registry == right.registry && left.service == right.service &&
        left.registry_resource == right.registry_resource && left.registry_handle == right.registry_handle &&
        left.reader == right.reader && left.metadata == right.metadata && left.map == right.map &&
        left.buckets == right.buckets && left.keys == right.keys && left.values == right.values &&
        left.chunks == right.chunks && left.bucket_count == right.bucket_count &&
        left.key_count == right.key_count && left.value_count == right.value_count &&
        left.chunk_count == right.chunk_count;
}

bool span_from_entry(const Layout& layout, const View& view, std::size_t index,
    const unsigned char* entry, Span& span){
    if (index >= view.value_count || !entry) return false;
    std::uint32_t offset{}, packed{};
    std::memcpy(&offset, entry + layout.resource_offset_offset, sizeof(offset));
    std::memcpy(&packed, entry + layout.packed_size_offset, sizeof(packed));
    const auto chunk_id = entry[layout.chunk_index_offset];
    const auto size = static_cast<std::size_t>(packed & static_cast<std::uint32_t>(layout.size_mask));
    if (size > layout.maximum_resource_bytes || chunk_id >= view.chunk_count) return false;
    std::uintptr_t chunk_address{};
    if (!add(view.chunks, static_cast<std::size_t>(chunk_id) * layout.chunk_stride, chunk_address)) return false;
    std::uintptr_t chunk_base{};
    std::uint64_t chunk_size{};
    if (!read_value(chunk_address + layout.chunk_base_pointer_offset, chunk_base) ||
        !read_value(chunk_address + layout.chunk_size_offset, chunk_size) || !chunk_base ||
        offset > chunk_size || size > chunk_size - offset) return false;
    std::uintptr_t data{};
    if (!add(chunk_base, offset, data) || size > UINTPTR_MAX - data) return false;
    span = {data, size, static_cast<std::uint32_t>(index + 1), chunk_id};
    return true;
}

bool registry_is_bound(const Layout& layout, const View& view){
    if (!view.registry_handle || view.registry_handle > view.value_count) return false;
    std::array<unsigned char, 8> entry{};
    std::uintptr_t entry_address{};
    const auto index = static_cast<std::size_t>(view.registry_handle - 1);
    if (!add(view.values, index * layout.value_stride, entry_address) ||
        !copy_memory(entry_address, entry.data(), entry.size())) return false;
    Span span{};
    return span_from_entry(layout, view, index, entry.data(), span) &&
        span.handle == view.registry_handle && span.data == view.registry_resource;
}

bool locate(const Layout& layout, const View& view, const KfcRuntimeResourceId& id,
    std::uint32_t static_hash, Span& span){
    if (std::all_of(std::begin(id.guid), std::end(id.guid), [](auto byte) { return byte == 0; })) return false;
    const auto bucket_index = static_cast<std::size_t>(static_hash) & (view.bucket_count - 1);
    std::uintptr_t bucket_address{};
    if (!add(view.buckets, bucket_index * layout.bucket_stride, bucket_address)) return false;
    std::uint32_t first{}, count{};
    if (!read_value(bucket_address, first) || !read_value(bucket_address + sizeof(first), count) ||
        first > view.key_count || count > view.key_count - first) return false;
    std::array<unsigned char, 24> wanted{};
    std::memcpy(wanted.data(), id.guid, 16);
    std::memcpy(wanted.data() + 16, &id.type_hash, sizeof(id.type_hash));
    std::memcpy(wanted.data() + 20, &id.part_index, sizeof(id.part_index));
    std::size_t found = view.key_count;
    for (std::size_t position = first; position < static_cast<std::size_t>(first) + count; ++position) {
        std::array<unsigned char, 24> key{};
        std::uintptr_t key_address{};
        if (!add(view.keys, position * layout.key_stride, key_address) ||
            !copy_memory(key_address, key.data(), key.size())) return false;
        if (key == wanted) {
            if (found != view.key_count) return false;
            found = position;
        }
    }
    if (found == view.key_count) return false;
    std::array<unsigned char, 8> entry{};
    std::uintptr_t entry_address{};
    if (!add(view.values, found * layout.value_stride, entry_address) ||
        !copy_memory(entry_address, entry.data(), entry.size())) return false;
    return span_from_entry(layout, view, found, entry.data(), span);
}

// Same canonical StaticHash as kfc-base/src/guid/resource.rs. This native
// resolver learns the GUID inside the guarded lookup, before a Rust caller can
// hash it. Conformance vectors cover GUID seeding, byte order, type and part.
std::uint32_t canonical_resource_hash(const KfcRuntimeResourceId& id){
    std::uint32_t hash{};
    std::memcpy(&hash, id.guid, sizeof(hash));
    const std::uint32_t fields[]{id.type_hash, id.part_index};
    for (const auto field : fields) {
        for (unsigned shift = 0; shift < 32; shift += 8) {
            hash ^= (field >> shift) & 0xff;
            hash *= 0x01000193u;
        }
    }
    return hash;
}




bool relative_pointer(std::uintptr_t field, std::uintptr_t& pointer){
    std::int32_t relative{};
    if (!read_value(field, relative)) return false;
    if (!relative) {
        pointer = 0;
        return true;
    }
    if (relative < 0) {
        const auto magnitude = static_cast<std::uint64_t>(-static_cast<std::int64_t>(relative));
        if (magnitude > field) return false;
        pointer = field - static_cast<std::uintptr_t>(magnitude);
        return true;
    }
    return add(field, static_cast<std::size_t>(relative), pointer);
}
}
