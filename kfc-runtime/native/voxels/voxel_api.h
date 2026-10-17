#pragma once
#include "ecs/entities.h"
#include <cstddef>
#include <cstdint>
#include <string>
namespace WorldRuntime {
bool ReadVoxels(const std::int32_t origin[3], const std::uint32_t dimensions[3],
                std::uint16_t* values, std::size_t capacity, std::size_t* actual);
bool GetGridSpec(const char* grid_id, KfcRuntimeGridSpec* spec);
bool WriteVoxels(const std::int32_t origin[3], const std::uint32_t dimensions[3],
                 const std::uint16_t* values, std::size_t count, std::uint32_t* outcome);
}
