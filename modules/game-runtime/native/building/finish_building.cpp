#include "building/building_request_state.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
bool native_finish(void* context, bool complete){
    const auto* profile = find_operation("runtime.world.entity.finish_building");
    const auto event_rva = KfcRuntimeCompatibility::EnshroudedClient::world_finish_event_id_rva;
    std::uint32_t event_id{};
    const auto base = image_base();
    if (!context || !profile || !profile->available || !event_rva ||
        !read_memory(base + event_rva, &event_id, sizeof(event_id))) return false;
    // The original client and server call sites pass this value even when it
    // is zero. Zero is not a missing event/address sentinel.
    auto function = reinterpret_cast<NativeFinish>(base + profile->function_rva);
    __try { function(context, nullptr, event_id, complete); return true; }
    __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}
}

namespace WorldRuntime {
bool FinishBuilding(bool complete, std::uint32_t* outcome) {
    if (!outcome || !OperationAvailable("runtime.world.entity.finish_building")) return false;
    auto request = std::make_shared<EntityRequest>();
    request->kind = EntityRequest::Kind::Finish;
    request->complete = complete;
    return invoke_entity_request(request, outcome);
}
}

extern "C" {
bool __cdecl KfcRuntimeWorldEntityFinishBuilding(bool complete, std::uint32_t* outcome) {
    return WorldRuntime::FinishBuilding(complete, outcome);
}
}
