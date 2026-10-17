#include "ecs/ecs_session_state.h"

namespace EcsImplementation {
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

using namespace EcsImplementation;
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
