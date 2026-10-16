#pragma once

#include "runtime.h"
#include "engine_context.h"
#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

#define KFC_EXPORT KFC_RUNTIME_API

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
bool SnapshotEntityIds(std::vector<std::uint32_t>& ids);
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop);
bool EntityIdentity(std::uint32_t handle, std::uint32_t* entity_id);
// Only from a live engine callback. Checks generation and layout epoch.
bool MatchesComponentAddress(std::uint32_t handle, const char* name, const void* expected);
void Shutdown();
}
