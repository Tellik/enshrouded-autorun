#pragma once

#include "ecs/entity_session.h"
#include "ecs/entity_storage.h"
#include "ecs/component_access.h"
#include "ecs/snapshot_entity_ids.h"
#include "ecs/read_building_prop.h"
#include "ecs/registry/read_component_registry.h"
#include "game-thread/run_game_thread_operations.h"
#include <nlohmann/json.hpp>
#include "profiles/game_profile.h"
#include "engine-context/engine_owners.h"
#include "engine-context/hold_resource_lifetime.h"
#include "engine-context/lock_scene_access.h"
#include "engine-context/world_context.h"
#include "voxels/voxel_operations.h"
#include "building/building_operations.h"
#include "player-input/sample_world_cursor.h"
#include "patches/set_runtime_patch.h"
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

extern std::atomic<bool> stop_requested;

extern std::uintptr_t image_base;

extern std::size_t image_size;

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

std::string query_key(const QueryOperation& operation);

std::uint64_t current_epoch();

bool invoke_read_operation(GameThreadDispatcher::Operation operation, const std::shared_ptr<void>& context,
                           std::uint32_t timeout_ms = 75);

void publish_current_query_result(QueryOperation& operation, const QueryCacheEntry& cached,
                                  const ResolvedLayout& layout, const std::vector<ComponentType>& components);

void query_on_game_thread(void* opaque);

void query_bounds_on_game_thread(void* opaque);

}
