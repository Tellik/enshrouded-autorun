#include "ecs/ecs_session_state.h"

namespace EcsImplementation {
bool compact_entity_pointers(const std::uintptr_t* slots, std::size_t count,
                            const std::uint64_t* occupancy, std::size_t occupancy_count,
                            std::vector<EntityPointer>& pointers){
    if ((!slots && count) || (occupancy && occupancy_count != (count + 63) / 64) ||
        (!occupancy && occupancy_count)) return false;
    pointers.clear();
    if (occupancy) {
        std::size_t occupied{};
        for (std::size_t word = 0; word < occupancy_count; ++word)
            occupied += std::popcount(occupancy[word]);
        pointers.reserve(occupied);
    } else {
        pointers.reserve(count);
    }
    for (std::size_t index = 0; index < count; ++index) {
        const bool active = !occupancy ||
            (occupancy[index / 64] & (std::uint64_t{1} << (index % 64)));
        if (active && slots[index]) pointers.push_back({slots[index], index});
    }
    return true;
}

bool readable(std::uintptr_t address, std::size_t size, ReadRegionCache* regions){
    if (regions) return regions->check(address, size);
    if (!address || !size || address > UINTPTR_MAX - size) return false;
    MEMORY_BASIC_INFORMATION memory{};
    if (!VirtualQuery(reinterpret_cast<const void*>(address), &memory, sizeof(memory))) return false;
    const auto end = reinterpret_cast<std::uintptr_t>(memory.BaseAddress) + memory.RegionSize;
    return memory.State == MEM_COMMIT && !(memory.Protect & (PAGE_GUARD | PAGE_NOACCESS)) &&
        address + size <= end;
}

// Runtime reads originate from the live ECS entity table in this process.
// ReadProcessMemory plus VirtualQuery for every scalar made the incremental
// scan advance only one or two entities per game-thread callback. Keep the
// same fault containment, but copy directly under SEH; callers still validate
// externally supplied ranges where required and invalid/stale pointers fail
// closed on an access violation.
bool guarded_copy(void* destination, const void* source, std::size_t size){
    __try {
        std::memcpy(destination, source, size);
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return false;
    }
}

bool read_bytes(std::uintptr_t address, void* value, std::size_t size){
    return address && value && size && address <= UINTPTR_MAX - size &&
        guarded_copy(value, reinterpret_cast<const void*>(address), size);
}

bool layout_snapshot(ResolvedLayout& result){
    std::scoped_lock lock(state_mutex);
    if (!layout_ready) return false;
    result = live_layout;
    return true;
}

bool entity_pointers(const ResolvedLayout& layout, std::vector<EntityPointer>& pointers){
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
        !read(layout.table_address, table) || !table) {
        std::scoped_lock lock(state_mutex);
        layout_ready = false;
        ++layout_epoch;
        handles.clear();
        reverse_handles.clear();
        return false;
    }
    std::vector<std::uintptr_t> slots(static_cast<std::size_t>(count));
    if (!read_bytes(table, slots.data(), slots.size() * sizeof(slots[0]))) return false;
    pointers.clear();
    if (layout.occupancy_address) {
        std::uintptr_t occupancy{};
        if (!read(layout.occupancy_address, occupancy) || !occupancy) return false;
        std::vector<std::uint64_t> bits((slots.size() + 63) / 64);
        if (!read_bytes(occupancy, bits.data(), bits.size() * sizeof(bits[0]))) return false;
        if (!compact_entity_pointers(slots.data(), slots.size(), bits.data(), bits.size(), pointers)) return false;
        std::uintptr_t occupancy_again{}, table_again{};
        std::uint64_t count_again{};
        if (!read(layout.occupancy_address, occupancy_again) || occupancy_again != occupancy ||
            !read(layout.table_address, table_again) || table_again != table ||
            !read(layout.count_address, count_again) || count_again != count) return false;
    } else {
        if (!compact_entity_pointers(slots.data(), slots.size(), nullptr, 0, pointers)) return false;
    }
    return true;
}

// The original table contains non-null pointers in erased buckets. Match the
// original ID hash lookup instead of treating these stale allocations as live
// entities. Profiles without this independently validated table layout retain
// their existing dispatcher-backed lookup contract.
bool entity_registered(const EntityView& entity, const ResolvedLayout& layout){
    if (!layout.occupancy_address && !layout.keys_address) return true;
    std::uint64_t count{};
    std::uintptr_t table{}, occupancy{}, keys{};
    if (!layout.occupancy_address || !layout.keys_address ||
        !read(layout.count_address, count) || !count || count > (1u << 20) || (count & (count - 1)) ||
        !read(layout.table_address, table) || !table ||
        !read(layout.occupancy_address, occupancy) || !occupancy ||
        !read(layout.keys_address, keys) || !keys) return false;
    auto hash = entity.id;
    hash = ((hash >> 16) ^ hash) * 0x45d9f3bu;
    hash = ((hash >> 16) ^ hash) * 0x45d9f3bu;
    hash = (hash >> 16) ^ hash;
    auto slot = static_cast<std::size_t>(hash & (count - 1));
    for (std::size_t visited = 0; visited < count; ++visited, slot = (slot + 1) & (count - 1)) {
        std::uint64_t bits{};
        std::uint32_t key{};
        if (!read(occupancy + (slot / 64) * 8, bits) || !(bits & (std::uint64_t{1} << (slot % 64)))) return false;
        if (!read(keys + slot * sizeof(key), key)) return false;
        if (key != entity.id) continue;
        std::uintptr_t current{};
        std::uint64_t again{};
        std::uint64_t count_again{};
        std::uintptr_t table_again{}, occupancy_again{}, keys_again{};
        std::uint32_t key_again{}, id_again{}, generation_again{};
        return read(table + slot * sizeof(current), current) && current == entity.pointer &&
            read(occupancy + (slot / 64) * 8, again) && (again & (std::uint64_t{1} << (slot % 64))) &&
            read(keys + slot * sizeof(key_again), key_again) && key_again == entity.id &&
            read(entity.pointer + layout.entity_id, id_again) && id_again == entity.id &&
            read(entity.pointer + layout.entity_generation, generation_again) && generation_again == entity.generation &&
            read(layout.count_address, count_again) && count_again == count &&
            read(layout.table_address, table_again) && table_again == table &&
            read(layout.occupancy_address, occupancy_again) && occupancy_again == occupancy &&
            read(layout.keys_address, keys_again) && keys_again == keys;
    }
    return false;
}

bool entity_view(std::uintptr_t pointer, const ResolvedLayout& layout, EntityView& entity){
    entity.pointer = pointer;
    if (!pointer || !read(pointer + layout.entity_layout, entity.layout) || !entity.layout ||
        !read(pointer + layout.entity_storage, entity.storage) || !entity.storage ||
        !read(pointer + layout.entity_row, entity.row) || entity.row > (1u << 24) ||
        !read(pointer + layout.entity_id, entity.id) || !entity.id ||
        !read(pointer + layout.entity_generation, entity.generation)) return false;
    read(pointer + layout.entity_definition, entity.definition);
    return entity_registered(entity, layout);
}
}
