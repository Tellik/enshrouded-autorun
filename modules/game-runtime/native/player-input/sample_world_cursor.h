#pragma once
#include "abi/runtime_api.h"
#include <cstddef>
#include <cstdint>
#include <string>
namespace WorldRuntime {
void OnCursorUpdate(void* cursor, void* execution_view, void* cursor_frame = nullptr);
void SetCursorHookReady(bool ready);
bool ReadCursorSnapshot(std::uint8_t* bytes, std::size_t capacity, std::uint64_t* sequence);
}
