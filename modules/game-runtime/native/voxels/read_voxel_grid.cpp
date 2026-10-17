#include "voxels/voxel_request.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

extern "C" {
bool __cdecl KfcRuntimeWorldGridGetSpec(const char* grid_id, KfcRuntimeGridSpec* spec) {
    return WorldRuntime::GetGridSpec(grid_id, spec);
}
}

namespace WorldRuntime {
bool GetGridSpec(const char* grid_id, KfcRuntimeGridSpec* spec){
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    if (!grid_id || !spec || !OperationAvailable("runtime.world.voxel.read")) return false;
    const auto found = std::find_if(runtime_world_grids.begin(), runtime_world_grids.end(),
        [grid_id](const auto& grid) { return grid.id == grid_id; });
    if (found == runtime_world_grids.end() || found->id.size() >= sizeof(spec->id)) return false;
    *spec = {};
    std::memcpy(spec->id, found->id.c_str(), found->id.size() + 1);
    for (int axis = 0; axis < 3; ++axis) {
        spec->origin[axis] = found->origin[axis];
        spec->cell_size[axis] = found->cell_size[axis];
        spec->maximum[axis] = found->maximum[axis];
    }
    return true;
}
}
