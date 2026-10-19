#include "ecs/building_prop_query_state.h"
#include "ecs/ecs_session_state.h"

using namespace EcsImplementation;

namespace EcsImplementation {

bool recipe_bounds_intersect(const KfcRuntimePropRecord& prop,
                             const std::array<float, 6>& bounds,
                             const double* query_bounds){
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) ||
            bounds[axis] > bounds[axis + 3]) return false;
    }
    const double qx = prop.orientation[0], qy = prop.orientation[1];
    const double qz = prop.orientation[2], qw = prop.orientation[3];
    const double norm = qx*qx + qy*qy + qz*qz + qw*qw;
    const double sx = prop.scale[0], sy = prop.scale[1], sz = prop.scale[2];
    if (!std::isfinite(norm) || norm <= 1e-12 || !std::isfinite(sx) ||
        !std::isfinite(sy) || !std::isfinite(sz)) return false;
    const double factor = 2.0 / norm;
    const double rotation[3][3] = {
        {1-factor*(qy*qy+qz*qz), factor*(qx*qy-qz*qw), factor*(qx*qz+qy*qw)},
        {factor*(qx*qy+qz*qw), 1-factor*(qx*qx+qz*qz), factor*(qy*qz-qx*qw)},
        {factor*(qx*qz-qy*qw), factor*(qy*qz+qx*qw), 1-factor*(qx*qx+qy*qy)},
    };
    const double scale[3]{sx, sy, sz};
    double local_center[3]{}, local_half[3]{};
    for (int axis = 0; axis < 3; ++axis) {
        local_center[axis] = (static_cast<double>(bounds[axis]) + bounds[axis + 3]) * 0.5;
        local_half[axis] = std::abs(static_cast<double>(bounds[axis + 3]) - bounds[axis]) * 0.5 * std::abs(scale[axis]);
        local_center[axis] *= scale[axis];
    }
    double world_center[3]{}, world_half[3]{};
    for (int row = 0; row < 3; ++row) {
        world_center[row] = static_cast<double>(prop.position[row]) / 4294967296.0;
        for (int column = 0; column < 3; ++column) {
            world_center[row] += rotation[row][column] * local_center[column];
            world_half[row] += std::abs(rotation[row][column]) * local_half[column];
        }
    }
    if (world_half[0] + world_half[1] + world_half[2] < 1e-6)
        world_half[0] = world_half[1] = world_half[2] = 0.25;
    for (int axis = 0; axis < 3; ++axis) {
        if (world_center[axis] + world_half[axis] < query_bounds[axis] ||
            world_center[axis] - world_half[axis] >= query_bounds[axis + 3]) return false;
    }
    return true;
}

void validate_prop_query(PropQueryOperation& operation, const ResolvedLayout& layout,
                         const ComponentType& transform, const ComponentType& item){
    std::uintptr_t table{};
    std::uint64_t count{};
    if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
        !read(layout.table_address, table) || !table || table > UINTPTR_MAX - count * sizeof(std::uintptr_t)) return;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(2);
    std::size_t visited{};
    while (operation.validation_cursor < operation.output.size()) {
        if (visited >= 8192 || (visited && std::chrono::steady_clock::now() >= deadline)) {
            operation.result = SIZE_MAX - 1;
            return;
        }
        const auto& prop = operation.output[operation.validation_cursor++];
        ++visited;
        HandleRecord record{};
        {
            std::scoped_lock lock(state_mutex);
            const auto found = handles.find(prop.entity_handle);
            if (found == handles.end() || found->second.epoch != layout_epoch) return;
            record = found->second;
        }
        std::uintptr_t pointer{};
        EntityView entity{};
        KfcRuntimePropRecord current{};
        // A changed count/retrieval snapshot is retried by the caller. Do not
        // rediscover an erased handle through entity_for_handle's full-table
        // fallback inside this bounded validation slice.
        if (record.table_slot < table || record.table_slot >= table + count * sizeof(pointer) ||
            (record.table_slot - table) % sizeof(pointer) || !read(record.table_slot, pointer) || pointer != record.pointer ||
            !entity_view(pointer, layout, entity) || entity.id != record.id || entity.generation != record.generation ||
            !read_prop_payload(entity, layout, transform, item, current, operation.exact_recipe_bounds)) return;
        if (current.item_id != prop.item_id || current.entity_id != prop.entity_id ||
            std::memcmp(current.position, prop.position, sizeof(prop.position)) ||
            std::memcmp(current.orientation, prop.orientation, sizeof(prop.orientation)) ||
            std::memcmp(current.scale, prop.scale, sizeof(prop.scale)) ||
            std::memcmp(current.template_uuid, prop.template_uuid, sizeof(prop.template_uuid))) return;
    }
    operation.result = operation.output.size();
}

void query_props_native(PropQueryOperation& operation){
    std::scoped_lock operation_lock(operation_mutex);
    ComponentType transform_component{}, item_component{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform_component) || transform_component.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item_component) || item_component.size != 4) return;
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    if (!operation.initialized) {
        if (!entity_pointers(layout, operation.pointers) || operation.pointers.size() > (1u << 20)) return;
        if (operation.exact_recipe_bounds) {
            std::scoped_lock lock(prop_recipe_mutex);
            if (!prop_recipe_catalog_ready) return;
            operation.recipes = prop_recipes;
        }
        operation.seen.reserve(operation.pointers.size());
        operation.initialized = true;
    }
    const auto slice_deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(2);
    std::size_t visited{};
    while (operation.cursor < operation.pointers.size()) {
        if (operation.exact_recipe_bounds &&
            (visited >= 8192 || (visited > 0 && std::chrono::steady_clock::now() >= slice_deadline))) {
            operation.result = SIZE_MAX - 1;
            return;
        }
        const auto pointer = operation.pointers[operation.cursor++];
        ++visited;
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) || !operation.seen.insert(identity_key(entity.id, entity.generation)).second) continue;
        KfcRuntimePropRecord prop{};
        if (!read_prop_payload(entity, layout, transform_component, item_component, prop, false)) continue;
        if (operation.exact_recipe_bounds) {
            const auto recipe = operation.recipes.find(prop.item_id);
            if (recipe == operation.recipes.end() || !recipe_bounds_intersect(prop, recipe->second.bounds, operation.bounds)) continue;
            // UUIDs are needed by the editor only for props inside the selected
            // region. Reading them before the spatial test made every capture
            // touch the template definition of every UsedItem in the world.
            if (!read_prop_template_uuid(entity, layout, prop)) continue;
        } else {
            const double scale = (std::max)({std::abs(static_cast<double>(prop.scale[0])),
                std::abs(static_cast<double>(prop.scale[1])), std::abs(static_cast<double>(prop.scale[2]))});
            if (!std::isfinite(scale)) continue;
            const auto margin = operation.padding * scale;
            bool inside = true;
            for (int axis = 0; axis < 3; ++axis) {
                const auto position = static_cast<double>(prop.position[axis]) / 4294967296.0;
                if (!std::isfinite(position) || position < operation.bounds[axis] - margin ||
                    position >= operation.bounds[axis + 3] + margin) { inside = false; break; }
            }
            if (!inside) continue;
        }
        prop.entity_handle = handle_for(entity, pointer.table_index);
        prop.entity_id = entity.id;
        if (prop.entity_handle) operation.output.push_back(prop);
    }
    operation.result = operation.output.size();
}

namespace {
void query_prop_operation(void* opaque){ query_props_native(*static_cast<PropQueryOperation*>(opaque)); }

void validate_cached_props(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<PropQueryOperation*>(opaque);
    ComponentType transform{}, item{};
    ResolvedLayout layout{};
    if (layout_snapshot(layout) && resolve_component("keen::ecs::CurrentTransform", transform) &&
        resolve_component("keen::ecs::UsedItem", item)) validate_prop_query(operation, layout, transform, item);
}

bool invoke_prop_query(const std::shared_ptr<PropQueryOperation>& operation, bool cached = false){
    std::scoped_lock operation_lock(operation_mutex);
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (direct) return invoke_read_operation(cached ? validate_cached_props : query_prop_operation, operation, 500);
    if (!cached) query_props_native(*operation);
    else operation->result = operation->output.size();
    return true;
}
}
}

extern "C" bool __cdecl KfcRuntimeEcsPropQueryReady() {
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return false;
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (direct && (!layout.entity_definition_uuid_validated || !KfcRuntimeEcsReady())) return false;
    ComponentType transform{}, item{};
    return resolve_component("keen::ecs::CurrentTransform", transform) && transform.size == 0x38 &&
        resolve_component("keen::ecs::UsedItem", item) && item.size == 4;
}

extern "C" std::size_t __cdecl KfcRuntimeWorldEntityQueryProps(const double* bounds, double padding,
                                                                  KfcRuntimePropRecord* props, std::size_t capacity) {
    if (!bounds || !std::isfinite(padding) || padding < 0 || capacity > (1u << 20) || (!props && capacity)) return SIZE_MAX;
    auto operation = std::make_shared<PropQueryOperation>();
    operation->padding = padding;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) || bounds[axis] >= bounds[axis + 3]) return SIZE_MAX;
        operation->bounds[axis] = bounds[axis];
        operation->bounds[axis + 3] = bounds[axis + 3];
    }
    if (!invoke_prop_query(operation)) return SIZE_MAX;
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (props) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), props);
    return operation->result;
}

extern "C" std::size_t __cdecl KfcRuntimeWorldEntityQueryPropsInBounds(
    const double* bounds, KfcRuntimePropRecord* props, std::size_t capacity) {
    if (!bounds || capacity > (1u << 20) || (!props && capacity)) return SIZE_MAX;
    std::unique_lock query_lock(prop_query_call_mutex, std::try_to_lock);
    if (!query_lock.owns_lock()) return SIZE_MAX - 1;
    auto operation = std::make_shared<PropQueryOperation>();
    operation->exact_recipe_bounds = true;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) ||
            bounds[axis] >= bounds[axis + 3]) return SIZE_MAX;
        operation->bounds[axis] = bounds[axis];
        operation->bounds[axis + 3] = bounds[axis + 3];
    }
    const auto epoch = current_epoch();
    const auto now = GetTickCount64();
    const auto same_bounds = [&] {
        for (int axis = 0; axis < 6; ++axis)
            if (prop_query_result_cache.bounds[axis] != operation->bounds[axis]) return false;
        return true;
    };
    if (props && prop_query_result_cache.ready && prop_query_result_cache.epoch == epoch && same_bounds()) {
        operation->output = std::move(prop_query_result_cache.props);
        operation->validation_cursor = prop_query_result_cache.validation_cursor;
        operation->validating_cached = true;
        operation->epoch = epoch;
        operation->started_ms = operation->progressed_ms = now;
        pending_prop_query = operation;
        prop_query_result_cache.ready = false;
    }
    if (pending_prop_query) {
        const bool same = std::equal(std::begin(operation->bounds), std::end(operation->bounds),
            std::begin(pending_prop_query->bounds));
        if (pending_prop_query->epoch != epoch || now - pending_prop_query->progressed_ms > 30000 ||
            now - pending_prop_query->started_ms > 120000) {
            pending_prop_query.reset();
            prop_query_result_cache.ready = false;
            return SIZE_MAX; // changed world, stalled scan, or exceeded capture deadline; never publish partial results
        }
        if (same) operation = pending_prop_query;
        else if (now - pending_prop_query->touched_ms < 2000) return SIZE_MAX - 1;
        else pending_prop_query.reset();
    }
    if (!pending_prop_query) {
        operation->epoch = epoch;
        operation->started_ms = operation->progressed_ms = now;
        pending_prop_query = operation;
    }
    operation->touched_ms = now;
    operation->result = SIZE_MAX;
    const auto previous_cursor = operation->cursor;
    const auto previous_validation_cursor = operation->validation_cursor;
    if (!invoke_prop_query(operation, operation->validating_cached)) {
        pending_prop_query.reset();
        return SIZE_MAX;
    }
    if (operation->cursor > previous_cursor || operation->validation_cursor > previous_validation_cursor)
        operation->progressed_ms = GetTickCount64();
    if (current_epoch() != epoch) {
        pending_prop_query.reset();
        return SIZE_MAX;
    }
    if (operation->result == SIZE_MAX - 1) return operation->result;
    pending_prop_query.reset();
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (!props && capacity == 0 && !operation->output.empty()) {
        prop_query_result_cache.ready = true;
        prop_query_result_cache.epoch = epoch;
        prop_query_result_cache.validation_cursor = operation->validating_cached ? operation->validation_cursor : 0;
        std::copy_n(operation->bounds, 6, prop_query_result_cache.bounds.begin());
        prop_query_result_cache.props = std::move(operation->output);
    } else {
        prop_query_result_cache.ready = false;
        std::vector<KfcRuntimePropRecord>().swap(prop_query_result_cache.props);
    }
    if (props) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), props);
    return operation->result;
}

namespace EcsImplementation {
std::mutex prop_query_call_mutex;

PropQueryResultCache prop_query_result_cache;




std::shared_ptr<PropQueryOperation> pending_prop_query;
}
