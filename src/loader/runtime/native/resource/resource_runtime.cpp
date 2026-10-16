#include "resource_runtime.h"

#include "embedded_profile_ids.h"
#include "engine_context.h"
#include "ecs/ecs_runtime.h"
#include "profile.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstring>
#include <limits>
#include <map>
#include <mutex>
#include <set>
#include <stdexcept>
#include <type_traits>
#include <vector>
#include <nlohmann/json.hpp>

namespace {
using nlohmann::json;
using KfcRuntimeCompatibility::EnshroudedClient::image_sha256;
using KfcRuntimeCompatibility::EnshroudedClient::image_size;
using KfcRuntimeCompatibility::EnshroudedClient::image_target;
using KfcRuntimeCompatibility::EnshroudedClient::image_timestamp;
using KfcRuntimeCompatibility::EnshroudedClient::attribute_calculation_model;

struct Layout {
    std::string profile_id;
    std::size_t main_active_context_offset{}, context_item_registry_offset{};
    std::size_t item_registry_service_offset{}, item_registry_handle_offset{};
    std::size_t item_registry_resource_pointer_offset{};
    std::size_t resource_service_reader_offset{}, resource_service_chunk_table_offset{};
    std::size_t resource_service_id_callback_offset{}, resource_service_handle_callback_offset{};
    std::uintptr_t resource_service_id_callback_rva{}, resource_service_handle_callback_rva{};
    std::size_t resource_service_chunk_count_offset{}, reader_metadata_offset{}, metadata_map_offset{};
    std::size_t bucket_rel32_offset{}, bucket_count_offset{}, keys_rel32_offset{}, key_count_offset{};
    std::size_t values_rel32_offset{}, value_count_offset{}, map_header_size{};
    std::size_t bucket_stride{}, key_stride{}, resource_key_size{}, value_stride{};
    std::size_t resource_offset_offset{}, packed_size_offset{}, chunk_index_offset{}, size_mask{};
    std::size_t chunk_stride{}, chunk_base_pointer_offset{}, chunk_size_offset{};
    std::size_t maximum_entries{}, maximum_chunks{}, maximum_resource_bytes{};
};

struct View {
    std::uintptr_t image{}, root{}, client{}, controller{}, main_owner{}, context{};
    std::uintptr_t registry{}, registry_resource{}, service{}, reader{}, metadata{}, map{};
    std::uintptr_t buckets{}, keys{}, values{}, chunks{};
    std::uint32_t registry_handle{}, bucket_count{}, key_count{}, value_count{}, chunk_count{};
};

struct Span {
    std::uintptr_t data{};
    std::size_t size{};
    std::uint32_t handle{};
    std::uint8_t chunk{};
};

std::mutex state_mutex;
Layout active_layout;
bool profile_loaded{};
bool owns_attribute_model{};
std::string profile_status{"not-initialized"};
// Protected by state_mutex. Literal pointers avoid allocation while the
// original lifetime guard is held. An uncertain operation disables every mod's
// subsequent writes until the resource runtime is explicitly reinitialized.
const char* scalar_write_fault{};

void clear_owned_attribute_model() {
    if (owns_attribute_model && attribute_calculation_model == "attribute-command-v1") attribute_calculation_model.clear();
    owns_attribute_model = false;
}
void publish_attribute_model(const json& exact_profile) {
    const auto model = exact_profile.value("attributeCalculationModel", std::string{});
    const auto& provenance = exact_profile.at("provenance");
    const auto evidence = provenance.find("attributeCalculationEvidence");
    if (model != "attribute-command-v1" || evidence == provenance.end() ||
        !evidence->is_string() || evidence->get_ref<const std::string&>().empty()) return;
    attribute_calculation_model = model;
    owns_attribute_model = true;
}

using RegionCache = std::map<std::uintptr_t, MEMORY_BASIC_INFORMATION>;
bool readable(std::uintptr_t address, std::size_t size, RegionCache& regions) {
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

bool copy_memory(std::uintptr_t address, void* destination, std::size_t size) {
    if (!size) return true;
    // ReadProcessMemory already rejects unreadable/unmapped ranges. Repeating
    // VirtualQuery for each scalar walks large engine allocations at high cost.
    if (!address || !destination || size > std::numeric_limits<std::uintptr_t>::max() - address) return false;
    SIZE_T actual{};
    return ReadProcessMemory(GetCurrentProcess(), reinterpret_cast<const void*>(address),
        destination, size, &actual) && actual == size;
}

template <typename T>
bool read_value(std::uintptr_t address, T& value) {
    static_assert(std::is_trivially_copyable_v<T>);
    return copy_memory(address, &value, sizeof(value));
}

bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& result) {
    if (!base || offset > std::numeric_limits<std::uintptr_t>::max() - base) return false;
    result = base + offset;
    return true;
}

bool relative_pointer(std::uintptr_t field, std::uintptr_t& pointer) {
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

std::size_t json_size(const json& value, const char* key, std::size_t maximum) {
    const auto result = value.at(key).get<std::size_t>();
    if (result > maximum) throw std::runtime_error(std::string("out-of-range resource layout field: ") + key);
    return result;
}

Layout parse_layout(const json& candidate) {
    const auto& value = candidate.at("resourceLayout");
    Layout result{};
    result.profile_id = candidate.at("id").get<std::string>();
    result.main_active_context_offset = json_size(value, "mainActiveContextOffset", 0x1000000);
    result.context_item_registry_offset = json_size(value, "contextItemRegistryOffset", 0x1000000);
    result.item_registry_service_offset = json_size(value, "itemRegistryServiceOffset", 0x1000000);
    result.item_registry_handle_offset = json_size(value, "itemRegistryHandleOffset", 0x1000000);
    result.item_registry_resource_pointer_offset = json_size(value, "itemRegistryResourcePointerOffset", 0x1000000);
    result.resource_service_reader_offset = json_size(value, "resourceServiceReaderOffset", 0x1000000);
    result.resource_service_id_callback_offset = json_size(value, "resourceServiceIdCallbackOffset", 0x1000000);
    result.resource_service_id_callback_rva = json_size(value, "resourceServiceIdCallbackRva", image_size);
    result.resource_service_handle_callback_offset = json_size(value, "resourceServiceHandleCallbackOffset", 0x1000000);
    result.resource_service_handle_callback_rva = json_size(value, "resourceServiceHandleCallbackRva", image_size);
    result.resource_service_chunk_table_offset = json_size(value, "resourceServiceChunkTableOffset", 0x1000000);
    result.resource_service_chunk_count_offset = json_size(value, "resourceServiceChunkCountOffset", 0x1000000);
    result.reader_metadata_offset = json_size(value, "readerMetadataOffset", 0x1000000);
    result.metadata_map_offset = json_size(value, "metadataMapOffset", 0x1000000);

    const auto& map = value.at("map");
    result.bucket_rel32_offset = json_size(map, "bucketRel32Offset", 0x1000);
    result.bucket_count_offset = json_size(map, "bucketCountOffset", 0x1000);
    result.keys_rel32_offset = json_size(map, "keysRel32Offset", 0x1000);
    result.key_count_offset = json_size(map, "keyCountOffset", 0x1000);
    result.values_rel32_offset = json_size(map, "valuesRel32Offset", 0x1000);
    result.value_count_offset = json_size(map, "valueCountOffset", 0x1000);
    result.map_header_size = json_size(map, "headerSize", 0x1000);
    result.bucket_stride = json_size(map, "bucketStride", 64);
    result.key_stride = json_size(map, "keyStride", 64);
    result.resource_key_size = json_size(map, "resourceKeySize", 64);
    result.value_stride = json_size(map, "valueStride", 64);
    if (result.bucket_stride != 8 || result.key_stride != 32 ||
        result.resource_key_size != 24 || result.value_stride != 8)
        throw std::runtime_error("unsupported ResourceService map record layout");

    const auto& entry = value.at("valueEntry");
    result.resource_offset_offset = json_size(entry, "resourceOffsetOffset", 7);
    result.packed_size_offset = json_size(entry, "packedSizeOffset", 7);
    result.chunk_index_offset = json_size(entry, "chunkIndexOffset", 7);
    result.size_mask = json_size(entry, "sizeMask", 0xffffff);
    if (result.resource_offset_offset + sizeof(std::uint32_t) > result.value_stride ||
        result.packed_size_offset + sizeof(std::uint32_t) > result.value_stride ||
        result.chunk_index_offset >= result.value_stride || result.size_mask != 0xffffff)
        throw std::runtime_error("invalid ResourceService value record layout");

    const auto& chunk = value.at("chunkEntry");
    result.chunk_stride = json_size(chunk, "stride", 64);
    result.chunk_base_pointer_offset = json_size(chunk, "basePointerOffset", 64);
    result.chunk_size_offset = json_size(chunk, "sizeOffset", 64);
    if (result.chunk_stride != 16 || result.chunk_base_pointer_offset != 0 || result.chunk_size_offset != 8)
        throw std::runtime_error("unsupported ResourceService chunk layout");

    result.maximum_entries = json_size(value, "maximumEntries", 1u << 20);
    result.maximum_chunks = json_size(value, "maximumChunks", 1u << 16);
    result.maximum_resource_bytes = json_size(value, "maximumResourceBytes", 256u << 20);
    if (!result.resource_service_id_callback_rva || result.resource_service_id_callback_rva >= image_size ||
        !result.resource_service_handle_callback_rva || result.resource_service_handle_callback_rva >= image_size ||
        result.bucket_count_offset + sizeof(std::uint32_t) > result.map_header_size ||
        result.value_count_offset + sizeof(std::uint32_t) > result.map_header_size ||
        result.key_count_offset + sizeof(std::uint32_t) > result.map_header_size ||
        result.bucket_rel32_offset + sizeof(std::int32_t) > result.map_header_size ||
        result.keys_rel32_offset + sizeof(std::int32_t) > result.map_header_size ||
        result.values_rel32_offset + sizeof(std::int32_t) > result.map_header_size ||
        !result.maximum_entries || !result.maximum_chunks || !result.maximum_resource_bytes)
        throw std::runtime_error("incomplete ResourceService profile layout");
    return result;
}

bool resolve_view(const Layout& layout, View& view, std::string* reason = nullptr,
    const EngineContext::Owners* guarded_owners = nullptr) {
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

bool same_view(const View& left, const View& right) {
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
    const unsigned char* entry, Span& span) {
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

bool registry_is_bound(const Layout& layout, const View& view) {
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
    std::uint32_t static_hash, Span& span) {
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

bool snapshot_arrays(const Layout& layout, const View& view,
    std::vector<unsigned char>& keys, std::vector<unsigned char>& values,
    std::vector<unsigned char>& chunks) {
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
    std::vector<KfcRuntimeResourceId>& result) {
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
        Span span{};
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

bool compare_exchange_32(std::uintptr_t address, std::uint32_t expected,
    std::uint32_t desired, std::uint32_t& observed) noexcept {
    __try {
        observed = static_cast<std::uint32_t>(InterlockedCompareExchange(
            reinterpret_cast<volatile LONG*>(address), static_cast<LONG>(desired), static_cast<LONG>(expected)));
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

// Caller-owned full-resource expectations are captured before original gates.
// Native wrappers reject invalid pointers without leaving an exception across
// the ABI. These bounded helpers allocate nothing and do not query host locks.
bool copy_expected_snapshot(const void* source, unsigned char* destination, std::size_t size) noexcept {
    const auto address = reinterpret_cast<std::uintptr_t>(source);
    if (!source || !destination || !size || size > UINTPTR_MAX - address) return false;
    __try { std::memcpy(destination, source, size); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
bool snapshot_matches(std::uintptr_t address, const unsigned char* expected, std::size_t size) noexcept {
    if (!size) return true;
    if (!address || !expected || size > UINTPTR_MAX - address) return false;
    __try { return std::memcmp(reinterpret_cast<const void*>(address), expected, size) == 0; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

class ScalarDataProtection {
public:
    ~ScalarDataProtection() { Restore(); }
    bool Prepare(std::uintptr_t address, std::uintptr_t allocation_base) {
        MEMORY_BASIC_INFORMATION page{};
        if (VirtualQuery(reinterpret_cast<const void*>(address), &page, sizeof(page)) != sizeof(page) ||
            page.State != MEM_COMMIT || page.Type != MEM_PRIVATE ||
            reinterpret_cast<std::uintptr_t>(page.AllocationBase) != allocation_base ||
            (page.Protect != PAGE_READONLY && page.Protect != PAGE_READWRITE)) return false;
        const auto base = reinterpret_cast<std::uintptr_t>(page.BaseAddress);
        if (address < base || address - base > page.RegionSize || sizeof(LONG) > page.RegionSize - (address - base))
            return false;
        address_ = address;
        allocation_base_ = allocation_base;
        old_ = page.Protect;
        if (page.Protect == PAGE_READWRITE) return true;
        DWORD previous{};
        if (!VirtualProtect(reinterpret_cast<void*>(address_), sizeof(LONG), PAGE_READWRITE, &previous)) return false;
        old_ = previous;
        changed_ = true;
        // A changed protection is not permission to write a different kind of
        // page. Restore even if the original observation no longer matches.
        if (previous != PAGE_READONLY) return false;
        MEMORY_BASIC_INFORMATION after{};
        return VirtualQuery(reinterpret_cast<const void*>(address_), &after, sizeof(after)) == sizeof(after) &&
            after.State == MEM_COMMIT && after.Type == MEM_PRIVATE && after.Protect == PAGE_READWRITE &&
            after.AllocationBase == page.AllocationBase;
    }
    bool Restore() noexcept {
        if (!address_) return !restore_failed_;
        if (changed_) {
            DWORD ignored{};
            if (!VirtualProtect(reinterpret_cast<void*>(address_), sizeof(LONG), old_, &ignored)) {
                restore_failed_ = true;
                // The destructor can make one last cleanup attempt. This sticky
                // failure still prevents reporting a confirmed outcome.
                return false;
            }
            changed_ = false;
        }
        MEMORY_BASIC_INFORMATION restored{};
        if (VirtualQuery(reinterpret_cast<const void*>(address_), &restored, sizeof(restored)) != sizeof(restored) ||
            restored.Protect != old_ || restored.State != MEM_COMMIT || restored.Type != MEM_PRIVATE ||
            reinterpret_cast<std::uintptr_t>(restored.AllocationBase) != allocation_base_)
            restore_failed_ = true;
        return !restore_failed_;
    }
private:
    std::uintptr_t address_{}, allocation_base_{};
    DWORD old_{};
    bool changed_{}, restore_failed_{};
};
}

namespace ResourceRuntime {
bool Initialize() {
    std::scoped_lock lock(state_mutex);
    clear_owned_attribute_model();
    EngineContext::Shutdown();
    profile_loaded = false;
    active_layout = {};
    scalar_write_fault = nullptr;
    profile_status = "no-exact-resource-read-profile";
    std::size_t exact_matches{};
    Layout selected{};
    json selected_profile;
    for (const auto resource_id : KfcRuntimeEmbedded::resource_read_profile_ids) {
        if (!resource_id) continue;
        HMODULE self{};
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
                GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                reinterpret_cast<LPCWSTR>(&Initialize), &self)) continue;
        const auto resource = FindResourceW(self, MAKEINTRESOURCEW(resource_id), MAKEINTRESOURCEW(10));
        if (!resource) continue;
        const auto loaded = LoadResource(self, resource);
        const auto size = SizeofResource(self, resource);
        const auto* data = static_cast<const char*>(LockResource(loaded));
        if (!loaded || !data || !size) continue;
        try {
            const auto candidate = json::parse(data, data + size);
            if (candidate.value("schemaVersion", 0) != 1 ||
                candidate.value("target", std::string{}) != image_target ||
                candidate.value("kfcFormat", 0) != 4 ||
                !candidate.contains("provenance") ||
                candidate["provenance"].value("status", std::string{}) != "read-validated" ||
                !candidate["provenance"].value("resourceReadValidated", false)) continue;
            const auto& image = candidate.at("image");
            if (image.value("timestamp", 0u) != image_timestamp ||
                image.value("size", 0u) != image_size ||
                image.value("sha256", std::string{}) != image_sha256) continue;
            ++exact_matches;
            selected = parse_layout(candidate);
            selected_profile = candidate;
        } catch (const std::exception& error) {
            profile_status = std::string("resource-profile-error:") + error.what();
            return false;
        }
    }
    if (exact_matches != 1) {
        profile_status = exact_matches ? "ambiguous-exact-resource-read-profile" : "no-exact-resource-read-profile";
        return false;
    }
    std::string owner_error;
    if (!EngineContext::Configure(selected_profile, owner_error)) {
        profile_status = "engine-owner-profile-error:" + owner_error;
        return false;
    }
    publish_attribute_model(selected_profile);
    active_layout = std::move(selected);
    profile_loaded = true;
    profile_status = active_layout.profile_id + ":exact-image-resource-service";
    return true;
}

void Shutdown() {
    std::scoped_lock lock(state_mutex);
    clear_owned_attribute_model();
    EngineContext::Shutdown();
    profile_loaded = false;
    active_layout = {};
    scalar_write_fault = nullptr;
    profile_status = "shutdown";
}

bool Ready() {
    Layout layout;
    {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded) return false;
        layout = active_layout;
    }
    View view{};
    return resolve_view(layout, view) && registry_is_bound(layout, view);
}

bool CanWrite() {
    std::scoped_lock lock(state_mutex);
    if (!profile_loaded || scalar_write_fault || !EngineContext::ResourceWriteConfigured()) return false;
    View before{}, after{};
    if (!resolve_view(active_layout, before) || !registry_is_bound(active_layout, before)) return false;
    EngineContext::ResourceLifeGuard lifetime;
    EngineContext::Owners owners;
    return lifetime.TryAcquire(before.service, before.reader) && lifetime.Validate(owners) &&
        resolve_view(active_layout, after, nullptr, &owners) && same_view(before, after) &&
        registry_is_bound(active_layout, after);
}

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

static std::uint32_t compare_exchange_scalar(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width,
    std::uint64_t expected_session) {
    if (width != sizeof(LONG) || !expected || !desired || !resource_size ||
        offset > resource_size || width > resource_size - offset) return 0;
    std::uint32_t expected_bits{}, desired_bits{};
    if ((!expected_session && !copy_memory(reinterpret_cast<std::uintptr_t>(expected), &expected_bits, sizeof(expected_bits))) ||
        !copy_memory(reinterpret_cast<std::uintptr_t>(desired), &desired_bits, sizeof(desired_bits))) return 0;
    // Serialize our temporary data-page protections. Take this mutex before the
    // original lifetime guard, never while holding an engine gate.
    std::scoped_lock lock(state_mutex);
    if (!profile_loaded || scalar_write_fault) return 0;
    const auto& layout = active_layout;
    std::vector<unsigned char> expected_resource;
    if (expected_session) {
        if (resource_size > layout.maximum_resource_bytes) return 0;
        expected_resource.resize(resource_size);
        if (!copy_expected_snapshot(expected, expected_resource.data(), expected_resource.size())) return 0;
        std::memcpy(&expected_bits, expected_resource.data() + offset, sizeof(expected_bits));
    }
    // The ECS binding mutex is acquired before original engine gates and held
    // until after them. No artificial session is generated by this operation.
    EcsRuntime::SessionGuard session;
    if (expected_session && (!session.TryAcquire(expected_session) || !session.WritesAllowed())) return 0;
    const auto uncertain = [&](const char* reason) -> std::uint32_t {
        scalar_write_fault = reason;
        if (expected_session) session.MarkWriteUncertain(reason);
        return 3;
    };
    View initial{};
    if (!resolve_view(layout, initial) || !registry_is_bound(layout, initial)) return 0;
    const auto session_owners_match = [&](const EngineContext::Owners& owners) {
        return !expected_session || owners == session.View().owners;
    };
    // Prepare the content-worker snapshot before entering Scene. Its lifetime
    // guard then borrows Scene's identical original registry Shared ownership.
    // Declaration order releases Scene before destroying prepared heap storage.
    EngineContext::ResourceLifeGuard lifetime;
    EngineContext::EcsWriteGuard scene;
    if (expected_session) {
        if (!lifetime.Prepare() || !scene.TryAcquire(session.View()) ||
            !lifetime.TryAcquire(scene, initial.service, initial.reader)) return 0;
    } else if (!lifetime.TryAcquire(initial.service, initial.reader)) return 0;
    EngineContext::Owners owners;
    View before{};
    Span span{};
    if (!lifetime.Validate(owners) || !session_owners_match(owners) ||
        (expected_session && !scene.Validate()) || !resolve_view(layout, before, nullptr, &owners) ||
        !same_view(initial, before) || !registry_is_bound(layout, before) ||
        !locate(layout, before, resource, static_hash, span) || span.size != resource_size ||
        !lifetime.Contains(span.data, span.size)) return 0;
    std::uintptr_t scalar{};
    if (!add(span.data, offset, scalar) || scalar % alignof(LONG)) return 0;
    const auto stable = [&]() {
        EngineContext::Owners current_owners;
        View after{};
        Span current{};
        return lifetime.Validate(current_owners) && session_owners_match(current_owners) &&
            (!expected_session || scene.Validate()) && resolve_view(layout, after, nullptr, &current_owners) &&
            same_view(before, after) && registry_is_bound(layout, after) &&
            locate(layout, after, resource, static_hash, current) && current.data == span.data &&
            current.size == span.size && current.handle == span.handle && current.chunk == span.chunk &&
            lifetime.Contains(current.data, current.size);
    };
    ScalarDataProtection protection;
    if (!protection.Prepare(scalar, lifetime.PayloadBase()))
        return protection.Restore() ? 0 : uncertain("resource-page-protection-restore-uncertain");
    if (!stable())
        return protection.Restore() ? 0 : uncertain("resource-page-protection-restore-uncertain");
    // A reflected BlobVariant field offset is meaningful only for the complete
    // resource image used by the Lua codec, including every descriptor. Equal
    // size, ResourceId and scalar bits alone are insufficient after a reload.
    if (expected_session && !snapshot_matches(span.data, expected_resource.data(), expected_resource.size()))
        return protection.Restore() ? 2 : uncertain("resource-page-protection-restore-uncertain");
    std::uint32_t observed{};
    const bool executed = compare_exchange_32(scalar, expected_bits, desired_bits, observed);
    bool unchanged_payload = true;
    if (expected_session && executed) {
        if (observed == expected_bits) {
            std::memcpy(expected_resource.data() + offset, &desired_bits, sizeof(desired_bits));
            unchanged_payload = snapshot_matches(span.data, expected_resource.data(), expected_resource.size());
        } else {
            // A scalar conflict has no mutation by us. All other bytes,
            // especially the variant descriptors, must still match.
            unchanged_payload = snapshot_matches(span.data, expected_resource.data(), offset) &&
                snapshot_matches(span.data + offset + width, expected_resource.data() + offset + width,
                    expected_resource.size() - offset - width);
        }
    }
    const bool unchanged_owner = stable();
    const bool restored = protection.Restore();
    if (!restored) return uncertain("resource-page-protection-restore-uncertain");
    if (!executed) return uncertain("resource-scalar-cas-uncertain");
    if (!unchanged_owner) return uncertain("resource-scalar-owner-changed");
    if (!unchanged_payload) return uncertain("resource-scalar-payload-changed-after-cas");
    return observed == expected_bits ? 1 : 2;
}
std::uint32_t CompareExchangeScalar(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width) {
    return compare_exchange_scalar(resource, static_hash, resource_size, offset, expected, desired, width, 0);
}
std::uint32_t CompareExchangeScalarForSession(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width,
    std::uint64_t expected_session) {
    if (!expected_session) return 0;
    return compare_exchange_scalar(resource, static_hash, resource_size, offset, expected, desired, width, expected_session);
}

std::string Status() {
    Layout layout;
    std::string detail;
    const char* write_fault{};
    {
        std::scoped_lock lock(state_mutex);
        detail = profile_status;
        write_fault = scalar_write_fault;
        if (!profile_loaded) return json({{"available", false}, {"profile", detail},
            {"mode", "resource-service"}, {"resourceWritesConfigured", false}, {"resourceWrites", false}}).dump();
        layout = active_layout;
    }
    View view{};
    std::string reason;
    const bool available = resolve_view(layout, view, &reason) && registry_is_bound(layout, view);
    if (!available && reason.empty()) reason = "item-registry-resource-handle-mismatch";
    const bool writes_configured = EngineContext::ResourceWriteConfigured();
    const bool writes_available = writes_configured && CanWrite();
    return json({{"available", available}, {"profile", detail},
        {"mode", "resource-service"}, {"reason", available ? "ready" : reason},
        {"bucketCount", view.bucket_count}, {"resourceIndexCount", view.key_count},
        {"chunkCount", view.chunk_count}, {"resourceEngineCalls", false},
        {"resourceWrites", writes_available}, {"resourceWritesConfigured", writes_configured},
        {"resourceWriteFault", write_fault != nullptr},
        {"resourceWriteReason", write_fault ? write_fault :
            (writes_available ? "ready" : (writes_configured ? "resource-write-gate-unavailable" : "not-configured"))},
        {"resourceWritePolicy", "atomic-aligned-32-bit-private-bulk-scalar"},
        {"resourceWriteWidth", 4}, {"resourceWriteTransactions", false},
        {"resourceHooksInstalled", false}}).dump();
}
}

extern "C" bool __cdecl KfcRuntimeResourceReady() {
    try { return ResourceRuntime::Ready(); } catch (...) { return false; }
}
extern "C" bool __cdecl KfcRuntimeResourceCanWrite() {
    try { return ResourceRuntime::CanWrite(); } catch (...) { return false; }
}
extern "C" std::uint32_t __cdecl KfcRuntimeResourceCompareExchangeScalar(const KfcRuntimeResourceId* resource,
    std::uint32_t static_hash, std::size_t resource_size, std::size_t offset,
    const void* expected, const void* desired, std::size_t width) {
    if (!resource) return 0;
    try { return ResourceRuntime::CompareExchangeScalar(*resource, static_hash, resource_size, offset, expected, desired, width); }
    catch (...) { return 0; }
}
extern "C" std::uint32_t __cdecl KfcRuntimeResourceCompareExchangeScalarForSession(const KfcRuntimeResourceId* resource,
    std::uint32_t static_hash, std::size_t resource_size, std::size_t offset,
    const void* expected, const void* desired, std::size_t width, std::uint64_t expected_session) {
    if (!resource || !expected_session) return 0;
    try { return ResourceRuntime::CompareExchangeScalarForSession(*resource, static_hash, resource_size, offset,
        expected, desired, width, expected_session); }
    catch (...) { return 0; }
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

extern "C" std::size_t __cdecl KfcRuntimeResourceStatus(char* buffer, std::size_t capacity) {
    try {
        const auto text = ResourceRuntime::Status();
        const auto required = text.size() + 1;
        if (buffer && capacity >= required) std::memcpy(buffer, text.c_str(), required);
        return required;
    } catch (...) {
        constexpr char error[] = "{\"available\":false,\"reason\":\"resource-status-exception\"}";
        if (buffer && capacity >= sizeof(error)) std::memcpy(buffer, error, sizeof(error));
        return sizeof(error);
    }
}
