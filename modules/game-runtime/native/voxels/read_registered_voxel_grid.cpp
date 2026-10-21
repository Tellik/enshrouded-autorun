#include "voxels/read_registered_voxel_grid.h"
#include "voxels/voxel_request.h"
#include "named-services/named_service_contract.h"
#include <limits>

namespace WorldRuntime::RegisteredVoxelGrid {
namespace {
using NamedServiceImplementation::add;
using NamedServiceImplementation::copy;
using NamedServiceImplementation::field;
using NamedServiceImplementation::BindingContract;
constexpr std::uint32_t voxel_mode = 6;
constexpr double cell_size = 0.5;
constexpr std::uint16_t buffer_guard = 0xa56c;
constexpr std::size_t guard_cells = 8;
constexpr auto maximum_coordinate = (std::numeric_limits<std::int32_t>::max)() - 7;
std::atomic<bool> reader_fault{};

// The six original backend slots belong to the named ecs::VoxelGrid service.
// They are interface roles, never image addresses or function fingerprints.
struct Interface {
    std::uintptr_t allocator{}, worker{};
    std::uint32_t block_shift{}, width{}, height{}, width_shift{}, height_shift{}, padding{};
    std::uintptr_t world_wrapper{};
    std::array<std::uintptr_t,6> callbacks{};
    bool operator==(const Interface&) const = default;
};
static_assert(sizeof(Interface) == 96 && offsetof(Interface, world_wrapper) == 40 &&
    offsetof(Interface, callbacks) == 48);

struct Level {
    std::uint32_t tile_x{}, tile_z{}, count_x{}, count_z{};
    std::uintptr_t allocator{}, handles{};
    std::uint64_t handle_count{};
    bool operator==(const Level&) const = default;
};
static_assert(sizeof(Level) == 40);

struct TaskPool {
    std::uintptr_t groups{};
    std::uint64_t count{}, shift{}, mask{};
    bool operator==(const TaskPool&) const = default;
};
struct Task {
    std::uintptr_t slot{}, address{};
    std::uint32_t id{}, generation{}, state{};
    bool operator==(const Task&) const = default;
};
struct Snapshot {
    std::uintptr_t grid{}, wrapper{}, world{}, resources{}, storage{};
    Interface backend;
    Level level;
    TaskPool pool;
    std::array<float,3> origin{};
    std::uint32_t initialized{}, global_task{};
    std::uint8_t storage_unavailable{};
    bool operator==(const Snapshot&) const = default;
};
struct Request {
    std::uint16_t* data{};
    std::uint64_t count{};
    std::array<std::uint32_t,3> dimensions{};
    std::uint32_t mode{voxel_mode};
    std::array<std::int32_t,3> origin{};
    std::uint32_t padding{};
    bool operator==(const Request&) const = default;
};
static_assert(sizeof(Request) == 48 && offsetof(Request, dimensions) == 16 &&
    offsetof(Request, mode) == 28 && offsetof(Request, origin) == 32);
struct alignas(16) Point {
    std::array<float,4> values{};
    bool operator==(const Point&) const = default;
};

bool acquire_world_reader(std::uintptr_t address) {
    if (!address || address % alignof(LONG)) return false;
    __try {
        auto* lock = reinterpret_cast<volatile LONG*>(address);
        const auto current = static_cast<std::uint32_t>(*lock);
        return current < 0xffff &&
            static_cast<std::uint32_t>(InterlockedCompareExchange(lock,
                static_cast<LONG>(current + 1), static_cast<LONG>(current))) == current;
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        reader_fault.store(true, std::memory_order_release);
        return false;
    }
}
void release_world_reader(std::uintptr_t address) noexcept {
    __try { InterlockedDecrement(reinterpret_cast<volatile LONG*>(address)); }
    __except (EXCEPTION_EXECUTE_HANDLER) { reader_fault.store(true, std::memory_order_release); }
}
struct WorldReader {
    std::uintptr_t address{};
    explicit WorldReader(std::uintptr_t world) {
        std::uintptr_t candidate{};
        if (add(world, 0x174, candidate) && acquire_world_reader(candidate)) address = candidate;
    }
    ~WorldReader() { if (address) release_world_reader(address); }
};

bool read_snapshot(const BindingContract& grid_binding, const BindingContract& world_binding,
    const EngineContext::EcsView& owner, Snapshot& snapshot) {
    if (!Healthy()) return false;
    Snapshot value;
    if (!NamedServiceImplementation::resolve_binding(grid_binding, owner, value.grid) ||
        !NamedServiceImplementation::resolve_binding(world_binding, owner, value.wrapper) ||
        !field(value.grid, 0, value.backend) || value.backend.world_wrapper != value.wrapper ||
        !field(value.wrapper, 8, value.world) || !value.world ||
        value.backend.width_shift >= 31 || value.backend.height_shift >= 31 ||
        value.backend.width != (std::uint32_t{1} << value.backend.width_shift) ||
        value.backend.height != (std::uint32_t{1} << value.backend.height_shift)) return false;
    for (const auto callback : value.backend.callbacks)
        if (!NamedServiceImplementation::executable_callback(callback)) return false;
    // This guard protects bounded metadata copies only. Release it before any
    // original callback. A queued writer sets upper bits and would otherwise
    // deadlock a nested original Shared acquisition against our own reader.
    WorldReader reader(value.world);
    if (!reader.address || !field(value.world, 0x14, value.initialized) || !value.initialized ||
        !field(value.world, 0x40 + voxel_mode * sizeof(Level), value.level) ||
        !field(value.world, 0x160, value.resources) || !value.resources ||
        !field(value.world, 0x1220, value.storage) || !value.storage ||
        !field(value.world, 0x1a34, value.global_task) ||
        !field(value.storage, 0x2c3, value.storage_unavailable) || value.storage_unavailable ||
        !field(value.world, 0x30, value.origin) ||
        !std::all_of(value.origin.begin(), value.origin.end(), [](float f) { return std::isfinite(f); }) ||
        !field(value.resources, 0x190, value.pool.groups) || !value.pool.groups ||
        !field(value.resources, 0x198, value.pool.count) || !value.pool.count ||
        !field(value.resources, 0x1b8, value.pool.shift) || value.pool.shift > 24 ||
        !field(value.resources, 0x1c8, value.pool.mask) ||
        value.pool.mask != ((std::uint64_t{1} << value.pool.shift) - 1)) return false;
    const auto& level = value.level;
    const auto x = std::uint64_t{level.tile_x} * level.count_x;
    const auto z = std::uint64_t{level.tile_z} * level.count_z;
    if (level.tile_x < 8 || level.tile_z < 8 || level.tile_x % 8 || level.tile_z % 8 ||
        !level.count_x || !level.count_z || !level.handles ||
        level.handle_count != std::uint64_t{level.count_x} * level.count_z ||
        x > maximum_coordinate || z > maximum_coordinate ||
        level.handle_count > (UINTPTR_MAX - level.handles) / sizeof(std::uint32_t)) return false;
    snapshot = value;
    return true;
}

bool task(const Snapshot& world, std::uint32_t id, std::uintptr_t slot, Task& result) {
    if (!id) return false;
    const auto index = std::uint64_t{id & 0xffffff};
    const auto group_index = index >> world.pool.shift;
    const auto element = index & world.pool.mask;
    std::uintptr_t group{}, rows{}, address{};
    std::uint64_t count{};
    Task value;
    value.id = id; value.slot = slot;
    if (group_index >= world.pool.count || group_index > UINTPTR_MAX / 64 ||
        !add(world.pool.groups, static_cast<std::size_t>(group_index * 64), group) ||
        !field(group, 8, rows) || !rows || !field(group, 16, count) || element >= count ||
        element > UINTPTR_MAX / 80 || !add(rows, static_cast<std::size_t>(element * 80), address) ||
        !field(address, 0, value.generation) || (value.generation & 0x7fffffff) != id >> 24 ||
        !field(address, 0x48, value.state) || value.state != 2) return false;
    value.address = address;
    result = value;
    return true;
}

bool region(const Snapshot& world, const Request& request) {
    const std::array<std::uint64_t,3> maximum{
        std::uint64_t{world.level.tile_x} * world.level.count_x,
        static_cast<std::uint64_t>(maximum_coordinate),
        std::uint64_t{world.level.tile_z} * world.level.count_z};
    for (std::size_t axis = 0; axis < 3; ++axis) {
        if (request.origin[axis] < 0 || !request.dimensions[axis]) return false;
        const auto high = std::uint64_t{static_cast<std::uint32_t>(request.origin[axis])} + request.dimensions[axis];
        if (high > maximum[axis]) return false;
    }
    return true;
}

bool tasks(const Snapshot& world, const Request& request, std::vector<Task>& prepared,
    std::size_t& used) {
    if (!Healthy()) return false;
    // The caller allocates a conservative bound for intersecting X/Z tiles
    // before entering the original gates. Valid tile dimensions are >= 8.
    WorldReader reader(world.world);
    if (!reader.address) return false;
    used = 0;
    if (world.global_task) {
        Task ignored;
        if (!task(world, world.global_task, 0, ignored)) return false;
    }
    const auto x0 = static_cast<std::uint32_t>(request.origin[0]) / world.level.tile_x;
    const auto z0 = static_cast<std::uint32_t>(request.origin[2]) / world.level.tile_z;
    const auto x1 = (static_cast<std::uint64_t>(request.origin[0]) + request.dimensions[0] - 1) / world.level.tile_x;
    const auto z1 = (static_cast<std::uint64_t>(request.origin[2]) + request.dimensions[2] - 1) / world.level.tile_z;
    for (auto z = std::uint64_t{z0}; z <= z1; ++z) {
        for (auto x = std::uint64_t{x0}; x <= x1; ++x) {
            if (used >= prepared.size()) return false;
            std::uintptr_t slot{};
            std::uint32_t id{};
            const auto index = z * world.level.count_x + x;
            if (!add(world.level.handles, static_cast<std::size_t>(index * sizeof(id)), slot) ||
                !field(slot, 0, id) || !task(world, id, slot, prepared[used])) return false;
            ++used;
        }
    }
    return true;
}

bool revalidate(const BindingContract& grid_binding, const BindingContract& world_binding,
    const EngineContext::EcsView& owner, const Snapshot& expected,
    const std::vector<Task>& expected_tasks, std::size_t used) {
    if (!Healthy()) return false;
    Snapshot again;
    if (!read_snapshot(grid_binding, world_binding, owner, again) || again != expected) return false;
    WorldReader reader(expected.world);
    if (!reader.address) return false;
    if (expected.global_task) {
        Task global;
        if (!task(expected, expected.global_task, 0, global)) return false;
    }
    for (std::size_t index = 0; index < used; ++index) {
        Task repeated;
        std::uint32_t id{};
        const auto& before = expected_tasks[index];
        if (!field(before.slot, 0, id) || id != before.id || !task(expected, id, before.slot, repeated) ||
            repeated != before) return false;
    }
    return true;
}

bool invoke_origin(std::uintptr_t callback, Point* output, std::uintptr_t wrapper, void*& returned) {
    using Callback = void*(__fastcall*)(Point*, const void*);
    __try { returned = reinterpret_cast<Callback>(callback)(output, reinterpret_cast<const void*>(wrapper)); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { reader_fault.store(true, std::memory_order_release); return false; }
}
bool invoke_ready(std::uintptr_t callback, std::uintptr_t wrapper, const Point* point, bool& ready) {
    using Callback = bool(__fastcall*)(const void*, const Point*);
    __try { ready = reinterpret_cast<Callback>(callback)(reinterpret_cast<const void*>(wrapper), point); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { reader_fault.store(true, std::memory_order_release); return false; }
}
bool invoke_read(std::uintptr_t callback, Request* request, std::uintptr_t wrapper, bool& read) {
    using Callback = bool(__fastcall*)(Request*, const void*);
    __try { read = reinterpret_cast<Callback>(callback)(request, reinterpret_cast<const void*>(wrapper)); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { reader_fault.store(true, std::memory_order_release); return false; }
}

struct Bindings {
    BindingContract grid{NamedServiceImplementation::binding_contract("ecs::VoxelGrid")};
    BindingContract world{NamedServiceImplementation::binding_contract("VoxelWorld")};
};
}

bool Healthy() noexcept { return !reader_fault.load(std::memory_order_acquire); }

Result ReadForSession(std::uint64_t expected_session, const std::int32_t origin[3],
    const std::uint32_t dimensions[3], std::uint16_t* output, std::size_t capacity, std::size_t* actual) {
    std::size_t count{};
    if (!expected_session || !origin || !output || !actual ||
        !Implementation::cell_count(dimensions, count) || count > capacity) return Result::Rejected;
    if (!Healthy()) return Result::Fault;
    if (KfcRuntimeWorldSessionId() != expected_session) return Result::StaleSession;
    // Resolve ecs::VoxelGrid and VoxelWorld from the live named-service registry,
    // then call their original registered interface roles. Read() owns its
    // Scene and task validation and publishes only after its original reader
    // has completed. Keep caller memory untouched until the session is checked
    // again and the complete staging buffer is ready.
    try {
        std::vector<std::uint16_t> staging(count);
        std::size_t read{};
        const auto result = Read(origin, dimensions, staging.data(), staging.size(), &read);
        if (result != Result::Success) {
            if (KfcRuntimeWorldSessionId() != expected_session) return Result::StaleSession;
            return Healthy() ? result : Result::Fault;
        }
        if (KfcRuntimeWorldSessionId() != expected_session) return Result::StaleSession;
        if (read != count) return Result::Rejected;
        EcsRuntime::SessionGuard publish;
        if (!publish.TryAcquire(expected_session)) return Result::StaleSession;
        if (!copy(output, staging.data(), count * sizeof(std::uint16_t)) ||
            !copy(actual, &count, sizeof(count))) return Result::Rejected;
        return Result::Success;
    } catch (...) { return Result::Unavailable; }
}

bool Available() {
    if (!Healthy()) return false;
    try {
        const Bindings bindings;
        const auto token = KfcRuntimeWorldSessionId();
        std::scoped_lock operation(NamedServiceImplementation::operation_mutex);
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(token)) return false;
        EngineContext::EcsWriteGuard scene;
        if (!scene.TryAcquire(session.View())) return false;
        Snapshot snapshot, again;
        return Healthy() && scene.Validate() &&
            read_snapshot(bindings.grid, bindings.world, session.View(), snapshot) &&
            read_snapshot(bindings.grid, bindings.world, session.View(), again) && snapshot == again && scene.Validate() && Healthy();
    } catch (...) { return false; }
}

Result GetSpec(const char* grid_id, KfcRuntimeGridSpec* output) {
    return GetSpecForSession(KfcRuntimeWorldSessionId(), grid_id, output);
}

Result GetSpecForSession(std::uint64_t expected_session, const char* grid_id, KfcRuntimeGridSpec* output) {
    if (!expected_session || !grid_id || !output || std::strcmp(grid_id, "voxel")) return Result::Rejected;
    if (!Healthy()) return Result::Fault;
    if (KfcRuntimeWorldSessionId() != expected_session) return Result::StaleSession;
    try {
        const Bindings bindings;
        const auto token = expected_session;
        KfcRuntimeGridSpec spec{};
        {
            std::scoped_lock operation(NamedServiceImplementation::operation_mutex);
            EcsRuntime::SessionGuard session;
            if (!session.TryAcquire(token)) return Result::Unavailable;
            EngineContext::EcsWriteGuard scene;
            if (!scene.TryAcquire(session.View())) return Result::Pending;
            Snapshot snapshot, again;
            if (!scene.Validate() || !read_snapshot(bindings.grid, bindings.world, session.View(), snapshot))
                return Healthy() ? Result::Pending : Result::Fault;
            Point origin;
            void* returned{};
            if (!invoke_origin(snapshot.backend.callbacks[1], &origin, snapshot.wrapper, returned))
                return Result::Fault;
            if (returned != &origin) { reader_fault.store(true, std::memory_order_release); return Result::Fault; }
            if (!Healthy() || !std::equal(snapshot.origin.begin(), snapshot.origin.end(), origin.values.begin()) ||
                !scene.Validate() || !read_snapshot(bindings.grid, bindings.world, session.View(), again) ||
                again != snapshot || !Healthy()) return Healthy() ? Result::Pending : Result::Fault;
            std::memcpy(spec.id, "voxel", sizeof("voxel"));
            for (std::size_t axis = 0; axis < 3; ++axis) {
                spec.origin[axis] = origin.values[axis];
                spec.cell_size[axis] = cell_size;
            }
            spec.maximum[0] = std::uint64_t{snapshot.level.tile_x} * snapshot.level.count_x;
            spec.maximum[1] = maximum_coordinate;
            spec.maximum[2] = std::uint64_t{snapshot.level.tile_z} * snapshot.level.count_z;
        }
        if (!Healthy()) return Result::Fault;
        EcsRuntime::SessionGuard publish;
        if (!publish.TryAcquire(token)) return Result::StaleSession;
        if (!Healthy()) return Result::Fault;
        if (!copy(output, &spec, sizeof(spec))) return Result::Rejected;
        return Result::Success;
    } catch (...) { return Result::Unavailable; }
}

Result Read(const std::int32_t origin[3], const std::uint32_t dimensions[3],
    std::uint16_t* output, std::size_t capacity, std::size_t* actual) {
    std::size_t count{};
    if (!origin || !output || !actual || !Implementation::cell_count(dimensions, count) || count > capacity)
        return Result::Rejected;
    if (!Healthy()) return Result::Fault;
    try {
        const Bindings bindings;
        const auto token = KfcRuntimeWorldSessionId();
        std::vector<std::uint16_t> cells(count + guard_cells * 2, buffer_guard);
        // For any origin, an interval of n cells intersects at most
        // ceil((n + 7) / 8) tiles when every validated tile is at least 8 wide.
        const auto maximum_tiles_x = (static_cast<std::size_t>(dimensions[0]) + 14) / 8;
        const auto maximum_tiles_z = (static_cast<std::size_t>(dimensions[2]) + 14) / 8;
        std::vector<Task> prepared((std::min)(count, maximum_tiles_x * maximum_tiles_z));
        Request request{cells.data() + guard_cells, count};
        std::copy_n(origin, 3, request.origin.begin());
        std::copy_n(dimensions, 3, request.dimensions.begin());
        const auto original_request = request;
        {
            std::scoped_lock operation(NamedServiceImplementation::operation_mutex);
            EcsRuntime::SessionGuard session;
            if (!session.TryAcquire(token)) return Result::Unavailable;
            EngineContext::EcsWriteGuard scene;
            if (!scene.TryAcquire(session.View())) return Result::Pending;
            Snapshot snapshot;
            std::size_t used{};
            if (!scene.Validate() || !read_snapshot(bindings.grid, bindings.world, session.View(), snapshot))
                return Healthy() ? Result::Pending : Result::Fault;
            if (!region(snapshot, request)) return Result::Rejected;
            if (!tasks(snapshot, request, prepared, used) ||
                !revalidate(bindings.grid, bindings.world, session.View(), snapshot, prepared, used) || !Healthy())
                return Healthy() ? Result::Pending : Result::Fault;
            for (std::size_t index = 0; index < used; ++index) {
                const auto flat = (prepared[index].slot - snapshot.level.handles) / sizeof(std::uint32_t);
                Point point;
                point.values = {static_cast<float>(snapshot.origin[0] +
                    (flat % snapshot.level.count_x * snapshot.level.tile_x + 0.5) * cell_size),
                    snapshot.origin[1], static_cast<float>(snapshot.origin[2] +
                    (flat / snapshot.level.count_x * snapshot.level.tile_z + 0.5) * cell_size), 0};
                const auto before = point;
                bool ready{};
                // The short metadata reader has ended. No World/Task reader
                // survives across this original nested-locking callback.
                if (!Healthy() || !invoke_ready(snapshot.backend.callbacks[2], snapshot.wrapper, &point, ready))
                    return Result::Fault;
                if (point != before) { reader_fault.store(true, std::memory_order_release); return Result::Fault; }
                if (!ready || !Healthy()) return Healthy() ? Result::Pending : Result::Fault;
            }
            if (!scene.Validate() || !revalidate(bindings.grid, bindings.world, session.View(), snapshot, prepared, used) || !Healthy())
                return Healthy() ? Result::Pending : Result::Fault;
            bool read{};
            if (!invoke_read(snapshot.backend.callbacks[0], &request, snapshot.wrapper, read)) return Result::Fault;
            if (request != original_request ||
                !std::all_of(cells.begin(), cells.begin() + guard_cells, [](auto value) { return value == buffer_guard; }) ||
                !std::all_of(cells.end() - guard_cells, cells.end(), [](auto value) { return value == buffer_guard; })) {
                reader_fault.store(true, std::memory_order_release);
                return Result::Fault;
            }
            if (!Healthy() || !read || !scene.Validate() ||
                !revalidate(bindings.grid, bindings.world, session.View(), snapshot, prepared, used) || !Healthy())
                return Healthy() ? Result::Pending : Result::Fault;
        }
        if (!Healthy()) return Result::Fault;
        EcsRuntime::SessionGuard publish;
        if (!publish.TryAcquire(token)) return Result::StaleSession;
        if (!Healthy()) return Result::Fault;
        if (!copy(output, cells.data() + guard_cells, count * sizeof(std::uint16_t)) ||
            !copy(actual, &count, sizeof(count))) return Result::Rejected;
        return Result::Success;
    } catch (...) { return Result::Unavailable; }
}
}
