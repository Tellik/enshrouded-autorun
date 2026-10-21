#pragma once
#include "engine-context/world_context_state.h"
#include "voxels/voxel_operations.h"
namespace WorldRuntime::Implementation {
constexpr std::size_t MaximumCells = 65'536;

struct Cell { std::uint8_t material{}, density{}; };

static_assert(sizeof(Cell) == 2);

struct CellSpan { Cell* data{}; std::uint64_t size{}; };

// Expand partial requests to complete 8x8x8 chunks while preserving every
// surrounding cell through the existing writer's read/modify/write path.
struct WriteRegion {
    std::array<std::int32_t,3> origin{};
    std::array<std::uint32_t,3> dimensions{};
    std::array<std::size_t,3> offset{};
    std::size_t count{};
};
bool plan_write_region(const std::int32_t* origin, const std::uint32_t* dimensions, WriteRegion& output);

using NativeRead = bool (__fastcall*)(CellSpan*, const std::uint32_t*, const void*, std::uint32_t, const std::int32_t*);

struct Operation {
    enum class Kind { Read, Write } kind{};
    std::array<std::int32_t, 3> origin{};
    std::array<std::uint32_t, 3> dimensions{};
    std::vector<Cell> cells;
    bool result{};
    bool write_attempted{};
    bool rollback_verified{};
    std::uint64_t context_generation{};
};

bool safe_read(NativeRead function, CellSpan* span, const std::uint32_t* dimensions,
               const void* world, std::uint32_t mode, const std::int32_t* origin);

void execute(void* opaque);

bool cell_count(const std::uint32_t dimensions[3], std::size_t& count);

}
