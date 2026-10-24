#pragma once
#include <cstdint>

namespace AutoRunMovement {
bool Install();
void Stop();
bool Available();
bool Set(bool enabled, std::uint64_t expected_session);
void Status(std::uint64_t& hook_hits, std::uint64_t& local_hits,
            std::uint64_t& writes, bool& active, float& last_x, float& last_y);
}
