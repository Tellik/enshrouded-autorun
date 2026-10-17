#include "loaded-resources/resource_state.h"



using namespace ResourceImplementation;

namespace ResourceImplementation {
std::mutex state_mutex;

Layout active_layout;

bool profile_loaded{};

bool owns_attribute_model{};

std::string profile_status{"not-initialized"};

// Protected by state_mutex. Literal pointers avoid allocation while the
// original lifetime guard is held. An uncertain operation disables every mod's
// subsequent writes until the resource runtime is explicitly reinitialized.
const char* scalar_write_fault{};

void clear_owned_attribute_model(){
    if (owns_attribute_model && attribute_calculation_model == "attribute-command-v1") attribute_calculation_model.clear();
    owns_attribute_model = false;
}

void publish_attribute_model(const json& exact_profile){
    const auto model = exact_profile.value("attributeCalculationModel", std::string{});
    const auto& provenance = exact_profile.at("provenance");
    const auto evidence = provenance.find("attributeCalculationEvidence");
    if (model != "attribute-command-v1" || evidence == provenance.end() ||
        !evidence->is_string() || evidence->get_ref<const std::string&>().empty()) return;
    attribute_calculation_model = model;
    owns_attribute_model = true;
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

std::size_t json_size(const json& value, const char* key, std::size_t maximum){
    const auto result = value.at(key).get<std::size_t>();
    if (result > maximum) throw std::runtime_error(std::string("out-of-range resource layout field: ") + key);
    return result;
}

Layout parse_layout(const json& candidate){
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
    if (value.contains("itemIdMap")) {
        const auto& model = value.at("itemIdMap");
        const auto& provenance = candidate.at("provenance");
        if (model.at("id") != "original-item-id-map-v1" || model.at("registry") != "g38_items::ItemRegistry" ||
            model.at("targetType") != "keen::ItemInfo" || !provenance.value("itemIdMapValidated", false) ||
            !model.at("source").is_string() || model.at("source").get_ref<const std::string&>().empty() ||
            !provenance.at("itemIdMapEvidence").is_string() ||
            provenance.at("itemIdMapEvidence").get_ref<const std::string&>().empty())
            throw std::runtime_error("unproven original ItemRegistry map contract");
        auto& item = result.item_map;
        item.type_hash = model.at("targetHash").get<std::uint32_t>();
        item.part_index = model.at("partIndex").get<std::uint32_t>();
        item.root_size = json_size(model, "targetSize", result.maximum_resource_bytes);
        item.item_id_offset = json_size(model, "itemIdOffset", result.maximum_resource_bytes);
        item.guid_offset = json_size(model, "guidOffset", result.maximum_resource_bytes);
        item.map_offset = json_size(model, "mapOffset", 0x1000000);
        item.header_size = json_size(model, "headerSize", 128);
        item.bits_offset = json_size(model, "bitsPointerOffset", 128);
        item.capacity_offset = json_size(model, "capacityOffset", 128);
        item.keys_offset = json_size(model, "keysPointerOffset", 128);
        item.values_offset = json_size(model, "valuesPointerOffset", 128);
        item.count_offset = json_size(model, "countOffset", 128);
        item.maximum_capacity = json_size(model, "maximumCapacity", 1u << 20);
        item.maximum_probes = json_size(model, "maximumProbes", 256);
        if (!item.type_hash || item.root_size < 20 || item.item_id_offset > item.root_size - 4 ||
            item.guid_offset > item.root_size - 16 || item.item_id_offset % 4 ||
            item.header_size < 8 || item.bits_offset > item.header_size - 8 ||
            item.capacity_offset > item.header_size - 8 || item.keys_offset > item.header_size - 8 ||
            item.values_offset > item.header_size - 8 || item.count_offset > item.header_size - 4 ||
            !item.maximum_capacity || !item.maximum_probes || item.part_index != 0)
            throw std::runtime_error("invalid original ItemRegistry map contract");
        item.enabled = true;
    }
    return result;
}
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
        {"itemRegistryResolverConfigured", layout.item_map.enabled},
        {"itemRegistryResolverScope", "resource-service-owner-registry"},
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
