#pragma once

#include "abi/runtime_api.h"
#include "engine-context/engine_owners.h"
#include "engine-context/hold_resource_lifetime.h"
#include "engine-context/lock_scene_access.h"
#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

namespace EcsRuntime {
// Pins the existing ECS binding epoch before any original engine gate is
// acquired. Declare before EcsWriteGuard and destroy after it. The original
// Scene guard must still validate View() before reading or writing payloads.
class SessionGuard {
public:
    SessionGuard();
    ~SessionGuard();
    SessionGuard(const SessionGuard&) = delete;
    SessionGuard& operator=(const SessionGuard&) = delete;
    bool TryAcquire(std::uint64_t expected_session);
    const EngineContext::EcsView& View() const;
    bool WritesAllowed() const;
    void MarkWriteUncertain(const char* reason);
private:
    struct State;
    std::unique_ptr<State> state_;
};
bool Initialize();
void Tick();
std::string Status();
std::string Diagnostics();
bool EntityIdentity(std::uint32_t handle, std::uint32_t* entity_id);
void Shutdown();
}
