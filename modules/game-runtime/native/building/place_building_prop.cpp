#include "building/building_request_state.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
bool valid_transform(const EntityRequest& request, EngineTransform& transform){
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(request.position[axis])) return false;
        const auto fixed = std::ldexp(request.position[axis], 32);
        if (!std::isfinite(fixed) || fixed < static_cast<double>(INT64_MIN) || fixed > static_cast<double>(INT64_MAX)) return false;
        transform.position[axis] = static_cast<std::int64_t>(std::llround(fixed));
    }
    long double norm{};
    for (int index = 0; index < 4; ++index) {
        if (!std::isfinite(request.rotation[index])) return false;
        norm += static_cast<long double>(request.rotation[index]) * request.rotation[index];
    }
    if (!std::isfinite(norm) || norm < 1e-12L) return false;
    const auto inverse_norm = 1.0L / std::sqrt(norm);
    for (int index = 0; index < 4; ++index)
        transform.rotation[index] = static_cast<float>(request.rotation[index] * inverse_norm);
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(request.scale[axis])) return false;
        transform.scale[axis] = request.scale[axis];
    }
    return true;
}

}

namespace WorldRuntime {

bool PlaceEntity(const double position[3], const double rotation[4], const float bounds[6],
                 std::uint32_t tracking, std::uint32_t feedback, std::uint32_t* outcome) {
    if (!position || !rotation || !bounds || !tracking || !feedback || !outcome ||
        !OperationAvailable("runtime.world.entity.place") ||
        !OperationAvailable("runtime.world.entity.finish_building")) return false;
    for (int axis = 0; axis < 3; ++axis)
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) || bounds[axis] > bounds[axis + 3]) return false;
    auto request = std::make_shared<EntityRequest>();
    request->kind = EntityRequest::Kind::Place;
    std::copy_n(position, 3, request->position);
    std::copy_n(rotation, 4, request->rotation);
    std::copy_n(bounds, 6, request->bounds);
    request->tracking = tracking;
    request->feedback = feedback;
    return invoke_entity_request(request, outcome);
}

}

extern "C" {
bool __cdecl KfcRuntimeWorldEntityPlace(const double* position, const double* rotation,
    const float* bounds, std::uint32_t tracking, std::uint32_t feedback, std::uint32_t* outcome) {
    return WorldRuntime::PlaceEntity(position, rotation, bounds, tracking, feedback, outcome);
}

}
