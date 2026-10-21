#include "building/building_request_state.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
void complete_entity_request(const std::shared_ptr<EntityRequest>& request, bool success, std::uint32_t token){
    request->queue_token = token;
    request->success.store(success, std::memory_order_release);
    request->done.store(true, std::memory_order_release);
    if (request->completed) SetEvent(request->completed);
    auto expected = request;
    pending_entity_request.compare_exchange_strong(expected, {}, std::memory_order_acq_rel);
}

void execute_entity_request(const std::shared_ptr<EntityRequest>& request, void* execution_view, void* actor_frame){
    if (!request || request->cancelled.load(std::memory_order_acquire) || request->done.load(std::memory_order_acquire)) return;
    if (request->kind == EntityRequest::Kind::Spawn) {
        // Spawn is published only by the named, session-pinned queue adapter.
        // Stale legacy requests may never enter an address-based hook fallback.
        complete_entity_request(request, false);
        return;
    }
    const auto* place_profile = find_operation(request->kind == EntityRequest::Kind::Destroy
        ? "runtime.world.entity.destroy" : "runtime.world.entity.place");
    const auto* finish_profile = find_operation("runtime.world.entity.finish_building");
    const auto base = image_base();
    if (request->kind != EntityRequest::Kind::Finish && (!place_profile || !place_profile->available)) {
        complete_entity_request(request, false); return;
    }
    if (!finish_profile || !finish_profile->available || !actor_frame) { complete_entity_request(request, false); return; }
    const auto frame = reinterpret_cast<std::uintptr_t>(actor_frame);
    if (!actor_frame_matches_active_world(actor_frame)) { complete_entity_request(request, false); return; }
    const auto& layout = KfcRuntimeCompatibility::EnshroudedClient::world_context_layout;
    const auto native_address = frame + layout.placement_context;
    std::uintptr_t root{}, place_queue{}, remove_queue{}, publish_state{}, publish_commands{};
    std::uint32_t owner{};
    if (!read_memory(native_address, &root, sizeof(root)) || root != reinterpret_cast<std::uintptr_t>(execution_view) ||
        !read_memory(native_address + layout.place_queue, &place_queue, sizeof(place_queue)) ||
        !read_memory(native_address + layout.remove_queue, &remove_queue, sizeof(remove_queue)) ||
        !read_memory(native_address + layout.publish_state, &publish_state, sizeof(publish_state)) ||
        !read_memory(native_address + layout.publish_commands, &publish_commands, sizeof(publish_commands)) ||
        !read_memory(native_address + layout.owner, &owner, sizeof(owner)) || !owner || !publish_state || !publish_commands ||
        (request->kind == EntityRequest::Kind::Destroy ? !remove_queue : request->kind == EntityRequest::Kind::Place && !place_queue)) {
        complete_entity_request(request, false); return;
    }
    bool called = true;
    if (request->kind == EntityRequest::Kind::Finish) {
        called = native_finish(reinterpret_cast<void*>(native_address), request->complete);
    } else {
        EngineTransform transform{};
        if (request->has_exact_transform) transform = request->exact_transform;
        else if (!valid_transform(*request, transform)) { complete_entity_request(request, false); return; }
        PlacementBounds bounds{};
        std::copy_n(request->bounds, 3, bounds.minimum);
        std::copy_n(request->bounds + 3, 3, bounds.maximum);
        if (request->kind == EntityRequest::Kind::Place) {
            auto function = reinterpret_cast<NativePlace>(base + place_profile->function_rva);
            // Original code writes argument 4 to BuildingPlaceEvent.material
            // (MaterialFeedbackId), argument 5 to trackingItemId (ItemId).
            // The public Lua/C ABI keeps its existing tracking, feedback order.
            __try { function(reinterpret_cast<void*>(native_address), &transform, bounds.minimum, request->feedback, request->tracking); }
            __except(EXCEPTION_EXECUTE_HANDLER) { called = false; }
        } else {
            auto function = reinterpret_cast<NativeDestroy>(base + place_profile->function_rva);
            // Shroudtopia passes the recipe's materialFeedbackId here; the
            // fourth native argument is not the prop item/tracking ID.
            const auto was_inside_dispatch = inside_building_dispatch;
            inside_building_dispatch = true;
            called = safe_destroy(function, reinterpret_cast<void*>(native_address), &transform,
                bounds.minimum, request->feedback);
            if (called) called = native_finish(reinterpret_cast<void*>(native_address), true);
            inside_building_dispatch = was_inside_dispatch;
            complete_entity_request(request, called);
            return;
        }
        if (called) {
            if (request->kind != EntityRequest::Kind::Finish) inside_building_dispatch = true;
            called = native_finish(reinterpret_cast<void*>(native_address), request->kind == EntityRequest::Kind::Finish
                ? request->complete : true);
            inside_building_dispatch = false;
        }
    }
    complete_entity_request(request, called);
}

bool invoke_entity_request(const std::shared_ptr<EntityRequest>& request, std::uint32_t* outcome){
    if (!outcome) return false;
    *outcome = 1;
    if (!request->completed) return false;
    std::shared_ptr<EntityRequest> empty;
    if (!pending_entity_request.compare_exchange_strong(empty, request, std::memory_order_acq_rel)) return false;
    if (WaitForSingleObject(request->completed, OperationTimeoutMs) != WAIT_OBJECT_0) {
        request->cancelled.store(true, std::memory_order_release);
        auto expected = request;
        pending_entity_request.compare_exchange_strong(expected, {}, std::memory_order_acq_rel);
        *outcome = 3;
        return false;
    }
    if (request->success.load(std::memory_order_acquire)) { *outcome = 0; return true; }
    *outcome = 2;
    return false;
}
}

namespace WorldRuntime {
void OnPropUpdate(void* execution_view, void* actor_frame) {
    prop_update_hook_hits.fetch_add(1, std::memory_order_relaxed);
    if (execution_view) observed_execution_view.store(reinterpret_cast<std::uintptr_t>(execution_view), std::memory_order_release);
    observe_actor_world(actor_frame);
    const auto request = pending_entity_request.load(std::memory_order_acquire);
    if (!request) return;
    const auto* context = find_operation("runtime.world.context.active");
    const bool server_actor_context = context && context->actor_context;
    if (request->kind != EntityRequest::Kind::Spawn &&
        !(server_actor_context && request->kind == EntityRequest::Kind::Destroy)) return;
    prop_update_request_hits.fetch_add(1, std::memory_order_relaxed);
    execute_entity_request(request, execution_view, actor_frame);
}

void OnActorPlacement(void* execution_view, void* actor_frame) {
    actor_placement_hook_hits.fetch_add(1, std::memory_order_relaxed);
    if (execution_view) observed_execution_view.store(reinterpret_cast<std::uintptr_t>(execution_view), std::memory_order_release);
    observe_actor_world(actor_frame);
    if (InterlockedCompareExchange(&pending_removal.phase, 0, 0) == 1) {
        actor_placement_request_hits.fetch_add(1, std::memory_order_relaxed);
        execute_pending_removal(execution_view, actor_frame);
        if (InterlockedCompareExchange(&pending_removal.phase, 0, 0) == 3)
            entity_dispatch_completions.fetch_add(1, std::memory_order_relaxed);
        return;
    }
    const auto request = pending_entity_request.load(std::memory_order_acquire);
    if (!request || request->kind == EntityRequest::Kind::Spawn) return;
    actor_placement_request_hits.fetch_add(1, std::memory_order_relaxed);
    execute_entity_request(request, execution_view, actor_frame);
    if (request->done.load(std::memory_order_acquire))
        entity_dispatch_completions.fetch_add(1, std::memory_order_relaxed);
}

void OnBuildingDispatch(void* placement_context) {
    building_dispatch_hook_hits.fetch_add(1, std::memory_order_relaxed);
    if (inside_building_dispatch || !placement_context) return;
    const auto request = pending_entity_request.load(std::memory_order_acquire);
    if (!request || request->kind != EntityRequest::Kind::Destroy ||
        request->cancelled.load(std::memory_order_acquire) || request->done.load(std::memory_order_acquire)) return;
    building_dispatch_request_hits.fetch_add(1, std::memory_order_relaxed);
    const auto* operation = find_operation("runtime.world.entity.destroy");
    const auto* finish = find_operation("runtime.world.entity.finish_building");
    if (!operation || !operation->available || !finish || !finish->available) {
        complete_entity_request(request, false);
        return;
    }
    const auto context = reinterpret_cast<std::uintptr_t>(placement_context);
    const auto& layout = KfcRuntimeCompatibility::EnshroudedClient::world_context_layout;
    std::uintptr_t root{}, remove_queue{}, publish_state{}, publish_commands{};
    std::uint32_t owner{};
    const auto active_execution_view = observed_execution_view.load(std::memory_order_acquire);
    if (!active_execution_view || !read_memory(context, &root, sizeof(root)) || root != active_execution_view ||
        !read_memory(context + layout.remove_queue, &remove_queue, sizeof(remove_queue)) || !remove_queue ||
        !read_memory(context + layout.publish_state, &publish_state, sizeof(publish_state)) || !publish_state ||
        !read_memory(context + layout.publish_commands, &publish_commands, sizeof(publish_commands)) || !publish_commands ||
        !read_memory(context + layout.owner, &owner, sizeof(owner)) || !owner) {
        complete_entity_request(request, false);
        return;
    }
    EngineTransform transform{};
    if (!valid_transform(*request, transform)) {
        complete_entity_request(request, false);
        return;
    }
    PlacementBounds bounds{};
    std::copy_n(request->bounds, 3, bounds.minimum);
    std::copy_n(request->bounds + 3, 3, bounds.maximum);
    auto function = reinterpret_cast<NativeDestroy>(image_base() + operation->function_rva);
    inside_building_dispatch = true;
    bool called = safe_destroy(function, placement_context, &transform, bounds.minimum, request->feedback);
    if (called) called = native_finish(placement_context, true);
    inside_building_dispatch = false;
    complete_entity_request(request, called);
    entity_dispatch_completions.fetch_add(1, std::memory_order_relaxed);
}

std::string EntityHookStatus() {
    return "actor-placement=" + std::to_string(actor_placement_hook_hits.load(std::memory_order_relaxed)) +
        "/" + std::to_string(actor_placement_request_hits.load(std::memory_order_relaxed)) +
        ",building-dispatch=" + std::to_string(building_dispatch_hook_hits.load(std::memory_order_relaxed)) +
        "/" + std::to_string(building_dispatch_request_hits.load(std::memory_order_relaxed)) +
        ",prop-update-requests=" + std::to_string(prop_update_request_hits.load(std::memory_order_relaxed)) +
        ",completed=" + std::to_string(entity_dispatch_completions.load(std::memory_order_relaxed)) +
        ",removal{phase=" + std::to_string(InterlockedCompareExchange(&pending_removal.phase, 0, 0)) +
        ",success=" + std::to_string(pending_removal.success ? 1 : 0) +
        ",thread=" + std::to_string(pending_removal.thread_id) +
        ",owner=" + std::to_string(pending_removal.owner) + '}';
}
}

namespace WorldRuntime::Implementation {
std::atomic<std::shared_ptr<EntityRequest>> pending_entity_request;

RemovalRequest pending_removal;

std::mutex removal_mutex;

thread_local bool inside_building_dispatch{};




std::atomic<std::uint64_t> actor_placement_hook_hits{}, actor_placement_request_hits{};



std::atomic<std::uint64_t> building_dispatch_hook_hits{}, building_dispatch_request_hits{};



std::atomic<std::uint64_t> entity_dispatch_completions{};



std::atomic<std::uint64_t> prop_update_request_hits{};



std::atomic<std::uintptr_t> observed_execution_view{};
}
