#pragma once
#include "engine-context/world_context_state.h"
#include "player-input/sample_world_cursor.h"
namespace WorldRuntime::Implementation {
constexpr std::size_t NativeCursorSize = 0xa0;

struct CursorMailbox {
    volatile long lock{};
    volatile long hook_ready{};
    std::uint64_t sequence{};
    std::uint64_t context_generation{};
    std::array<std::uint8_t, NativeCursorSize> bytes{};
};

extern CursorMailbox cursor_mailbox;

bool copy_cursor_safely(const void* source, std::uint8_t* output);

bool LocalCursorInputAvailable();
}
