#include "loaded-resources/resource_state.h"



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
    if (!add(view.main_owner, layout.main_active_context_offset, active_context_address) ||
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

bool Read(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    unsigned char* buffer, std::size_t capacity, std::size_t* actual) {
    if (!actual || (!buffer && capacity)) return false;
    *actual = 0;
    Layout layout;
    {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded) return false;
        layout = active_layout;
    }
    View before{};
    Span span{};
    if (!resolve_view(layout, before) || !registry_is_bound(layout, before) ||
        !locate(layout, before, resource, static_hash, span)) return false;
    *actual = span.size;
    const auto stable_span = [&] {
        View after{};
        Span after_span{};
        return resolve_view(layout, after) && registry_is_bound(layout, after) && same_view(before, after) &&
            locate(layout, after, resource, static_hash, after_span) &&
            span.data == after_span.data && span.size == after_span.size && span.handle == after_span.handle;
    };
    if (!buffer && capacity == 0) {
        // The size query also backs Lua's direct GUID existence check. Preserve
        // its readable-payload filter without querying every metadata scalar.
        RegionCache regions;
        return readable(span.data, span.size, regions) && stable_span();
    }
    if (capacity < span.size || (!buffer && span.size)) return false;
    std::vector<unsigned char> snapshot(span.size), confirmation(span.size);
    if (!copy_memory(span.data, snapshot.data(), span.size) ||
        !copy_memory(span.data, confirmation.data(), span.size) ||
        snapshot != confirmation) return false;
    if (!stable_span()) return false;
    if (!snapshot.empty()) std::memcpy(buffer, snapshot.data(), snapshot.size());
    return true;
}

std::uint32_t ReadScalarBatch(const KfcRuntimeResourceScalarRequest* requests,
    std::size_t count, std::uint32_t type_hash, std::size_t root_size,
    std::size_t offset, std::uint64_t expected_session,
    std::uint32_t* scalar_bits, std::size_t capacity, std::size_t* actual) {
    constexpr std::size_t maximum_batch = 64;
    constexpr std::uint32_t success = 1, invalid_argument = 2, unsupported = 3,
        stale_context = 4, retry_incomplete = 5;
    static_assert(sizeof(KfcRuntimeResourceId) == 24);
    static_assert(sizeof(KfcRuntimeResourceScalarRequest) == 28);
    if (!actual || !set_batch_count(actual, 0)) return invalid_argument;
    if (!requests || !scalar_bits || !count || count > maximum_batch || capacity < count ||
        !expected_session || root_size < sizeof(std::uint32_t) ||
        offset % alignof(std::uint32_t) || offset > root_size - sizeof(std::uint32_t))
        return invalid_argument;

    std::array<KfcRuntimeResourceScalarRequest, maximum_batch> copied_requests{};
    std::array<std::uint32_t, maximum_batch> keys{};
    std::array<Span, maximum_batch> spans{};
    if (!copy_memory(reinterpret_cast<std::uintptr_t>(requests), copied_requests.data(),
        count * sizeof(KfcRuntimeResourceScalarRequest))) return invalid_argument;
    for (std::size_t index = 0; index < count; ++index) {
        const auto& id = copied_requests[index].resource;
        if (id.type_hash != type_hash ||
            std::all_of(std::begin(id.guid), std::end(id.guid), [](auto value) { return value == 0; }))
            return invalid_argument;
    }

    // All allocation and our own mutex acquisitions precede original engine
    // gates. SessionGuard pins the binding epoch, not mutable resource contents.
    const auto result = [&]() -> std::uint32_t {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded || !EngineContext::ResourceWriteConfigured()) return unsupported;
        const auto& layout = active_layout;
        if (root_size > layout.maximum_resource_bytes) return invalid_argument;
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) {
            // SessionGuard intentionally admits only local authority. A stable
            // client replica still supports the established guarded Read path,
            // so decline this optimization without weakening stale-session errors.
            EngineContext::EcsView replica{}, confirmation{};
            View before{}, after{};
            if (KfcRuntimeWorldSessionId() == expected_session &&
                EngineContext::ResolveEcs(replica) && replica.role == EngineContext::EcsRole::ClientReplica &&
                resolve_view(layout, before, nullptr, &replica.owners) && registry_is_bound(layout, before) &&
                EngineContext::ResolveEcs(confirmation) && confirmation == replica &&
                resolve_view(layout, after, nullptr, &confirmation.owners) && registry_is_bound(layout, after) && same_view(before, after) &&
                KfcRuntimeWorldSessionId() == expected_session) return unsupported;
            return stale_context;
        }
        View initial{};
        if (!resolve_view(layout, initial) || !registry_is_bound(layout, initial)) return retry_incomplete;
        EngineContext::ResourceLifeGuard lifetime;
        if (!lifetime.TryAcquire(initial.service, initial.reader)) return retry_incomplete;
        EngineContext::Owners owners;
        View before{};
        if (!lifetime.Validate(owners) || owners != session.View().owners ||
            !resolve_view(layout, before, nullptr, &owners) || !same_view(initial, before) ||
            !registry_is_bound(layout, before)) return stale_context;

        // Read only the fixed inline key. Full resources are still read by the
        // final candidate lookup and every existing guarded resource writer.
        for (std::size_t index = 0; index < count; ++index) {
            const auto& request = copied_requests[index];
            auto& span = spans[index];
            if (!locate(layout, before, request.resource, request.static_hash, span) ||
                span.size < root_size || !lifetime.Contains(span.data, span.size)) return retry_incomplete;
            std::uintptr_t scalar{};
            if (!add(span.data, offset, scalar) || scalar % alignof(std::uint32_t)) return retry_incomplete;
            if (!read_value(scalar, keys[index])) return retry_incomplete;
        }

        // The lifetime guard does not freeze keys or map entries. Check every
        // selected identity/span and key again before returning sampled rows.
        for (std::size_t index = 0; index < count; ++index) {
            const auto& request = copied_requests[index];
            const auto& span = spans[index];
            Span current{};
            std::uint32_t confirmation{};
            std::uintptr_t scalar{};
            if (!locate(layout, before, request.resource, request.static_hash, current) ||
                current.data != span.data || current.size != span.size ||
                current.handle != span.handle || current.chunk != span.chunk ||
                !lifetime.Contains(current.data, current.size) || !add(current.data, offset, scalar) ||
                !read_value(scalar, confirmation) || confirmation != keys[index]) return retry_incomplete;
        }
        EngineContext::Owners after_owners;
        View after{};
        if (!lifetime.Validate(after_owners) || after_owners != owners ||
            !resolve_view(layout, after, nullptr, &after_owners) || !same_view(before, after) ||
            !registry_is_bound(layout, after)) return stale_context;
        return success;
    }();
    if (result != success) return result;
    return publish_scalar_batch(keys.data(), count, scalar_bits, actual) ? success : invalid_argument;
}
}

extern "C" bool __cdecl KfcRuntimeResourcesByType(std::uint32_t type_hash,
    KfcRuntimeResourceId* resources, std::size_t capacity, std::size_t* count) {
    try { return ResourceRuntime::ResourcesByType(type_hash, resources, capacity, count); }
    catch (...) { if (count) *count = 0; return false; }
}

extern "C" bool __cdecl KfcRuntimeResourceRead(const KfcRuntimeResourceId* resource,
    std::uint32_t static_hash, unsigned char* buffer, std::size_t capacity, std::size_t* actual) {
    if (!resource) return false;
    try { return ResourceRuntime::Read(*resource, static_hash, buffer, capacity, actual); }
    catch (...) { if (actual) *actual = 0; return false; }
}

extern "C" std::uint32_t __cdecl KfcRuntimeResourceReadScalarBatch(
    const KfcRuntimeResourceScalarRequest* requests, std::size_t count, std::uint32_t type_hash,
    std::size_t root_size, std::size_t offset, std::uint64_t expected_session,
    std::uint32_t* scalar_bits, std::size_t capacity, std::size_t* actual) {
    try { return ResourceRuntime::ReadScalarBatch(requests, count, type_hash, root_size, offset,
        expected_session, scalar_bits, capacity, actual); }
    catch (...) { if (actual) set_batch_count(actual, 0); return 5; }
}
