#pragma once
#include "abi/runtime_api.h"
#include <cstddef>
#include <cstdint>
namespace EcsRuntime {
inline constexpr std::size_t MaximumReadManyComponents = 64;
inline constexpr std::size_t MaximumReadManyBytes = 1u << 20;
// Only from a live engine callback. Checks generation and layout epoch.
bool MatchesComponentAddress(std::uint32_t handle, const char* name, const void* expected);
}
