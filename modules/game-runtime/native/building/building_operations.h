#pragma once
#include "abi/runtime_api.h"
#include <cstddef>
#include <cstdint>
#include <string>
namespace WorldRuntime {
bool SpawnEntity(const std::uint64_t template_uuid[2], const double position[3],
                 const double rotation[4], std::uint32_t tracking, std::uint32_t flags,
                 std::uint32_t* entity_handle, std::uint32_t* outcome);
bool PlaceEntity(const double position[3], const double rotation[4], const float bounds[6],
                 std::uint32_t tracking, std::uint32_t feedback, std::uint32_t* outcome);
bool DestroyEntity(const double position[3], const double rotation[4], const float bounds[6],
                   std::uint32_t tracking, std::uint32_t feedback, std::uint32_t* outcome);
bool DestroyEntityHandle(std::uint32_t entity_handle, std::uint32_t* outcome);
bool FinishBuilding(bool complete, std::uint32_t* outcome);
}
