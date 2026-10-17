#include "engine-context/engine_context_state.h"

using namespace EngineContextImplementation;

namespace EngineContext {
struct ResourceLifeGuard::State {
    Layout layout;
    Owners owners;
    std::uintptr_t registry_lock{}, service{}, reader{}, thread{}, handle{}, payload{};
    std::uint64_t payload_size{};
    bool owns_registry{};
};

ResourceLifeGuard::ResourceLifeGuard() = default;

ResourceLifeGuard::~ResourceLifeGuard() { Release(); }

void ResourceLifeGuard::Release() noexcept {
    if (state_ && state_->registry_lock && state_->owns_registry) {
        release_shared(state_->registry_lock);
    }
    // Retain prepared storage until destruction or the next Prepare, so a
    // rejected borrowed acquisition never frees heap storage under Scene.
    if (state_) { state_->registry_lock = 0; state_->owns_registry = false; }
}

bool ResourceLifeGuard::Validate(Owners& owners) const {
    if (!state_ || !state_->registry_lock) return false;
    const auto& s = *state_;
    const auto& w = s.layout.write;
    const auto& r = s.layout.resource_write;
    std::uintptr_t name{}, handle{}, callback{}, context{}, reader{}, payload{};
    std::uint64_t length{}, capacity{}, payload_size{};
    std::uint32_t stop{}, initialized{};
    Owners current;
    if (!field(s.thread, w.name_pointer, name) || name != s.thread + w.inline_name ||
        !field(s.thread, w.name_length, length) || length != r.name_size ||
        !field(s.thread, w.name_capacity, capacity) || capacity != w.name_capacity_value ||
        !field(s.thread, w.thread_handle, handle) || handle != s.handle ||
        !field(s.thread, w.thread_callback, callback) || callback != s.owners.image + r.callback ||
        !field(s.thread, w.thread_context, context) || context != s.thread ||
        !field(s.thread, w.thread_stop, stop) || stop != 0 ||
        !field(s.service, r.initialized, initialized) || initialized != r.initialized_value ||
        !field(s.service, r.reader, reader) || reader != s.reader ||
        !field(s.service, r.payload_pointer, payload) || payload != s.payload ||
        !field(s.service, r.payload_size, payload_size) || payload_size != s.payload_size ||
        !resolve(s.layout, current, nullptr, s.owners.image) || current != s.owners) return false;
    owners = current;
    return true;
}

bool ResourceLifeGuard::Contains(std::uintptr_t address, std::size_t size) const {
    return state_ && state_->registry_lock && address >= state_->payload &&
        address - state_->payload <= state_->payload_size &&
        size <= state_->payload_size - (address - state_->payload);
}

std::uintptr_t ResourceLifeGuard::PayloadBase() const { return state_ ? state_->payload : 0; }

bool ResourceLifeGuard::TryAcquire(std::uintptr_t expected_service, std::uintptr_t expected_reader) {
    return Prepare() && AcquirePrepared(expected_service, expected_reader, 0);
}

bool ResourceLifeGuard::Prepare() {
    if (state_ && state_->registry_lock) return false;
    // Allocate/snapshot before the original Shared guard. Its critical section
    // never takes a caller mutex, allocates, waits or invokes engine code.
    auto state = std::make_unique<State>();
    {
        std::scoped_lock lock(context_mutex);
        if (!configured || !active_layout.resource_write.enabled) return false;
        state->layout = active_layout;
    }
    if (!resolve(state->layout, state->owners, nullptr)) return false;
    state_ = std::move(state);
    return true;
}

bool ResourceLifeGuard::TryAcquire(const EcsWriteGuard& scene, std::uintptr_t expected_service,
    std::uintptr_t expected_reader) {
    if (!state_ || !scene.scene_lock_ || !scene.registry_lock_ || !scene.Validate()) return false;
    return AcquirePrepared(expected_service, expected_reader, scene.registry_lock_);
}

bool ResourceLifeGuard::AcquirePrepared(std::uintptr_t expected_service, std::uintptr_t expected_reader,
    std::uintptr_t borrowed_registry) {
    if (!state_ || state_->registry_lock || !expected_service || !expected_reader) return false;
    const auto& w = state_->layout.write;
    const auto& r = state_->layout.resource_write;
    const auto image = state_->owners.image;
    std::uintptr_t lock{};
    if (!add(image, w.registry_lock, lock) ||
        (borrowed_registry ? borrowed_registry != lock : !try_shared(lock))) return false;
    state_->registry_lock = lock;
    state_->owns_registry = !borrowed_registry;
    const auto fail = [&] { Release(); return false; };
    std::uint16_t count{};
    if (!field(image, w.registry_count, count) || count > w.capacity) return fail();
    unsigned matches{};
    for (std::size_t index = 0; index < count; ++index) {
        const auto row = image + w.registry_entries + index * w.stride;
        std::uintptr_t name{}, handle{};
        std::uint64_t length{};
        if (!field(row, w.row_length, length) || !field(row, w.row_name, name) || !field(row, w.row_handle, handle))
            return fail();
        if (length != r.name_size) continue;
        std::array<char, 64> text{};
        if (!read_span(name, text.data(), length)) return fail();
        if (std::memcmp(text.data(), r.name.data(), length)) continue;
        if (++matches != 1 || !handle || name < w.inline_name || name - w.inline_name < r.service_thread)
            return fail();
        state_->thread = name - w.inline_name;
        state_->service = state_->thread - r.service_thread;
        state_->handle = handle;
    }
    if (matches != 1 || state_->service != expected_service) return fail();
    state_->reader = expected_reader;
    if (!field(state_->service, r.payload_pointer, state_->payload) || !state_->payload ||
        !field(state_->service, r.payload_size, state_->payload_size) || !state_->payload_size ||
        state_->payload_size > UINTPTR_MAX - state_->payload) return fail();
    Owners owners;
    if (!Validate(owners)) return fail();
    return true;
}
}
