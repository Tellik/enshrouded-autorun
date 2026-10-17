#include "engine-context/session_internal.h"



using namespace WorldRuntime::Implementation;

namespace WorldRuntime::Implementation {
bool safe_read(NativeRead function, CellSpan* span, const std::uint32_t* dimensions,
               const void* world, std::uint32_t mode, const std::int32_t* origin){
    __try { return function(span, dimensions, world, mode, origin); }
    __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool safe_write(NativeWrite function, void* world, CellSpan* span,
                const std::uint32_t* dimensions, const std::int32_t* origin){
    __try { function(world, span, dimensions, origin); return true; }
    __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}

void execute(void* opaque){
    auto& op = *static_cast<Operation*>(opaque);
    std::uintptr_t world{};
    const bool direct_context = resolve_voxel_world(world);
    if (!direct_context && (op.kind != Operation::Kind::Read || !resolve_client_read_world(world))) return;
    if (op.kind == Operation::Kind::Read && op.context_generation != context_resets.load(std::memory_order_acquire)) return;
    const auto* read_profile = find_operation("runtime.world.voxel.read");
    const auto* write_profile = find_operation("runtime.world.voxel.write");
    const auto base = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    if (!read_profile || !read_profile->available) return;
    auto read_fn = reinterpret_cast<NativeRead>(base + read_profile->function_rva);
    CellSpan span{op.cells.data(), op.cells.size()};
    if (op.kind == Operation::Kind::Read) {
        op.result = safe_read(read_fn, &span, op.dimensions.data(), reinterpret_cast<void*>(world), read_profile->mode, op.origin.data());
        if (op.result && direct_context) {
            std::uintptr_t unset{};
            preferred_voxel_world.compare_exchange_strong(unset, world, std::memory_order_acq_rel);
        }
        return;
    }
    if (!write_profile || !write_profile->available) return;
    auto write_fn = reinterpret_cast<NativeWrite>(base + write_profile->function_rva);
    std::array<std::int32_t, 3> expanded_origin{};
    std::array<std::uint32_t, 3> expanded_dimensions{};
    std::size_t expanded_count = 1;
    std::array<std::size_t, 3> offset{};
    for (int axis = 0; axis < 3; ++axis) {
        const auto low = static_cast<std::int64_t>(op.origin[axis]);
        const auto high = low + op.dimensions[axis];
        auto chunk_low = low / 8;
        if (low < 0 && low % 8) --chunk_low;
        auto chunk_high = high / 8;
        if (high > 0 && high % 8) ++chunk_high;
        const auto aligned_low = chunk_low * 8;
        const auto aligned_high = chunk_high * 8;
        if (aligned_low < INT32_MIN || aligned_low > INT32_MAX || aligned_high <= aligned_low ||
            static_cast<std::uint64_t>(aligned_high - aligned_low) > MaximumCells / expanded_count) return;
        expanded_origin[axis] = static_cast<std::int32_t>(aligned_low);
        expanded_dimensions[axis] = static_cast<std::uint32_t>(aligned_high - aligned_low);
        offset[axis] = static_cast<std::size_t>(low - aligned_low);
        expanded_count *= expanded_dimensions[axis];
    }
    std::vector<Cell> expanded(expanded_count);
    CellSpan expanded_span{expanded.data(), expanded.size()};
    if (!safe_read(read_fn, &expanded_span, expanded_dimensions.data(), reinterpret_cast<void*>(world), read_profile->mode, expanded_origin.data())) return;
    auto original = expanded;
    for (std::size_t z = 0; z < op.dimensions[2]; ++z)
        for (std::size_t y = 0; y < op.dimensions[1]; ++y)
            for (std::size_t x = 0; x < op.dimensions[0]; ++x) {
                const auto source = (z * op.dimensions[1] + y) * op.dimensions[0] + x;
                const auto target = ((z + offset[2]) * expanded_dimensions[1] + y + offset[1]) * expanded_dimensions[0] + x + offset[0];
                expanded[target] = op.cells[source];
            }
    const auto restore_original = [&]() {
        CellSpan restore_span{original.data(), original.size()};
        if (!safe_write(write_fn, reinterpret_cast<void*>(world), &restore_span, expanded_dimensions.data(), expanded_origin.data())) return;
        std::vector<Cell> restored(expanded_count);
        CellSpan restored_span{restored.data(), restored.size()};
        op.rollback_verified = safe_read(read_fn, &restored_span, expanded_dimensions.data(), reinterpret_cast<void*>(world), read_profile->mode, expanded_origin.data()) &&
            std::memcmp(restored.data(), original.data(), original.size() * sizeof(Cell)) == 0;
    };
    op.result = false;
    op.write_attempted = true;
    if (!safe_write(write_fn, reinterpret_cast<void*>(world), &expanded_span, expanded_dimensions.data(), expanded_origin.data())) {
        restore_original();
        op.result = false;
        return;
    }
    std::vector<Cell> verify(expanded_count);
    CellSpan verify_span{verify.data(), verify.size()};
    if (!safe_read(read_fn, &verify_span, expanded_dimensions.data(), reinterpret_cast<void*>(world), read_profile->mode, expanded_origin.data()) ||
        std::memcmp(verify.data(), expanded.data(), expanded.size() * sizeof(Cell)) != 0) {
        restore_original();
        op.result = false;
        return;
    }
    op.result = true;
    std::uintptr_t unset{};
    preferred_voxel_world.compare_exchange_strong(unset, world, std::memory_order_acq_rel);
}

bool cell_count(const std::uint32_t dimensions[3], std::size_t& count){
    if (!dimensions) return false;
    count = 1;
    for (int axis = 0; axis < 3; ++axis) {
        if (!dimensions[axis] || dimensions[axis] > MaximumCells / count) return false;
        count *= dimensions[axis];
    }
    return count <= MaximumCells;
}
}

namespace WorldRuntime {
bool ReadVoxels(const std::int32_t origin[3], const std::uint32_t dimensions[3],
                std::uint16_t* values, std::size_t capacity, std::size_t* actual) {
    std::size_t count{};
    if (!origin || !values || !actual || !cell_count(dimensions, count) || count > capacity ||
        !OperationAvailable("runtime.world.voxel.read") || !GameThreadDispatcher::Ready()) return false;
    auto op = std::make_shared<Operation>();
    op->kind = Operation::Kind::Read;
    op->context_generation = context_resets.load(std::memory_order_acquire);
    std::copy_n(origin, 3, op->origin.begin());
    std::copy_n(dimensions, 3, op->dimensions.begin());
    op->cells.resize(count);
    if (!GameThreadDispatcher::Invoke(execute, op, 500) || !op->result) return false;
    for (std::size_t i = 0; i < count; ++i)
        values[i] = static_cast<std::uint16_t>(op->cells[i].material | (static_cast<std::uint16_t>(op->cells[i].density) << 8));
    *actual = count;
    return true;
}

bool WriteVoxels(const std::int32_t origin[3], const std::uint32_t dimensions[3],
                 const std::uint16_t* values, std::size_t count, std::uint32_t* outcome) {
    if (!outcome) return false;
    *outcome = 1; // rejected before writing
    std::size_t expected{};
    if (!origin || !values || !cell_count(dimensions, expected) || count != expected ||
        !OperationAvailable("runtime.world.voxel.read") ||
        !OperationAvailable("runtime.world.voxel.write") || !GameThreadDispatcher::Ready()) return false;
    auto op = std::make_shared<Operation>();
    op->kind = Operation::Kind::Write;
    std::copy_n(origin, 3, op->origin.begin());
    std::copy_n(dimensions, 3, op->dimensions.begin());
    op->cells.resize(count);
    for (std::size_t i = 0; i < count; ++i) {
        op->cells[i].material = static_cast<std::uint8_t>(values[i]);
        op->cells[i].density = static_cast<std::uint8_t>(values[i] >> 8);
    }
    if (!GameThreadDispatcher::Invoke(execute, op, 500)) {
        *outcome = 3; // the game-thread request may have started before timeout
        return false;
    }
    if (op->result) { *outcome = 0; return true; }
    *outcome = op->write_attempted ? (op->rollback_verified ? 2u : 3u) : 1u;
    return false;
}
}

extern "C" {
bool __cdecl KfcRuntimeWorldVoxelRead(const std::int32_t* origin, const std::uint32_t* dimensions,
    std::uint16_t* values, std::size_t capacity, std::size_t* actual) {
    return WorldRuntime::ReadVoxels(origin, dimensions, values, capacity, actual);
}

bool __cdecl KfcRuntimeWorldVoxelWrite(const std::int32_t* origin, const std::uint32_t* dimensions,
    const std::uint16_t* values, std::size_t count, std::uint32_t* outcome) {
    return WorldRuntime::WriteVoxels(origin, dimensions, values, count, outcome);
}
}
