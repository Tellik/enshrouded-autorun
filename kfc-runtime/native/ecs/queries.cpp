#include "ecs/ecs_state.h"



using namespace EcsImplementation;

namespace EcsImplementation {
std::string query_key(const QueryOperation& operation){
    std::string key;
    for (const auto& name : operation.owned_names) {
        key.append(std::to_string(name.size()));
        key.push_back(':');
        key.append(name);
        key.push_back(';');
    }
    return key;
}

void publish_query_result(QueryOperation& operation, const std::vector<std::uint32_t>& matches){
    operation.result = matches.size();
    if (operation.entities && operation.capacity)
        std::copy_n(matches.begin(), (std::min)(operation.capacity, matches.size()), operation.entities);
}

void publish_current_query_result(QueryOperation& operation, const QueryCacheEntry& cached,
                                  const ResolvedLayout& layout, const std::vector<ComponentType>& components){
    // Discovery is incremental, but a known entity need not disappear from every
    // caller while that scan refreshes. Revalidate its real identity, live table
    // membership and requested components. Never retain component addresses.
    std::vector<std::uint32_t> matches;
    matches.reserve(cached.matches.size());
    for (const auto handle : cached.matches) {
        EntityView entity{};
        if (!entity_for_handle(handle, layout, entity)) continue;
        if (std::all_of(components.begin(), components.end(), [&](const auto& component) {
                return component_present(entity, layout, component);
            })) matches.push_back(handle);
    }
    publish_query_result(operation, matches);
}

void query_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<QueryOperation*>(opaque);
    std::vector<ComponentType> components(operation.count);
    for (std::size_t index = 0; index < operation.count; ++index)
        if (!resolve_component(operation.names[index], components[index])) return;
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    const auto epoch = current_epoch();
    const auto now_ms = GetTickCount64();
    const auto key = query_key(operation);
    for (auto iterator = query_scans.begin(); iterator != query_scans.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = query_scans.erase(iterator);
        else ++iterator;
    }
    for (auto iterator = query_cache.begin(); iterator != query_cache.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = query_cache.erase(iterator);
        else ++iterator;
    }
    auto cached = query_cache.find(key);
    if (cached != query_cache.end()) {
        if (cached->second.epoch != epoch) {
            query_cache.erase(cached);
            cached = query_cache.end();
        } else {
            cached->second.touched_ms = now_ms;
        }
        if (cached != query_cache.end() && now_ms - cached->second.completed_ms <= 50) {
            publish_current_query_result(operation, cached->second, layout, components);
            return;
        }
    }
    auto scan = query_scans.find(key);
    if (scan == query_scans.end() || scan->second.epoch != epoch) {
        QueryScanState fresh{};
        fresh.epoch = epoch;
        fresh.touched_ms = now_ms;
        if (!entity_pointers(layout, fresh.pointers)) return;
        scan = query_scans.insert_or_assign(key, std::move(fresh)).first;
        active_query_cursor.store(0, std::memory_order_relaxed);
        active_query_total.store(scan->second.pointers.size(), std::memory_order_relaxed);
        active_query_matches.store(0, std::memory_order_relaxed);
    }
    auto& progress = scan->second;
    progress.touched_ms = now_ms;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(1);
    std::size_t visited = 0;
    for (; progress.cursor < progress.pointers.size() && visited < 8192; ++progress.cursor, ++visited) {
        const auto pointer = progress.pointers[progress.cursor];
        EntityView entity{};
        if (entity_view(pointer.pointer, layout, entity)) {
            bool include = true;
            for (const auto& component : components) {
                if (!component_present(entity, layout, component)) { include = false; break; }
            }
            if (include && progress.seen.insert(identity_key(entity.id, entity.generation)).second)
                progress.matches.push_back(handle_for(entity, pointer.table_index));
        }
        if (std::chrono::steady_clock::now() >= deadline) {
            ++progress.cursor;
            break;
        }
    }
    if (progress.cursor < progress.pointers.size()) {
        // Keep publishing validated members of the last complete discovery
        // snapshot while advancing the next scan. Only the first scan is
        // pending. New members become visible when discovery completes.
        if (cached != query_cache.end())
            publish_current_query_result(operation, cached->second, layout, components);
        else operation.result = SIZE_MAX - 1;
        active_query_cursor.store(progress.cursor, std::memory_order_relaxed);
        active_query_total.store(progress.pointers.size(), std::memory_order_relaxed);
        active_query_matches.store(progress.matches.size(), std::memory_order_relaxed);
        return;
    }
    active_query_cursor.store(progress.cursor, std::memory_order_relaxed);
    active_query_total.store(progress.pointers.size(), std::memory_order_relaxed);
    active_query_matches.store(progress.matches.size(), std::memory_order_relaxed);
    auto completed = std::move(progress.matches);
    query_scans.erase(scan);
    const auto completed_ms = GetTickCount64();
    auto [refreshed, _] = query_cache.insert_or_assign(key,
        QueryCacheEntry{epoch, completed_ms, completed_ms, std::move(completed)});
    publish_current_query_result(operation, refreshed->second, layout, components);
}

void query_bounds_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<BoundsQueryOperation*>(opaque);
    std::vector<ComponentType> components;
    components.reserve(operation.names.size());
    for (const auto& name : operation.names) {
        ComponentType component{};
        if (!resolve_component(name.c_str(), component)) return;
        components.push_back(component);
    }
    const auto transform_type = std::find_if(operation.names.begin(), operation.names.end(),
        [](const auto& name) { return name == "keen::ecs::CurrentTransform"; });
    if (transform_type == operation.names.end()) return;
    const auto component_index = static_cast<std::size_t>(transform_type - operation.names.begin());
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    std::vector<EntityPointer> pointers;
    if (!entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return;
    operation.output.clear();
    std::unordered_set<std::uint32_t> seen;
    seen.reserve(pointers.size());
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity)) continue;
        if (!seen.insert(entity.id).second) continue;
        bool include = true;
        for (const auto& component : components) {
            if (!component_present(entity, layout, component)) { include = false; break; }
        }
        if (!include) continue;
        std::uintptr_t address{};
        if (!component_address(entity, layout, components[component_index], address)) continue;
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        static_assert(sizeof(NativeTransform) == 0x38);
        if (!read_bytes(address, &transform, sizeof(transform))) continue;
        const auto scale = (std::max)({std::abs(static_cast<double>(transform.scale[0])),
            std::abs(static_cast<double>(transform.scale[1])), std::abs(static_cast<double>(transform.scale[2]))});
        if (!std::isfinite(scale)) continue;
        const auto margin = operation.padding * scale;
        bool inside = true;
        for (int axis = 0; axis < 3; ++axis) {
            const auto world_position = static_cast<double>(transform.position[axis]) / 4294967296.0;
            if (!std::isfinite(world_position) || world_position < operation.bounds[axis] - margin ||
                world_position >= operation.bounds[axis + 3] + margin) { inside = false; break; }
        }
        if (inside) operation.output.push_back(handle_for(entity, pointer.table_index));
    }
    operation.result = operation.output.size();
}
}

extern "C" std::size_t __cdecl KfcRuntimeEcsQuery(const char* const* names, std::size_t count,
                                                    std::uint32_t* entities, std::size_t capacity) {
    operation_counters.queries.fetch_add(1, std::memory_order_relaxed);
    if (!names || !count || count > max_components || capacity > (1u << 20) || !KfcRuntimeEcsReady()) {
        operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
        return SIZE_MAX;
    }
    auto operation = std::make_shared<QueryOperation>();
    for (std::size_t index = 0; index < count; ++index) {
        if (!names[index]) {
            operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
            return SIZE_MAX;
        }
        operation->owned_names.emplace_back(names[index]);
    }
    for (const auto& name : operation->owned_names) operation->name_pointers.push_back(name.c_str());
    operation->names = operation->name_pointers.data();
    operation->count = count;
    operation->output.resize(capacity);
    operation->entities = operation->output.data();
    operation->capacity = capacity;
    if (!invoke_read_operation(query_on_game_thread, operation)) {
        operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
        return SIZE_MAX;
    }
    if (operation->result >= SIZE_MAX - 1) {
        if (operation->result == SIZE_MAX - 1) operation_counters.query_incomplete.fetch_add(1, std::memory_order_relaxed);
        else operation_counters.query_failures.fetch_add(1, std::memory_order_relaxed);
        return operation->result;
    }
    if (entities) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->result), entities);
    operation_counters.query_successes.fetch_add(1, std::memory_order_relaxed);
    return operation->result;
}

extern "C" std::size_t __cdecl KfcRuntimeEcsQueryBounds(const char* const* names, std::size_t count,
                                                           const double* bounds, double padding, std::uint32_t* entities,
                                                           std::size_t capacity) {
    if (!names || !count || count > max_components || !bounds || !std::isfinite(padding) || padding < 0 ||
        capacity > (1u << 20) || !KfcRuntimeEcsReady())
        return SIZE_MAX;
    auto operation = std::make_shared<BoundsQueryOperation>();
    operation->padding = padding;
    for (std::size_t index = 0; index < count; ++index) {
        if (!names[index]) return SIZE_MAX;
        operation->names.emplace_back(names[index]);
    }
    bool has_transform = false;
    for (int axis = 0; axis < 3; ++axis) {
        const auto minimum = bounds[axis], maximum = bounds[axis + 3];
        if (!std::isfinite(minimum) || !std::isfinite(maximum) || minimum >= maximum) return SIZE_MAX;
        operation->bounds[axis] = minimum;
        operation->bounds[axis + 3] = maximum;
    }
    has_transform = std::find(operation->names.begin(), operation->names.end(), "keen::ecs::CurrentTransform") != operation->names.end();
    if (!has_transform) return SIZE_MAX;
    operation->capacity = capacity;
    if (!invoke_read_operation(query_bounds_on_game_thread, operation, 500)) return SIZE_MAX;
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (entities) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), entities);
    return operation->result;
}
