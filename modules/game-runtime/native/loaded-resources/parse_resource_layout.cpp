#include "loaded-resources/loaded_resource_state.h"

namespace ResourceImplementation {
std::size_t json_size(const json& value, const char* key, std::size_t maximum){
    const auto result = value.at(key).get<std::size_t>();
    if (result > maximum) throw std::runtime_error(std::string("out-of-range resource layout field: ") + key);
    return result;
}

Layout parse_layout(const json& candidate){
    const auto& value = candidate.at("resourceLayout");
    Layout result{};
    result.profile_id = candidate.at("id").get<std::string>();
    result.dedicated_server = candidate.at("target") == "enshrouded_server.exe";
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
