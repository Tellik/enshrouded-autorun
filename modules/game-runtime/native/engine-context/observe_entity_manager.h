#pragma once
#include <atomic>
#include <cstdint>
namespace GameThreadImplementation {
extern std::atomic<std::uintptr_t> entity_manager;
extern std::atomic<std::uint64_t> manager_changes, last_manager_observation_ms;
void __cdecl capture_entity_manager(void*, void*);
}
namespace GameThreadDispatcher {
std::uintptr_t EntityManager();
// Observe a current view without retaining its stack address.
void ObserveExecutionView(void* execution_view, bool local_player = false);
}
