#include "building/building_request_state.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
bool perform_spawn(const std::shared_ptr<EntityRequest>& request, void* execution_view){
    const auto* profile = find_operation("runtime.world.entity.spawn");
    const auto base = image_base();
    if (!execution_view || !profile || !profile->available ||
        !(request->template_uuid[0] || request->template_uuid[1])) return false;
    EngineTransform transform{};
    if (!valid_transform(*request, transform)) return false;
    float position[4]{};
    for (int axis = 0; axis < 3; ++axis) {
        position[axis] = static_cast<float>(request->position[axis]);
        if (!std::isfinite(position[axis])) return false;
    }
    std::uintptr_t context[1]{reinterpret_cast<std::uintptr_t>(execution_view)};
    const std::uint64_t auxiliary[2]{};
    std::uint32_t token{};
    auto function = reinterpret_cast<NativeCreate>(base + profile->function_rva);
    __try { token = function(context, request->template_uuid, position, transform.rotation,
                             request->tracking, request->flags, auxiliary); }
    __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
    request->queue_token = token;
    return token != 0;
}

bool verify_spawn(const EntityRequest& request, const std::unordered_set<std::uint32_t>& before_ids,
                  std::uint32_t& created_handle){
    const auto deadline = GetTickCount64() + 2000;
    auto next_scan = GetTickCount64();
    do {
        const auto now = GetTickCount64();
        if (now >= next_scan) {
            std::vector<std::uint32_t> after_ids;
            if (EcsRuntime::SnapshotEntityIds(after_ids)) {
                for (const auto id : after_ids) {
                    if (before_ids.contains(id)) continue;
                    KfcRuntimePropRecord prop{};
                    if (!EcsRuntime::ResolvePropEntityId(id, &prop) || prop.item_id != request.tracking ||
                        prop.template_uuid[0] != request.template_uuid[0] ||
                        prop.template_uuid[1] != request.template_uuid[1]) continue;
                    bool position_matches = true;
                    for (int axis = 0; axis < 3; ++axis) {
                        const auto expected = std::ldexp(request.position[axis], 32);
                        if (!std::isfinite(expected) || std::abs(static_cast<long double>(prop.position[axis]) - expected) > (1LL << 24)) {
                            position_matches = false;
                            break;
                        }
                    }
                    if (!position_matches) continue;
                    created_handle = prop.entity_handle;
                    return true;
                }
            }
            next_scan = now + 50;
        }
        if (GetTickCount64() < deadline) Sleep(5);
    } while (GetTickCount64() < deadline);
    return false;
}
}

namespace WorldRuntime {
bool SpawnEntity(const std::uint64_t template_uuid[2], const double position[3],
                 const double rotation[4], std::uint32_t tracking, std::uint32_t flags,
                 std::uint32_t* entity_handle, std::uint32_t* outcome) {
    if (!entity_handle || !template_uuid || !position || !rotation || !outcome || !tracking ||
        !OperationAvailable("runtime.world.entity.spawn")) return false;
    *entity_handle = 0;
    auto request = std::make_shared<EntityRequest>();
    request->kind = EntityRequest::Kind::Spawn;
    std::copy_n(template_uuid, 2, request->template_uuid);
    std::copy_n(position, 3, request->position);
    std::copy_n(rotation, 4, request->rotation);
    request->tracking = tracking;
    request->flags = flags;
    // Match Shroudtopia: snapshot existing entity IDs, dispatch creation in the
    // prop-update hook, then resolve only newly appeared IDs to live props.
    std::vector<std::uint32_t> id_snapshot;
    if (!EcsRuntime::SnapshotEntityIds(id_snapshot)) { *outcome = 1; return false; }
    const std::unordered_set<std::uint32_t> before_ids(id_snapshot.begin(), id_snapshot.end());
    const auto ok = invoke_entity_request(request, outcome);
    if (!ok) return false;
    if (verify_spawn(*request, before_ids, *entity_handle)) return true;
    *outcome = 4; // command ran, but no matching live ECS entity was observed
    return false;
}
}

extern "C" {
bool __cdecl KfcRuntimeWorldEntitySpawn(const std::uint64_t* template_uuid, const double* position,
    const double* rotation, std::uint32_t tracking, std::uint32_t flags,
    std::uint32_t* entity_handle, std::uint32_t* outcome) {
    return WorldRuntime::SpawnEntity(template_uuid, position, rotation, tracking, flags, entity_handle, outcome);
}
}
