#pragma once
#include "engine-context/world_context_state.h"
#include "building/building_operations.h"
namespace WorldRuntime::Implementation {
constexpr std::uint32_t OperationTimeoutMs = 3000;

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

void execute_entity_request(const std::shared_ptr<EntityRequest>& request, void* execution_view, void* actor_frame);

void execute_pending_removal(void* execution_view, void* actor_frame);

bool queue_removal(const EngineTransform& transform, const float bounds[6],
                   std::uint32_t tracking, std::uint32_t feedback);

bool invoke_entity_request(const std::shared_ptr<EntityRequest>& request, std::uint32_t* outcome);

bool matching_prop_handles(const EntityRequest& request, std::vector<std::uint32_t>& handles);

bool count_spawn_matches(const EntityRequest& request, std::size_t& matches);

bool verify_destroy(const EntityRequest& request, std::size_t previous_matches);

}
