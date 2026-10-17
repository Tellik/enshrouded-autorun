#include "engine-context/observe_entity_manager.h"
#include "profiles/game_profile.h"
#include "engine-context/world_context.h"
#include <windows.h>

namespace GameThreadImplementation {
std::atomic<std::uintptr_t> entity_manager{};

std::atomic<std::uint64_t> local_player_manager_ms{};

std::atomic<std::uint64_t> manager_changes{}, last_manager_observation_ms{};

void __cdecl capture_entity_manager(void* lookup_context, void*) {
    GameThreadDispatcher::ObserveExecutionView(lookup_context, false);
}
}

using namespace GameThreadImplementation;

namespace GameThreadDispatcher {
void ObserveExecutionView(void* lookup_context, bool local_player) {
    if (!lookup_context) return;
    __try {
        const auto root = *static_cast<std::uintptr_t*>(lookup_context);
        if (!root) return;
        const auto manager = *reinterpret_cast<std::uintptr_t*>(root + KfcRuntimeCompatibility::EnshroudedClient::lookup_manager);
        if (manager) {
            // A released world retains a readable manager with an empty table.
            // Do not replace a current manager with that loading/preview context.
            const auto count = *reinterpret_cast<const std::uint64_t*>(manager + KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count);
            const auto table = *reinterpret_cast<const std::uintptr_t*>(manager + KfcRuntimeCompatibility::EnshroudedClient::entity_manager_table);
            if (!count || count > (1u << 20) || !table) return;
            // Probe both ends under SEH before publishing the borrowed manager.
            const volatile auto first = *reinterpret_cast<const std::uintptr_t*>(table);
            const volatile auto last = *reinterpret_cast<const std::uintptr_t*>(table + (count - 1) * sizeof(std::uintptr_t));
            (void)first; (void)last;
            const auto now = GetTickCount64();
            const auto local_observation = local_player_manager_ms.load(std::memory_order_acquire);
            // Generic lookups also run for preview worlds. While the local
            // player's recurring cursor callback is live, it is authoritative.
            // Otherwise these two hooks alternate managers and invalidate every
            // ECS handle/scan on successive frames (observed in the live test).
            if (!local_player && local_observation && now - local_observation < 1000 &&
                entity_manager.load(std::memory_order_acquire) != manager) return;
            if (local_player) local_player_manager_ms.store(now, std::memory_order_release);
            if (entity_manager.exchange(manager, std::memory_order_acq_rel) != manager) {
                manager_changes.fetch_add(1, std::memory_order_relaxed);
                WorldRuntime::ResetContext();
            }
            last_manager_observation_ms.store(GetTickCount64(), std::memory_order_relaxed);
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {}
}

std::uintptr_t EntityManager() { return entity_manager.load(std::memory_order_acquire); }
}
