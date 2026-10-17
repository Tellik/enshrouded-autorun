#include "ecs/ecs_state.h"



using namespace EcsImplementation;

namespace EcsImplementation {
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop){
    if (!entity_id || !prop) return false;
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return false; }
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType transform_component{}, item_component{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform_component) || transform_component.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item_component) || item_component.size != sizeof(std::uint32_t)) return false;
    ResolvedLayout layout{};
    std::vector<EntityPointer> pointers;
    if (!layout_snapshot(layout) || !entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) || entity.id != entity_id) continue;
        std::uintptr_t transform_address{}, item_address{};
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        std::uint32_t item_id{};
        std::uint64_t template_uuid[2]{};
        if (!component_address(entity, layout, transform_component, transform_address) ||
            !component_address(entity, layout, item_component, item_address) ||
            !read_bytes(transform_address, &transform, sizeof(transform)) ||
            !read_bytes(item_address, &item_id, sizeof(item_id)) || !item_id || !entity.definition ||
            !read_bytes(entity.definition + definition_uuid, template_uuid, sizeof(template_uuid)) ||
            !(template_uuid[0] || template_uuid[1])) return false;
        *prop = {};
        prop->entity_handle = handle_for(entity, pointer.table_index);
        prop->entity_id = entity.id;
        prop->item_id = item_id;
        std::copy_n(transform.position, 3, prop->position);
        std::copy_n(transform.rotation, 4, prop->orientation);
        std::copy_n(transform.scale, 3, prop->scale);
        std::copy_n(template_uuid, 2, prop->template_uuid);
        if (!prop->entity_handle) return false;
        return true;
    }
    return false;
}

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

void query_props_native(PropQueryOperation& operation){
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
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
        std::uintptr_t transform_address{}, item_address{};
        if (!component_address(entity, layout, transform_component, transform_address, false) ||
            !component_address(entity, layout, item_component, item_address, false)) continue;
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        static_assert(sizeof(NativeTransform) == 0x38);
        std::uint32_t item_id{};
        if (!read_bytes(transform_address, &transform, sizeof(transform)) ||
            !read_bytes(item_address, &item_id, sizeof(item_id)) || !item_id) continue;
        KfcRuntimePropRecord prop{};
        prop.item_id = item_id;
        std::copy_n(transform.position, 3, prop.position);
        std::copy_n(transform.rotation, 4, prop.orientation);
        std::copy_n(transform.scale, 3, prop.scale);
        if (operation.exact_recipe_bounds) {
            const auto recipe = operation.recipes.find(item_id);
            if (recipe == operation.recipes.end() || !recipe_bounds_intersect(prop, recipe->second.bounds, operation.bounds)) continue;
            // UUIDs are needed by the editor only for props inside the selected
            // region. Reading them before the spatial test made every capture
            // touch the template definition of every UsedItem in the world.
            if (!entity.definition || !read_bytes(entity.definition + definition_uuid,
                    prop.template_uuid, sizeof(prop.template_uuid)) ||
                !(prop.template_uuid[0] || prop.template_uuid[1])) continue;
        } else {
            const double scale = (std::max)({std::abs(static_cast<double>(transform.scale[0])),
                std::abs(static_cast<double>(transform.scale[1])), std::abs(static_cast<double>(transform.scale[2]))});
            if (!std::isfinite(scale)) continue;
            const auto margin = operation.padding * scale;
            bool inside = true;
            for (int axis = 0; axis < 3; ++axis) {
                const auto position = static_cast<double>(transform.position[axis]) / 4294967296.0;
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
}

namespace EcsRuntime {
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop) {
    return ::ResolvePropEntityId(entity_id, prop);
}
}

extern "C" bool __cdecl KfcRuntimeEcsPropQueryReady() {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return false; }
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return false;
    ComponentType transform{}, item{};
    return resolve_component("keen::ecs::CurrentTransform", transform) && transform.size == 0x38 &&
        resolve_component("keen::ecs::UsedItem", item) && item.size == 4;
}

extern "C" std::size_t __cdecl KfcRuntimeWorldEntityQueryProps(const double* bounds, double padding,
                                                                  KfcRuntimePropRecord* props, std::size_t capacity) {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return SIZE_MAX; }
    if (!bounds || !std::isfinite(padding) || padding < 0 || capacity > (1u << 20) || (!props && capacity)) return SIZE_MAX;
    auto operation = std::make_shared<PropQueryOperation>();
    operation->padding = padding;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(bounds[axis]) || !std::isfinite(bounds[axis + 3]) || bounds[axis] >= bounds[axis + 3]) return SIZE_MAX;
        operation->bounds[axis] = bounds[axis];
        operation->bounds[axis + 3] = bounds[axis + 3];
    }
    query_props_native(*operation);
    if (operation->result == SIZE_MAX) return SIZE_MAX;
    if (props) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), props);
    return operation->result;
}

extern "C" std::size_t __cdecl KfcRuntimeWorldEntityQueryPropsInBounds(
    const double* bounds, KfcRuntimePropRecord* props, std::size_t capacity) {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return SIZE_MAX; }
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
    const auto same_bounds = [&] {
        for (int axis = 0; axis < 6; ++axis)
            if (prop_query_result_cache.bounds[axis] != operation->bounds[axis]) return false;
        return true;
    };
    if (props && prop_query_result_cache.ready && prop_query_result_cache.epoch == epoch && same_bounds()) {
        const auto count = prop_query_result_cache.props.size();
        std::copy_n(prop_query_result_cache.props.begin(), (std::min)(capacity, count), props);
        prop_query_result_cache.ready = false;
        std::vector<KfcRuntimePropRecord>().swap(prop_query_result_cache.props);
        return count;
    }
    const auto now = GetTickCount64();
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
    query_props_native(*operation);
    if (operation->cursor > previous_cursor) operation->progressed_ms = GetTickCount64();
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
        std::copy_n(operation->bounds, 6, prop_query_result_cache.bounds.begin());
        prop_query_result_cache.props = std::move(operation->output);
    } else {
        prop_query_result_cache.ready = false;
        std::vector<KfcRuntimePropRecord>().swap(prop_query_result_cache.props);
    }
    if (props) std::copy_n(operation->output.begin(), (std::min)(capacity, operation->output.size()), props);
    return operation->result;
}

extern "C" bool __cdecl KfcRuntimeWorldEntityRegisterPropRecipes(
    const KfcRuntimePropRecipe* recipes, std::size_t count) {
    if ((!recipes && count) || count > 1'000'000) return false;
    std::unordered_map<std::uint32_t, RegisteredPropRecipe> resolved;
    resolved.reserve(count);
    for (std::size_t index = 0; index < count; ++index) {
        const auto& recipe = recipes[index];
        if (!recipe.item_id || !std::all_of(std::begin(recipe.bounds), std::end(recipe.bounds),
                [](float value) { return std::isfinite(value); }) ||
            recipe.bounds[0] > recipe.bounds[3] || recipe.bounds[1] > recipe.bounds[4] ||
            recipe.bounds[2] > recipe.bounds[5]) return false;
        if (!resolved.emplace(recipe.item_id,
                RegisteredPropRecipe{std::to_array(recipe.bounds), recipe.feedback}).second) return false;
    }
    {
        std::scoped_lock lock(prop_recipe_mutex);
        prop_recipes = std::move(resolved);
        prop_recipe_catalog_ready = true;
    }
    return true;
}

extern "C" bool __cdecl KfcRuntimeWorldEntityGetPropRecipe(
    std::uint32_t item_id, KfcRuntimePropRecipe* recipe) {
    if (!item_id || !recipe) return false;
    std::scoped_lock lock(prop_recipe_mutex);
    const auto found = prop_recipes.find(item_id);
    if (found == prop_recipes.end()) return false;
    recipe->item_id = item_id;
    std::copy(found->second.bounds.begin(), found->second.bounds.end(), recipe->bounds);
    recipe->feedback = found->second.feedback;
    return true;
}

extern "C" bool __cdecl KfcRuntimeWorldEntityGetTransform(std::uint32_t handle, KfcRuntimePropRecord* prop) {
    { std::scoped_lock lock(state_mutex); if (read_only_binding) return false; }
    if (!handle || !prop) return false;
    HandleRecord record{};
    {
        std::scoped_lock lock(state_mutex);
        const auto found = handles.find(handle);
        if (found == handles.end() || found->second.epoch != layout_epoch) return false;
        record = found->second;
    }
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return false;
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType transform_component{}, item_component{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform_component) || transform_component.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item_component) || item_component.size != 4) return false;
    std::vector<EntityPointer> pointers;
    if (!entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    // Keep table membership and live id/generation validation, but inspect the
    // captured pointer first. Undo used to decode every preceding world entity
    // again for each prop and each verification pass (O(props * world entities)).
    const auto cached = std::find_if(pointers.begin(), pointers.end(),
        [&](const EntityPointer& entry) { return entry.pointer == record.pointer; });
    if (cached != pointers.end()) std::iter_swap(pointers.begin(), cached);
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) || entity.id != record.id || entity.generation != record.generation) continue;
        std::uintptr_t transform_address{}, item_address{};
        struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; } transform{};
        std::uint32_t item_id{};
        if (!component_address(entity, layout, transform_component, transform_address) ||
            !component_address(entity, layout, item_component, item_address) ||
            !read_bytes(transform_address, &transform, sizeof(transform)) ||
            !read_bytes(item_address, &item_id, sizeof(item_id)) || !item_id) return false;
        const auto current_handle = handle_for(entity, pointer.table_index);
        if (current_handle != handle) return false;
        *prop = {};
        prop->entity_handle = handle;
        prop->entity_id = entity.id;
        prop->item_id = item_id;
        std::copy_n(transform.position, 3, prop->position);
        std::copy_n(transform.rotation, 4, prop->orientation);
        std::copy_n(transform.scale, 3, prop->scale);
        using namespace KfcRuntimeCompatibility::EnshroudedClient;
        if (!entity.definition || !read_bytes(entity.definition + definition_uuid,
                prop->template_uuid, sizeof(prop->template_uuid)) ||
            !(prop->template_uuid[0] || prop->template_uuid[1])) return false;
        return true;
    }
    return false;
}

struct SetEntityScaleOperation {
    std::uint32_t handle{};
    float scale[3]{};
    bool result{};
};

void set_entity_scale_on_game_thread(void* opaque) {
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<SetEntityScaleOperation*>(opaque);
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    ComponentType component{};
    if (!resolve_component("keen::ecs::CurrentTransform", component) || component.size != 0x38) return;
    ResolvedLayout layout{};
    EntityView entity{};
    std::uintptr_t address{};
    if (!layout_snapshot(layout) || !entity_for_handle(operation.handle, layout, entity) ||
        !component_address(entity, layout, component, address)) return;
    struct NativeTransform { std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding; };
    static_assert(sizeof(NativeTransform) == 0x38);
    constexpr auto scale_offset = offsetof(NativeTransform, scale);
    float before[3]{}, verified[3]{};
    if (!read_bytes(address + scale_offset, before, sizeof(before))) return;
    SIZE_T written{};
    if (!WriteProcessMemory(GetCurrentProcess(), reinterpret_cast<void*>(address + scale_offset),
            operation.scale, sizeof(operation.scale), &written) || written != sizeof(operation.scale) ||
        !read_bytes(address + scale_offset, verified, sizeof(verified)) ||
        std::memcmp(verified, operation.scale, sizeof(verified))) {
        SIZE_T restored{};
        WriteProcessMemory(GetCurrentProcess(), reinterpret_cast<void*>(address + scale_offset),
            before, sizeof(before), &restored);
        return;
    }
    operation.result = true;
}

extern "C" bool __cdecl KfcRuntimeWorldEntitySetScale(std::uint32_t handle, const double* scale) {
    if (!handle || !scale || !KfcRuntimeEcsCanWrite()) return false;
    auto operation = std::make_shared<SetEntityScaleOperation>();
    operation->handle = handle;
    for (int axis = 0; axis < 3; ++axis) {
        if (!std::isfinite(scale[axis]) || std::abs(scale[axis]) > std::numeric_limits<float>::max()) return false;
        operation->scale[axis] = static_cast<float>(scale[axis]);
    }
    return GameThreadDispatcher::Invoke(set_entity_scale_on_game_thread, operation) && operation->result;
}
