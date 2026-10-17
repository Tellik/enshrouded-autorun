#include "ecs/ecs_state.h"



using namespace EcsImplementation;

namespace EcsImplementation {
bool component_presence(const EntityView& entity, const ResolvedLayout& layout,
                        const ComponentType& component, bool& present){
    std::uint64_t bits{};
    if (component.index >= max_components ||
        !read(entity.layout + layout.component_bits + (component.index / 64) * 8, bits)) return false;
    present = (bits & (std::uint64_t{1} << (component.index % 64))) != 0;
    return true;
}

bool component_present(const EntityView& entity, const ResolvedLayout& layout, const ComponentType& component){
    bool present{};
    return component_presence(entity, layout, component, present) && present;
}

bool component_address(const EntityView& entity, const ResolvedLayout& layout,
                       const ComponentType& component, std::uintptr_t& address, bool check_page,
                       ReadRegionCache* regions){
    std::uint16_t offset{}, stride{};
    if (!component_present(entity, layout, component) ||
        !read(entity.layout + layout.component_offsets + component.index * 2, offset) ||
        !read(entity.layout + layout.component_strides + component.index * 2, stride) || stride != component.size) return false;
    const auto delta = offset + static_cast<std::uintptr_t>(entity.row) * stride;
    if (entity.storage > UINTPTR_MAX - delta) return false;
    address = entity.storage + delta;
    // Read-only scan callers immediately copy through guarded_copy (SEH).
    // Re-querying the page for every component doubles kernel transitions for
    // every prop in the world; writes and address-only queries keep the check.
    return !check_page || readable(address, component.size, regions);
}

bool resolve_component(const char* name, ComponentType& component){
    if (!name) return false;
    std::scoped_lock lock(state_mutex);
    const auto found = types.find(name);
    if (found == types.end()) return false;
    component = found->second;
    return true;
}

void read_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ReadOperation*>(opaque);
    ComponentType component{};
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    if (!(operation.value && resolve_component(operation.name, component) &&
        component.size == operation.size && layout_snapshot(layout) &&
        entity_for_handle(operation.handle, layout, entity) &&
        component_address(entity, layout, component, address) &&
        read_bytes(address, operation.value, operation.size))) return;
    // A non-dispatched read may race an archetype move or an engine update.
    // Publish only equal payload copies belonging to the same current entity.
    std::vector<std::uint8_t> again(operation.size);
    EntityView current{};
    std::uintptr_t current_address{};
    operation.result = read_bytes(address, again.data(), again.size()) &&
        std::memcmp(operation.value, again.data(), operation.size) == 0 &&
        entity_for_handle(operation.handle, layout, current) && current == entity &&
        component_address(current, layout, component, current_address) && current_address == address;
}

void read_many_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ReadManyOperation*>(opaque);
    operation.regions = {};
    ResolvedLayout layout{};
    EntityView entity{};
    if (!layout_snapshot(layout) || !entity_for_handle(operation.handle, layout, entity)) return;
    for (std::size_t index = 0; index < operation.names.size(); ++index) {
        auto& component = operation.components[index];
        if (!resolve_component(operation.names[index].c_str(), component) ||
            component.size != operation.sizes[index]) return;
    }
    std::size_t offset{};
    for (std::size_t index = 0; index < operation.names.size(); ++index) {
        const auto& component = operation.components[index];
        const auto size = operation.sizes[index];
        bool present{};
        // An unreadable presence mask cannot be reported as a missing
        // component: that might conceal another initialized attribute root.
        if (!component_presence(entity, layout, component, present)) return;
        if (present) {
            auto& address = operation.addresses[index];
            auto* output = operation.output.data() + offset;
            EntityView current{};
            std::uintptr_t current_address{};
            if (!component_address(entity, layout, component, address, true, &operation.regions) ||
                !read_bytes(address, output, size) ||
                !read_bytes(address, operation.verification.data(), size) ||
                std::memcmp(output, operation.verification.data(), size) != 0 ||
                !entity_for_handle(operation.handle, layout, current) || current != entity ||
                !component_address(current, layout, component, current_address, true, &operation.regions) || current_address != address) return;
            operation.present[index] = 1;
        }
        offset += size;
    }
    EntityView current{};
    if (!entity_for_handle(operation.handle, layout, current) || current != entity) return;
    // Preserve absence and address identity for every candidate through the
    // end of this owner operation. Individual equal copies do not establish
    // simultaneous values across different engine components.
    for (std::size_t index = 0; index < operation.names.size(); ++index) {
        bool present{};
        if (!component_presence(current, layout, operation.components[index], present) ||
            present != (operation.present[index] == 1)) return;
        if (present) {
            std::uintptr_t address{};
            if (!component_address(current, layout, operation.components[index], address, true, &operation.regions) ||
                address != operation.addresses[index]) return;
        }
    }
    operation.result = operation.regions.revalidate();
}

// Called with operation_mutex held, before any original engine gate. All maps,
// buffers and host mutexes are prepared here, outside the short memory scope.
bool prepare_write(WriteOperation& operation){
    if (stop_requested.load(std::memory_order_acquire) || !operation.mask || !operation.value ||
        operation.before.size() != operation.size || operation.merged.size() != operation.size ||
        operation.verified.size() != operation.size || !resolve_component(operation.name, operation.component) ||
        operation.component.size != operation.size || !layout_snapshot(operation.layout)) return false;
    std::scoped_lock lock(state_mutex);
    if (write_fault_active_locked()) {
        operation.blocked_by_write_fault = true;
        return false;
    }
    const auto found = handles.find(operation.handle);
    if (found == handles.end() || found->second.epoch != layout_epoch) return false;
    operation.record = found->second;
    operation.epoch = layout_epoch;
    operation.direct = read_only_binding;
    if (operation.direct) {
        if (!component_registry_ready || !component_registry_world_bound ||
            component_registry.owner != read_only_view.registry_owner || !operation.record.table_slot) return false;
        operation.owner = read_only_view;
    }
    return true;
}

// No allocation, mutex acquisition or task dispatch in this body. The guarded
// path records changes through the validated native tracker before mutation.
// Direct callers own both original gates. Legacy callers own a dispatcher phase.
void write_payload(WriteOperation& operation, const EngineContext::EcsWriteGuard* guard){
    if (stop_requested.load(std::memory_order_acquire) || (guard && !guard->Validate())) return;
    const auto& layout = operation.layout;
    const auto& record = operation.record;
    if (operation.direct) {
        std::uint64_t count{};
        std::uintptr_t table{}, pointer{};
        if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
            !read(layout.table_address, table) || table != operation.owner.entity_table ||
            table > UINTPTR_MAX - count * sizeof(pointer) || record.table_slot < table ||
            record.table_slot >= table + count * sizeof(pointer) || (record.table_slot - table) % sizeof(pointer) ||
            !read(record.table_slot, pointer) || pointer != record.pointer) return;
    }
    EntityView entity{};
    std::uintptr_t address{};
    if (!entity_view(record.pointer, layout, entity) || entity.id != record.id || entity.generation != record.generation ||
        !component_address(entity, layout, operation.component, address, false) ||
        !read_bytes(address, operation.before.data(), operation.size)) return;
    if (!operation.expected.empty() && operation.before != operation.expected) {
        operation.conflict = true;
        return;
    }
    std::copy(operation.before.begin(), operation.before.end(), operation.merged.begin());
    const auto* mask = static_cast<const std::uint8_t*>(operation.mask);
    const auto* value = static_cast<const std::uint8_t*>(operation.value);
    for (std::size_t index = 0; index < operation.size; ++index)
        if (mask[index]) operation.merged[index] = value[index];
    if (guard && !guard->ComponentSupported(operation.component.index)) return;
    if (operation.merged == operation.before) { operation.result = true; return; }
    if (guard && !guard->TrackChange(entity.pointer, operation.component.index)) return;
    operation.mutation_attempted = true;
    if (!guarded_copy(reinterpret_cast<void*>(address), operation.merged.data(), operation.size) ||
        !read_bytes(address, operation.verified.data(), operation.size) || operation.verified != operation.merged) {
        guarded_copy(reinterpret_cast<void*>(address), operation.before.data(), operation.size);
        // A failed copy can leave a partial mutation. Only a complete equal
        // readback establishes rollback, even when the restore copy faulted on
        // a read-only page that already contains the original payload.
        operation.uncertain = !read_bytes(address, operation.verified.data(), operation.size) ||
            operation.verified != operation.before;
        return;
    }
    operation.result = true;
}

void write_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<WriteOperation*>(opaque);
    if (!prepare_write(operation) || operation.direct) return;
    std::scoped_lock transaction(write_mutex);
    write_payload(operation);
    if (operation.uncertain) {
        std::scoped_lock lock(state_mutex);
        if (layout_epoch == operation.epoch) {
            write_fault_epoch = layout_epoch;
            write_fault_reason = "ecs-write-outcome-uncertain-rollback-unverified";
        }
    }
}

bool invoke_write_operation(const std::shared_ptr<WriteOperation>& operation){
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (!direct) {
        const bool completed = GameThreadDispatcher::Invoke(write_on_game_thread, operation);
        if (completed && operation->uncertain)
            last_write_reason.store("ecs-write-outcome-uncertain-rollback-unverified", std::memory_order_relaxed);
        return completed;
    }
    std::scoped_lock operation_lock(operation_mutex);
    if (!prepare_write(*operation) || !operation->direct) {
        last_write_reason.store(operation->blocked_by_write_fault ?
            "ecs-writes-disabled-after-uncertain-rollback" : "ecs-write-handle-or-layout-invalid", std::memory_order_relaxed);
        return false;
    }
    std::scoped_lock transaction(write_mutex);
    EcsRuntime::SessionGuard session;
    if (!session.TryAcquire(operation->epoch + 1) || session.View() != operation->owner || !session.WritesAllowed()) {
        last_write_reason.store("ecs-write-session-changed-or-disabled", std::memory_order_relaxed);
        return false;
    }
    EngineContext::EcsWriteGuard guard;
    const char* reason{};
    if (!guard.TryAcquire(operation->owner, &reason)) {
        last_write_reason.store(reason, std::memory_order_relaxed);
        return false;
    }
    if (operation->size > guard.MaximumBytes()) {
        guard.Release();
        last_write_reason.store("ecs-write-size-exceeds-gate-budget", std::memory_order_relaxed);
        return false;
    }
    write_payload(*operation, &guard);
    if (!guard.Validate()) {
        operation->result = false;
        operation->uncertain = operation->uncertain || operation->mutation_attempted;
    }
    if (operation->uncertain) {
        session.MarkWriteUncertain("ecs-write-outcome-uncertain-rollback-unverified");
    }
    guard.Release();
    last_write_reason.store(operation->uncertain ? "ecs-write-outcome-uncertain-rollback-unverified" :
        operation->result ? "success" : operation->conflict ? "compare-conflict" :
        "ecs-write-identity-or-memory-check-failed", std::memory_order_relaxed);
    return true;
}
}

namespace EcsRuntime {
bool MatchesComponentAddress(std::uint32_t handle, const char* name, const void* expected) {
    ComponentType component{};
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    return resolve_component(name, component) && layout_snapshot(layout) &&
        entity_for_handle(handle, layout, entity) &&
        component_address(entity, layout, component, address) &&
        address == reinterpret_cast<std::uintptr_t>(expected);
}
}

extern "C" bool __cdecl KfcRuntimeEcsCanWrite() {
    std::scoped_lock lock(state_mutex);
    if (!layout_ready || types.empty() || write_fault_active_locked()) return false;
    if (!read_only_binding) return GameThreadDispatcher::Ready() &&
        (!component_registry_ready || component_registry_world_bound);
    EngineContext::EcsView current;
    return component_registry_ready && component_registry_world_bound &&
        read_only_view.role == EngineContext::EcsRole::LocalAuthority && EngineContext::EcsWriteConfigured() &&
        EngineContext::ResolveEcs(current) && current == read_only_view;
}

extern "C" bool __cdecl KfcRuntimeEcsCanWriteComponent(const char* name) {
    std::scoped_lock operation_lock(operation_mutex);
    ComponentType component{};
    if (!KfcRuntimeEcsCanWrite() || !resolve_component(name, component)) return false;
    EngineContext::EcsView owner;
    {
        std::scoped_lock lock(state_mutex);
        if (!read_only_binding) return true;
        owner = read_only_view;
    }
    return EngineContext::EcsComponentWriteSupported(owner, component.index);
}

extern "C" bool __cdecl KfcRuntimeEcsRead(std::uint32_t handle, const char* name,
                                           void* value, std::size_t size) {
    operation_counters.reads.fetch_add(1, std::memory_order_relaxed);
    if (!name || !value || !size || size > (1u << 20) || !KfcRuntimeEcsReady()) {
        operation_counters.read_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    auto operation = std::make_shared<ReadOperation>();
    operation->handle = handle;
    operation->owned_name = name;
    operation->name = operation->owned_name.c_str();
    operation->output.resize(size);
    operation->value = operation->output.data();
    operation->size = size;
    if (!invoke_read_operation(read_on_game_thread, operation) || !operation->result) {
        operation_counters.read_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    std::memcpy(value, operation->output.data(), size);
    operation_counters.read_successes.fetch_add(1, std::memory_order_relaxed);
    return true;
}

extern "C" bool __cdecl KfcRuntimeEcsReadMany(std::uint32_t handle, const char* const* names,
    const std::uint32_t* sizes, std::size_t count, std::uint8_t* buffer,
    std::size_t capacity, std::uint8_t* present) {
    operation_counters.read_batches.fetch_add(1, std::memory_order_relaxed);
    const auto fail = [] {
        operation_counters.read_batch_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    };
    try {
        if (!handle || !names || !sizes || !buffer || !present || !count ||
            count > EcsRuntime::MaximumReadManyComponents || !capacity ||
            capacity > EcsRuntime::MaximumReadManyBytes || !KfcRuntimeEcsReady()) return fail();
        auto operation = std::make_shared<ReadManyOperation>();
        operation->handle = handle;
        operation->names.reserve(count);
        operation->sizes.assign(sizes, sizes + count);
        std::size_t total{}, maximum_size{};
        for (std::size_t index = 0; index < count; ++index) {
            const auto size = operation->sizes[index];
            if (!names[index] || !size || size > EcsRuntime::MaximumReadManyBytes - total) return fail();
            const auto length = strnlen_s(names[index], 512);
            if (!length || length == 512) return fail();
            operation->names.emplace_back(names[index], length);
            total += size;
            maximum_size = (std::max)(maximum_size, static_cast<std::size_t>(size));
        }
        if (total != capacity) return fail();
        // No caller-owned pointers enter the dispatcher. A timed-out queued
        // operation can finish against its own names and buffers safely.
        operation->components.resize(count);
        operation->addresses.resize(count);
        operation->output.resize(total);
        operation->present.resize(count);
        operation->verification.resize(maximum_size);
        if (!invoke_read_operation(read_many_on_game_thread, operation) || !operation->result) return fail();
        std::memcpy(buffer, operation->output.data(), total);
        std::memcpy(present, operation->present.data(), count);
        const auto copied = std::count(operation->present.begin(), operation->present.end(), std::uint8_t{1});
        operation_counters.read_batch_successes.fetch_add(1, std::memory_order_relaxed);
        operation_counters.read_batch_components.fetch_add(static_cast<std::uint64_t>(copied), std::memory_order_relaxed);
        operation_counters.read_batch_absent.fetch_add(count - copied, std::memory_order_relaxed);
        return true;
    } catch (...) { return fail(); }
}

extern "C" bool __cdecl KfcRuntimeEcsWrite(std::uint32_t handle, const char* name,
                                            const void* mask, const void* value,
                                            std::size_t size) {
    operation_counters.writes.fetch_add(1, std::memory_order_relaxed);
    if (!name || !mask || !value || !size || size > (1u << 20) || !KfcRuntimeEcsCanWrite()) {
        operation_counters.write_failures.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    auto operation = std::make_shared<WriteOperation>();
    operation->handle = handle;
    operation->owned_name = name;
    operation->name = operation->owned_name.c_str();
    const auto first = static_cast<const std::uint8_t*>(mask);
    const auto last = static_cast<const std::uint8_t*>(value);
    operation->mask_bytes.assign(first, first + size);
    operation->destination.assign(last, last + size);
    operation->mask = operation->mask_bytes.data();
    operation->value = operation->destination.data();
    operation->size = size;
    operation->before.resize(size);
    operation->merged.resize(size);
    operation->verified.resize(size);
    const bool written = invoke_write_operation(operation) && operation->result;
    if (written) operation_counters.write_successes.fetch_add(1, std::memory_order_relaxed);
    else operation_counters.write_failures.fetch_add(1, std::memory_order_relaxed);
    return written;
}

extern "C" std::uint32_t __cdecl KfcRuntimeEcsCompareExchange(std::uint32_t handle, const char* name,
    const void* expected, const void* mask, const void* value, std::size_t size) {
    if (!name || !expected || !mask || !value || !size || size > (1u << 20) || !KfcRuntimeEcsCanWrite()) return 0;
    auto operation = std::make_shared<WriteOperation>();
    operation->handle = handle;
    operation->owned_name = name;
    operation->name = operation->owned_name.c_str();
    const auto first = static_cast<const std::uint8_t*>(mask);
    const auto last = static_cast<const std::uint8_t*>(value);
    const auto before = static_cast<const std::uint8_t*>(expected);
    operation->mask_bytes.assign(first, first + size);
    operation->destination.assign(last, last + size);
    operation->expected.assign(before, before + size);
    operation->mask = operation->mask_bytes.data();
    operation->value = operation->destination.data();
    operation->size = size;
    operation->before.resize(size);
    operation->merged.resize(size);
    operation->verified.resize(size);
    if (!invoke_write_operation(operation)) return 0;
    return operation->uncertain ? 3 : operation->result ? 1 : operation->conflict ? 2 : 0;
}
