#include "ecs/ecs_session_state.h"

using namespace EcsImplementation;

struct SetEntityScaleOperation {
    std::uint32_t handle{};
    float scale[3]{};
    bool result{};
};

void set_entity_scale_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<SetEntityScaleOperation*>(opaque);
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType component{};
    if (!resolve_component("keen::ecs::CurrentTransform", component) || component.size != 0x38) return;
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    if (!layout_snapshot(layout) || !entity_for_handle(operation.handle, layout, entity) ||
        !component_address(entity, layout, component, address)) return;
    struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; };
    static_assert(sizeof(NativeTransform) == 0x38);
    constexpr auto scale_offset = offsetof(NativeTransform, scale);
    float before[3]{}, verified[3]{};
    if (!read_bytes(address + scale_offset, before, sizeof(before))) return;
    SIZE_T written{};
    if (!WriteProcessMemory(GetCurrentProcess(), reinterpret_cast<void*>(address + scale_offset),
            operation.scale, sizeof(operation.scale), &written) || written != sizeof(operation.scale) ||
        !read_bytes(address + scale_offset, verified, sizeof(verified)) ||
        std::memcmp(verified, operation.scale, sizeof(verified))) {
        SIZE_T restored{};
        WriteProcessMemory(GetCurrentProcess(), reinterpret_cast<void*>(address + scale_offset),
            before, sizeof(before), &restored);
        return;
    }
    operation.result = true;
}

extern "C" bool __cdecl KfcRuntimeWorldEntitySetScale(std::uint32_t handle, const double* scale) {
    if (!handle || !scale || !KfcRuntimeEcsCanWrite()) return false;
    auto operation = std::make_shared<SetEntityScaleOperation>();
    operation->handle = handle;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(scale[axis]) || std::abs(scale[axis]) > std::numeric_limits<float>::max()) return false;
        operation->scale[axis] = static_cast<float>(scale[axis]);
    }
    return GameThreadDispatcher::Invoke(set_entity_scale_on_game_thread, operation) && operation->result;
}
