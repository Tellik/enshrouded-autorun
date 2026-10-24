#pragma once
#include <cstdint>

namespace AutoRunMovement {
void ObserveInputForSession(const void* input, std::uint64_t session, std::uint64_t now);
const void* StableInputForSession(std::uint64_t session, std::uint64_t now);
bool Install();
void Stop();
bool Available();
bool Set(bool enabled, std::uint64_t expected_session);
void Status(std::uint64_t& hook_hits, std::uint64_t& local_hits,
            std::uint64_t& writes, bool& active, float& last_x, float& last_y);
}
