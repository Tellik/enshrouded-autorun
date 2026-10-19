#include "voxels/read_registered_voxel_grid.h"
#include "voxels/voxel_operations.h"

extern "C" {
bool __cdecl KfcRuntimeWorldGridGetSpec(const char* grid_id, KfcRuntimeGridSpec* spec) {
    return WorldRuntime::GetGridSpec(grid_id, spec);
}
std::uint32_t __cdecl KfcRuntimeWorldGridGetSpecForSession(std::uint64_t expected_session,
    const char* grid_id, KfcRuntimeGridSpec* spec) {
    return static_cast<std::uint32_t>(WorldRuntime::RegisteredVoxelGrid::GetSpecForSession(
        expected_session, grid_id, spec));
}
}

namespace WorldRuntime {
bool GetGridSpec(const char* grid_id, KfcRuntimeGridSpec* spec){
    return RegisteredVoxelGrid::GetSpec(grid_id, spec) == RegisteredVoxelGrid::Result::Success;
}
}
