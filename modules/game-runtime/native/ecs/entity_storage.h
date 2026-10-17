#pragma once
#include "abi/runtime_api.h"
#include "engine-context/engine_owners.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <mutex>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>

namespace EcsImplementation {
struct ComponentType {
    std::uint16_t index; std::uint32_t size;
    bool operator==(const ComponentType&) const = default;
};

struct ResolvedLayout {
    std::uintptr_t count_address{};
    std::uintptr_t table_address{};
    std::uintptr_t occupancy_address{};
    std::uintptr_t keys_address{};
    std::size_t entity_layout{};
    std::size_t entity_storage{};
    std::size_t entity_row{};
    std::size_t entity_id{};
    std::size_t entity_generation{};
    std::size_t entity_definition{};
    std::size_t entity_definition_uuid{};
    bool entity_definition_uuid_validated{};
    std::size_t component_bits{};
    std::size_t component_offsets{};
    std::size_t component_strides{};
};

struct EntityView {
    std::uintptr_t pointer{};
    std::uintptr_t layout{};
    std::uintptr_t storage{};
    std::uintptr_t definition{};
    std::uint32_t row{};
    std::uint32_t id{};
    std::uint32_t generation{};
    bool operator==(const EntityView&) const = default;
};

struct EntityPointer {
    std::uintptr_t pointer{};
    std::size_t table_index{};
};

bool compact_entity_pointers(const std::uintptr_t* slots, std::size_t count,
                            const std::uint64_t* occupancy, std::size_t occupancy_count,
                            std::vector<EntityPointer>& pointers);

struct HandleRecord {
    std::uint32_t id{};
    std::uint32_t generation{};
    std::uint64_t epoch{};
    std::uintptr_t pointer{};
    std::uintptr_t table_slot{};
};

// Read batches reuse page observations only inside this one owner operation.
// SEH still protects every actual payload copy. Each observed region must be
// queried again with identical properties before any output is published.
struct ReadRegionCache {
    struct Region {
        MEMORY_BASIC_INFORMATION memory{};
        std::uintptr_t begin{}, end{};
    };
    std::array<Region, 64> regions{};
    std::size_t count{};

    static bool covers(const MEMORY_BASIC_INFORMATION& memory, std::uintptr_t address, std::size_t size) {
        const auto base = reinterpret_cast<std::uintptr_t>(memory.BaseAddress);
        return memory.State == MEM_COMMIT && !(memory.Protect & (PAGE_GUARD | PAGE_NOACCESS)) &&
            memory.RegionSize <= UINTPTR_MAX - base && address >= base &&
            address - base <= memory.RegionSize && size <= memory.RegionSize - (address - base);
    }
    bool check(std::uintptr_t address, std::size_t size) {
        if (!address || !size || address > UINTPTR_MAX - size) return false;
        for (std::size_t index = 0; index < count; ++index) {
            auto& region = regions[index];
            if (!covers(region.memory, address, size)) continue;
            region.begin = (std::min)(region.begin, address);
            region.end = (std::max)(region.end, address + size);
            return true;
        }
        MEMORY_BASIC_INFORMATION memory{};
        if (!VirtualQuery(reinterpret_cast<const void*>(address), &memory, sizeof(memory)) ||
            !covers(memory, address, size) || count == regions.size()) return false;
        regions[count++] = {memory, address, address + size};
        return true;
    }
    bool revalidate() const {
        for (std::size_t index = 0; index < count; ++index) {
            const auto& region = regions[index];
            MEMORY_BASIC_INFORMATION memory{};
            if (!VirtualQuery(reinterpret_cast<const void*>(region.begin), &memory, sizeof(memory)) ||
                !covers(memory, region.begin, region.end - region.begin) ||
                memory.BaseAddress != region.memory.BaseAddress ||
                memory.AllocationBase != region.memory.AllocationBase ||
                memory.AllocationProtect != region.memory.AllocationProtect ||
                memory.RegionSize != region.memory.RegionSize || memory.State != region.memory.State ||
                memory.Protect != region.memory.Protect || memory.Type != region.memory.Type) return false;
        }
        return true;
    }
};

bool readable(std::uintptr_t address, std::size_t size, ReadRegionCache* regions = nullptr);

bool guarded_copy(void* destination, const void* source, std::size_t size);

bool read_bytes(std::uintptr_t address, void* value, std::size_t size);

template<class T> bool read(std::uintptr_t address, T& value) {
    return read_bytes(address, &value, sizeof(value));
}

bool layout_snapshot(ResolvedLayout& result);

bool entity_pointers(const ResolvedLayout& layout, std::vector<EntityPointer>& pointers);

bool entity_registered(const EntityView& entity, const ResolvedLayout& layout);

bool entity_view(std::uintptr_t pointer, const ResolvedLayout& layout, EntityView& entity);

bool component_presence(const EntityView& entity, const ResolvedLayout& layout,
                        const ComponentType& component, bool& present);

bool component_present(const EntityView& entity, const ResolvedLayout& layout, const ComponentType& component);

bool component_address(const EntityView& entity, const ResolvedLayout& layout,
                       const ComponentType& component, std::uintptr_t& address, bool check_page = true,
                       ReadRegionCache* regions = nullptr);

bool resolve_component(const char* name, ComponentType& component);

std::uint64_t identity_key(std::uint32_t id, std::uint32_t generation);

std::uint32_t handle_for(const EntityView& entity, std::size_t table_index = SIZE_MAX);

bool SnapshotEntityIds(std::vector<std::uint32_t>& ids);

bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop);

bool entity_for_handle(std::uint32_t handle, const ResolvedLayout& layout, EntityView& entity);

struct ResolveOperation { std::uint32_t entity_id{}, result{}; };

void resolve_on_game_thread(void* opaque);

struct TemplateUuidOperation {
    std::uint32_t handle{};
    std::array<std::uint8_t, 16> uuid{};
    bool result{};
};

void template_uuid_on_game_thread(void* opaque);

}
