#pragma once
#include "abi/runtime_api.h"
#include <cstddef>
#include <cstdint>
#include <memory>

namespace EngineContext { struct EcsView; }

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

// Internal transaction reader. Construct before taking the original gates.
// The caller owns operation_mutex, SessionGuard and a validated Scene guard.
// No gate, allocation or session query occurs in the pinned methods. The first
// read pins the exact DI/World/task snapshot until this object is destroyed.
class PinnedRegion {
public:
    PinnedRegion(const std::int32_t origin[3], const std::uint32_t dimensions[3]);
    ~PinnedRegion();
    PinnedRegion(const PinnedRegion&) = delete;
    PinnedRegion& operator=(const PinnedRegion&) = delete;
    bool AvailablePinned(const EngineContext::EcsView& owner) const;
    Result ReadPinned(const EngineContext::EcsView& owner, std::uint16_t* output, std::size_t capacity);
    bool ValidatePinned(const EngineContext::EcsView& owner) const;
    std::uintptr_t WorldPayload() const;
private:
    struct State;
    std::unique_ptr<State> state_;
};
}
