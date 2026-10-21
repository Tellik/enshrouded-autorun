#include "engine-context/engine_context_state.h"

using namespace EngineContextImplementation;

namespace EngineContextImplementation {
bool untracked_component(std::uintptr_t owner, std::uint16_t index,
                         std::size_t mask_a, std::size_t mask_b){
    // ComponentRegistry owner layout v1 owns the original registration count at
    // +16. The policy is by current registration index, never a type-name list.
    std::uint64_t count{}, count_again{}, a{}, b{}, a_again{}, b_again{};
    const auto word = static_cast<std::size_t>(index / 64) * sizeof(std::uint64_t);
    if (!owner || !mask_a || !mask_b || !field(owner, 16, count) || !count || count > 1024 || index >= count ||
        !field(owner, mask_a + word, a) || !field(owner, mask_b + word, b) ||
        !field(owner, mask_a + word, a_again) || !field(owner, mask_b + word, b_again) ||
        !field(owner, 16, count_again) || count_again != count || a_again != a || b_again != b)
        return false;
    return ((a | b) & (std::uint64_t{1} << (index % 64))) == 0;
}

bool try_shared(std::uintptr_t address){
    if (!address || address % alignof(LONG)) return false;
    __try {
        auto* lock = reinterpret_cast<volatile LONG*>(address);
        const LONG current = *lock;
        return current >= 0 && current < 0x7fffffff &&
            InterlockedCompareExchange(lock, current + 1, current) == current;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool try_exclusive(std::uintptr_t address){
    if (!address || address % alignof(LONG)) return false;
    __try {
        return InterlockedCompareExchange(reinterpret_cast<volatile LONG*>(address),
            static_cast<LONG>(0x80000000u), 0) == 0;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

void release_shared(std::uintptr_t address) noexcept{
    __try { InterlockedDecrement(reinterpret_cast<volatile LONG*>(address)); }
    __except (EXCEPTION_EXECUTE_HANDLER) {}
}

void release_exclusive(std::uintptr_t address) noexcept{
    __try { InterlockedExchange(reinterpret_cast<volatile LONG*>(address), 0); }
    __except (EXCEPTION_EXECUTE_HANDLER) {}
}

bool tracking_call(std::uintptr_t function, const std::array<std::uint8_t, 33>& expected,
                   std::uintptr_t manager, std::uintptr_t context, std::uintptr_t entity, std::uint16_t index){
    std::array<std::uint8_t, 33> bytes{};
    if (!read_span(function, bytes.data(), bytes.size()) || bytes != expected) return false;
    using Callback = void(__fastcall*)(std::uintptr_t, std::uintptr_t, std::uintptr_t, std::uint16_t, std::uint8_t);
    __try { reinterpret_cast<Callback>(function)(manager, context, entity, index, 0); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
}

namespace EngineContext {
bool EcsComponentWriteSupported(const EcsView& expected, std::uint16_t index) {
    Layout layout;
    {
        std::scoped_lock lock(context_mutex);
        if (!configured || !active_layout.write.enabled) return false;
        layout = active_layout;
    }
    if (expected.role != EcsRole::LocalAuthority) return false;
    EcsView current, again;
    const auto& w = layout.write;
    return resolve_ecs(layout, current, nullptr) && current == expected &&
        untracked_component(current.registry_owner, index, w.tracking_registry_a, w.tracking_registry_b) &&
        resolve_ecs(layout, again, nullptr) && again == current &&
        untracked_component(again.registry_owner, index, w.tracking_registry_a, w.tracking_registry_b);
}

struct EcsWriteGuard::State {
    Layout layout;
    EcsView expected;
    std::uintptr_t thread{}, handle{};
};

EcsWriteGuard::EcsWriteGuard() = default;

EcsWriteGuard::~EcsWriteGuard() { Release(); }

void EcsWriteGuard::Release() noexcept {
    if (scene_lock_) { release_exclusive(scene_lock_); scene_lock_ = 0; }
    if (registry_lock_) { release_shared(registry_lock_); registry_lock_ = 0; }
    maximum_bytes_ = 0;
    manager_ = tracker_ = tracking_context_ = registry_owner_ = 0;
    state_.reset();
}

bool EcsWriteGuard::Validate() const {
    if (!state_ || !registry_lock_ || !scene_lock_) return false;
    EcsView current;
    std::uint8_t mode{};
    const auto& s = *state_;
    const auto& w = s.layout.write;
    std::uintptr_t name{}, handle{}, callback{}, context{}, wanted_context{};
    std::uint64_t length{}, capacity{};
    std::uint32_t stop{};
    std::uint8_t running{};
    return field(s.thread, w.name_pointer, name) && name == s.thread + w.inline_name &&
        field(s.thread, w.name_length, length) && length == w.name_size &&
        field(s.thread, w.name_capacity, capacity) && capacity == w.name_capacity_value &&
        field(s.thread, w.thread_handle, handle) && handle == s.handle &&
        field(s.thread, w.thread_callback, callback) && callback == s.expected.owners.image + w.callback &&
        add(s.expected.scene_owner, w.scene_context, wanted_context) &&
        field(s.thread, w.thread_context, context) && context == wanted_context &&
        field(s.thread, w.thread_stop, stop) && stop == 0 &&
        field(s.expected.scene_owner, w.scene_state, running) && running == w.running_state &&
        resolve_ecs(s.layout, current, nullptr, s.expected.owners.image) &&
        current == state_->expected &&
        field(current.execution_owner, w.execution_mode, mode) && mode == w.authority_mode;
}

bool EcsWriteGuard::ComponentSupported(std::uint16_t component_index) const {
    if (!registry_lock_ || !scene_lock_ || component_index >= 1024) return false;
    std::uintptr_t owner{};
    return field(manager_, 0, owner) && owner == registry_owner_ &&
        untracked_component(owner, component_index, tracking_registry_a_, tracking_registry_b_);
}

bool EcsWriteGuard::TrackChange(std::uintptr_t entity, std::uint16_t component_index) const {
    std::uintptr_t manager{};
    if (!entity || !ComponentSupported(component_index) ||
        !field(tracking_context_, tracking_manager_offset_, manager) || manager != manager_) return false;
    // Both tracking category bits are absent. The original recorder has no
    // queue, stamp, alias traversal or allocator effects for this component.
    // Repeat the policy after the separately guarded original entry returns.
    return tracking_call(tracker_, tracker_bytes_, manager_, tracking_context_, entity, component_index) &&
        ComponentSupported(component_index);
}

bool EcsWriteGuard::TryAcquire(const EcsView& expected, const char** reason) {
    const auto fail = [&](const char* message) {
        Release();
        if (reason) *reason = message;
        return false;
    };
    if (registry_lock_ || scene_lock_) {
        if (reason) *reason = "ecs-write-guard-already-held";
        return false;
    }
    Layout layout;
    {
        std::scoped_lock lock(context_mutex);
        if (!configured || !active_layout.write.enabled) return fail("ecs-write-gate-disabled");
        layout = active_layout;
    }
    auto prepared = std::make_unique<State>();
    prepared->layout = layout;
    prepared->expected = expected;
    const auto& w = layout.write;
    const auto image = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    if (!image || expected.owners.image != image || expected.role != EcsRole::LocalAuthority)
        return fail("ecs-write-requires-current-local-authority");
    std::uintptr_t address{};
    if (!add(image, w.registry_lock, address) || !try_shared(address))
        return fail("ecs-thread-registry-busy");
    registry_lock_ = address;
    // No allocation, caller mutex acquisition or queue wait from this point
    // until Release. Only TrackChange may enter its validated native recorder.
    // The image-global guard prevents unregister and
    // Scene free before the borrowed Scene lock is touched.
    RegisteredScene selected;
    const char* registration_reason{};
    if (!resolve_registered_scene(layout, image, selected, &registration_reason))
        return fail(registration_reason);
    if (!add(selected.scene, w.scene_lock, address) || !try_exclusive(address))
        return fail("ecs-scene-frame-busy");
    scene_lock_ = address;
    EcsView current;
    RegisteredScene registered_again;
    std::uint8_t mode{};
    if (!resolve_registered_scene(layout, image, registered_again) || registered_again != selected ||
        !resolve_ecs(layout, current, nullptr, image) || current.scene_owner != selected.scene || current != expected ||
        !field(current.execution_owner, w.execution_mode, mode) || mode != w.authority_mode)
        return fail("ecs-write-owner-changed-or-not-authoritative");
    maximum_bytes_ = w.maximum_bytes;
    manager_ = current.manager;
    registry_owner_ = current.registry_owner;
    tracker_ = image + w.tracker;
    if (!add(manager_, w.tracking_context, tracking_context_)) return fail("ecs-change-tracker-context-invalid");
    tracker_bytes_ = w.tracker_bytes;
    tracking_manager_offset_ = w.tracking_manager;
    tracking_registry_a_ = w.tracking_registry_a;
    tracking_registry_b_ = w.tracking_registry_b;
    prepared->thread = selected.thread;
    prepared->handle = selected.handle;
    state_ = std::move(prepared);
    if (reason) *reason = nullptr;
    return true;
}
}
