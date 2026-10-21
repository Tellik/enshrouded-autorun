#include "ecs/ecs_session_state.h"

namespace EcsImplementation {
std::uint64_t identity_key(std::uint32_t id, std::uint32_t generation){
    return static_cast<std::uint64_t>(generation) << 32 | id;
}

std::uint32_t handle_for(const EntityView& entity, std::size_t table_index){
    std::scoped_lock lock(state_mutex);
    const auto table_slot = read_only_binding && table_index != SIZE_MAX ?
        read_only_view.entity_table + table_index * sizeof(std::uintptr_t) : std::uintptr_t{};
    const auto key = identity_key(entity.id, entity.generation);
    if (const auto found = reverse_handles.find(key); found != reverse_handles.end()) {
        auto& record = handles.at(found->second);
        if (record.pointer != entity.pointer) record.table_slot = 0;
        record.pointer = entity.pointer;
        if (table_slot) record.table_slot = table_slot;
        return found->second;
    }
    // Never recycle an opaque handle during this process, including world changes.
    if (!next_handle) return 0;
    const auto handle = next_handle++;
    handles.emplace(handle, HandleRecord{entity.id, entity.generation, layout_epoch, entity.pointer, table_slot});
    reverse_handles.emplace(key, handle);
    return handle;
}

bool SnapshotEntityIds(std::vector<std::uint32_t>& ids){
    ResolvedLayout layout{};
    std::vector<EntityPointer> pointers;
    if (!layout_snapshot(layout) || !entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    ids.clear();
    ids.reserve(pointers.size());
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (entity_view(pointer.pointer, layout, entity)) ids.push_back(entity.id);
    }
    return true;
}

bool entity_for_handle(std::uint32_t handle, const ResolvedLayout& layout, EntityView& entity){
    HandleRecord record{};
    bool direct{};
    std::uintptr_t bound_table{};
    {
        std::scoped_lock lock(state_mutex);
        const auto found = handles.find(handle);
        if (found == handles.end() || found->second.epoch != layout_epoch) return false;
        record = found->second;
        direct = read_only_binding;
        bound_table = read_only_view.entity_table;
    }
    if (direct) {
        std::uint64_t count{};
        std::uintptr_t table{}, pointer{};
        if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
            !read(layout.table_address, table) || table != bound_table) return false;
        const auto end = table + count * sizeof(std::uintptr_t);
        if (record.table_slot < table || record.table_slot >= end ||
            !read(record.table_slot, pointer) || pointer != record.pointer) {
            if (layout.occupancy_address && layout.keys_address) {
                // A removed or relocated cached handle must not trigger a
                // whole-world pointer scan on every component read. Reuse the
                // original bounded ID lookup and retain generation checks.
                EntityView current{};
                std::size_t slot{};
                if (entity_for_id(record.id, layout, current, slot) != EntityMembership::Present ||
                    current.pointer != record.pointer || current.generation != record.generation) return false;
                record.table_slot = table + slot * sizeof(pointer);
            } else {
                // Legacy layouts lack the reviewed original ID index.
                std::array<std::uintptr_t, 256> batch{};
                record.table_slot = 0;
                for (std::size_t offset = 0; offset < count && !record.table_slot; offset += batch.size()) {
                    const auto size = (std::min)(batch.size(), static_cast<std::size_t>(count) - offset);
                    if (!read_bytes(table + offset * sizeof(pointer), batch.data(), size * sizeof(pointer))) return false;
                    for (std::size_t index = 0; index < size; ++index) if (batch[index] == record.pointer) {
                        record.table_slot = table + (offset + index) * sizeof(pointer);
                        break;
                    }
                }
            }
            if (!record.table_slot || !read(record.table_slot, pointer) || pointer != record.pointer) return false;
            std::scoped_lock lock(state_mutex);
            const auto found = handles.find(handle);
            if (found == handles.end() || found->second.epoch != record.epoch || found->second.pointer != record.pointer) return false;
            found->second.table_slot = record.table_slot;
        }
    }
    // Re-read the live identity and layout; never cache component addresses.
    // A moved/deleted entity is rediscovered by the next query, not a full-world
    // scan for each individual component read and write.
    return entity_view(record.pointer, layout, entity) && entity.id == record.id &&
        entity.generation == record.generation;
}

void resolve_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ResolveOperation*>(opaque);
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    if (layout.occupancy_address && layout.keys_address) {
        // The reviewed original table already owns an ID index. Resolve with
        // its checked hash lookup rather than starting a world scan per ID.
        EntityView entity{};
        std::size_t slot{};
        if (entity_for_id(operation.entity_id, layout, entity, slot) == EntityMembership::Present)
            operation.result = handle_for(entity, slot);
        return;
    }
    const auto epoch = current_epoch();
    const auto now_ms = GetTickCount64();
    for (auto iterator = resolve_scans.begin(); iterator != resolve_scans.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = resolve_scans.erase(iterator);
        else ++iterator;
    }
    for (auto iterator = resolve_cache.begin(); iterator != resolve_cache.end();) {
        if (now_ms - iterator->second.completed_ms > 2000) iterator = resolve_cache.erase(iterator);
        else ++iterator;
    }
    if (auto cached = resolve_cache.find(operation.entity_id); cached != resolve_cache.end()) {
        if (cached->second.epoch == epoch && now_ms - cached->second.completed_ms <= 50) {
            operation.result = cached->second.handle;
            return;
        }
        resolve_cache.erase(cached);
    }
    auto scan = resolve_scans.find(operation.entity_id);
    if (scan == resolve_scans.end() || scan->second.epoch != epoch) {
        ResolveScanState fresh{};
        fresh.epoch = epoch;
        fresh.touched_ms = now_ms;
        if (!entity_pointers(layout, fresh.pointers)) return;
        scan = resolve_scans.insert_or_assign(operation.entity_id, std::move(fresh)).first;
    }
    auto& progress = scan->second;
    progress.touched_ms = now_ms;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(1);
    std::size_t visited = 0;
    for (; progress.cursor < progress.pointers.size() && visited < 256; ++progress.cursor, ++visited) {
        EntityView candidate{};
        if (entity_view(progress.pointers[progress.cursor].pointer, layout, candidate) &&
            candidate.id == operation.entity_id) {
            operation.result = handle_for(candidate, progress.pointers[progress.cursor].table_index);
            resolve_cache.insert_or_assign(operation.entity_id,
                ResolveCacheEntry{epoch, now_ms, operation.result});
            resolve_scans.erase(scan);
            return;
        }
        if (std::chrono::steady_clock::now() >= deadline) {
            ++progress.cursor;
            break;
        }
    }
    if (progress.cursor >= progress.pointers.size()) {
        resolve_cache.insert_or_assign(operation.entity_id, ResolveCacheEntry{epoch, now_ms, 0});
        resolve_scans.erase(scan);
    }
}

void template_uuid_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<TemplateUuidOperation*>(opaque);
    ResolvedLayout layout{};
    EntityView before{}, after{};
    std::array<std::uint8_t, 16> again{};
    if (!layout_snapshot(layout) || !layout.entity_definition_uuid_validated ||
        !entity_for_handle(operation.handle, layout, before) || !before.definition ||
        before.definition > UINTPTR_MAX - layout.entity_definition_uuid ||
        before.definition + layout.entity_definition_uuid > UINTPTR_MAX - operation.uuid.size()) return;
    const auto address = before.definition + layout.entity_definition_uuid;
    if (!read_bytes(address, operation.uuid.data(), operation.uuid.size()) ||
        std::all_of(operation.uuid.begin(), operation.uuid.end(), [](auto byte) { return byte == 0; }) ||
        !read_bytes(address, again.data(), again.size()) || again != operation.uuid ||
        !entity_for_handle(operation.handle, layout, after) || after != before) return;
    operation.result = true;
}
}

using namespace EcsImplementation;

namespace EcsRuntime {
bool EntityIdentity(std::uint32_t handle, std::uint32_t* entity_id) {
    if (!handle || !entity_id) return false;
    std::scoped_lock operation_lock(operation_mutex);
    bool direct{};
    EngineContext::EcsView expected, before, after;
    {
        std::scoped_lock lock(state_mutex);
        direct = read_only_binding;
        expected = read_only_view;
    }
    if (direct && (!EngineContext::ResolveEcs(before) || before != expected)) return false;
    ResolvedLayout layout{};
    EntityView entity{};
    if (!layout_snapshot(layout) || !entity_for_handle(handle, layout, entity)) return false;
    if (direct && (!EngineContext::ResolveEcs(after) || after != before)) return false;
    *entity_id = entity.id;
    return entity.id != 0;
}

bool SnapshotEntityIds(std::vector<std::uint32_t>& ids) {
    return EcsImplementation::SnapshotEntityIds(ids);
}
}

extern "C" std::uint32_t __cdecl KfcRuntimeEcsResolve(std::uint32_t entity_id) {
    operation_counters.resolves.fetch_add(1, std::memory_order_relaxed);
    if (!entity_id || !KfcRuntimeEcsReady()) {
        operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
        return 0;
    }
    auto operation = std::make_shared<ResolveOperation>();
    operation->entity_id = entity_id;
    if (!invoke_read_operation(resolve_on_game_thread, operation)) {
        operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
        return 0;
    }
    if (operation->result) operation_counters.resolve_successes.fetch_add(1, std::memory_order_relaxed);
    else operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
    return operation->result;
}

extern "C" bool __cdecl KfcRuntimeEcsTemplateUuid(std::uint32_t handle, std::uint8_t* uuid) {
    if (!handle || !uuid || !KfcRuntimeEcsReady()) return false;
    auto operation = std::make_shared<TemplateUuidOperation>();
    operation->handle = handle;
    if (!invoke_read_operation(template_uuid_on_game_thread, operation) || !operation->result) return false;
    std::memcpy(uuid, operation->uuid.data(), operation->uuid.size());
    return true;
}

extern "C" bool __cdecl KfcRuntimeEcsEntityIdentity(std::uint32_t handle, std::uint32_t* entity_id) {
    return EcsRuntime::EntityIdentity(handle, entity_id);
}

namespace EcsImplementation {
std::unordered_map<std::uint32_t, ResolveScanState> resolve_scans;

std::unordered_map<std::uint32_t, ResolveCacheEntry> resolve_cache;

}
