#pragma once

#include "engine-context/session.h"
#include "voxels/voxel_api.h"
#include "building/building_api.h"
#include "player-input/cursor.h"
#include "player-input/building.h"
#include "ecs/components.h"
#include "ecs/queries.h"
#include "ecs/props.h"
#include "profiles/profile_contract.h"
#include "execution/dispatcher.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstring>
#include <memory>
#include <mutex>
#include <atomic>
#include <cmath>
#include <condition_variable>
#include <cstdint>
#include <vector>
#include <unordered_set>
#include <nlohmann/json.hpp>

// Private implementation contract. State is defined once, in the owning unit.

namespace WorldRuntime::Implementation {
constexpr std::size_t MaximumCells = 65'536;

constexpr std::size_t NativeCursorSize = 0xa0;

constexpr std::uint32_t OperationTimeoutMs = 3000;

struct Cell { std::uint8_t material{}, density{}; };

static_assert(sizeof(Cell) == 2);

struct CellSpan { Cell* data{}; std::uint64_t size{}; };

using NativeRead = bool (__fastcall*)(CellSpan*, const std::uint32_t*, const void*, std::uint32_t, const std::int32_t*);

using NativeWrite = void (__fastcall*)(void*, CellSpan*, const std::uint32_t*, const std::int32_t*);

struct Operation {
    enum class Kind { Read, Write } kind{};
    std::array<std::int32_t, 3> origin{};
    std::array<std::uint32_t, 3> dimensions{};
    std::vector<Cell> cells;
    bool result{};
    bool write_attempted{};
    bool rollback_verified{};
    std::uint64_t context_generation{};
};

struct EngineTransform {
    std::int64_t position[3]{};
    float rotation[4]{};
    float scale[3]{1,1,1};
    std::uint32_t padding{};
};

static_assert(sizeof(EngineTransform) == 0x38);

struct alignas(16) PlacementBounds { float minimum[4]{}; float maximum[4]{}; };

static_assert(sizeof(PlacementBounds) == 32 && offsetof(PlacementBounds, maximum) == 16);

struct EntityRequest {
    enum class Kind { Spawn, Place, Destroy, Finish } kind{};
    std::uint64_t template_uuid[2]{};
    double position[3]{}, rotation[4]{0,0,0,1};
    float scale[3]{1,1,1};
    float bounds[6]{};
    EngineTransform exact_transform{};
    std::uint32_t tracking{}, feedback{}, flags{}, event_id{};
    bool complete{};
    bool has_exact_transform{};
    HANDLE completed{CreateEventW(nullptr, TRUE, FALSE, nullptr)};
    std::atomic<bool> done{}, success{};
    std::atomic<bool> cancelled{};
    std::uint32_t queue_token{};
    ~EntityRequest() { if (completed) CloseHandle(completed); }
};

// Prop deletion follows Shroudtopia's synchronous WorldApi path. The entity
// placement callback owns this POD request until it publishes phase 3; unlike
// the generic request/event path it cannot be cancelled after the game hook
// has started consuming it.
struct RemovalRequest {
    EngineTransform transform{};
    PlacementBounds bounds{};
    std::uint32_t tracking{}, feedback{};
    volatile long phase{}; // 0 idle, 1 queued, 2 consumed, 3 complete
    bool success{};
    std::uint32_t thread_id{};
    std::uintptr_t actor_world{}, remove_queue{};
    std::uint32_t owner{};
};

extern std::atomic<std::shared_ptr<EntityRequest>> pending_entity_request;

extern RemovalRequest pending_removal;

extern std::mutex removal_mutex;

extern thread_local bool inside_building_dispatch;

extern std::atomic<std::uintptr_t> observed_actor_world;

extern std::atomic<std::uint64_t> observed_actor_world_ms;

extern std::atomic<std::uintptr_t> preferred_voxel_world;

extern std::atomic<std::uint64_t> actor_placement_hook_hits, actor_placement_request_hits;

extern std::atomic<std::uint64_t> building_dispatch_hook_hits, building_dispatch_request_hits;

extern std::atomic<std::uint64_t> entity_dispatch_completions;

extern std::atomic<std::uint64_t> prop_update_request_hits;

extern std::atomic<std::uintptr_t> observed_execution_view;

extern std::atomic<std::uint64_t> prop_update_hook_hits, cursor_hook_hits, context_resets;

extern std::atomic<std::uintptr_t> local_execution_root;

extern std::atomic<std::uintptr_t> client_read_world, client_read_manager;

extern std::atomic<std::uint64_t> client_read_ms, client_read_generation;

enum class ActorProbe { Never, MissingFrame, MissingLayout, MissingService, InvalidWorld, Valid };

extern std::atomic<ActorProbe> actor_probe;

struct CursorMailbox {
    volatile long lock{};
    volatile long hook_ready{};
    std::uint64_t sequence{};
    std::uint64_t context_generation{};
    std::array<std::uint8_t, NativeCursorSize> bytes{};
};

extern CursorMailbox cursor_mailbox;

bool read_memory(std::uintptr_t address, void* output, std::size_t size);

bool copy_cursor_safely(const void* source, std::uint8_t* output);

const KfcRuntimeCompatibility::EnshroudedClient::RuntimeOperation* find_operation(const char* name);

bool valid_world_context(std::uintptr_t world);

bool resolve_voxel_world(std::uintptr_t& world);

bool resolve_client_read_world(std::uintptr_t& world);

void observe_client_read_world(void* cursor_frame, void* execution_view);

void observe_actor_world(void* actor_frame);

bool actor_frame_matches_active_world(void* actor_frame);

bool safe_read(NativeRead function, CellSpan* span, const std::uint32_t* dimensions,
               const void* world, std::uint32_t mode, const std::int32_t* origin);

bool safe_write(NativeWrite function, void* world, CellSpan* span,
                const std::uint32_t* dimensions, const std::int32_t* origin);

using NativeCreate = std::uint32_t (__fastcall*)(void*, const std::uint64_t*, const float*, const float*,
                                                  std::uint32_t, std::uint32_t, const std::uint64_t*);

using NativePlace = void (__fastcall*)(void*, const EngineTransform*, const float*,
                                       std::uint32_t material_feedback_id, std::uint32_t tracking_item_id);

using NativeDestroy = void (__fastcall*)(void*, const EngineTransform*, const float*, std::uint32_t);

using NativeFinish = void (__fastcall*)(void*, void*, std::uint32_t, bool);

bool safe_destroy(NativeDestroy function, void* context, const EngineTransform* transform,
                  const float* bounds, std::uint32_t feedback);

bool valid_transform(const EntityRequest& request, EngineTransform& transform);

std::uintptr_t image_base();

bool native_finish(void* context, bool complete);

void complete_entity_request(const std::shared_ptr<EntityRequest>& request, bool success, std::uint32_t token = 0);

bool perform_spawn(const std::shared_ptr<EntityRequest>& request, void* execution_view);

void execute_entity_request(const std::shared_ptr<EntityRequest>& request, void* execution_view, void* actor_frame);

void execute_pending_removal(void* execution_view, void* actor_frame);

bool queue_removal(const EngineTransform& transform, const float bounds[6],
                   std::uint32_t tracking, std::uint32_t feedback);

bool invoke_entity_request(const std::shared_ptr<EntityRequest>& request, std::uint32_t* outcome);

bool matching_prop_handles(const EntityRequest& request, std::vector<std::uint32_t>& handles);

bool count_spawn_matches(const EntityRequest& request, std::size_t& matches);

bool verify_spawn(const EntityRequest& request, const std::unordered_set<std::uint32_t>& before_ids,
                  std::uint32_t& created_handle);

bool verify_destroy(const EntityRequest& request, std::size_t previous_matches);

void execute(void* opaque);

bool cell_count(const std::uint32_t dimensions[3], std::size_t& count);

bool get_grid_spec_native(const char* grid_id, KfcRuntimeGridSpec* spec);

bool matching_prop_handles(const EntityRequest& request, std::vector<std::uint32_t>& handles);
}
