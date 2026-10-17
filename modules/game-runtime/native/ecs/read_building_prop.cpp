#include "ecs/ecs_session_state.h"

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
}

namespace EcsRuntime {
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop) {
    return EcsImplementation::ResolvePropEntityId(entity_id, prop);
}
}

using namespace EcsImplementation;

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
