#pragma once
#include "engine-context/world_context_state.h"
#include "voxels/voxel_operations.h"
namespace WorldRuntime::Implementation {
constexpr std::size_t MaximumCells = 65'536;

struct Cell { std::uint8_t material{}, density{}; };

static_assert(sizeof(Cell) == 2);

struct CellSpan { Cell* data{}; std::uint64_t size{}; };

using NativeRead = bool (__fastcall*)(CellSpan*, const std::uint32_t*, const void*, std::uint32_t, const std::int32_t*);

using NativeWrite = void (__fastcall*)(void*, CellSpan*, const std::uint32_t*, const std::int32_t*);

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

bool safe_write(NativeWrite function, void* world, CellSpan* span,
                const std::uint32_t* dimensions, const std::int32_t* origin);

void execute(void* opaque);

bool cell_count(const std::uint32_t dimensions[3], std::size_t& count);

}
