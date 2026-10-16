#include "ecs_runtime.h"
#include "component_registry.h"
#include "registry_discovery.h"
#include "dispatcher.h"
#include <nlohmann/json.hpp>
#include "profile.h"
#include "engine_context.h"
#include "world_runtime.h"
#include "patch_runtime.h"

#include <windows.h>
#include <algorithm>
#include <array>
#include <atomic>
#include <bit>
#include <chrono>
#include <cmath>
#include <cstddef>
#include <cstring>
#include <mutex>
#include <memory>
#include <sstream>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>

static_assert(sizeof(KfcRuntimePropRecord) == 88);
static_assert(sizeof(KfcRuntimePropRecipe) == 32);
static_assert(offsetof(KfcRuntimePropRecord, position) == 8);
static_assert(offsetof(KfcRuntimePropRecord, orientation) == 32);
static_assert(offsetof(KfcRuntimePropRecord, scale) == 48);
static_assert(offsetof(KfcRuntimePropRecord, template_uuid) == 64);
static_assert(offsetof(KfcRuntimePropRecord, entity_id) == 80);

namespace {
constexpr std::size_t max_components = 1024;

struct OperationCounters {
    std::atomic<std::uint64_t> queries{}, query_successes{}, query_incomplete{}, query_failures{};
    std::atomic<std::uint64_t> resolves{}, resolve_successes{}, resolve_failures{};
    std::atomic<std::uint64_t> reads{}, read_successes{}, read_failures{};
    std::atomic<std::uint64_t> read_batches{}, read_batch_successes{}, read_batch_failures{};
    std::atomic<std::uint64_t> read_batch_components{}, read_batch_absent{};
    std::atomic<std::uint64_t> writes{}, write_successes{}, write_failures{};
};
OperationCounters operation_counters;
std::atomic<std::uint64_t> active_query_cursor{}, active_query_total{}, active_query_matches{};

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
                            std::vector<EntityPointer>& pointers) {
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
struct ComponentSlot { std::uint16_t index{}, stride{}, offset{}; };
struct TemplateLayoutSample {
    std::uint64_t uuid[2]{};
    std::uintptr_t layout{};
    std::string name;
    std::uint64_t entity_count{};
    std::vector<ComponentSlot> components;
};
struct HandleRecord {
    std::uint32_t id{};
    std::uint32_t generation{};
    std::uint64_t epoch{};
    std::uintptr_t pointer{};
    std::uintptr_t table_slot{};
};

std::mutex state_mutex;
// Entity operations and lifecycle invalidation share this outer lock. Discovery
// scans run outside it. Lock order is operation, discovery, then state.
std::recursive_mutex operation_mutex;
std::unordered_map<std::string, ComponentType> types;
std::unordered_map<std::string, std::uint32_t> configured_types;
ComponentRegistry::Snapshot component_registry;
bool component_registry_ready{};
bool component_registry_world_bound{};
bool component_registry_seen{};
std::string component_registry_error{"awaiting entity manager"};
std::uint64_t component_registry_last_read{};
std::uintptr_t component_registry_manager{};
std::string component_registry_json;
std::unordered_map<std::uint32_t, std::unordered_set<std::uint16_t>> observed_indices_by_size;
std::unordered_set<std::uintptr_t> discovered_layouts;
std::unordered_map<std::uintptr_t, std::vector<ComponentSlot>> discovered_layout_components;
std::unordered_map<std::string, TemplateLayoutSample> template_layout_samples;
std::uint64_t template_layout_samples_dropped{};
std::vector<std::uintptr_t> discovery_entities;
std::size_t discovery_cursor{};
std::uintptr_t discovery_table{};
std::uint64_t discovery_last_restart{};
std::uint64_t discovery_scanned_entities{};
std::uint64_t discovery_layout_count{};
std::uint64_t discovery_restarts{};
bool discovery_complete{};
std::mutex discovery_mutex;
ComponentRegistry::Discovery registry_discovery;
ResolvedLayout live_layout{};
bool layout_ready{};
bool read_only_binding{};
EngineContext::EcsView read_only_view;
std::uintptr_t live_table{};
std::unordered_map<std::uint32_t, HandleRecord> handles;
std::unordered_map<std::uint64_t, std::uint32_t> reverse_handles;
struct QueryScanState {
    std::uint64_t epoch{};
    std::uint64_t touched_ms{};
    std::vector<EntityPointer> pointers;
    std::size_t cursor{};
    std::vector<std::uint32_t> matches;
    std::unordered_set<std::uint64_t> seen;
};
struct QueryCacheEntry {
    std::uint64_t epoch{};
    std::uint64_t completed_ms{};
    std::uint64_t touched_ms{};
    std::vector<std::uint32_t> matches;
};
std::unordered_map<std::string, QueryScanState> query_scans;
std::unordered_map<std::string, QueryCacheEntry> query_cache;
struct ResolveScanState {
    std::uint64_t epoch{};
    std::uint64_t touched_ms{};
    std::vector<EntityPointer> pointers;
    std::size_t cursor{};
};
struct ResolveCacheEntry {
    std::uint64_t epoch{};
    std::uint64_t completed_ms{};
    std::uint32_t handle{};
};
std::unordered_map<std::uint32_t, ResolveScanState> resolve_scans;
std::unordered_map<std::uint32_t, ResolveCacheEntry> resolve_cache;
std::uint32_t next_handle{1};
std::uint64_t layout_epoch{1};
// Protected by state_mutex. A confirmed new native binding epoch is the only
// reset. Busy guards and ordinary expected-value conflicts never set this.
std::uint64_t write_fault_epoch{};
const char* write_fault_reason{};
bool write_fault_active_locked() { return write_fault_epoch == layout_epoch && write_fault_reason; }
std::mutex write_mutex;
std::mutex prop_recipe_mutex;
struct RegisteredPropRecipe { std::array<float, 6> bounds{}; std::uint32_t feedback{}; };
std::unordered_map<std::uint32_t, RegisteredPropRecipe> prop_recipes;
bool prop_recipe_catalog_ready{};
std::mutex prop_query_call_mutex;
struct PropQueryResultCache {
    std::array<double, 6> bounds{};
    std::uint64_t epoch{};
    std::vector<KfcRuntimePropRecord> props;
    bool ready{};
};
PropQueryResultCache prop_query_result_cache;
std::atomic<bool> stop_requested{};
std::uintptr_t image_base{};
std::size_t image_size{};

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

bool readable(std::uintptr_t address, std::size_t size, ReadRegionCache* regions = nullptr) {
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
bool guarded_copy(void* destination, const void* source, std::size_t size) {
    __try {
        std::memcpy(destination, source, size);
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return false;
    }
}

bool read_bytes(std::uintptr_t address, void* value, std::size_t size) {
    return address && value && size && address <= UINTPTR_MAX - size &&
        guarded_copy(value, reinterpret_cast<const void*>(address), size);
}
template<class T> bool read(std::uintptr_t address, T& value) {
    return read_bytes(address, &value, sizeof(value));
}


bool layout_snapshot(ResolvedLayout& result) {
    std::scoped_lock lock(state_mutex);
    if (!layout_ready) return false;
    result = live_layout;
    return true;
}
bool entity_pointers(const ResolvedLayout& layout, std::vector<EntityPointer>& pointers) {
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
bool entity_registered(const EntityView& entity, const ResolvedLayout& layout) {
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
bool entity_view(std::uintptr_t pointer, const ResolvedLayout& layout, EntityView& entity) {
    entity.pointer = pointer;
    if (!pointer || !read(pointer + layout.entity_layout, entity.layout) || !entity.layout ||
        !read(pointer + layout.entity_storage, entity.storage) || !entity.storage ||
        !read(pointer + layout.entity_row, entity.row) || entity.row > (1u << 24) ||
        !read(pointer + layout.entity_id, entity.id) || !entity.id ||
        !read(pointer + layout.entity_generation, entity.generation)) return false;
    read(pointer + layout.entity_definition, entity.definition);
    return entity_registered(entity, layout);
}
bool component_presence(const EntityView& entity, const ResolvedLayout& layout,
                        const ComponentType& component, bool& present) {
    std::uint64_t bits{};
    if (component.index >= max_components ||
        !read(entity.layout + layout.component_bits + (component.index / 64) * 8, bits)) return false;
    present = (bits & (std::uint64_t{1} << (component.index % 64))) != 0;
    return true;
}
bool component_present(const EntityView& entity, const ResolvedLayout& layout, const ComponentType& component) {
    bool present{};
    return component_presence(entity, layout, component, present) && present;
}
bool component_address(const EntityView& entity, const ResolvedLayout& layout,
                       const ComponentType& component, std::uintptr_t& address, bool check_page = true,
                       ReadRegionCache* regions = nullptr) {
    std::uint16_t offset{}, stride{};
    if (!component_present(entity, layout, component) ||
        !read(entity.layout + layout.component_offsets + component.index * 2, offset) ||
        !read(entity.layout + layout.component_strides + component.index * 2, stride) || stride != component.size) return false;
    const auto delta = offset + static_cast<std::uintptr_t>(entity.row) * stride;
    if (entity.storage > UINTPTR_MAX - delta) return false;
    address = entity.storage + delta;
    // Read-only scan callers immediately copy through guarded_copy (SEH).
    // Re-querying the page for every component doubles kernel transitions for
    // every prop in the world; writes and address-only queries keep the check.
    return !check_page || readable(address, component.size, regions);
}
bool resolve_component(const char* name, ComponentType& component) {
    if (!name) return false;
    std::scoped_lock lock(state_mutex);
    const auto found = types.find(name);
    if (found == types.end()) return false;
    component = found->second;
    return true;
}
std::uint64_t identity_key(std::uint32_t id, std::uint32_t generation) {
    return static_cast<std::uint64_t>(generation) << 32 | id;
}
std::uint32_t handle_for(const EntityView& entity, std::size_t table_index = SIZE_MAX) {
    std::scoped_lock lock(state_mutex);
    const auto table_slot = read_only_binding && table_index != SIZE_MAX ?
        read_only_view.entity_table + table_index * sizeof(std::uintptr_t) : std::uintptr_t{};
    const auto key = identity_key(entity.id, entity.generation);
    if (const auto found = reverse_handles.find(key); found != reverse_handles.end()) {
        auto& record = handles.at(found->second);
        if (record.pointer != entity.pointer) record.table_slot = 0;
        record.pointer = entity.pointer;
        if (table_slot) record.table_slot = table_slot;
        return found->second;
    }
    // Never recycle an opaque handle during this process, including world changes.
    if (!next_handle) return 0;
    const auto handle = next_handle++;
    handles.emplace(handle, HandleRecord{entity.id, entity.generation, layout_epoch, entity.pointer, table_slot});
    reverse_handles.emplace(key, handle);
    return handle;
}

bool SnapshotEntityIds(std::vector<std::uint32_t>& ids) {
    ResolvedLayout layout{};
    std::vector<EntityPointer> pointers;
    if (!layout_snapshot(layout) || !entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    ids.clear();
    ids.reserve(pointers.size());
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (entity_view(pointer.pointer, layout, entity)) ids.push_back(entity.id);
    }
    return true;
}

bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop) {
    if (!entity_id || !prop) return false;
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return false; }
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType transform_component{}, item_component{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform_component) || transform_component.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item_component) || item_component.size != sizeof(std::uint32_t)) return false;
    ResolvedLayout layout{};
    std::vector<EntityPointer> pointers;
    if (!layout_snapshot(layout) || !entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) || entity.id != entity_id) continue;
        std::uintptr_t transform_address{}, item_address{};
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        std::uint32_t item_id{};
        std::uint64_t template_uuid[2]{};
        if (!component_address(entity, layout, transform_component, transform_address) ||
            !component_address(entity, layout, item_component, item_address) ||
            !read_bytes(transform_address, &transform, sizeof(transform)) ||
            !read_bytes(item_address, &item_id, sizeof(item_id)) || !item_id || !entity.definition ||
            !read_bytes(entity.definition + definition_uuid, template_uuid, sizeof(template_uuid)) ||
            !(template_uuid[0] || template_uuid[1])) return false;
        *prop = {};
        prop->entity_handle = handle_for(entity, pointer.table_index);
        prop->entity_id = entity.id;
        prop->item_id = item_id;
        std::copy_n(transform.position, 3, prop->position);
        std::copy_n(transform.rotation, 4, prop->orientation);
        std::copy_n(transform.scale, 3, prop->scale);
        std::copy_n(template_uuid, 2, prop->template_uuid);
        if (!prop->entity_handle) return false;
        return true;
    }
    return false;
}
bool entity_for_handle(std::uint32_t handle, const ResolvedLayout& layout, EntityView& entity) {
    HandleRecord record{};
    bool direct{};
    std::uintptr_t bound_table{};
    {
        std::scoped_lock lock(state_mutex);
        const auto found = handles.find(handle);
        if (found == handles.end() || found->second.epoch != layout_epoch) return false;
        record = found->second;
        direct = read_only_binding;
        bound_table = read_only_view.entity_table;
    }
    if (direct) {
        std::uint64_t count{};
        std::uintptr_t table{}, pointer{};
        if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
            !read(layout.table_address, table) || table != bound_table) return false;
        const auto end = table + count * sizeof(std::uintptr_t);
        if (record.table_slot < table || record.table_slot >= end ||
            !read(record.table_slot, pointer) || pointer != record.pointer) {
            // Discover the slot once, then validate it on every read. A readable
            // object with an old ID is insufficient after entity removal.
            std::array<std::uintptr_t, 256> batch{};
            record.table_slot = 0;
            for (std::size_t offset = 0; offset < count && !record.table_slot; offset += batch.size()) {
                const auto size = (std::min)(batch.size(), static_cast<std::size_t>(count) - offset);
                if (!read_bytes(table + offset * sizeof(pointer), batch.data(), size * sizeof(pointer))) return false;
                for (std::size_t index = 0; index < size; ++index) if (batch[index] == record.pointer) {
                    record.table_slot = table + (offset + index) * sizeof(pointer);
                    break;
                }
            }
            if (!record.table_slot || !read(record.table_slot, pointer) || pointer != record.pointer) return false;
            std::scoped_lock lock(state_mutex);
            const auto found = handles.find(handle);
            if (found == handles.end() || found->second.epoch != record.epoch || found->second.pointer != record.pointer) return false;
            found->second.table_slot = record.table_slot;
        }
    }
    // Re-read the live identity and layout; never cache component addresses.
    // A moved/deleted entity is rediscovered by the next query, not a full-world
    // scan for each individual component read and write.
    return entity_view(record.pointer, layout, entity) && entity.id == record.id &&
        entity.generation == record.generation;
}

struct QueryOperation {
    const char* const* names{};
    std::size_t count{};
    std::uint32_t* entities{};
    std::size_t capacity{};
    std::size_t result{SIZE_MAX};
    std::vector<std::string> owned_names;
    std::vector<const char*> name_pointers;
    std::vector<std::uint32_t> output;
};
struct BoundsQueryOperation {
    std::vector<std::string> names;
    double bounds[6]{};
    double padding{};
    std::vector<std::uint32_t> output;
    std::size_t capacity{};
    std::size_t result{SIZE_MAX};
};
struct PropQueryOperation {
    double bounds[6]{};
    double padding{};
    bool exact_recipe_bounds{};
    std::vector<KfcRuntimePropRecord> output;
    std::size_t result{SIZE_MAX};
    std::vector<EntityPointer> pointers;
    std::unordered_set<std::uint64_t> seen;
    std::unordered_map<std::uint32_t, RegisteredPropRecipe> recipes;
    std::size_t cursor{};
    std::uint64_t epoch{}, started_ms{}, touched_ms{}, progressed_ms{};
    bool initialized{};
};
std::shared_ptr<PropQueryOperation> pending_prop_query;
std::string query_key(const QueryOperation& operation) {
    std::string key;
    for (const auto& name : operation.owned_names) {
        key.append(std::to_string(name.size()));
        key.push_back(':');
        key.append(name);
        key.push_back(';');
    }
    return key;
}
std::uint64_t current_epoch() {
    std::scoped_lock lock(state_mutex);
    return layout_epoch;
}
bool invoke_read_operation(GameThreadDispatcher::Operation operation, const std::shared_ptr<void>& context,
                           std::uint32_t timeout_ms = 75) {
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (!direct) return GameThreadDispatcher::Invoke(operation, context, timeout_ms);
    std::scoped_lock operation_lock(operation_mutex);
    EngineContext::EcsView expected, before, after;
    std::uint64_t epoch{};
    {
        std::scoped_lock lock(state_mutex);
        if (!read_only_binding || !layout_ready || !component_registry_ready || !component_registry_world_bound) return false;
        expected = read_only_view;
        epoch = layout_epoch;
    }
    if (!EngineContext::ResolveEcs(before) || before != expected) return false;
    operation(context.get());
    return EngineContext::ResolveEcs(after) && after == before && current_epoch() == epoch;
}
void publish_query_result(QueryOperation& operation, const std::vector<std::uint32_t>& matches) {
    operation.result = matches.size();
    if (operation.entities && operation.capacity)
        std::copy_n(matches.begin(), (std::min)(operation.capacity, matches.size()), operation.entities);
}
void publish_current_query_result(QueryOperation& operation, const QueryCacheEntry& cached,
                                  const ResolvedLayout& layout, const std::vector<ComponentType>& components) {
    // Discovery is incremental, but a known entity need not disappear from every
    // caller while that scan refreshes. Revalidate its real identity, live table
    // membership and requested components. Never retain component addresses.
    std::vector<std::uint32_t> matches;
    matches.reserve(cached.matches.size());
    for (const auto handle : cached.matches) {
        EntityView entity{};
        if (!entity_for_handle(handle, layout, entity)) continue;
        if (std::all_of(components.begin(), components.end(), [&](const auto& component) {
                return component_present(entity, layout, component);
            })) matches.push_back(handle);
    }
    publish_query_result(operation, matches);
}
void query_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<QueryOperation*>(opaque);
    std::vector<ComponentType> components(operation.count);
    for (std::size_t index = 0; index < operation.count; ++index)
        if (!resolve_component(operation.names[index], components[index])) return;
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    const auto epoch = current_epoch();
    const auto now_ms = GetTickCount64();
    const auto key = query_key(operation);
    for (auto iterator = query_scans.begin(); iterator != query_scans.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = query_scans.erase(iterator);
        else ++iterator;
    }
    for (auto iterator = query_cache.begin(); iterator != query_cache.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = query_cache.erase(iterator);
        else ++iterator;
    }
    auto cached = query_cache.find(key);
    if (cached != query_cache.end()) {
        if (cached->second.epoch != epoch) {
            query_cache.erase(cached);
            cached = query_cache.end();
        } else {
            cached->second.touched_ms = now_ms;
        }
        if (cached != query_cache.end() && now_ms - cached->second.completed_ms <= 50) {
            publish_current_query_result(operation, cached->second, layout, components);
            return;
        }
    }
    auto scan = query_scans.find(key);
    if (scan == query_scans.end() || scan->second.epoch != epoch) {
        QueryScanState fresh{};
        fresh.epoch = epoch;
        fresh.touched_ms = now_ms;
        if (!entity_pointers(layout, fresh.pointers)) return;
        scan = query_scans.insert_or_assign(key, std::move(fresh)).first;
        active_query_cursor.store(0, std::memory_order_relaxed);
        active_query_total.store(scan->second.pointers.size(), std::memory_order_relaxed);
        active_query_matches.store(0, std::memory_order_relaxed);
    }
    auto& progress = scan->second;
    progress.touched_ms = now_ms;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(1);
    std::size_t visited = 0;
    for (; progress.cursor < progress.pointers.size() && visited < 8192; ++progress.cursor, ++visited) {
        const auto pointer = progress.pointers[progress.cursor];
        EntityView entity{};
        if (entity_view(pointer.pointer, layout, entity)) {
            bool include = true;
            for (const auto& component : components) {
                if (!component_present(entity, layout, component)) { include = false; break; }
            }
            if (include && progress.seen.insert(identity_key(entity.id, entity.generation)).second)
                progress.matches.push_back(handle_for(entity, pointer.table_index));
        }
        if (std::chrono::steady_clock::now() >= deadline) {
            ++progress.cursor;
            break;
        }
    }
    if (progress.cursor < progress.pointers.size()) {
        // Keep publishing validated members of the last complete discovery
        // snapshot while advancing the next scan. Only the first scan is
        // pending. New members become visible when discovery completes.
        if (cached != query_cache.end())
            publish_current_query_result(operation, cached->second, layout, components);
        else operation.result = SIZE_MAX - 1;
        active_query_cursor.store(progress.cursor, std::memory_order_relaxed);
        active_query_total.store(progress.pointers.size(), std::memory_order_relaxed);
        active_query_matches.store(progress.matches.size(), std::memory_order_relaxed);
        return;
    }
    active_query_cursor.store(progress.cursor, std::memory_order_relaxed);
    active_query_total.store(progress.pointers.size(), std::memory_order_relaxed);
    active_query_matches.store(progress.matches.size(), std::memory_order_relaxed);
    auto completed = std::move(progress.matches);
    query_scans.erase(scan);
    const auto completed_ms = GetTickCount64();
    auto [refreshed, _] = query_cache.insert_or_assign(key,
        QueryCacheEntry{epoch, completed_ms, completed_ms, std::move(completed)});
    publish_current_query_result(operation, refreshed->second, layout, components);
}
void query_bounds_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<BoundsQueryOperation*>(opaque);
    std::vector<ComponentType> components;
    components.reserve(operation.names.size());
    for (const auto& name : operation.names) {
        ComponentType component{};
        if (!resolve_component(name.c_str(), component)) return;
        components.push_back(component);
    }
    const auto transform_type = std::find_if(operation.names.begin(), operation.names.end(),
        [](const auto& name) { return name == "keen::ecs::CurrentTransform"; });
    if (transform_type == operation.names.end()) return;
    const auto component_index = static_cast<std::size_t>(transform_type - operation.names.begin());
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    std::vector<EntityPointer> pointers;
    if (!entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return;
    operation.output.clear();
    std::unordered_set<std::uint32_t> seen;
    seen.reserve(pointers.size());
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity)) continue;
        if (!seen.insert(entity.id).second) continue;
        bool include = true;
        for (const auto& component : components) {
            if (!component_present(entity, layout, component)) { include = false; break; }
        }
        if (!include) continue;
        std::uintptr_t address{};
        if (!component_address(entity, layout, components[component_index], address)) continue;
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        static_assert(sizeof(NativeTransform) == 0x38);
        if (!read_bytes(address, &transform, sizeof(transform))) continue;
        const auto scale = (std::max)({std::abs(static_cast<double>(transform.scale[0])),
            std::abs(static_cast<double>(transform.scale[1])), std::abs(static_cast<double>(transform.scale[2]))});
        if (!std::isfinite(scale)) continue;
        const auto margin = operation.padding * scale;
        bool inside = true;
        for (int axis = 0; axis < 3; ++axis) {
            const auto world_position = static_cast<double>(transform.position[axis]) / 4294967296.0;
            if (!std::isfinite(world_position) || world_position < operation.bounds[axis] - margin ||
                world_position >= operation.bounds[axis + 3] + margin) { inside = false; break; }
        }
        if (inside) operation.output.push_back(handle_for(entity, pointer.table_index));
    }
    operation.result = operation.output.size();
}
bool recipe_bounds_intersect(const KfcRuntimePropRecord& prop,
                             const std::array<float, 6>& bounds,
                             const double* query_bounds) {
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) ||
            bounds[axis] > bounds[axis + 3]) return false;
    }
    const double qx = prop.orientation[0], qy = prop.orientation[1];
    const double qz = prop.orientation[2], qw = prop.orientation[3];
    const double norm = qx*qx + qy*qy + qz*qz + qw*qw;
    const double sx = prop.scale[0], sy = prop.scale[1], sz = prop.scale[2];
    if (!std::isfinite(norm) || norm <= 1e-12 || !std::isfinite(sx) ||
        !std::isfinite(sy) || !std::isfinite(sz)) return false;
    const double factor = 2.0 / norm;
    const double rotation[3][3] = {
        {1-factor*(qy*qy+qz*qz), factor*(qx*qy-qz*qw), factor*(qx*qz+qy*qw)},
        {factor*(qx*qy+qz*qw), 1-factor*(qx*qx+qz*qz), factor*(qy*qz-qx*qw)},
        {factor*(qx*qz-qy*qw), factor*(qy*qz+qx*qw), 1-factor*(qx*qx+qy*qy)},
    };
    const double scale[3]{sx, sy, sz};
    double local_center[3]{}, local_half[3]{};
    for (int axis = 0; axis < 3; ++axis) {
        local_center[axis] = (static_cast<double>(bounds[axis]) + bounds[axis + 3]) * 0.5;
        local_half[axis] = std::abs(static_cast<double>(bounds[axis + 3]) - bounds[axis]) * 0.5 * std::abs(scale[axis]);
        local_center[axis] *= scale[axis];
    }
    double world_center[3]{}, world_half[3]{};
    for (int row = 0; row < 3; ++row) {
        world_center[row] = static_cast<double>(prop.position[row]) / 4294967296.0;
        for (int column = 0; column < 3; ++column) {
            world_center[row] += rotation[row][column] * local_center[column];
            world_half[row] += std::abs(rotation[row][column]) * local_half[column];
        }
    }
    if (world_half[0] + world_half[1] + world_half[2] < 1e-6)
        world_half[0] = world_half[1] = world_half[2] = 0.25;
    for (int axis = 0; axis < 3; ++axis) {
        if (world_center[axis] + world_half[axis] < query_bounds[axis] ||
            world_center[axis] - world_half[axis] >= query_bounds[axis + 3]) return false;
    }
    return true;
}
void query_props_native(PropQueryOperation& operation) {
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType transform_component{}, item_component{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform_component) || transform_component.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item_component) || item_component.size != 4) return;
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    if (!operation.initialized) {
        if (!entity_pointers(layout, operation.pointers) || operation.pointers.size() > (1u << 20)) return;
        if (operation.exact_recipe_bounds) {
            std::scoped_lock lock(prop_recipe_mutex);
            if (!prop_recipe_catalog_ready) return;
            operation.recipes = prop_recipes;
        }
        operation.seen.reserve(operation.pointers.size());
        operation.initialized = true;
    }
    const auto slice_deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(2);
    std::size_t visited{};
    while (operation.cursor < operation.pointers.size()) {
        if (operation.exact_recipe_bounds &&
            (visited >= 8192 || (visited > 0 && std::chrono::steady_clock::now() >= slice_deadline))) {
            operation.result = SIZE_MAX - 1;
            return;
        }
        const auto pointer = operation.pointers[operation.cursor++];
        ++visited;
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) || !operation.seen.insert(identity_key(entity.id, entity.generation)).second) continue;
        std::uintptr_t transform_address{}, item_address{};
        if (!component_address(entity, layout, transform_component, transform_address, false) ||
            !component_address(entity, layout, item_component, item_address, false)) continue;
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        static_assert(sizeof(NativeTransform) == 0x38);
        std::uint32_t item_id{};
        if (!read_bytes(transform_address, &transform, sizeof(transform)) ||
            !read_bytes(item_address, &item_id, sizeof(item_id)) || !item_id) continue;
        KfcRuntimePropRecord prop{};
        prop.item_id = item_id;
        std::copy_n(transform.position, 3, prop.position);
        std::copy_n(transform.rotation, 4, prop.orientation);
        std::copy_n(transform.scale, 3, prop.scale);
        if (operation.exact_recipe_bounds) {
            const auto recipe = operation.recipes.find(item_id);
            if (recipe == operation.recipes.end() || !recipe_bounds_intersect(prop, recipe->second.bounds, operation.bounds)) continue;
            // UUIDs are needed by the editor only for props inside the selected
            // region. Reading them before the spatial test made every capture
            // touch the template definition of every UsedItem in the world.
            if (!entity.definition || !read_bytes(entity.definition + definition_uuid,
                    prop.template_uuid, sizeof(prop.template_uuid)) ||
                !(prop.template_uuid[0] || prop.template_uuid[1])) continue;
        } else {
            const double scale = (std::max)({std::abs(static_cast<double>(transform.scale[0])),
                std::abs(static_cast<double>(transform.scale[1])), std::abs(static_cast<double>(transform.scale[2]))});
            if (!std::isfinite(scale)) continue;
            const auto margin = operation.padding * scale;
            bool inside = true;
            for (int axis = 0; axis < 3; ++axis) {
                const auto position = static_cast<double>(transform.position[axis]) / 4294967296.0;
                if (!std::isfinite(position) || position < operation.bounds[axis] - margin ||
                    position >= operation.bounds[axis + 3] + margin) { inside = false; break; }
            }
            if (!inside) continue;
        }
        prop.entity_handle = handle_for(entity, pointer.table_index);
        prop.entity_id = entity.id;
        if (prop.entity_handle) operation.output.push_back(prop);
    }
    operation.result = operation.output.size();
}

struct ResolveOperation { std::uint32_t entity_id{}, result{}; };
void resolve_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ResolveOperation*>(opaque);
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    const auto epoch = current_epoch();
    const auto now_ms = GetTickCount64();
    for (auto iterator = resolve_scans.begin(); iterator != resolve_scans.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = resolve_scans.erase(iterator);
        else ++iterator;
    }
    for (auto iterator = resolve_cache.begin(); iterator != resolve_cache.end();) {
        if (now_ms - iterator->second.completed_ms > 2000) iterator = resolve_cache.erase(iterator);
        else ++iterator;
    }
    if (auto cached = resolve_cache.find(operation.entity_id); cached != resolve_cache.end()) {
        if (cached->second.epoch == epoch && now_ms - cached->second.completed_ms <= 50) {
            operation.result = cached->second.handle;
            return;
        }
        resolve_cache.erase(cached);
    }
    auto scan = resolve_scans.find(operation.entity_id);
    if (scan == resolve_scans.end() || scan->second.epoch != epoch) {
        ResolveScanState fresh{};
        fresh.epoch = epoch;
        fresh.touched_ms = now_ms;
        if (!entity_pointers(layout, fresh.pointers)) return;
        scan = resolve_scans.insert_or_assign(operation.entity_id, std::move(fresh)).first;
    }
    auto& progress = scan->second;
    progress.touched_ms = now_ms;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(1);
    std::size_t visited = 0;
    for (; progress.cursor < progress.pointers.size() && visited < 256; ++progress.cursor, ++visited) {
        EntityView candidate{};
        if (entity_view(progress.pointers[progress.cursor].pointer, layout, candidate) &&
            candidate.id == operation.entity_id) {
            operation.result = handle_for(candidate, progress.pointers[progress.cursor].table_index);
            resolve_cache.insert_or_assign(operation.entity_id,
                ResolveCacheEntry{epoch, now_ms, operation.result});
            resolve_scans.erase(scan);
            return;
        }
        if (std::chrono::steady_clock::now() >= deadline) {
            ++progress.cursor;
            break;
        }
    }
    if (progress.cursor >= progress.pointers.size()) {
        resolve_cache.insert_or_assign(operation.entity_id, ResolveCacheEntry{epoch, now_ms, 0});
        resolve_scans.erase(scan);
    }
}

struct ReadOperation {
    std::uint32_t handle{};
    const char* name{};
    void* value{};
    std::size_t size{};
    bool result{};
    std::string owned_name;
    std::vector<std::uint8_t> output;
};
void read_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ReadOperation*>(opaque);
    ComponentType component{};
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    if (!(operation.value && resolve_component(operation.name, component) &&
        component.size == operation.size && layout_snapshot(layout) &&
        entity_for_handle(operation.handle, layout, entity) &&
        component_address(entity, layout, component, address) &&
        read_bytes(address, operation.value, operation.size))) return;
    // A non-dispatched read may race an archetype move or an engine update.
    // Publish only equal payload copies belonging to the same current entity.
    std::vector<std::uint8_t> again(operation.size);
    EntityView current{};
    std::uintptr_t current_address{};
    operation.result = read_bytes(address, again.data(), again.size()) &&
        std::memcmp(operation.value, again.data(), operation.size) == 0 &&
        entity_for_handle(operation.handle, layout, current) && current == entity &&
        component_address(current, layout, component, current_address) && current_address == address;
}

struct ReadManyOperation {
    std::uint32_t handle{};
    std::vector<std::string> names;
    std::vector<std::uint32_t> sizes;
    std::vector<ComponentType> components;
    std::vector<std::uintptr_t> addresses;
    std::vector<std::uint8_t> output, present, verification;
    ReadRegionCache regions;
    bool result{};
};
void read_many_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ReadManyOperation*>(opaque);
    operation.regions = {};
    ResolvedLayout layout{};
    EntityView entity{};
    if (!layout_snapshot(layout) || !entity_for_handle(operation.handle, layout, entity)) return;
    for (std::size_t index = 0; index < operation.names.size(); ++index) {
        auto& component = operation.components[index];
        if (!resolve_component(operation.names[index].c_str(), component) ||
            component.size != operation.sizes[index]) return;
    }
    std::size_t offset{};
    for (std::size_t index = 0; index < operation.names.size(); ++index) {
        const auto& component = operation.components[index];
        const auto size = operation.sizes[index];
        bool present{};
        // An unreadable presence mask cannot be reported as a missing
        // component: that might conceal another initialized attribute root.
        if (!component_presence(entity, layout, component, present)) return;
        if (present) {
            auto& address = operation.addresses[index];
            auto* output = operation.output.data() + offset;
            EntityView current{};
            std::uintptr_t current_address{};
            if (!component_address(entity, layout, component, address, true, &operation.regions) ||
                !read_bytes(address, output, size) ||
                !read_bytes(address, operation.verification.data(), size) ||
                std::memcmp(output, operation.verification.data(), size) != 0 ||
                !entity_for_handle(operation.handle, layout, current) || current != entity ||
                !component_address(current, layout, component, current_address, true, &operation.regions) || current_address != address) return;
            operation.present[index] = 1;
        }
        offset += size;
    }
    EntityView current{};
    if (!entity_for_handle(operation.handle, layout, current) || current != entity) return;
    // Preserve absence and address identity for every candidate through the
    // end of this owner operation. Individual equal copies do not establish
    // simultaneous values across different engine components.
    for (std::size_t index = 0; index < operation.names.size(); ++index) {
        bool present{};
        if (!component_presence(current, layout, operation.components[index], present) ||
            present != (operation.present[index] == 1)) return;
        if (present) {
            std::uintptr_t address{};
            if (!component_address(current, layout, operation.components[index], address, true, &operation.regions) ||
                address != operation.addresses[index]) return;
        }
    }
    operation.result = operation.regions.revalidate();
}

struct TemplateUuidOperation {
    std::uint32_t handle{};
    std::array<std::uint8_t, 16> uuid{};
    bool result{};
};
void template_uuid_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<TemplateUuidOperation*>(opaque);
    ResolvedLayout layout{};
    EntityView before{}, after{};
    std::array<std::uint8_t, 16> again{};
    if (!layout_snapshot(layout) || !layout.entity_definition_uuid_validated ||
        !entity_for_handle(operation.handle, layout, before) || !before.definition ||
        before.definition > UINTPTR_MAX - layout.entity_definition_uuid ||
        before.definition + layout.entity_definition_uuid > UINTPTR_MAX - operation.uuid.size()) return;
    const auto address = before.definition + layout.entity_definition_uuid;
    if (!read_bytes(address, operation.uuid.data(), operation.uuid.size()) ||
        std::all_of(operation.uuid.begin(), operation.uuid.end(), [](auto byte) { return byte == 0; }) ||
        !read_bytes(address, again.data(), again.size()) || again != operation.uuid ||
        !entity_for_handle(operation.handle, layout, after) || after != before) return;
    operation.result = true;
}

struct WriteOperation {
    std::uint32_t handle{};
    const char* name{};
    const void* mask{};
    const void* value{};
    std::size_t size{};
    bool result{}, conflict{}, uncertain{}, direct{}, mutation_attempted{};
    std::string owned_name;
    std::vector<std::uint8_t> mask_bytes, destination, expected, before, merged, verified;
    ComponentType component{};
    ResolvedLayout layout{};
    HandleRecord record{};
    EngineContext::EcsView owner;
    std::uint64_t epoch{};
    bool blocked_by_write_fault{};
};
std::atomic<const char*> last_write_reason{"not-attempted"};
// Called with operation_mutex held, before any original engine gate. All maps,
// buffers and host mutexes are prepared here, outside the short memory scope.
bool prepare_write(WriteOperation& operation) {
    if (stop_requested.load(std::memory_order_acquire) || !operation.mask || !operation.value ||
        operation.before.size() != operation.size || operation.merged.size() != operation.size ||
        operation.verified.size() != operation.size || !resolve_component(operation.name, operation.component) ||
        operation.component.size != operation.size || !layout_snapshot(operation.layout)) return false;
    std::scoped_lock lock(state_mutex);
    if (write_fault_active_locked()) {
        operation.blocked_by_write_fault = true;
        return false;
    }
    const auto found = handles.find(operation.handle);
    if (found == handles.end() || found->second.epoch != layout_epoch) return false;
    operation.record = found->second;
    operation.epoch = layout_epoch;
    operation.direct = read_only_binding;
    if (operation.direct) {
        if (!component_registry_ready || !component_registry_world_bound ||
            component_registry.owner != read_only_view.registry_owner || !operation.record.table_slot) return false;
        operation.owner = read_only_view;
    }
    return true;
}
// No allocation, mutex acquisition or task dispatch in this body. The guarded
// path records changes through the validated native tracker before mutation.
// Direct callers own both original gates. Legacy callers own a dispatcher phase.
void write_payload(WriteOperation& operation, const EngineContext::EcsWriteGuard* guard = nullptr) {
    if (stop_requested.load(std::memory_order_acquire) || (guard && !guard->Validate())) return;
    const auto& layout = operation.layout;
    const auto& record = operation.record;
    if (operation.direct) {
        std::uint64_t count{};
        std::uintptr_t table{}, pointer{};
        if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
            !read(layout.table_address, table) || table != operation.owner.entity_table ||
            table > UINTPTR_MAX - count * sizeof(pointer) || record.table_slot < table ||
            record.table_slot >= table + count * sizeof(pointer) || (record.table_slot - table) % sizeof(pointer) ||
            !read(record.table_slot, pointer) || pointer != record.pointer) return;
    }
    EntityView entity{};
    std::uintptr_t address{};
    if (!entity_view(record.pointer, layout, entity) || entity.id != record.id || entity.generation != record.generation ||
        !component_address(entity, layout, operation.component, address, false) ||
        !read_bytes(address, operation.before.data(), operation.size)) return;
    if (!operation.expected.empty() && operation.before != operation.expected) {
        operation.conflict = true;
        return;
    }
    std::copy(operation.before.begin(), operation.before.end(), operation.merged.begin());
    const auto* mask = static_cast<const std::uint8_t*>(operation.mask);
    const auto* value = static_cast<const std::uint8_t*>(operation.value);
    for (std::size_t index = 0; index < operation.size; ++index)
        if (mask[index]) operation.merged[index] = value[index];
    if (guard && !guard->ComponentSupported(operation.component.index)) return;
    if (operation.merged == operation.before) { operation.result = true; return; }
    if (guard && !guard->TrackChange(entity.pointer, operation.component.index)) return;
    operation.mutation_attempted = true;
    if (!guarded_copy(reinterpret_cast<void*>(address), operation.merged.data(), operation.size) ||
        !read_bytes(address, operation.verified.data(), operation.size) || operation.verified != operation.merged) {
        guarded_copy(reinterpret_cast<void*>(address), operation.before.data(), operation.size);
        // A failed copy can leave a partial mutation. Only a complete equal
        // readback establishes rollback, even when the restore copy faulted on
        // a read-only page that already contains the original payload.
        operation.uncertain = !read_bytes(address, operation.verified.data(), operation.size) ||
            operation.verified != operation.before;
        return;
    }
    operation.result = true;
}
void write_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<WriteOperation*>(opaque);
    if (!prepare_write(operation) || operation.direct) return;
    std::scoped_lock transaction(write_mutex);
    write_payload(operation);
    if (operation.uncertain) {
        std::scoped_lock lock(state_mutex);
        if (layout_epoch == operation.epoch) {
            write_fault_epoch = layout_epoch;
            write_fault_reason = "ecs-write-outcome-uncertain-rollback-unverified";
        }
    }
}
bool invoke_write_operation(const std::shared_ptr<WriteOperation>& operation) {
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (!direct) {
        const bool completed = GameThreadDispatcher::Invoke(write_on_game_thread, operation);
        if (completed && operation->uncertain)
            last_write_reason.store("ecs-write-outcome-uncertain-rollback-unverified", std::memory_order_relaxed);
        return completed;
    }
    std::scoped_lock operation_lock(operation_mutex);
    if (!prepare_write(*operation) || !operation->direct) {
        last_write_reason.store(operation->blocked_by_write_fault ?
            "ecs-writes-disabled-after-uncertain-rollback" : "ecs-write-handle-or-layout-invalid", std::memory_order_relaxed);
        return false;
    }
    std::scoped_lock transaction(write_mutex);
    EcsRuntime::SessionGuard session;
    if (!session.TryAcquire(operation->epoch + 1) || session.View() != operation->owner || !session.WritesAllowed()) {
        last_write_reason.store("ecs-write-session-changed-or-disabled", std::memory_order_relaxed);
        return false;
    }
    EngineContext::EcsWriteGuard guard;
    const char* reason{};
    if (!guard.TryAcquire(operation->owner, &reason)) {
        last_write_reason.store(reason, std::memory_order_relaxed);
        return false;
    }
    if (operation->size > guard.MaximumBytes()) {
        guard.Release();
        last_write_reason.store("ecs-write-size-exceeds-gate-budget", std::memory_order_relaxed);
        return false;
    }
    write_payload(*operation, &guard);
    if (!guard.Validate()) {
        operation->result = false;
        operation->uncertain = operation->uncertain || operation->mutation_attempted;
    }
    if (operation->uncertain) {
        session.MarkWriteUncertain("ecs-write-outcome-uncertain-rollback-unverified");
    }
    guard.Release();
    last_write_reason.store(operation->uncertain ? "ecs-write-outcome-uncertain-rollback-unverified" :
        operation->result ? "success" : operation->conflict ? "compare-conflict" :
        "ecs-write-identity-or-memory-check-failed", std::memory_order_relaxed);
    return true;
}

void reset_component_discovery() {
    std::scoped_lock lock(discovery_mutex);
    registry_discovery.Reset();
    observed_indices_by_size.clear();
    discovered_layouts.clear();
    discovered_layout_components.clear();
    template_layout_samples.clear();
    template_layout_samples_dropped = 0;
    discovery_entities.clear();
    discovery_cursor = 0;
    discovery_table = 0;
    discovery_last_restart = 0;
    discovery_scanned_entities = 0;
    discovery_layout_count = 0;
    discovery_restarts = 0;
    discovery_complete = false;
}

bool collect_layout_component_candidates(const EntityView& entity, const ResolvedLayout& layout,
                                         std::vector<ComponentSlot>& components) {
    std::size_t count{};
    {
        std::scoped_lock lock(state_mutex);
        if (component_registry_ready) count = component_registry.entries.size();
        else for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components)
            count = (std::max)(count, static_cast<std::size_t>(component.index) + 1);
    }
    if (!count || count > max_components) return false;
    std::uint64_t component_bits[16]{};
    if (!read_bytes(entity.layout + layout.component_bits, component_bits, ((count + 63) / 64) * 8)) return false;
    for (std::size_t index = 0; index < count; ++index) {
        if (!(component_bits[index / 64] & (std::uint64_t{1} << (index % 64)))) continue;
        std::uint16_t stride{}, offset{};
        if (!read(entity.layout + layout.component_strides + index * sizeof(stride), stride) || !stride ||
            !read(entity.layout + layout.component_offsets + index * sizeof(offset), offset)) continue;
        observed_indices_by_size[stride].insert(static_cast<std::uint16_t>(index));
        components.push_back({static_cast<std::uint16_t>(index), stride, offset});
    }
    ++discovery_layout_count;
    return true;
}

void collect_template_layout_sample(const EntityView& entity,
                                    const std::vector<ComponentSlot>& components) {
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    if (!entity.definition) return;
    struct DefinitionHeader {
        std::uint64_t uuid[2]{};
        std::uintptr_t name{};
        std::uint64_t name_size{};
    } definition{};
    if (!read_bytes(entity.definition + definition_uuid, definition.uuid, sizeof(definition.uuid)) ||
        !read(entity.definition + definition_name, definition.name) || !definition.name ||
        !read(entity.definition + definition_name_size, definition.name_size) ||
        !definition.name_size || definition.name_size > 128) return;
    std::string name(static_cast<std::size_t>(definition.name_size), '\0');
    if (!read_bytes(definition.name, name.data(), name.size())) return;
    if (const auto end = name.find('\0'); end != std::string::npos) name.resize(end);
    if (name.empty()) return;

    std::string key(sizeof(definition.uuid) + sizeof(entity.layout), '\0');
    std::memcpy(key.data(), definition.uuid, sizeof(definition.uuid));
    std::memcpy(key.data() + sizeof(definition.uuid), &entity.layout, sizeof(entity.layout));
    auto found = template_layout_samples.find(key);
    if (found == template_layout_samples.end()) {
        constexpr std::size_t max_template_layout_samples = 4096;
        if (template_layout_samples.size() >= max_template_layout_samples) {
            ++template_layout_samples_dropped;
            return;
        }
        TemplateLayoutSample sample{};
        std::memcpy(sample.uuid, definition.uuid, sizeof(sample.uuid));
        sample.layout = entity.layout;
        sample.name = std::move(name);
        sample.components = components;
        found = template_layout_samples.emplace(std::move(key), std::move(sample)).first;
    }
    ++found->second.entity_count;
}

void publish_discovered_component_indices() {
    std::unordered_map<std::string, std::uint32_t> contract;
    std::unordered_map<std::string, ComponentType> previously_resolved;
    {
        std::scoped_lock lock(state_mutex);
        if (component_registry_seen) return;
        contract = configured_types;
        previously_resolved = types;
    }
    std::unordered_map<std::string, ComponentType> discovered;
    for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components) {
        const auto known = contract.find(component.qualified_name);
        if (known != contract.end() && known->second == component.size)
            discovered.emplace(known->first, ComponentType{component.index, component.size});
    }
    for (const auto& [name, component] : previously_resolved) {
        const auto configured = contract.find(name);
        if (configured != contract.end() && configured->second == component.size)
            discovered.emplace(name, component);
    }
    // Strides remain useful evidence, but a matching size cannot prove identity.
    // Only an engine registration or an exact build profile may publish a mapping.
    std::scoped_lock lock(state_mutex);
    if (configured_types != contract) return;
    types = std::move(discovered);
}

nlohmann::json registration_type_json(const ComponentRegistry::Type& type) {
    if (type.name.empty()) return nullptr;
    return {{"qualified_name",type.name},{"qualified_hash",type.hash},
        {"size",type.size},{"metadata_rva",type.metadata_rva}};
}

void read_component_registry(std::uintptr_t manager) {
    const auto now = GetTickCount64();
    if (!manager) {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (component_registry_manager || layout_ready ||
            !handles.empty() || !reverse_handles.empty()) {
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
        component_registry_world_bound = false;
        read_only_binding = false;
        read_only_view = {};
        component_registry_manager = 0;
        live_layout = {};
        layout_ready = false;
        live_table = 0;
    }
    std::unordered_map<std::string, std::uint32_t> contract;
    {
        std::scoped_lock lock(state_mutex);
        if (manager && manager == component_registry_manager && now - component_registry_last_read < 1000) return;
        if (manager != component_registry_manager) component_registry_world_bound = false;
        component_registry_manager = manager;
        component_registry_last_read = now;
        contract = configured_types;
    }
    if (contract.empty()) return;
    ComponentRegistry::Snapshot snapshot;
    std::string error;
    const auto validate_contract = [&](const ComponentRegistry::Snapshot& candidate, std::string& reason) {
        for (const auto& entry : candidate.entries) {
            for (const auto* type : {&entry.storage, &entry.configuration}) {
                if (type->name.empty()) continue;
                const auto found = contract.find(type->name);
                if (found == contract.end() || found->second != type->size) {
                    reason = "engine registration disagrees with the parser type contract: " + type->name;
                    return false;
                }
            }
        }
        return true;
    };
    bool valid{};
    if (manager) {
        valid = ComponentRegistry::Read(manager, image_base, image_size, read_bytes, snapshot, error);
    } else {
        // Registration metadata has its own original owner. Resolving it does
        // not require an execution hook or establish an active ECS world.
        std::scoped_lock discovery_lock(discovery_mutex);
        valid = registry_discovery.Tick(image_base, image_size, read_bytes,
            validate_contract, snapshot);
        const auto& progress = registry_discovery.Status();
        if (!valid && progress.available) return; // cached catalog, revalidation is throttled
        if (!valid) error = progress.reason;
    }
    std::unordered_map<std::string, ComponentType> resolved;
    nlohmann::json entries = nlohmann::json::array();
    if (valid) valid = validate_contract(snapshot, error);
    if (valid) for (const auto& entry : snapshot.entries) {
        if (!entry.storage.name.empty())
            resolved.emplace(entry.storage.name, ComponentType{entry.index, entry.size});
        nlohmann::json callbacks = nlohmann::json::array();
        for (const auto& callback : entry.callbacks) {
            std::array<std::uint8_t, 512> code{};
            std::string code_hex;
            const auto code_address = image_base + callback.function_rva;
            if (image_size >= code.size() && callback.function_rva <= image_size - code.size() &&
                read_bytes(code_address, code.data(), code.size())) {
                static constexpr char digits[] = "0123456789abcdef";
                for (const auto byte : code) { code_hex += digits[byte >> 4]; code_hex += digits[byte & 15]; }
                std::array<std::uint8_t, 512> again{};
                if (!read_bytes(code_address, again.data(), again.size()) || again != code) code_hex.clear();
            }
            callbacks.push_back({{"slot_offset",callback.slot_offset},{"function_rva",callback.function_rva},
                {"origin",callback.in_record ? "registration-record" : "parallel-callback-table"},
                {"code_hex",std::move(code_hex)},
                {"callable",false},{"reason","native callback ABI and effects are not resolved"}});
        }
        entries.push_back({{"index",entry.index},{"qualified_name",entry.name},
            {"qualified_hash",entry.hash},{"runtime_type",registration_type_json(entry.storage)},
            {"template_type",registration_type_json(entry.configuration)}, {"runtime_size",entry.size},
            {"flags_bits",entry.flags},{"storage_flags_bits",entry.storage_flags},
            {"storage",entry.storage.name.empty() ? "template-only" : "entity"},
            {"callbacks",std::move(callbacks)}});
    }
    std::scoped_lock operation_lock(operation_mutex);
    std::scoped_lock lock(state_mutex);
    if (configured_types != contract) return;
    component_registry_error = error;
    if (!valid) {
        if (component_registry_ready) types.clear();
        component_registry_ready = false;
        component_registry_world_bound = false;
        component_registry_json.clear();
        return;
    }
    const bool changed = component_registry.owner != snapshot.owner ||
        component_registry.records != snapshot.records || !component_registry_ready || types != resolved;
    types = std::move(resolved);
    component_registry = std::move(snapshot);
    component_registry_ready = true;
    component_registry_world_bound = manager != 0;
    component_registry_seen = true;
    component_registry_json = nlohmann::json({{"schema_version",1},{"layout_version",1},
        {"available",true},{"source","live-engine-registration"},{"count",entries.size()},
        {"discovery",manager ? "active-world-manager" : "resident-private-memory"},
        {"manager_bound",component_registry_world_bound},
        {"runtime_type_count",types.size()},{"entries",std::move(entries)}}).dump();
    if (changed) {
        ++layout_epoch;
        handles.clear();
        reverse_handles.clear();
        // Dispatcher scans invalidate their own epoch lazily. Clearing these
        // maps from this discovery thread races an active engine callback.
    }
}

void component_discovery_tick(std::uintptr_t manager) {
    std::scoped_lock discovery_lock(discovery_mutex);
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    const auto count_address = manager + KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count;
    const auto table_address = manager + KfcRuntimeCompatibility::EnshroudedClient::entity_manager_table;
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!read(count_address, count) || !count || count > (1u << 20) ||
        !read(table_address, table) || !table || !readable(table, count * sizeof(std::uintptr_t))) return;

    const auto now = GetTickCount64();
    if (table != discovery_table || discovery_entities.size() != count ||
        (discovery_complete && now - discovery_last_restart >= 5000)) {
        discovery_entities.resize(static_cast<std::size_t>(count));
        if (!read_bytes(table, discovery_entities.data(), discovery_entities.size() * sizeof(std::uintptr_t))) {
            discovery_entities.clear();
            return;
        }
        discovery_table = table;
        discovery_cursor %= static_cast<std::size_t>(count);
        discovery_last_restart = now;
        discovery_scanned_entities = 0;
        ++discovery_restarts;
        discovery_complete = false;
    }
    if (discovery_complete || discovery_entities.empty()) return;

    constexpr std::size_t entities_per_tick = 128;
    const auto begin = discovery_cursor;
    const auto end = (std::min)(discovery_cursor + entities_per_tick, discovery_entities.size());
    for (; discovery_cursor < end; ++discovery_cursor) {
        EntityView entity{};
        if (!entity_view(discovery_entities[discovery_cursor], layout, entity)) continue;
        auto components = discovered_layout_components.find(entity.layout);
        if (components == discovered_layout_components.end()) {
            std::vector<ComponentSlot> discovered_components;
            if (!collect_layout_component_candidates(entity, layout, discovered_components)) continue;
            discovered_layouts.insert(entity.layout);
            components = discovered_layout_components.emplace(entity.layout, std::move(discovered_components)).first;
        }
        collect_template_layout_sample(entity, components->second);
    }
    discovery_scanned_entities += end - begin;
    if (discovery_cursor >= discovery_entities.size()) {
        discovery_complete = true;
    }
    // Publish every batch so a changing entity table cannot withhold mappings
    // until an entire snapshot has been traversed.
    publish_discovered_component_indices();
}
}

namespace EcsRuntime {
struct SessionGuard::State {
    std::unique_lock<std::mutex> lock{state_mutex};
    EngineContext::EcsView view;
    std::uint64_t epoch{};
};
SessionGuard::SessionGuard() = default;
SessionGuard::~SessionGuard() = default;
bool SessionGuard::TryAcquire(std::uint64_t expected_session) {
    if (state_ || !expected_session) return false;
    auto state = std::make_unique<State>();
    if (stop_requested.load(std::memory_order_acquire) || !read_only_binding || !layout_ready ||
        !component_registry_ready || !component_registry_world_bound || expected_session != layout_epoch + 1 ||
        read_only_view.role != EngineContext::EcsRole::LocalAuthority ||
        component_registry.owner != read_only_view.registry_owner ||
        component_registry_manager != read_only_view.manager) return false;
    EngineContext::EcsView current;
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!EngineContext::ResolveEcs(current) || current != read_only_view ||
        live_layout.count_address != current.manager + current.layout.manager_count ||
        live_layout.table_address != current.manager + current.layout.manager_table ||
        !read(live_layout.count_address, count) || !count || count > (1u << 20) ||
        !read(live_layout.table_address, table) || !table || table != live_table ||
        table != current.entity_table) return false;
    state->view = current;
    state->epoch = layout_epoch;
    state_ = std::move(state);
    return true;
}
const EngineContext::EcsView& SessionGuard::View() const { return state_->view; }
bool SessionGuard::WritesAllowed() const { return state_ && !write_fault_active_locked(); }
void SessionGuard::MarkWriteUncertain(const char* reason) {
    if (!state_ || !reason) return;
    write_fault_epoch = state_->epoch;
    write_fault_reason = reason;
    last_write_reason.store(reason, std::memory_order_relaxed);
}
bool EntityIdentity(std::uint32_t handle, std::uint32_t* entity_id) {
    if (!handle || !entity_id) return false;
    std::scoped_lock operation_lock(operation_mutex);
    bool direct{};
    EngineContext::EcsView expected, before, after;
    {
        std::scoped_lock lock(state_mutex);
        direct = read_only_binding;
        expected = read_only_view;
    }
    if (direct && (!EngineContext::ResolveEcs(before) || before != expected)) return false;
    ResolvedLayout layout{};
    EntityView entity{};
    if (!layout_snapshot(layout) || !entity_for_handle(handle, layout, entity)) return false;
    if (direct && (!EngineContext::ResolveEcs(after) || after != before)) return false;
    *entity_id = entity.id;
    return entity.id != 0;
}
bool MatchesComponentAddress(std::uint32_t handle, const char* name, const void* expected) {
    ComponentType component{};
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    return resolve_component(name, component) && layout_snapshot(layout) &&
        entity_for_handle(handle, layout, entity) &&
        component_address(entity, layout, component, address) &&
        address == reinterpret_cast<std::uintptr_t>(expected);
}
bool SnapshotEntityIds(std::vector<std::uint32_t>& ids) {
    return ::SnapshotEntityIds(ids);
}
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop) {
    return ::ResolvePropEntityId(entity_id, prop);
}
bool Initialize() {
    const bool execution_profile = KfcRuntimeCompatibility::EnshroudedClient::Load();
    image_base = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    if (!image_base) return false;
    const auto dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(image_base);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE) return false;
    const auto nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(image_base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE) return false;
    image_size = nt->OptionalHeader.SizeOfImage;
    stop_requested.store(false, std::memory_order_release);
    if (execution_profile) GameThreadDispatcher::Initialize();
    // Metadata discovery remains available when a build lacks proven native
    // execution adapters. Original-owner entity reads do not authorize writes.
    return true;
}
void Tick() {
    if (stop_requested.load(std::memory_order_acquire)) return;
    auto manager = GameThreadDispatcher::EntityManager();
    EngineContext::EcsView owners;
    const bool direct_read = (!manager || !GameThreadDispatcher::Ready()) && EngineContext::ResolveEcs(owners);
    if (direct_read) manager = owners.manager;
    read_component_registry(manager);
    if (!manager) {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (layout_ready) {
            layout_ready = false;
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
        return;
    }
    const auto count_address = manager + (direct_read ? owners.layout.manager_count : KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count);
    const auto table_address = manager + (direct_read ? owners.layout.manager_table : KfcRuntimeCompatibility::EnshroudedClient::entity_manager_table);
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!read(count_address, count) || !count || count > (1u << 20) ||
        !read(table_address, table) || !readable(table, sizeof(std::uintptr_t))) {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (layout_ready) {
            layout_ready = false;
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
        return;
    }
    {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (!layout_ready || live_table != table || live_layout.count_address != count_address ||
            live_layout.table_address != table_address || read_only_binding != direct_read ||
            (direct_read && read_only_view != owners)) {
            live_layout = {};
            live_table = table;
            read_only_binding = direct_read;
            read_only_view = direct_read ? owners : EngineContext::EcsView{};
            live_layout.count_address = count_address;
            live_layout.table_address = table_address;
            live_layout.occupancy_address = direct_read && owners.layout.manager_occupancy ? manager + owners.layout.manager_occupancy : 0;
            live_layout.keys_address = direct_read && owners.layout.manager_keys ? manager + owners.layout.manager_keys : 0;
            live_layout.entity_id = direct_read ? owners.layout.entity_id : KfcRuntimeCompatibility::EnshroudedClient::entity_id;
            live_layout.entity_generation = direct_read ? owners.layout.entity_generation : KfcRuntimeCompatibility::EnshroudedClient::entity_generation;
            live_layout.entity_layout = direct_read ? owners.layout.entity_layout : KfcRuntimeCompatibility::EnshroudedClient::entity_layout;
            live_layout.entity_storage = direct_read ? owners.layout.entity_storage : KfcRuntimeCompatibility::EnshroudedClient::entity_storage;
            live_layout.entity_definition = direct_read ? owners.layout.entity_definition : KfcRuntimeCompatibility::EnshroudedClient::entity_definition;
            live_layout.entity_definition_uuid = direct_read ? owners.layout.entity_definition_uuid : 0;
            live_layout.entity_definition_uuid_validated = direct_read && owners.layout.entity_definition_uuid_validated;
            live_layout.entity_row = direct_read ? owners.layout.entity_row : KfcRuntimeCompatibility::EnshroudedClient::entity_row;
            live_layout.component_bits = direct_read ? owners.layout.component_bits : KfcRuntimeCompatibility::EnshroudedClient::component_bits;
            live_layout.component_offsets = direct_read ? owners.layout.component_offsets : KfcRuntimeCompatibility::EnshroudedClient::component_offsets;
            live_layout.component_strides = direct_read ? owners.layout.component_strides : KfcRuntimeCompatibility::EnshroudedClient::component_strides;
            layout_ready = true;
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
    }
    if (!direct_read) component_discovery_tick(manager);
}
std::string Status() {
    std::scoped_lock discovery_lock(discovery_mutex);
    std::scoped_lock lock(state_mutex);
    std::ostringstream text;
    text << "types=" << types.size() << '/' << configured_types.size()
         << "{reflected_ecs_types=" << configured_types.size()
         << ",registered_storage_types=" << types.size()
         << ",registrations=" << (component_registry_ready ? component_registry.entries.size() : 0) << '}'
         << " component_index_discovery=" << (discovery_complete ? "complete" : "scanning")
         << '(' << discovery_cursor << '/' << discovery_entities.size()
         << ",layouts=" << discovered_layouts.size()
         << ",templates=" << template_layout_samples.size()
         << ",restarts=" << discovery_restarts << ')'
         << " profile=" << KfcRuntimeCompatibility::EnshroudedClient::status
         << " game_thread=" << GameThreadDispatcher::Status()
         << " entity_hooks{" << WorldRuntime::EntityHookStatus() << '}';
    for (const auto& operation : KfcRuntimeCompatibility::EnshroudedClient::runtime_operations)
        text << " world{" << operation.name << '=' << (operation.available ? operation.status : "unavailable") << '}';
    text << " voxel_context=" << (WorldRuntime::ActiveContextAvailable() ? "ready" : "waiting");
    text << " context_probe{" << WorldRuntime::ContextStatus() << '}';
    if (types.empty()) return text.str() + " registry=unresolved layout=unavailable";
    text << " registry=ready";
    if (!layout_ready || (component_registry_ready && !component_registry_world_bound))
        return text.str() + " layout=unavailable entity_access=unavailable(dispatcher-or-world-layout)";
    if (read_only_binding) {
        EngineContext::EcsView current;
        if (!EngineContext::ResolveEcs(current) || current != read_only_view)
            return text.str() + " layout=unavailable entity_access=unavailable(engine-owner-changed)";
        text << " entity_access=" << (EngineContext::EcsWriteConfigured() &&
            read_only_view.role == EngineContext::EcsRole::LocalAuthority ? "guarded-read-write" : "read-only")
            << " entity_realm=" << EngineContext::RoleName(read_only_view.role);
    } else text << " entity_access=" << (GameThreadDispatcher::Ready() ? "read-write" : "unavailable")
        << " entity_realm=engine-observed";
    text << " layout=ready"
         << " entity{id=0x" << std::hex << live_layout.entity_id
         << ",generation=0x" << live_layout.entity_generation
         << ",layout=0x" << live_layout.entity_layout
         << ",storage=0x" << live_layout.entity_storage
         << ",row=0x" << live_layout.entity_row << std::dec << '}'
         << " components{bits=0x" << std::hex << live_layout.component_bits
         << ",offsets=0x" << live_layout.component_offsets
         << ",strides=0x" << live_layout.component_strides << std::dec << '}'
         << " epoch=" << layout_epoch;
    return text.str();
}
std::string Diagnostics() {
    std::scoped_lock discovery_lock(discovery_mutex);
    std::scoped_lock lock(state_mutex);
    EngineContext::EcsView current_read_view;
    const bool current_read_binding = !read_only_binding ||
        (EngineContext::ResolveEcs(current_read_view) && current_read_view == read_only_view);
    std::size_t observed_indices{};
    for (const auto& entry : observed_indices_by_size) observed_indices += entry.second.size();
    nlohmann::json resolved_mappings = nlohmann::json::array();
    nlohmann::json unresolved_types = nlohmann::json::array();
    nlohmann::json stride_candidates = nlohmann::json::array();
    nlohmann::json template_layouts = nlohmann::json::array();
    nlohmann::json world_operations = nlohmann::json::object();
    for (const auto& operation : KfcRuntimeCompatibility::EnshroudedClient::runtime_operations)
        world_operations[operation.name] = {{"available", operation.available}, {"status", operation.status},
            {"abi", operation.abi}, {"thread", operation.thread}, {"context", operation.context}};
    std::unordered_map<std::uint32_t, std::vector<std::string>> names_by_size;
    std::unordered_set<std::string> profile_names;
    std::size_t dynamic_candidates{};
    std::size_t template_types{}, template_only{};
    if (component_registry_ready) for (const auto& entry : component_registry.entries) {
        template_types += !entry.configuration.name.empty();
        template_only += entry.storage.name.empty();
    }
    for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components)
        profile_names.insert(component.qualified_name);
    for (const auto& [name, size] : configured_types) {
        names_by_size[size].push_back(name);
        if (name.starts_with("keen::ecs::Dynamic")) ++dynamic_candidates;
    }
    for (const auto& [name, component] : types) {
        resolved_mappings.push_back({{"name",name},{"index",component.index},{"size",component.size},
            {"source",component_registry_ready ? "live-engine-registration" : "exact-build-profile"}});
    }
    for (const auto& [size, names] : names_by_size) {
        const auto candidate_set = observed_indices_by_size.find(size);
        nlohmann::json indices = nlohmann::json::array();
        if (candidate_set != observed_indices_by_size.end())
            for (const auto index : candidate_set->second) indices.push_back(index);
        stride_candidates.push_back({{"size",size},{"indices",std::move(indices)},
            {"reflectedTypeCount",names.size()}});
        for (const auto& name : names) {
            if (types.contains(name)) continue;
            const auto candidates = candidate_set == observed_indices_by_size.end() ? 0 : candidate_set->second.size();
            const auto reason = component_registry_ready ? "reflected-type-without-entity-storage" :
                names.size() != 1 ? "shared-reflected-size" :
                candidates == 0 ? "no-live-archetype-observed" :
                candidates != 1 ? "multiple-live-indices-for-size" : "index-conflict";
            unresolved_types.push_back({{"name",name},{"size",size},{"reason",reason}});
        }
    }
    for (const auto& [key, sample] : template_layout_samples) {
        (void)key;
        nlohmann::json component_slots = nlohmann::json::array();
        for (const auto& component : sample.components)
            component_slots.push_back({{"index", component.index}, {"stride", component.stride},
                {"offset", component.offset}});
        template_layouts.push_back({{"templateUuidQwords", {sample.uuid[0], sample.uuid[1]}},
            {"templateName", sample.name},
            {"entitiesSeen", sample.entity_count}, {"componentSlots", std::move(component_slots)}});
    }
    return nlohmann::json({
        {"schemaVersion",1}, {"providerAbi",KFC_RUNTIME_ABI_VERSION},
        {"profile",KfcRuntimeCompatibility::EnshroudedClient::status},
        {"kfcFormat",KfcRuntimeCompatibility::EnshroudedClient::kfc_format},
        {"candidateTypeBreakdown",nlohmann::json{
            {"total",configured_types.size()},
            {"otherEcsTypes",configured_types.size() - dynamic_candidates},
            {"dynamicNameTypes",dynamic_candidates},
            {"engineRegistrations",component_registry_ready ? component_registry.entries.size() : 0},
            {"registeredStorageTypes",component_registry_ready ? types.size() : 0},
            {"registeredTemplateTypes",template_types}, {"templateOnlyRegistrations",template_only},
            {"resolved",types.size()},
            {"unresolved",component_registry_ready ? 0 : configured_types.size() - types.size()},
            {"reflectedWithoutEntityStorage",configured_types.size() - types.size()}
        }},
        {"worldOperations",std::move(world_operations)},
        {"voxelContextActive",WorldRuntime::ActiveContextAvailable()},
        {"worldContext",nlohmann::json::parse(WorldRuntime::ContextDiagnostics())},
        {"entityContextReady",WorldRuntime::EntityContextReady()},
        {"entityHookStatus",WorldRuntime::EntityHookStatus()},
        {"configuredTypes",configured_types.size()}, {"resolvedTypes",types.size()},
        {"layoutReady",layout_ready}, {"layoutEpoch",layout_epoch},
        {"registryAvailable",component_registry_ready}, {"registryManagerBound",component_registry_world_bound},
        {"entityReadSource",read_only_binding ? "original-engine-owner-chain" : "game-thread-dispatcher"},
        {"entityReadRole",read_only_binding ? EngineContext::RoleName(read_only_view.role) : "engine-observed"},
        {"originalWriteGateConfigured",EngineContext::EcsWriteConfigured()},
        {"originalWriteGateLastReason",last_write_reason.load(std::memory_order_relaxed)},
        {"writeFault",write_fault_active_locked()},
        {"writeFaultReason",write_fault_active_locked() ? write_fault_reason : "none"},
        {"registryDiscovery",nlohmann::json{
            {"source","resident-private-memory"},
            {"scanning",registry_discovery.Status().scanning},
            {"available",registry_discovery.Status().available},
            {"reason",registry_discovery.Status().reason},
            {"scannedBytes",registry_discovery.Status().scanned_bytes},
            {"skippedNonresidentBytes",registry_discovery.Status().skipped_nonresident_bytes},
            {"validationBytes",registry_discovery.Status().validation_bytes},
            {"budgetLimitedSteps",registry_discovery.Status().budget_limited_steps},
            {"completedPasses",registry_discovery.Status().completed_passes},
            {"distinctRegistries",registry_discovery.Status().distinct_registries},
            {"distinctCatalogs",registry_discovery.Status().distinct_catalogs},
            {"entityAccessAuthorized",false}
        }},
        {"entityAccessReady",layout_ready && !types.empty() &&
            (read_only_binding ? current_read_binding && component_registry_ready && component_registry_world_bound :
                GameThreadDispatcher::Ready() && (!component_registry_ready || component_registry_world_bound))},
        {"componentDiscovery",nlohmann::json{
            {"complete",discovery_complete},
            {"scannedEntities",discovery_scanned_entities},
            {"totalEntities",discovery_entities.size()},
            {"archetypes",discovery_layout_count},
            {"distinctLayoutsSeen",discovered_layouts.size()},
            {"snapshotRestarts",discovery_restarts},
            {"observedStrideBuckets",observed_indices_by_size.size()},
            {"observedIndexCandidates",observed_indices},
            {"templateLayoutSamples",std::move(template_layouts)},
            {"templateLayoutSamplesDropped",template_layout_samples_dropped},
            {"mappingMethod",component_registry_ready ? "live engine registration, reflection and parallel arrays" :
                KfcRuntimeCompatibility::EnshroudedClient::runtime_components.empty() ? "waiting for live engine registration" : "exact build profile"},
            {"registrationReady",component_registry_ready}, {"registrationReason",component_registry_error},
            {"registrationCount",component_registry.entries.size()},
            {"resolved",std::move(resolved_mappings)},
            {"unresolved",component_registry_ready ? nlohmann::json::array() : unresolved_types},
            {"reflectedWithoutEntityStorage",component_registry_ready ? std::move(unresolved_types) : nlohmann::json::array()},
            {"strideCandidates",std::move(stride_candidates)}
        }},
        {"activeQueryScan",nlohmann::json{
            {"cursor",active_query_cursor.load(std::memory_order_relaxed)},
            {"total",active_query_total.load(std::memory_order_relaxed)},
            {"matches",active_query_matches.load(std::memory_order_relaxed)}
        }},
        {"operations", nlohmann::json{
            {"queries", operation_counters.queries.load()},
            {"querySuccesses", operation_counters.query_successes.load()},
            {"queryIncomplete", operation_counters.query_incomplete.load()},
            {"queryFailures", operation_counters.query_failures.load()},
            {"resolves", operation_counters.resolves.load()},
            {"resolveSuccesses", operation_counters.resolve_successes.load()},
            {"resolveFailures", operation_counters.resolve_failures.load()},
            {"reads", operation_counters.reads.load()},
            {"readSuccesses", operation_counters.read_successes.load()},
            {"readFailures", operation_counters.read_failures.load()},
            {"readBatches", operation_counters.read_batches.load()},
            {"readBatchSuccesses", operation_counters.read_batch_successes.load()},
            {"readBatchFailures", operation_counters.read_batch_failures.load()},
            {"readBatchComponents", operation_counters.read_batch_components.load()},
            {"readBatchAbsent", operation_counters.read_batch_absent.load()},
            {"writes", operation_counters.writes.load()},
            {"writeSuccesses", operation_counters.write_successes.load()},
            {"writeFailures", operation_counters.write_failures.load()},
            {"runtimePatches", nlohmann::json::parse(PatchRuntime::Diagnostics())}
        }},
        {"dispatcher",nlohmann::json::parse(GameThreadDispatcher::Diagnostics())}
    }).dump();
}
void Shutdown() {
    stop_requested.store(true, std::memory_order_release);
    GameThreadDispatcher::Shutdown();
    std::scoped_lock operation_lock(operation_mutex);
    reset_component_discovery();
    std::scoped_lock lock(state_mutex);
    types.clear();
    configured_types.clear();
    component_registry = {};
    component_registry_ready = false;
    component_registry_world_bound = false;
    component_registry_seen = false;
    component_registry_json.clear();
    component_registry_error = "runtime stopped";
    live_layout = {};
    layout_ready = false;
    read_only_binding = false;
    read_only_view = {};
    handles.clear();
    reverse_handles.clear();
    query_scans.clear();
    query_cache.clear();
    resolve_scans.clear();
    resolve_cache.clear();
}
}

extern "C" bool __cdecl KfcRuntimeEcsConfigure(const char* const* names,
                                                const std::uint32_t* sizes,
                                                std::size_t count) {
    if (!names || !sizes || !count || count > 20'000) return false;
    std::unordered_map<std::string, std::uint32_t> contract;
    for (std::size_t index = 0; index < count; ++index) {
        if (!names[index] || !sizes[index]) return false;
        const std::string name{names[index]};
        if (!name.starts_with("keen::ecs::")) return false;
        contract.emplace(name, sizes[index]);
    }
    std::scoped_lock operation_lock(operation_mutex);
    {
        std::scoped_lock lock(state_mutex);
        if (configured_types == contract) return true;
        configured_types = std::move(contract);
        component_registry = {};
        component_registry_ready = false;
        component_registry_world_bound = false;
        component_registry_seen = false;
        component_registry_last_read = 0;
        component_registry_json.clear();
        types.clear();
        for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components) {
            const auto configured = configured_types.find(std::string(component.qualified_name));
            if (configured != configured_types.end() && configured->second == component.size)
                types.emplace(configured->first, ComponentType{component.index, component.size});
        }
        live_layout = {};
        layout_ready = false;
        read_only_binding = false;
        read_only_view = {};
        ++layout_epoch;
        handles.clear();
        reverse_handles.clear();
    }
    reset_component_discovery();
    return true;
}

extern "C" std::uint64_t __cdecl KfcRuntimeWorldSessionId() {
    std::scoped_lock lock(state_mutex);
    if (read_only_binding) {
        EngineContext::EcsView current;
        if (!EngineContext::ResolveEcs(current) || current != read_only_view) return 0;
    }
    const auto manager = read_only_binding ? component_registry_manager : GameThreadDispatcher::EntityManager();
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!manager || !layout_ready ||
        live_layout.count_address != manager + (read_only_binding ? read_only_view.layout.manager_count : KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count) ||
        !read(live_layout.count_address, count) || !count || count > (1u << 20) ||
        !read(live_layout.table_address, table) || !table || table != live_table) return 0;
    // Conservative identity: rebuilding ECS layouts also invalidates old journals.
    return layout_epoch + 1;
}
extern "C" bool __cdecl KfcRuntimeEcsReady() {
    std::scoped_lock lock(state_mutex);
    if (!layout_ready || types.empty()) return false;
    if (!read_only_binding) return GameThreadDispatcher::Ready() &&
        (!component_registry_ready || component_registry_world_bound);
    EngineContext::EcsView current;
    return component_registry_ready && component_registry_world_bound &&
        EngineContext::ResolveEcs(current) && current == read_only_view;
}
extern "C" bool __cdecl KfcRuntimeEcsPropQueryReady() {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return false; }
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return false;
    ComponentType transform{}, item{};
    return resolve_component("keen::ecs::CurrentTransform", transform) && transform.size == 0x38 &&
        resolve_component("keen::ecs::UsedItem", item) && item.size == 4;
}
extern "C" bool __cdecl KfcRuntimeEcsCanWrite() {
    std::scoped_lock lock(state_mutex);
    if (!layout_ready || types.empty() || write_fault_active_locked()) return false;
    if (!read_only_binding) return GameThreadDispatcher::Ready() &&
        (!component_registry_ready || component_registry_world_bound);
    EngineContext::EcsView current;
    return component_registry_ready && component_registry_world_bound &&
        read_only_view.role == EngineContext::EcsRole::LocalAuthority && EngineContext::EcsWriteConfigured() &&
        EngineContext::ResolveEcs(current) && current == read_only_view;
}
extern "C" bool __cdecl KfcRuntimeEcsCanWriteComponent(const char* name) {
    std::scoped_lock operation_lock(operation_mutex);
    ComponentType component{};
    if (!KfcRuntimeEcsCanWrite() || !resolve_component(name, component)) return false;
    EngineContext::EcsView owner;
    {
        std::scoped_lock lock(state_mutex);
        if (!read_only_binding) return true;
        owner = read_only_view;
    }
    return EngineContext::EcsComponentWriteSupported(owner, component.index);
}
extern "C" bool __cdecl KfcRuntimeEcsDescribe(const char* name, std::uint32_t* size) {
    ComponentType component{};
    if (!size || !resolve_component(name, component)) return false;
    *size = component.size;
    return true;
}

extern "C" std::size_t __cdecl KfcRuntimeEcsRegistry(char* buffer, std::size_t capacity) {
    try {
    std::scoped_lock lock(state_mutex);
    const auto json = component_registry_ready ? component_registry_json : nlohmann::json({
        {"schema_version",1},{"available",false},{"reason",component_registry_error},
        {"entries",nlohmann::json::array()}}).dump();
    const auto required = json.size() + 1;
    if (buffer && capacity >= required) std::memcpy(buffer, json.c_str(), required);
    return required;
    } catch (...) { return 0; }
}

extern "C" std::size_t __cdecl KfcRuntimeEcsRegistration(const char* name, char* buffer, std::size_t capacity) {
    if (!name || !*name || strnlen_s(name, 512) == 512) return 0;
    try {
        std::scoped_lock lock(state_mutex);
        nlohmann::json report;
        if (!component_registry_ready) {
            report = {{"schema_version",1},{"available",false},{"reason",component_registry_error},
                {"entries",nlohmann::json::array()}};
        } else {
            auto entries = nlohmann::json::array();
            for (const auto& entry : component_registry.entries) {
                if (entry.name != name && entry.storage.name != name && entry.configuration.name != name) continue;
                entries.push_back({{"index",entry.index},{"qualified_name",entry.name},
                    {"qualified_hash",entry.hash},{"runtime_type",registration_type_json(entry.storage)},
                    {"template_type",registration_type_json(entry.configuration)},{"runtime_size",entry.size},
                    {"flags_bits",entry.flags},{"storage_flags_bits",entry.storage_flags},
                    {"storage",entry.storage.name.empty() ? "template-only" : "entity"}});
            }
            report = {{"schema_version",1},{"layout_version",1},{"layout_epoch",layout_epoch},
                {"available",true},{"source","live-engine-registration"},{"count",entries.size()},
                {"total_count",component_registry.entries.size()},
                {"discovery",component_registry_world_bound ? "active-world-manager" : "resident-private-memory"},
                {"manager_bound",component_registry_world_bound},{"runtime_type_count",types.size()},
                {"entries",std::move(entries)}};
        }
        const auto serialized = report.dump();
        const auto required = serialized.size() + 1;
        if (buffer && capacity >= required) std::memcpy(buffer, serialized.c_str(), required);
        return required;
    } catch (...) { return 0; }
}

extern "C" bool __cdecl KfcRuntimeEcsResolveType(const char* name, char* runtime_name, std::size_t capacity) {
    if (!name || !runtime_name || !capacity) return false;
    std::scoped_lock lock(state_mutex);
    const std::string* resolved{};
    if (component_registry_ready) {
        for (const auto& entry : component_registry.entries) {
            if (entry.name == name || entry.storage.name == name) {
                resolved = &entry.storage.name;
                break;
            }
        }
    } else {
        const auto found = types.find(name);
        if (found != types.end()) resolved = &found->first;
    }
    if (!resolved || resolved->empty() || resolved->size() + 1 > capacity) return false;
    std::memcpy(runtime_name, resolved->c_str(), resolved->size() + 1);
    return true;
}
extern "C" std::size_t __cdecl KfcRuntimeEcsQuery(const char* const* names, std::size_t count,
                                                    std::uint32_t* entities, std::size_t capacity) {
    operation_counters.queries.fetch_add(1, std::memory_order_relaxed);
    if (!names || !count || count > max_components || capacity > (1u << 20) || !KfcRuntimeEcsReady()) {
        operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
        return SIZE_MAX;
    }
    auto operation = std::make_shared<QueryOperation>();
    for (std::size_t index = 0; index < count; ++index) {
        if (!names[index]) {
            operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
            return SIZE_MAX;
        }
        operation->owned_names.emplace_back(names[index]);
    }
    for (const auto& name : operation->owned_names) operation->name_pointers.push_back(name.c_str());
    operation->names = operation->name_pointers.data();
    operation->count = count;
    operation->output.resize(capacity);
    operation->entities = operation->output.data();
    operation->capacity = capacity;
    if (!invoke_read_operation(query_on_game_thread, operation)) {
        operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
        return SIZE_MAX;
    }
    if (operation->result >= SIZE_MAX - 1) {
        if (operation->result == SIZE_MAX - 1) operation_counters.query_incomplete.fetch_add(1, std::memory_order_relaxed);
        else operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
        return operation->result;
    }
    if (entities) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->result), entities);
    operation_counters.query_successes.fetch_add(1, std::memory_order_relaxed);
    return operation->result;
}
extern "C" std::size_t __cdecl KfcRuntimeEcsQueryBounds(const char* const* names, std::size_t count,
                                                           const double* bounds, double padding, std::uint32_t* entities,
                                                           std::size_t capacity) {
    if (!names || !count || count > max_components || !bounds || !std::isfinite(padding) || padding < 0 ||
        capacity > (1u << 20) || !KfcRuntimeEcsReady())
        return SIZE_MAX;
    auto operation = std::make_shared<BoundsQueryOperation>();
    operation->padding = padding;
    for (std::size_t index = 0; index < count; ++index) {
        if (!names[index]) return SIZE_MAX;
        operation->names.emplace_back(names[index]);
    }
    bool has_transform = false;
    for (int axis = 0; axis < 3; ++axis) {
        const auto minimum = bounds[axis], maximum = bounds[axis + 3];
        if (!std::isfinite(minimum) || !std::isfinite(maximum) || minimum >= maximum) return SIZE_MAX;
        operation->bounds[axis] = minimum;
        operation->bounds[axis + 3] = maximum;
    }
    has_transform = std::find(operation->names.begin(), operation->names.end(), "keen::ecs::CurrentTransform") != operation->names.end();
    if (!has_transform) return SIZE_MAX;
    operation->capacity = capacity;
    if (!invoke_read_operation(query_bounds_on_game_thread, operation, 500)) return SIZE_MAX;
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (entities) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), entities);
    return operation->result;
}
extern "C" std::size_t __cdecl KfcRuntimeWorldEntityQueryProps(const double* bounds, double padding,
                                                                  KfcRuntimePropRecord* props, std::size_t capacity) {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return SIZE_MAX; }
    if (!bounds || !std::isfinite(padding) || padding < 0 || capacity > (1u << 20) || (!props && capacity)) return SIZE_MAX;
    auto operation = std::make_shared<PropQueryOperation>();
    operation->padding = padding;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) || bounds[axis] >= bounds[axis + 3]) return SIZE_MAX;
        operation->bounds[axis] = bounds[axis];
        operation->bounds[axis + 3] = bounds[axis + 3];
    }
    query_props_native(*operation);
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (props) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), props);
    return operation->result;
}
extern "C" std::size_t __cdecl KfcRuntimeWorldEntityQueryPropsInBounds(
    const double* bounds, KfcRuntimePropRecord* props, std::size_t capacity) {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return SIZE_MAX; }
    if (!bounds || capacity > (1u << 20) || (!props && capacity)) return SIZE_MAX;
    std::unique_lock query_lock(prop_query_call_mutex, std::try_to_lock);
    if (!query_lock.owns_lock()) return SIZE_MAX - 1;
    auto operation = std::make_shared<PropQueryOperation>();
    operation->exact_recipe_bounds = true;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) ||
            bounds[axis] >= bounds[axis + 3]) return SIZE_MAX;
        operation->bounds[axis] = bounds[axis];
        operation->bounds[axis + 3] = bounds[axis + 3];
    }
    const auto epoch = current_epoch();
    const auto same_bounds = [&] {
        for (int axis = 0; axis < 6; ++axis)
            if (prop_query_result_cache.bounds[axis] != operation->bounds[axis]) return false;
        return true;
    };
    if (props && prop_query_result_cache.ready && prop_query_result_cache.epoch == epoch && same_bounds()) {
        const auto count = prop_query_result_cache.props.size();
        std::copy_n(prop_query_result_cache.props.begin(), (std::min)(capacity, count), props);
        prop_query_result_cache.ready = false;
        std::vector<KfcRuntimePropRecord>().swap(prop_query_result_cache.props);
        return count;
    }
    const auto now = GetTickCount64();
    if (pending_prop_query) {
        const bool same = std::equal(std::begin(operation->bounds), std::end(operation->bounds),
            std::begin(pending_prop_query->bounds));
        if (pending_prop_query->epoch != epoch || now - pending_prop_query->progressed_ms > 30000 ||
            now - pending_prop_query->started_ms > 120000) {
            pending_prop_query.reset();
            prop_query_result_cache.ready = false;
            return SIZE_MAX; // changed world, stalled scan, or exceeded capture deadline; never publish partial results
        }
        if (same) operation = pending_prop_query;
        else if (now - pending_prop_query->touched_ms < 2000) return SIZE_MAX - 1;
        else pending_prop_query.reset();
    }
    if (!pending_prop_query) {
        operation->epoch = epoch;
        operation->started_ms = operation->progressed_ms = now;
        pending_prop_query = operation;
    }
    operation->touched_ms = now;
    operation->result = SIZE_MAX;
    const auto previous_cursor = operation->cursor;
    query_props_native(*operation);
    if (operation->cursor > previous_cursor) operation->progressed_ms = GetTickCount64();
    if (current_epoch() != epoch) {
        pending_prop_query.reset();
        return SIZE_MAX;
    }
    if (operation->result == SIZE_MAX - 1) return operation->result;
    pending_prop_query.reset();
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (!props && capacity == 0 && !operation->output.empty()) {
        prop_query_result_cache.ready = true;
        prop_query_result_cache.epoch = epoch;
        std::copy_n(operation->bounds, 6, prop_query_result_cache.bounds.begin());
        prop_query_result_cache.props = std::move(operation->output);
    } else {
        prop_query_result_cache.ready = false;
        std::vector<KfcRuntimePropRecord>().swap(prop_query_result_cache.props);
    }
    if (props) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), props);
    return operation->result;
}
extern "C" bool __cdecl KfcRuntimeWorldEntityRegisterPropRecipes(
    const KfcRuntimePropRecipe* recipes, std::size_t count) {
    if ((!recipes && count) || count > 1'000'000) return false;
    std::unordered_map<std::uint32_t, RegisteredPropRecipe> resolved;
    resolved.reserve(count);
    for (std::size_t index = 0; index < count; ++index) {
        const auto& recipe = recipes[index];
        if (!recipe.item_id || !std::all_of(std::begin(recipe.bounds), std::end(recipe.bounds),
                [](float value) { return std::isfinite(value); }) ||
            recipe.bounds[0] > recipe.bounds[3] || recipe.bounds[1] > recipe.bounds[4] ||
            recipe.bounds[2] > recipe.bounds[5]) return false;
        if (!resolved.emplace(recipe.item_id,
                RegisteredPropRecipe{std::to_array(recipe.bounds), recipe.feedback}).second) return false;
    }
    {
        std::scoped_lock lock(prop_recipe_mutex);
        prop_recipes = std::move(resolved);
        prop_recipe_catalog_ready = true;
    }
    return true;
}
extern "C" bool __cdecl KfcRuntimeWorldEntityGetPropRecipe(
    std::uint32_t item_id, KfcRuntimePropRecipe* recipe) {
    if (!item_id || !recipe) return false;
    std::scoped_lock lock(prop_recipe_mutex);
    const auto found = prop_recipes.find(item_id);
    if (found == prop_recipes.end()) return false;
    recipe->item_id = item_id;
    std::copy(found->second.bounds.begin(), found->second.bounds.end(), recipe->bounds);
    recipe->feedback = found->second.feedback;
    return true;
}
extern "C" bool __cdecl KfcRuntimeWorldEntityGetTransform(std::uint32_t handle, KfcRuntimePropRecord* prop) {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return false; }
    if (!handle || !prop) return false;
    HandleRecord record{};
    {
        std::scoped_lock lock(state_mutex);
        const auto found = handles.find(handle);
        if (found == handles.end() || found->second.epoch != layout_epoch) return false;
        record = found->second;
    }
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return false;
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType transform_component{}, item_component{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform_component) || transform_component.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item_component) || item_component.size != 4) return false;
    std::vector<EntityPointer> pointers;
    if (!entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    // Keep table membership and live id/generation validation, but inspect the
    // captured pointer first. Undo used to decode every preceding world entity
    // again for each prop and each verification pass (O(props * world entities)).
    const auto cached = std::find_if(pointers.begin(), pointers.end(),
        [&](const EntityPointer& entry) { return entry.pointer == record.pointer; });
    if (cached != pointers.end()) std::iter_swap(pointers.begin(), cached);
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) || entity.id != record.id || entity.generation != record.generation) continue;
        std::uintptr_t transform_address{}, item_address{};
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        std::uint32_t item_id{};
        if (!component_address(entity, layout, transform_component, transform_address) ||
            !component_address(entity, layout, item_component, item_address) ||
            !read_bytes(transform_address, &transform, sizeof(transform)) ||
            !read_bytes(item_address, &item_id, sizeof(item_id)) || !item_id) return false;
        const auto current_handle = handle_for(entity, pointer.table_index);
        if (current_handle != handle) return false;
        *prop = {};
        prop->entity_handle = handle;
        prop->entity_id = entity.id;
        prop->item_id = item_id;
        std::copy_n(transform.position, 3, prop->position);
        std::copy_n(transform.rotation, 4, prop->orientation);
        std::copy_n(transform.scale, 3, prop->scale);
        using namespace KfcRuntimeCompatibility::EnshroudedClient;
        if (!entity.definition || !read_bytes(entity.definition + definition_uuid,
                prop->template_uuid, sizeof(prop->template_uuid)) ||
            !(prop->template_uuid[0] || prop->template_uuid[1])) return false;
        return true;
    }
    return false;
}
struct SetEntityScaleOperation {
    std::uint32_t handle{};
    float scale[3]{};
    bool result{};
};
void set_entity_scale_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<SetEntityScaleOperation*>(opaque);
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType component{};
    if (!resolve_component("keen::ecs::CurrentTransform", component) || component.size != 0x38) return;
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    if (!layout_snapshot(layout) || !entity_for_handle(operation.handle, layout, entity) ||
        !component_address(entity, layout, component, address)) return;
    struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; };
    static_assert(sizeof(NativeTransform) == 0x38);
    constexpr auto scale_offset = offsetof(NativeTransform, scale);
    float before[3]{}, verified[3]{};
    if (!read_bytes(address + scale_offset, before, sizeof(before))) return;
    SIZE_T written{};
    if (!WriteProcessMemory(GetCurrentProcess(), reinterpret_cast<void*>(address + scale_offset),
            operation.scale, sizeof(operation.scale), &written) || written != sizeof(operation.scale) ||
        !read_bytes(address + scale_offset, verified, sizeof(verified)) ||
        std::memcmp(verified, operation.scale, sizeof(verified))) {
        SIZE_T restored{};
        WriteProcessMemory(GetCurrentProcess(), reinterpret_cast<void*>(address + scale_offset),
            before, sizeof(before), &restored);
        return;
    }
    operation.result = true;
}
extern "C" bool __cdecl KfcRuntimeWorldEntitySetScale(std::uint32_t handle, const double* scale) {
    if (!handle || !scale || !KfcRuntimeEcsCanWrite()) return false;
    auto operation = std::make_shared<SetEntityScaleOperation>();
    operation->handle = handle;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(scale[axis]) || std::abs(scale[axis]) > std::numeric_limits<float>::max()) return false;
        operation->scale[axis] = static_cast<float>(scale[axis]);
    }
    return GameThreadDispatcher::Invoke(set_entity_scale_on_game_thread, operation) && operation->result;
}
extern "C" std::uint32_t __cdecl KfcRuntimeEcsResolve(std::uint32_t entity_id) {
    operation_counters.resolves.fetch_add(1, std::memory_order_relaxed);
    if (!entity_id || !KfcRuntimeEcsReady()) {
        operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
        return 0;
    }
    auto operation = std::make_shared<ResolveOperation>();
    operation->entity_id = entity_id;
    if (!invoke_read_operation(resolve_on_game_thread, operation)) {
        operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
        return 0;
    }
    if (operation->result) operation_counters.resolve_successes.fetch_add(1, std::memory_order_relaxed);
    else operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
    return operation->result;
}
extern "C" bool __cdecl KfcRuntimeEcsRead(std::uint32_t handle, const char* name,
                                           void* value, std::size_t size) {
    operation_counters.reads.fetch_add(1, std::memory_order_relaxed);
    if (!name || !value || !size || size > (1u << 20) || !KfcRuntimeEcsReady()) {
        operation_counters.read_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    auto operation = std::make_shared<ReadOperation>();
    operation->handle = handle;
    operation->owned_name = name;
    operation->name = operation->owned_name.c_str();
    operation->output.resize(size);
    operation->value = operation->output.data();
    operation->size = size;
    if (!invoke_read_operation(read_on_game_thread, operation) || !operation->result) {
        operation_counters.read_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    std::memcpy(value, operation->output.data(), size);
    operation_counters.read_successes.fetch_add(1, std::memory_order_relaxed);
    return true;
}
extern "C" bool __cdecl KfcRuntimeEcsReadMany(std::uint32_t handle, const char* const* names,
    const std::uint32_t* sizes, std::size_t count, std::uint8_t* buffer,
    std::size_t capacity, std::uint8_t* present) {
    operation_counters.read_batches.fetch_add(1, std::memory_order_relaxed);
    const auto fail = [] {
        operation_counters.read_batch_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    };
    try {
        if (!handle || !names || !sizes || !buffer || !present || !count ||
            count > EcsRuntime::MaximumReadManyComponents || !capacity ||
            capacity > EcsRuntime::MaximumReadManyBytes || !KfcRuntimeEcsReady()) return fail();
        auto operation = std::make_shared<ReadManyOperation>();
        operation->handle = handle;
        operation->names.reserve(count);
        operation->sizes.assign(sizes, sizes + count);
        std::size_t total{}, maximum_size{};
        for (std::size_t index = 0; index < count; ++index) {
            const auto size = operation->sizes[index];
            if (!names[index] || !size || size > EcsRuntime::MaximumReadManyBytes - total) return fail();
            const auto length = strnlen_s(names[index], 512);
            if (!length || length == 512) return fail();
            operation->names.emplace_back(names[index], length);
            total += size;
            maximum_size = (std::max)(maximum_size, static_cast<std::size_t>(size));
        }
        if (total != capacity) return fail();
        // No caller-owned pointers enter the dispatcher. A timed-out queued
        // operation can finish against its own names and buffers safely.
        operation->components.resize(count);
        operation->addresses.resize(count);
        operation->output.resize(total);
        operation->present.resize(count);
        operation->verification.resize(maximum_size);
        if (!invoke_read_operation(read_many_on_game_thread, operation) || !operation->result) return fail();
        std::memcpy(buffer, operation->output.data(), total);
        std::memcpy(present, operation->present.data(), count);
        const auto copied = std::count(operation->present.begin(), operation->present.end(), std::uint8_t{1});
        operation_counters.read_batch_successes.fetch_add(1, std::memory_order_relaxed);
        operation_counters.read_batch_components.fetch_add(static_cast<std::uint64_t>(copied), std::memory_order_relaxed);
        operation_counters.read_batch_absent.fetch_add(count - copied, std::memory_order_relaxed);
        return true;
    } catch (...) { return fail(); }
}
extern "C" bool __cdecl KfcRuntimeEcsTemplateUuid(std::uint32_t handle, std::uint8_t* uuid) {
    if (!handle || !uuid || !KfcRuntimeEcsReady()) return false;
    auto operation = std::make_shared<TemplateUuidOperation>();
    operation->handle = handle;
    if (!invoke_read_operation(template_uuid_on_game_thread, operation) || !operation->result) return false;
    std::memcpy(uuid, operation->uuid.data(), operation->uuid.size());
    return true;
}
extern "C" bool __cdecl KfcRuntimeEcsWrite(std::uint32_t handle, const char* name,
                                            const void* mask, const void* value,
                                            std::size_t size) {
    operation_counters.writes.fetch_add(1, std::memory_order_relaxed);
    if (!name || !mask || !value || !size || size > (1u << 20) || !KfcRuntimeEcsCanWrite()) {
        operation_counters.write_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    auto operation = std::make_shared<WriteOperation>();
    operation->handle = handle;
    operation->owned_name = name;
    operation->name = operation->owned_name.c_str();
    const auto first = static_cast<const std::uint8_t*>(mask);
    const auto last = static_cast<const std::uint8_t*>(value);
    operation->mask_bytes.assign(first, first + size);
    operation->destination.assign(last, last + size);
    operation->mask = operation->mask_bytes.data();
    operation->value = operation->destination.data();
    operation->size = size;
    operation->before.resize(size);
    operation->merged.resize(size);
    operation->verified.resize(size);
    const bool written = invoke_write_operation(operation) && operation->result;
    if (written) operation_counters.write_successes.fetch_add(1, std::memory_order_relaxed);
    else operation_counters.write_failures.fetch_add(1, std::memory_order_relaxed);
    return written;
}
extern "C" std::uint32_t __cdecl KfcRuntimeEcsCompareExchange(std::uint32_t handle, const char* name,
    const void* expected, const void* mask, const void* value, std::size_t size) {
    if (!name || !expected || !mask || !value || !size || size > (1u << 20) || !KfcRuntimeEcsCanWrite()) return 0;
    auto operation = std::make_shared<WriteOperation>();
    operation->handle = handle;
    operation->owned_name = name;
    operation->name = operation->owned_name.c_str();
    const auto first = static_cast<const std::uint8_t*>(mask);
    const auto last = static_cast<const std::uint8_t*>(value);
    const auto before = static_cast<const std::uint8_t*>(expected);
    operation->mask_bytes.assign(first, first + size);
    operation->destination.assign(last, last + size);
    operation->expected.assign(before, before + size);
    operation->mask = operation->mask_bytes.data();
    operation->value = operation->destination.data();
    operation->size = size;
    operation->before.resize(size);
    operation->merged.resize(size);
    operation->verified.resize(size);
    if (!invoke_write_operation(operation)) return 0;
    return operation->uncertain ? 3 : operation->result ? 1 : operation->conflict ? 2 : 0;
}

extern "C" bool __cdecl KfcRuntimeEcsEntityIdentity(std::uint32_t handle, std::uint32_t* entity_id) {
    return EcsRuntime::EntityIdentity(handle, entity_id);
}
extern "C" bool __cdecl KfcRuntimePatchAvailable(const char* name) {
    if (name && std::strcmp(name, "runtime.gameplay.patch") == 0) return PatchRuntime::AnyAvailable();
    return PatchRuntime::Available(name);
}
extern "C" bool __cdecl KfcRuntimePatchSetEnabled(const char* name, bool enabled, std::uint32_t* outcome) {
    return PatchRuntime::SetEnabled(name, enabled, outcome);
}
