#pragma once

#include "ecs/entities.h"
#include "ecs/components.h"
#include "ecs/queries.h"
#include "ecs/props.h"
#include "ecs/registry/component_registry.h"
#include "ecs/registry/discovery.h"
#include "execution/dispatcher.h"
#include <nlohmann/json.hpp>
#include "profiles/profile_contract.h"
#include "engine-context/owners.h"
#include "engine-context/resource_lifetime.h"
#include "engine-context/scene_gate.h"
#include "engine-context/session.h"
#include "voxels/voxel_api.h"
#include "building/building_api.h"
#include "player-input/cursor.h"
#include "patches/patch_runtime.h"
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

// Private implementation contract. State is defined once, in the owning unit.

namespace EcsImplementation {
constexpr std::size_t max_components = 1024;

struct OperationCounters {
    std::atomic<std::uint64_t> queries{}, query_successes{}, query_incomplete{}, query_failures{};
    std::atomic<std::uint64_t> resolves{}, resolve_successes{}, resolve_failures{};
    std::atomic<std::uint64_t> reads{}, read_successes{}, read_failures{};
    std::atomic<std::uint64_t> read_batches{}, read_batch_successes{}, read_batch_failures{};
    std::atomic<std::uint64_t> read_batch_components{}, read_batch_absent{};
    std::atomic<std::uint64_t> writes{}, write_successes{}, write_failures{};
};

extern OperationCounters operation_counters;

extern std::atomic<std::uint64_t> active_query_cursor, active_query_total, active_query_matches;

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

extern std::mutex state_mutex;

extern std::recursive_mutex operation_mutex;

extern std::unordered_map<std::string, ComponentType> types;

extern std::unordered_map<std::string, std::uint32_t> configured_types;

extern ComponentRegistry::Snapshot component_registry;

extern bool component_registry_ready;

extern bool component_registry_world_bound;

extern bool component_registry_seen;

extern std::string component_registry_error;

extern std::uint64_t component_registry_last_read;

extern std::uintptr_t component_registry_manager;

extern std::string component_registry_json;

extern std::unordered_map<std::uint32_t, std::unordered_set<std::uint16_t>> observed_indices_by_size;

extern std::unordered_set<std::uintptr_t> discovered_layouts;

extern std::unordered_map<std::uintptr_t, std::vector<ComponentSlot>> discovered_layout_components;

extern std::unordered_map<std::string, TemplateLayoutSample> template_layout_samples;

extern std::uint64_t template_layout_samples_dropped;

extern std::vector<std::uintptr_t> discovery_entities;

extern std::size_t discovery_cursor;

extern std::uintptr_t discovery_table;

extern std::uint64_t discovery_last_restart;

extern std::uint64_t discovery_scanned_entities;

extern std::uint64_t discovery_layout_count;

extern std::uint64_t discovery_restarts;

extern bool discovery_complete;

extern std::mutex discovery_mutex;

extern ComponentRegistry::Discovery registry_discovery;

extern ResolvedLayout live_layout;

extern bool layout_ready;

extern bool read_only_binding;

extern EngineContext::EcsView read_only_view;

extern std::uintptr_t live_table;

extern std::unordered_map<std::uint32_t, HandleRecord> handles;

extern std::unordered_map<std::uint64_t, std::uint32_t> reverse_handles;

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

extern std::unordered_map<std::string, QueryScanState> query_scans;

extern std::unordered_map<std::string, QueryCacheEntry> query_cache;

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

extern std::unordered_map<std::uint32_t, ResolveScanState> resolve_scans;

extern std::unordered_map<std::uint32_t, ResolveCacheEntry> resolve_cache;

extern std::uint32_t next_handle;

extern std::uint64_t layout_epoch;

extern std::uint64_t write_fault_epoch;

extern const char* write_fault_reason;

bool write_fault_active_locked();

extern std::mutex write_mutex;

extern std::mutex prop_recipe_mutex;

struct RegisteredPropRecipe { std::array<float, 6> bounds{}; std::uint32_t feedback{}; };

extern std::unordered_map<std::uint32_t, RegisteredPropRecipe> prop_recipes;

extern bool prop_recipe_catalog_ready;

extern std::mutex prop_query_call_mutex;

struct PropQueryResultCache {
    std::array<double, 6> bounds{};
    std::uint64_t epoch{};
    std::vector<KfcRuntimePropRecord> props;
    bool ready{};
};

extern PropQueryResultCache prop_query_result_cache;

extern std::atomic<bool> stop_requested;

extern std::uintptr_t image_base;

extern std::size_t image_size;

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

extern std::shared_ptr<PropQueryOperation> pending_prop_query;

std::string query_key(const QueryOperation& operation);

std::uint64_t current_epoch();

bool invoke_read_operation(GameThreadDispatcher::Operation operation, const std::shared_ptr<void>& context,
                           std::uint32_t timeout_ms = 75);

void publish_query_result(QueryOperation& operation, const std::vector<std::uint32_t>& matches);

void publish_current_query_result(QueryOperation& operation, const QueryCacheEntry& cached,
                                  const ResolvedLayout& layout, const std::vector<ComponentType>& components);

void query_on_game_thread(void* opaque);

void query_bounds_on_game_thread(void* opaque);

bool recipe_bounds_intersect(const KfcRuntimePropRecord& prop,
                             const std::array<float, 6>& bounds,
                             const double* query_bounds);

void query_props_native(PropQueryOperation& operation);

struct ResolveOperation { std::uint32_t entity_id{}, result{}; };

void resolve_on_game_thread(void* opaque);

struct ReadOperation {
    std::uint32_t handle{};
    const char* name{};
    void* value{};
    std::size_t size{};
    bool result{};
    std::string owned_name;
    std::vector<std::uint8_t> output;
};

void read_on_game_thread(void* opaque);

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

void read_many_on_game_thread(void* opaque);

struct TemplateUuidOperation {
    std::uint32_t handle{};
    std::array<std::uint8_t, 16> uuid{};
    bool result{};
};

void template_uuid_on_game_thread(void* opaque);

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

extern std::atomic<const char*> last_write_reason;

bool prepare_write(WriteOperation& operation);

void write_payload(WriteOperation& operation, const EngineContext::EcsWriteGuard* guard = nullptr);

void write_on_game_thread(void* opaque);

bool invoke_write_operation(const std::shared_ptr<WriteOperation>& operation);

void reset_component_discovery();

bool collect_layout_component_candidates(const EntityView& entity, const ResolvedLayout& layout,
                                         std::vector<ComponentSlot>& components);

void collect_template_layout_sample(const EntityView& entity,
                                    const std::vector<ComponentSlot>& components);

void publish_discovered_component_indices();

nlohmann::json registration_type_json(const ComponentRegistry::Type& type);

void read_component_registry(std::uintptr_t manager);

void component_discovery_tick(std::uintptr_t manager);
}
