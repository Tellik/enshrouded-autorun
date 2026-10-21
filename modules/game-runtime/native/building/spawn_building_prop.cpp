#include "building/building_operations.h"
#include "impact/submit_building_prop_spawn.h"
#include "ecs/entity_session.h"
#include "ecs/read_building_prop.h"
#include <windows.h>
#include <algorithm>

namespace WorldRuntime {
bool SpawnEntity(const std::uint64_t template_uuid[2], const double position[3],
                 const double rotation[4], std::uint32_t tracking, std::uint32_t flags,
                 std::uint32_t* entity_handle, std::uint32_t* outcome) {
    if (!outcome) return false;
    *outcome = 1;
    if (!entity_handle) return false;
    *entity_handle = 0;
    if (!template_uuid || !position || !rotation || !tracking || flags) return false;
    const auto session = KfcRuntimeWorldSessionId();
    if (!session) return false;
    ImpactRuntime::BuildingPropSpawnReceipt receipt;
    using Result = ImpactRuntime::BuildingPropSpawnResult;
    auto submitted = Result::Unavailable;
    const auto dispatch_deadline = GetTickCount64() + 250;
    for (;;) {
        if (KfcRuntimeWorldSessionId() != session) { *outcome = 5; return false; }
        submitted = ImpactRuntime::SubmitBuildingPropSpawn(session,
            template_uuid, position, rotation, tracking, flags, receipt);
        if (submitted != Result::Unavailable || GetTickCount64() >= dispatch_deadline) break;
        // Unavailable is returned only before the original call starts. All
        // original gates have been released. Submitted and Uncertain must
        // never be resubmitted, even if completion is temporarily unreadable.
        Sleep(2);
    }
    if (submitted != Result::Submitted) {
        if (submitted == Result::StaleSession) *outcome = 5;
        else if (submitted == Result::Uncertain) *outcome = 6;
        return false;
    }
    // The receipt is the exact EntityId allocated by the original queue.
    // A public handle is returned only after that entity and its building
    // components exist in the same world epoch. Never infer it by proximity.
    const auto deadline = GetTickCount64() + 2000;
    do {
        if (KfcRuntimeWorldSessionId() != session) { *outcome = 5; return false; }
        KfcRuntimePropRecord prop{};
        std::uint8_t marker{};
        if (EcsRuntime::ResolvePropEntityId(receipt.entity_id, &prop) &&
            prop.entity_id == receipt.entity_id && prop.entity_handle && prop.item_id == tracking &&
            prop.template_uuid[0] == template_uuid[0] && prop.template_uuid[1] == template_uuid[1] &&
            std::equal(receipt.position.begin(), receipt.position.end(), prop.position) &&
            std::equal(receipt.rotation.begin(), receipt.rotation.end(), prop.orientation) &&
            std::all_of(std::begin(prop.scale), std::end(prop.scale), [](float value) { return value == 1.0f; }) &&
            KfcRuntimeEcsRead(prop.entity_handle, "keen::ecs::DisableRandomLootOnDestroy", &marker, sizeof(marker)) &&
            KfcRuntimeWorldSessionId() == session) {
            *entity_handle = prop.entity_handle;
            *outcome = 0;
            return true;
        }
        Sleep(10);
    } while (GetTickCount64() < deadline);
    *outcome = 4;
    EcsRuntime::MarkSessionWriteUncertain(session, "native-building-spawn-readback-unconfirmed");
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
