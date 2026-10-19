#include "named-services/named_service_contract.h"

using namespace NamedServiceImplementation;

namespace NamedServiceImplementation {
std::mutex operation_mutex;

bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& result){
    if (!base || offset > UINTPTR_MAX - base) return false;
    result = base + offset;
    return true;
}

std::uint32_t hash_name(const std::string& name){
    std::uint32_t hash = 0x811c9dc5;
    for (const unsigned char byte : name) hash = (hash ^ byte) * 0x1000193;
    return hash;
}

bool executable_callback(std::uintptr_t callback){
    const auto image = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    IMAGE_DOS_HEADER dos{};
    IMAGE_NT_HEADERS64 nt{};
    std::uintptr_t headers{}, sections{};
    if (!image || !copy(&dos, reinterpret_cast<const void*>(image), sizeof(dos)) ||
        dos.e_magic != IMAGE_DOS_SIGNATURE || dos.e_lfanew < 0 ||
        !add(image, static_cast<std::size_t>(dos.e_lfanew), headers) ||
        !copy(&nt, reinterpret_cast<const void*>(headers), sizeof(nt)) ||
        nt.Signature != IMAGE_NT_SIGNATURE || nt.OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC ||
        !nt.OptionalHeader.SizeOfImage || nt.FileHeader.NumberOfSections > 96 ||
        callback < image || callback - image >= nt.OptionalHeader.SizeOfImage ||
        !add(headers, offsetof(IMAGE_NT_HEADERS64, OptionalHeader) + nt.FileHeader.SizeOfOptionalHeader, sections))
        return false;
    bool executable{};
    const auto offset = callback - image;
    for (std::size_t i = 0; i < nt.FileHeader.NumberOfSections; ++i) {
        IMAGE_SECTION_HEADER section{};
        if (!field(sections, i * sizeof(section), section)) return false;
        if (offset >= section.VirtualAddress && offset - section.VirtualAddress < section.Misc.VirtualSize)
            executable = (section.Characteristics & IMAGE_SCN_MEM_EXECUTE) != 0;
    }
    MEMORY_BASIC_INFORMATION memory{};
    if (!executable || VirtualQuery(reinterpret_cast<const void*>(callback), &memory, sizeof(memory)) != sizeof(memory) ||
        memory.State != MEM_COMMIT || memory.Type != MEM_IMAGE ||
        reinterpret_cast<std::uintptr_t>(memory.AllocationBase) != image || (memory.Protect & PAGE_GUARD)) return false;
    const auto protection = memory.Protect & 0xff;
    return protection == PAGE_EXECUTE || protection == PAGE_EXECUTE_READ;
}

namespace {
std::size_t layout_offset(const nlohmann::json& model, const char* key){
    const auto value = model.at(key).get<std::uint64_t>();
    if (value > 0x10000000) throw std::runtime_error("invalid named service offset");
    return static_cast<std::size_t>(value);
}

BindingContract prepare_binding(const char* name, const nlohmann::json& model){
    if (!name || !*name || std::strlen(name) > 127)
        throw std::runtime_error("invalid service selector");
    if (model.at("id") != "engine-named-plain-services-v1")
        throw std::runtime_error("named services unavailable");
    BindingContract c;
    c.key = name; c.hash = hash_name(c.key);
    c.records = layout_offset(model, "recordsPointerOffset"); c.record_count = layout_offset(model, "recordCountOffset");
    c.stride = layout_offset(model, "recordStride"); c.record_hash = layout_offset(model, "recordHashOffset");
    c.normal = layout_offset(model, "recordNormalPointerOffset"); c.alternate = layout_offset(model, "recordAlternatePointerOffset");
    c.map_bits = layout_offset(model, "mapBitsOffset"); c.map_capacity = layout_offset(model, "mapCapacityOffset");
    c.map_keys = layout_offset(model, "mapKeysOffset"); c.map_values = layout_offset(model, "mapValuesOffset");
    c.map_count = layout_offset(model, "mapCountOffset");
    c.maximum_records = layout_offset(model, "maximumRecords"); c.maximum_capacity = layout_offset(model, "maximumMapCapacity");
    if (!c.maximum_records || c.maximum_records > 1024 ||
        !c.maximum_capacity || c.maximum_capacity > 4096 || c.stride < 0xb8 ||
        c.normal > c.stride - 8 || c.alternate > c.stride - 8 || c.record_hash > c.stride - 4)
        throw std::runtime_error("invalid named service record layout");
    return c;
}
}

BindingContract binding_contract(const char* name){
    return prepare_binding(name, EngineContext::ServiceLayout());
}

Contract contract(const char* name, std::uint32_t type_hash, std::uint32_t size){
    if (!name || !*name || std::strlen(name) > 127 || !size || size > 65536)
        throw std::runtime_error("invalid service selector");
    const auto model = EngineContext::ServiceLayout();
    if (model.at("id") != "engine-named-plain-services-v1")
        throw std::runtime_error("named services unavailable");
    const nlohmann::json* selected{};
    for (const auto& candidate : model.at("contracts")) {
        if (candidate.at("key") != name) continue;
        if (selected) throw std::runtime_error("ambiguous service contract");
        selected = &candidate;
    }
    if (!selected || selected->at("qualifiedType") != std::string("keen::") + name ||
        !selected->value("plainData", false) || selected->at("scope") != "host-world-shared" ||
        selected->at("size") != size || selected->at("qualifiedHash") != type_hash)
        throw std::runtime_error("no matching original plain-data service contract");
    Contract c;
    static_cast<BindingContract&>(c) = prepare_binding(name, model);
    c.type_hash = type_hash; c.size = size;
    c.descriptor = selected->at("mutableSystemDescriptorRva");
    c.function = selected->at("mutableSystemFunctionRva");
    c.metadata = selected->at("metadataRva"); c.metadata_slot = selected->at("registrySlotRva");
    auto off = [&](const char* key) -> std::size_t {
        return layout_offset(model, key);
    };
    c.systems = off("systemsOffset"); c.system_count = off("systemCountOffset"); c.system_stride = off("systemStride");
    c.normal_bindings = off("systemNormalBindingsOffset"); c.alternate_bindings = off("systemAlternateBindingsOffset");
    c.normal_count = off("systemNormalCountOffset"); c.alternate_count = off("systemAlternateCountOffset");
    c.maximum_systems = off("maximumSystems");
    if (!c.maximum_systems || c.maximum_systems > 2048 ||
        c.system_stride < 64 * sizeof(std::uintptr_t) ||
        c.normal_bindings > c.system_stride - 64 * sizeof(std::uintptr_t) ||
        c.alternate_bindings > c.system_stride - 64 * sizeof(std::uintptr_t) ||
        c.normal_count >= c.system_stride || c.alternate_count >= c.system_stride)
        throw std::runtime_error("invalid named service record layout");
    return c;
}

bool named(std::uintptr_t string, std::uint64_t length, const BindingContract& c){
    std::array<char, 128> bytes{};
    return length == c.key.size() && copy(bytes.data(), reinterpret_cast<const void*>(string), length) &&
        std::memcmp(bytes.data(), c.key.data(), length) == 0;
}

// Called only while the original Scene/lifetime gates are held. No allocations,
// caller mutexes, scheduler dispatch or engine calls occur in this resolver.
bool resolve_binding(const BindingContract& c, const EngineContext::EcsView& owner, std::uintptr_t& payload){
    const auto registry = owner.registry_owner;
    std::uintptr_t rows{}, bits{}, keys{}, values{};
    std::uint64_t count{}, capacity{};
    std::uint32_t used{};
    if (!field(registry, c.records, rows) || !rows || !field(registry, c.record_count, count) ||
        !count || count > c.maximum_records || !field(registry, c.map_bits, bits) || !bits ||
        !field(registry, c.map_capacity, capacity) || !capacity || capacity > c.maximum_capacity ||
        (capacity & (capacity - 1)) || !field(registry, c.map_keys, keys) || !keys ||
        !field(registry, c.map_values, values) || !values || !field(registry, c.map_count, used) ||
        !used || used > count || used > capacity || rows > UINTPTR_MAX - count * c.stride) return false;
    auto slot = static_cast<std::size_t>(c.hash) & (capacity - 1);
    std::uintptr_t record{};
    for (std::size_t visited = 0; visited < capacity; ++visited) {
        std::uint64_t word{}; std::uint32_t hash{};
        if (!field(bits, (slot / 64) * 8, word) || !(word & (std::uint64_t{1} << (slot % 64))) ||
            !field(keys, slot * 4, hash)) return false;
        if (hash == c.hash) {
            std::uint16_t index{};
            if (!field(values, slot * 2, index) || index >= count || rows > UINTPTR_MAX - index * c.stride) return false;
            record = rows + index * c.stride;
            break;
        }
        slot = (slot + 1) & (capacity - 1);
    }
    std::uintptr_t name{}, alternate{}, resolved{}; std::uint64_t length{}; std::uint32_t hash{};
    if (!record || !field(record, 0, name) || !field(record, 8, length) || !named(name, length, c) ||
        !field(record, c.record_hash, hash) || hash != c.hash || !field(record, c.normal, resolved) || !resolved ||
        !field(record, c.alternate, alternate) || alternate != resolved) return false;
    // Reject duplicate full names as well as hash collisions in the original
    // records, instead of trusting the original hash-only lookup blindly.
    unsigned matches{};
    for (std::size_t index = 0; index < count; ++index) {
        std::uintptr_t row{};
        if (!add(rows, index * c.stride, row) || !field(row, 0, name) || !field(row, 8, length)) return false;
        if (named(name, length, c) && ++matches > 1) return false;
    }
    if (matches != 1) return false;
    payload = resolved;
    return true;
}

bool resolve(const Contract& c, const EngineContext::EcsView& owner, std::uintptr_t& payload){
    const auto image = owner.owners.image;
    std::uintptr_t metadata{};
    std::uint32_t size{}, type_hash{};
    if (!field(image, c.metadata_slot, metadata) || metadata != image + c.metadata ||
        !field(metadata, 0x40, size) || size != c.size || !field(metadata, 0x50, type_hash) || type_hash != c.type_hash ||
        !resolve_binding(c, owner, payload)) return false;
    const auto registry = owner.registry_owner;
    std::uintptr_t name{};
    std::uint64_t length{};
    std::uint16_t systems{};
    if (!field(registry, c.system_count, systems) || !systems || systems > c.maximum_systems) return false;
    unsigned bound{};
    for (std::size_t index = 0; index < systems; ++index) {
        std::uintptr_t system{};
        if (!add(registry, c.systems + index * c.system_stride, system) ||
            system > UINTPTR_MAX - c.system_stride) return false;
        std::uintptr_t descriptor{};
        if (!field(system, 0, descriptor)) return false;
        if (descriptor != image + c.descriptor) continue;
        if (++bound != 1) return false;
        std::uintptr_t function{}, reads{}, writes{};
        std::uint64_t read_count{}, write_count{};
        if (!field(system, 0x28, function) || function != image + c.function ||
            !field(descriptor, 0x98, reads) || !field(descriptor, 0xa0, read_count) ||
            !field(descriptor, 0xb0, writes) || !field(descriptor, 0xb8, write_count) ||
            read_count > 64 || !write_count || read_count + write_count > 64) return false;
        std::size_t ordinal{}; unsigned mutable_matches{};
        for (std::size_t position = 0; position < read_count + write_count; ++position) {
            std::uintptr_t view{};
            if (!add(position < read_count ? reads : writes,
                    (position < read_count ? position : position - read_count) * 16, view) ||
                !field(view, 0, name) || !field(view, 8, length)) return false;
            if (!named(name, length, c)) continue;
            if (position < read_count || ++mutable_matches != 1) return false;
            ordinal = position;
        }
        std::uint8_t normal_count{}, alternate_count{};
        std::uintptr_t normal_value{}, alternate_value{};
        if (mutable_matches != 1 || !field(system, c.normal_count, normal_count) ||
            !field(system, c.alternate_count, alternate_count) || normal_count != read_count + write_count ||
            alternate_count != normal_count || !field(system, c.normal_bindings + ordinal * 8, normal_value) ||
            !field(system, c.alternate_bindings + ordinal * 8, alternate_value) ||
            normal_value != payload || alternate_value != payload) return false;
    }
    return bound == 1;
}
}
