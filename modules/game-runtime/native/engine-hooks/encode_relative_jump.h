#pragma once
#include <cstdint>
#include <limits>
namespace EngineHooks {
inline bool EncodeRelativeJump(std::uintptr_t instruction, std::uintptr_t destination, std::int32_t& value) {
    const auto delta = static_cast<std::int64_t>(destination) - static_cast<std::int64_t>(instruction + 5);
    if (delta < (std::numeric_limits<std::int32_t>::min)() ||
        delta > (std::numeric_limits<std::int32_t>::max)()) return false;
    value = static_cast<std::int32_t>(delta);
    return true;
}
}
