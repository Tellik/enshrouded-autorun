#pragma once
#include "abi/runtime_api.h"
#include <cstddef>
#include <cstdint>

namespace WorldRuntime::RegisteredVoxelGrid {
enum class Result : std::uint32_t {
    Success = 1, Pending = 2, Rejected = 3, StaleSession = 4, Unavailable = 5, Fault = 6
};

// These typed adapters resolve the original named DI objects for every call.
// No function-address discovery, code signature or per-build method profile is
// used. Existing Session/Scene owner infrastructure remains a prerequisite.
// The normal read path never publishes jobs or waits for streaming completion.
bool Healthy() noexcept;
bool Available();
Result GetSpec(const char* grid_id, KfcRuntimeGridSpec* output);
Result GetSpecForSession(std::uint64_t expected_session, const char* grid_id, KfcRuntimeGridSpec* output);
Result Read(const std::int32_t origin[3], const std::uint32_t dimensions[3],
    std::uint16_t* output, std::size_t capacity, std::size_t* actual);
// The public session route calls only the original registered voxel reader and
// publishes a complete buffer after the session epoch is checked again.
Result ReadForSession(std::uint64_t expected_session, const std::int32_t origin[3],
    const std::uint32_t dimensions[3], std::uint16_t* output, std::size_t capacity, std::size_t* actual);
}
