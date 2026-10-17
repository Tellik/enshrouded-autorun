#include "building/building_request_state.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
bool safe_destroy(NativeDestroy function, void* context, const EngineTransform* transform,
                  const float* bounds, std::uint32_t feedback){
    __try { function(context, transform, bounds, feedback); return true; }
    __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}

void execute_pending_removal(void* execution_view, void* actor_frame){
    if (!actor_frame) return;
    const auto frame = reinterpret_cast<std::uintptr_t>(actor_frame);
    const auto& layout = KfcRuntimeCompatibility::EnshroudedClient::world_context_layout;
    std::uintptr_t service_view{}, actor_world{};
    std::uintptr_t native_address{}, root{}, remove_queue{}, publish_state{}, publish_commands{};
    std::uint32_t owner{};
    bool success = false;
    if (!read_memory(frame + layout.actor_frame_service_view, &service_view, sizeof(service_view)) || !service_view ||
        !read_memory(service_view + layout.service_view_world, &actor_world, sizeof(actor_world)) || !actor_world)
        return;
    // Match Shroudtopia's ordering: discard preview-world ticks before claiming
    // the request, so a later tick can consume it against the pinned grid world.
    const auto preferred = preferred_voxel_world.load(std::memory_order_acquire);
    if (preferred && actor_world != preferred) return;
    if (InterlockedCompareExchange(&pending_removal.phase, 2, 1) != 1) return;
    if (execution_view &&
        (native_address = frame + layout.placement_context) != 0 &&
        read_memory(native_address, &root, sizeof(root)) && root == reinterpret_cast<std::uintptr_t>(execution_view) &&
        read_memory(native_address + layout.remove_queue, &remove_queue, sizeof(remove_queue)) && remove_queue &&
        read_memory(native_address + layout.publish_state, &publish_state, sizeof(publish_state)) && publish_state &&
        read_memory(native_address + layout.publish_commands, &publish_commands, sizeof(publish_commands)) && publish_commands &&
        read_memory(native_address + layout.owner, &owner, sizeof(owner)) && owner) {
        const auto* operation = find_operation("runtime.world.entity.destroy");
        const auto* finish = find_operation("runtime.world.entity.finish_building");
        if (operation && operation->available && finish && finish->available) {
            const auto base = image_base();
            auto destroy = reinterpret_cast<NativeDestroy>(base + operation->function_rva);
            success = safe_destroy(destroy, reinterpret_cast<void*>(native_address), &pending_removal.transform,
                pending_removal.bounds.minimum, pending_removal.feedback);
            if (success) success = native_finish(reinterpret_cast<void*>(native_address), true);
        }
    }
    pending_removal.success = success;
    pending_removal.thread_id = GetCurrentThreadId();
    pending_removal.actor_world = actor_world;
    pending_removal.remove_queue = remove_queue;
    pending_removal.owner = owner;
    InterlockedExchange(&pending_removal.phase, 3);
}

bool queue_removal(const EngineTransform& transform, const float bounds[6],
                   std::uint32_t tracking, std::uint32_t feedback){
    pending_removal.transform = transform;
    std::copy_n(bounds, 3, pending_removal.bounds.minimum);
    std::copy_n(bounds + 3, 3, pending_removal.bounds.maximum);
    pending_removal.tracking = tracking;
    pending_removal.feedback = feedback;
    pending_removal.success = false;
    pending_removal.thread_id = 0;
    pending_removal.actor_world = 0;
    pending_removal.remove_queue = 0;
    pending_removal.owner = 0;
    InterlockedExchange(&pending_removal.phase, 1);
    const auto deadline = GetTickCount64() + OperationTimeoutMs;
    for (;;) {
        const auto phase = InterlockedCompareExchange(&pending_removal.phase, 0, 0);
        if (phase == 3) {
            const auto success = pending_removal.success;
            return success;
        }
        // Match Shroudtopia: only cancel a request that no native hook has
        // consumed. Once phase 2 is published, let that synchronous operation
        // finish instead of freeing/cancelling state underneath the game.
        if (GetTickCount64() >= deadline &&
            InterlockedCompareExchange(&pending_removal.phase, 0, 1) == 1) return false;
        Sleep(1);
    }
}

bool verify_destroy(const EntityRequest& request, std::size_t previous_matches){
    const auto deadline = GetTickCount64() + 2000;
    do {
        std::size_t current_matches{};
        if (count_spawn_matches(request, current_matches) && current_matches < previous_matches) return true;
        if (GetTickCount64() < deadline) Sleep(10);
    } while (GetTickCount64() < deadline);
    return false;
}

bool matching_prop_handles(const EntityRequest& request, std::vector<std::uint32_t>& handles){
    constexpr double tolerance = 0.01;
    double bounds[6]{};
    for (int axis = 0; axis < 3; ++axis) {
        bounds[axis] = request.position[axis] - tolerance;
        bounds[axis + 3] = request.position[axis] + tolerance;
    }
    const auto count = KfcRuntimeWorldEntityQueryProps(bounds, 0.0, nullptr, 0);
    if (count == SIZE_MAX || count > 1'000'000) return false;
    std::vector<KfcRuntimePropRecord> props(count);
    if (count) {
        const auto actual = KfcRuntimeWorldEntityQueryProps(bounds, 0.0, props.data(), props.size());
        if (actual == SIZE_MAX || actual > props.size()) return false;
        props.resize(actual);
    }
    handles.clear();
    for (const auto& prop : props) {
        if (prop.item_id != request.tracking) continue;
        bool matches = true;
        for (int axis = 0; axis < 3; ++axis) {
            const auto expected = std::ldexp(request.position[axis], 32);
            if (!std::isfinite(expected) || std::abs(static_cast<long double>(prop.position[axis]) - expected) > (1LL << 24)) {
                matches = false;
                break;
            }
        }
        if (matches && prop.entity_handle) handles.push_back(prop.entity_handle);
    }
    return true;
}
}

namespace WorldRuntime {
bool DestroyEntity(const double position[3], const double rotation[4], const float bounds[6],
                   std::uint32_t tracking, std::uint32_t feedback, std::uint32_t* outcome) {
    if (!position || !rotation || !bounds || !tracking || !outcome ||
        !OperationAvailable("runtime.world.entity.destroy") ||
        !OperationAvailable("runtime.world.entity.finish_building")) return false;
    for (int axis = 0; axis < 3; ++axis)
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) || bounds[axis] > bounds[axis + 3]) return false;
    auto request = std::make_shared<EntityRequest>();
    request->kind = EntityRequest::Kind::Destroy;
    std::copy_n(position, 3, request->position);
    std::copy_n(rotation, 4, request->rotation);
    std::copy_n(bounds, 6, request->bounds);
    request->tracking = tracking;
    request->feedback = feedback;
    std::size_t previous_matches{};
    if (!count_spawn_matches(*request, previous_matches) || !previous_matches) { *outcome = 1; return false; }
    if (!invoke_entity_request(request, outcome)) return false;
    if (verify_destroy(*request, previous_matches)) return true;
    *outcome = 4;
    return false;
}

bool DestroyEntityHandle(std::uint32_t entity_handle, std::uint32_t* outcome) {
    if (!entity_handle || !outcome) return false;
    KfcRuntimePropRecord prop{};
    if (!KfcRuntimeWorldEntityGetTransform(entity_handle, &prop)) {
        *outcome = 1;
        return false;
    }
    KfcRuntimePropRecipe recipe{};
    if (!KfcRuntimeWorldEntityGetPropRecipe(prop.item_id, &recipe)) {
        *outcome = 1;
        return false;
    }
    if (!OperationAvailable("runtime.world.entity.destroy") ||
        !OperationAvailable("runtime.world.entity.finish_building")) { *outcome = 1; return false; }
    std::scoped_lock lock(removal_mutex);
    // Resolve the live handle again after acquiring the single-request lock,
    // then preserve its exact fixed-point transform and scale for native removal.
    if (!KfcRuntimeWorldEntityGetTransform(entity_handle, &prop) || prop.item_id != recipe.item_id) {
        *outcome = 1;
        return false;
    }
    const auto* context_profile = find_operation("runtime.world.context.active");
    if (context_profile && context_profile->actor_context) {
        // The headless server has no local player-input callback to consume
        // queue_removal(). Dispatch this exact handle through the server's
        // profiled prop-update frame, which carries the validated actor and
        // placement context used by its native destroy operation.
        auto request = std::make_shared<EntityRequest>();
        request->kind = EntityRequest::Kind::Destroy;
        request->has_exact_transform = true;
        std::copy_n(prop.position, 3, request->exact_transform.position);
        std::copy_n(prop.orientation, 4, request->exact_transform.rotation);
        std::copy_n(prop.scale, 3, request->exact_transform.scale);
        std::copy_n(recipe.bounds, 6, request->bounds);
        request->tracking = recipe.item_id;
        request->feedback = recipe.feedback;
        if (!invoke_entity_request(request, outcome)) return false;
        for (int attempt = 0; attempt < 200; ++attempt) {
            KfcRuntimePropRecord current{};
            if (!KfcRuntimeWorldEntityGetTransform(entity_handle, &current)) {
                *outcome = 0;
                return true;
            }
            Sleep(10);
        }
        *outcome = 4;
        return false;
    }
    EngineTransform transform{};
    std::copy_n(prop.position, 3, transform.position);
    std::copy_n(prop.orientation, 4, transform.rotation);
    std::copy_n(prop.scale, 3, transform.scale);
    const auto actor_hooks_before = actor_placement_hook_hits.load(std::memory_order_acquire);
    const auto actor_requests_before = actor_placement_request_hits.load(std::memory_order_acquire);
    const auto dispatch_hooks_before = building_dispatch_hook_hits.load(std::memory_order_acquire);
    const auto dispatch_requests_before = building_dispatch_request_hits.load(std::memory_order_acquire);
    if (!queue_removal(transform, recipe.bounds, recipe.item_id, recipe.feedback)) {
        const auto phase = InterlockedCompareExchange(&pending_removal.phase, 0, 0);
        if (phase == 0) {
            // Preserve hook evidence in the API result so the Lua error written
            // to shroudforge.log identifies which native callback actually ran.
            std::uint32_t flags{};
            if (actor_placement_hook_hits.load(std::memory_order_acquire) != actor_hooks_before) flags |= 1u;
            if (actor_placement_request_hits.load(std::memory_order_acquire) != actor_requests_before) flags |= 2u;
            if (building_dispatch_hook_hits.load(std::memory_order_acquire) != dispatch_hooks_before) flags |= 4u;
            if (building_dispatch_request_hits.load(std::memory_order_acquire) != dispatch_requests_before) flags |= 8u;
            if (GameThreadDispatcher::EntityContextReady()) flags |= 16u;
            *outcome = 30u + flags;
        } else {
            *outcome = 2;
        }
        return false;
    }
    for (int attempt = 0; attempt < 200; ++attempt) {
        KfcRuntimePropRecord current{};
        if (!KfcRuntimeWorldEntityGetTransform(entity_handle, &current)) { *outcome = 0; return true; }
        Sleep(10);
    }
    *outcome = 4;
    return false;
}
}

extern "C" {
bool __cdecl KfcRuntimeWorldEntityDestroy(const double* position, const double* rotation,
    const float* bounds, std::uint32_t tracking, std::uint32_t feedback, std::uint32_t* outcome) {
    return WorldRuntime::DestroyEntity(position, rotation, bounds, tracking, feedback, outcome);
}

bool __cdecl KfcRuntimeWorldEntityDestroyHandle(std::uint32_t entity_handle, std::uint32_t* outcome) {
    return WorldRuntime::DestroyEntityHandle(entity_handle, outcome);
}
}

namespace WorldRuntime::Implementation {
bool count_spawn_matches(const EntityRequest& request, std::size_t& matches){
    std::vector<std::uint32_t> handles;
    if (!matching_prop_handles(request, handles)) return false;
    matches = handles.size();
    return true;
}
}
