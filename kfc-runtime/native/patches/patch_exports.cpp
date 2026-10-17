#include "ecs/ecs_state.h"



using namespace EcsImplementation;

extern "C" bool __cdecl KfcRuntimePatchAvailable(const char* name) {
    if (name && std::strcmp(name, "runtime.gameplay.patch") == 0) return PatchRuntime::AnyAvailable();
    return PatchRuntime::Available(name);
}

extern "C" bool __cdecl KfcRuntimePatchSetEnabled(const char* name, bool enabled, std::uint32_t* outcome) {
    return PatchRuntime::SetEnabled(name, enabled, outcome);
}
