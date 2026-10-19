#include "player-input/world_cursor_state.h"
#include "player-input/apply_building_input.h"
#include "engine-context/observe_entity_manager.h"
#include "engine-context/world_context_state.h"

using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
bool copy_cursor_safely(const void* source, std::uint8_t* output){
    if (!source || !output) return false;
    __try { std::memcpy(output, source, NativeCursorSize); return true; }
    __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}
}

namespace WorldRuntime {
void SetCursorHookReady(bool ready) {
    InterlockedExchange(&cursor_mailbox.hook_ready, ready ? 1 : 0);
}

void OnCursorUpdate(void* cursor, void* execution_view, void* cursor_frame) {
    std::array<std::uint8_t, NativeCursorSize> sample{};
    if (!copy_cursor_safely(cursor, sample.data()) || sample[0x98] > 1) return;
    for (const auto base : {std::size_t{0}, std::size_t{0x38}}) {
        for (const auto offset : {std::size_t{0x18}, std::size_t{0x1c}, std::size_t{0x20}, std::size_t{0x24},
                                  std::size_t{0x28}, std::size_t{0x2c}, std::size_t{0x30}}) {
            float value{};
            std::memcpy(&value, sample.data() + base + offset, sizeof(value));
            if (!std::isfinite(value)) return;
        }
    }
    // R13 is the live execution view passed to this engine system (verified
    // at the profiled cursor call site). The entity lookup hook need not run
    // again when entering a remote world; refresh through this recurring hook.
    GameThreadDispatcher::ObserveExecutionView(execution_view, true);
    observe_client_read_world(cursor_frame, execution_view);
    // Evidence only: this root belongs to client_cursor, unlike the two
    // player_building_place_prop callbacks. No unverified world offset is used.
    std::uintptr_t root{};
    if (read_memory(reinterpret_cast<std::uintptr_t>(execution_view), &root, sizeof(root)))
        local_execution_root.store(root, std::memory_order_release);
    cursor_hook_hits.fetch_add(1, std::memory_order_relaxed);
    BuildingInput::Observe(cursor, execution_view);
    if (InterlockedCompareExchange(&cursor_mailbox.lock, 1, 0) != 0) return;
    cursor_mailbox.bytes = sample;
    cursor_mailbox.context_generation = context_resets.load(std::memory_order_acquire);
    ++cursor_mailbox.sequence;
    InterlockedExchange(&cursor_mailbox.lock, 0);
}

bool ReadCursorSnapshot(std::uint8_t* bytes, std::size_t capacity, std::uint64_t* sequence) {
    if (!bytes || capacity < NativeCursorSize || !sequence ||
        InterlockedCompareExchange(&cursor_mailbox.hook_ready, 0, 0) == 0) return false;
    if (InterlockedCompareExchange(&cursor_mailbox.lock, 1, 0) != 0) return false;
    const auto current_sequence = cursor_mailbox.sequence;
    const auto sample = cursor_mailbox.bytes;
    const auto sample_generation = cursor_mailbox.context_generation;
    InterlockedExchange(&cursor_mailbox.lock, 0);
    if (!current_sequence || sample_generation != context_resets.load(std::memory_order_acquire)) return false;
    std::memcpy(bytes, sample.data(), sample.size());
    *sequence = current_sequence;
    return true;
}
}

extern "C" {
bool __cdecl KfcRuntimeWorldCursorRead(std::uint8_t* cursor, std::size_t capacity, std::uint64_t* sequence) {
    return WorldRuntime::ReadCursorSnapshot(cursor, capacity, sequence);
}
}

namespace WorldRuntime::Implementation {
CursorMailbox cursor_mailbox;
}
