#pragma once
#include <cstdint>

namespace ImpactRuntime {
enum class DestructionResult {
    Submitted,
    Rejected,
    Unavailable,
    StaleSession,
    Uncertain,
};

// A read-only capability snapshot. Submission repeats every owner, operation,
// queue and opaque-handle check under the pinned session and original Scene.
bool EntityDestructionAvailable();

// Admits only the registered original destroyEntity callback. Submitted means
// one exact original command was appended, never that deletion has completed.
// No fallback or retry is permitted after Submitted or Uncertain.
DestructionResult SubmitEntityDestruction(std::uint64_t expected_session,
    std::uint32_t entity_handle);
}
