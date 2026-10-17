#pragma once
#include <atomic>
#include <cstdint>
#include <string>
namespace GameThreadImplementation {
extern std::atomic<bool> accepting;
extern std::atomic<bool> entity_context_hooks_ready;
extern std::atomic<std::uint32_t> engine_thread;
extern std::atomic<bool> command_observed;
extern std::string initialization_error;
void drain(void*, void*);
void CancelPendingOperations();
}
