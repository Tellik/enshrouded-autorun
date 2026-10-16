#include "runtime.h"
#include "ecs_runtime.h"
#include "engine_context.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstring>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>
#include <nlohmann/json.hpp>

// Original named DI storage. Services are not ECS components and therefore do
// not acquire a fabricated component registration or an ECS change-tracking bit.
// Each admitted plain-data contract must have an original mutable service
// binding and an independently established direct in-scene writer.
namespace {
std::mutex operation_mutex;
struct Contract {
    std::string key;
    std::uint32_t hash{}, type_hash{}, size{}, descriptor{}, function{}, metadata{}, metadata_slot{};
    std::size_t records{}, record_count{}, stride{}, record_hash{}, normal{}, alternate{},
        map_bits{}, map_capacity{}, map_keys{}, map_values{}, map_count{},
        systems{}, system_count{}, system_stride{}, normal_bindings{}, alternate_bindings{},
        normal_count{}, alternate_count{}, maximum_records{}, maximum_capacity{}, maximum_systems{};
};
template<class T> bool read(std::uintptr_t at, T& value) {
    if (!at || at > UINTPTR_MAX - sizeof(value)) return false;
    __try { std::memcpy(&value, reinterpret_cast<const void*>(at), sizeof(value)); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
bool copy(void* to, const void* from, std::size_t size) {
    __try { std::memcpy(to, from, size); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
template<class T> bool field(std::uintptr_t base, std::size_t offset, T& value) {
    return base && offset <= UINTPTR_MAX - base && read(base + offset, value);
}
bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& result) {
    if (!base || offset > UINTPTR_MAX - base) return false;
    result = base + offset;
    return true;
}
std::uint32_t hash_name(const std::string& name) {
    std::uint32_t hash = 0x811c9dc5;
    for (const unsigned char byte : name) hash = (hash ^ byte) * 0x1000193;
    return hash;
}
Contract contract(const char* name, std::uint32_t type_hash, std::uint32_t size) {
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
    c.key = name; c.hash = hash_name(c.key); c.type_hash = type_hash; c.size = size;
    c.descriptor = selected->at("mutableSystemDescriptorRva");
    c.function = selected->at("mutableSystemFunctionRva");
    c.metadata = selected->at("metadataRva"); c.metadata_slot = selected->at("registrySlotRva");
    auto off = [&](const char* key) -> std::size_t {
        const auto value = model.at(key).get<std::uint64_t>();
        if (value > 0x10000000) throw std::runtime_error("invalid named service offset");
        return static_cast<std::size_t>(value);
    };
    c.records = off("recordsPointerOffset"); c.record_count = off("recordCountOffset");
    c.stride = off("recordStride"); c.record_hash = off("recordHashOffset");
    c.normal = off("recordNormalPointerOffset"); c.alternate = off("recordAlternatePointerOffset");
    c.map_bits = off("mapBitsOffset"); c.map_capacity = off("mapCapacityOffset");
    c.map_keys = off("mapKeysOffset"); c.map_values = off("mapValuesOffset"); c.map_count = off("mapCountOffset");
    c.systems = off("systemsOffset"); c.system_count = off("systemCountOffset"); c.system_stride = off("systemStride");
    c.normal_bindings = off("systemNormalBindingsOffset"); c.alternate_bindings = off("systemAlternateBindingsOffset");
    c.normal_count = off("systemNormalCountOffset"); c.alternate_count = off("systemAlternateCountOffset");
    c.maximum_records = off("maximumRecords"); c.maximum_capacity = off("maximumMapCapacity");
    c.maximum_systems = off("maximumSystems");
    if (!c.maximum_records || c.maximum_records > 1024 || !c.maximum_systems || c.maximum_systems > 2048 ||
        !c.maximum_capacity || c.maximum_capacity > 4096 || c.stride < 0xb8 ||
        c.normal > c.stride - 8 || c.alternate > c.stride - 8 || c.record_hash > c.stride - 4 ||
        c.system_stride < 64 * sizeof(std::uintptr_t) ||
        c.normal_bindings > c.system_stride - 64 * sizeof(std::uintptr_t) ||
        c.alternate_bindings > c.system_stride - 64 * sizeof(std::uintptr_t) ||
        c.normal_count >= c.system_stride || c.alternate_count >= c.system_stride)
        throw std::runtime_error("invalid named service record layout");
    return c;
}
bool named(std::uintptr_t string, std::uint64_t length, const Contract& c) {
    std::array<char, 128> bytes{};
    return length == c.key.size() && copy(bytes.data(), reinterpret_cast<const void*>(string), length) &&
        std::memcmp(bytes.data(), c.key.data(), length) == 0;
}
// Called only while the original Scene/lifetime gates are held. No allocations,
// caller mutexes, scheduler dispatch or engine calls occur in this resolver.
bool resolve(const Contract& c, const EngineContext::EcsView& owner, std::uintptr_t& payload) {
    const auto image = owner.owners.image;
    std::uintptr_t metadata{};
    std::uint32_t size{}, type_hash{};
    if (!field(image, c.metadata_slot, metadata) || metadata != image + c.metadata ||
        !field(metadata, 0x40, size) || size != c.size || !field(metadata, 0x50, type_hash) || type_hash != c.type_hash)
        return false;
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
    std::uintptr_t name{}, alternate{}; std::uint64_t length{}; std::uint32_t hash{};
    if (!record || !field(record, 0, name) || !field(record, 8, length) || !named(name, length, c) ||
        !field(record, c.record_hash, hash) || hash != c.hash || !field(record, c.normal, payload) || !payload ||
        !field(record, c.alternate, alternate) || alternate != payload) return false;
    // Reject duplicate full names as well as hash collisions in the original
    // records, instead of trusting the original hash-only lookup blindly.
    unsigned matches{};
    for (std::size_t index = 0; index < count; ++index) {
        std::uintptr_t row{};
        if (!add(rows, index * c.stride, row) || !field(row, 0, name) || !field(row, 8, length)) return false;
        if (named(name, length, c) && ++matches > 1) return false;
    }
    if (matches != 1) return false;
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
bool writable(std::uintptr_t address, std::size_t size) {
    if (!address || address > UINTPTR_MAX - size) return false;
    const auto end = address + size;
    while (address < end) {
        MEMORY_BASIC_INFORMATION info{};
        if (VirtualQuery(reinterpret_cast<const void*>(address), &info, sizeof(info)) != sizeof(info) ||
            info.State != MEM_COMMIT || info.Type != MEM_PRIVATE || info.Protect != PAGE_READWRITE) return false;
        const auto base = reinterpret_cast<std::uintptr_t>(info.BaseAddress);
        if (base > UINTPTR_MAX - info.RegionSize || base + info.RegionSize <= address) return false;
        address = (std::min)(end, base + info.RegionSize);
    }
    return true;
}
}

extern "C" bool __cdecl KfcRuntimeServiceRead(const char* name, std::uint32_t type_hash, std::uint32_t size,
    std::uint64_t expected_session, unsigned char* buffer, std::size_t capacity) {
    if (!buffer || capacity != size || !expected_session) return false;
    try {
        const auto c = contract(name, type_hash, size);
        std::scoped_lock operation(operation_mutex);
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) return false;
        EngineContext::EcsWriteGuard guard;
        if (!guard.TryAcquire(session.View()) || size > guard.MaximumBytes()) return false;
        std::uintptr_t address{}, again{};
        return guard.Validate() && resolve(c, session.View(), address) && writable(address, size) &&
            copy(buffer, reinterpret_cast<const void*>(address), size) &&
            resolve(c, session.View(), again) && address == again && guard.Validate();
    } catch (...) { return false; }
}
extern "C" std::uint32_t __cdecl KfcRuntimeServiceCompareExchange(const char* name, std::uint32_t type_hash,
    std::uint32_t size, std::uint64_t expected_session, const void* expected, const void* mask, const void* desired) {
    if (!expected || !mask || !desired || !expected_session) return 0;
    try {
        const auto c = contract(name, type_hash, size);
        std::vector<std::uint8_t> expected_bytes(size), selected(size), wanted(size);
        if (!copy(expected_bytes.data(), expected, size) || !copy(selected.data(), mask, size) ||
            !copy(wanted.data(), desired, size) ||
            !std::all_of(selected.begin(), selected.end(), [](auto byte) { return byte <= 1; }) ||
            std::none_of(selected.begin(), selected.end(), [](auto byte) { return byte != 0; })) return 0;
        std::vector<std::uint8_t> before(size), after(size), verified(size);
        std::scoped_lock operation(operation_mutex);
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) return 0;
        if (!session.WritesAllowed()) return 0;
        EngineContext::EcsWriteGuard guard;
        if (!guard.TryAcquire(session.View()) || size > guard.MaximumBytes()) return 0;
        std::uintptr_t address{}, again{};
        if (!guard.Validate() || !resolve(c, session.View(), address) || !writable(address, size) ||
            !copy(before.data(), reinterpret_cast<const void*>(address), size)) return 0;
        if (before != expected_bytes) return 2;
        for (std::size_t index = 0; index < size; ++index)
            after[index] = selected[index] ? wanted[index] : before[index];
        if (after == before)
            return resolve(c, session.View(), again) && address == again && guard.Validate() ? 1 : 0;
        // Original admin_player writes this same plain-data service directly.
        // Its consumers read the service on every invocation, without ECS stamps.
        if (!copy(reinterpret_cast<void*>(address), after.data(), size) ||
            !copy(verified.data(), reinterpret_cast<const void*>(address), size) || verified != after) {
            copy(reinterpret_cast<void*>(address), before.data(), size);
            if (!copy(verified.data(), reinterpret_cast<const void*>(address), size) || verified != before ||
                !resolve(c, session.View(), again) || address != again || !guard.Validate()) {
                session.MarkWriteUncertain("named-service-write-outcome-uncertain");
                return 3;
            }
            return 0;
        }
        if (!resolve(c, session.View(), again) || address != again || !guard.Validate()) {
            session.MarkWriteUncertain("named-service-owner-changed-after-write");
            return 3;
        }
        return 1;
    } catch (...) { return 0; }
}
