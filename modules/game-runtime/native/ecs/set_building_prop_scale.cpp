#include "ecs/ecs_session_state.h"

extern "C" bool __cdecl KfcRuntimeWorldEntitySetScale(std::uint32_t handle, const double* scale) {
    constexpr const char* component = "keen::ecs::CurrentTransform";
    struct NativeTransform {
        std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding;
    };
    static_assert(sizeof(NativeTransform) == 0x38);
    if (!handle || !scale || !KfcRuntimeEcsCanWriteComponent(component)) return false;
    std::array<float, 3> requested{};
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(scale[axis]) || std::abs(scale[axis]) > std::numeric_limits<float>::max()) return false;
        requested[axis] = static_cast<float>(scale[axis]);
    }
    std::array<std::uint8_t, sizeof(NativeTransform)> before{}, desired{}, mask{};
    if (!KfcRuntimeEcsRead(handle, component, before.data(), before.size())) return false;
    desired = before;
    constexpr auto offset = offsetof(NativeTransform, scale);
    std::memcpy(desired.data() + offset, requested.data(), sizeof(requested));
    std::fill_n(mask.begin() + offset, sizeof(requested), std::uint8_t{1});
    // Reuse the canonical ECS ownership, original scene gates, tracking policy,
    // expected-value comparison and verified rollback. Scale never owns a
    // second memory writer or bypasses tracked-component restrictions.
    return KfcRuntimeEcsCompareExchange(handle, component, before.data(), mask.data(),
        desired.data(), desired.size()) == 1;
}
