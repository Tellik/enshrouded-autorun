#pragma once

#include <cstdint>
#include <cstddef>
#include <string>
#include <memory>

namespace GameThreadDispatcher {
using Operation = void (*)(void* context);

bool Ready();
bool EntityContextReady();
std::uint32_t ThreadId();
std::string Status();
std::string Diagnostics();

// Execute immediately only inside an active dispatcher callback. Otherwise
// enqueue and wait until the continuous Keen update consumes the command.
// Calls on the remembered engine thread outside that callback are rejected.
// Jobs own all input/output storage, including after a caller times out.
bool Invoke(Operation operation, std::shared_ptr<void> context, std::uint32_t timeout_ms = 75);
}
