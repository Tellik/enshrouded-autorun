#pragma once
#include <cstddef>
#include <cstdint>
namespace EngineHooks {
bool WriteExecutableBytes(std::uintptr_t address, const void* bytes, std::size_t size,
                          const void* expected = nullptr);
bool ReplaceExecutableBytes(std::uintptr_t address, const void* expected,
                            const void* replacement, std::size_t size);
}
