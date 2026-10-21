#pragma once
#include <array>
#include <cstdint>

namespace ImpactRuntime {
enum class BuildingPropSpawnResult {
    Submitted,
    Rejected,
    Unavailable,
    StaleSession,
    Uncertain,
};

// The original queue allocates an EntityId. It is not the public opaque handle.
// The caller confirms this exact ID after the normal native command drain.
struct BuildingPropSpawnReceipt {
    std::uint32_t entity_id{};
    std::array<std::int64_t, 3> position{};
    std::array<float, 4> rotation{};
};

bool BuildingPropSpawnAvailable();
BuildingPropSpawnResult SubmitBuildingPropSpawn(std::uint64_t expected_session,
    const std::uint64_t template_uuid[2], const double position[3],
    const double rotation[4], std::uint32_t tracking, std::uint32_t flags,
    BuildingPropSpawnReceipt& output);
}
