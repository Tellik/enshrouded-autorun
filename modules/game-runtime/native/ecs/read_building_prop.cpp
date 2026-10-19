#include "ecs/ecs_session_state.h"

namespace EcsImplementation {
bool read_prop_template_uuid(const EntityView& entity, const ResolvedLayout& layout,
                             KfcRuntimePropRecord& prop){
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (direct && !layout.entity_definition_uuid_validated) return false;
    const auto offset = direct ? layout.entity_definition_uuid :
        KfcRuntimeCompatibility::EnshroudedClient::definition_uuid;
    if (!entity.definition || entity.definition > UINTPTR_MAX - offset) return false;
    std::array<std::uint64_t, 2> uuid{}, again{};
    EntityView current{};
    if (!read_bytes(entity.definition + offset, uuid.data(), sizeof(uuid)) || !(uuid[0] || uuid[1]) ||
        !read_bytes(entity.definition + offset, again.data(), sizeof(again)) || again != uuid ||
        !entity_view(entity.pointer, layout, current) || current != entity) return false;
    std::copy(uuid.begin(), uuid.end(), prop.template_uuid);
    return true;
}

bool read_prop_payload(const EntityView& entity, const ResolvedLayout& layout,
                       const ComponentType& transform_component, const ComponentType& item_component,
                       KfcRuntimePropRecord& prop, bool include_template_uuid){
    struct NativeTransform {
        std::int64_t position[3]; float rotation[4]; float scale[3]; std::uint32_t padding;
    } transform{}, again{};
    static_assert(sizeof(NativeTransform) == 0x38);
    if (transform_component.size != sizeof(transform) || item_component.size != sizeof(std::uint32_t)) return false;
    std::uintptr_t transform_address{}, item_address{}, current_transform{}, current_item{};
    std::uint32_t item{}, item_again{};
    EntityView current{};
    if (!component_address(entity, layout, transform_component, transform_address, false) ||
        !component_address(entity, layout, item_component, item_address, false) ||
        !read_bytes(transform_address, &transform, sizeof(transform)) || !read(item_address, item) || !item ||
        !read_bytes(transform_address, &again, sizeof(again)) || std::memcmp(&again, &transform, sizeof(transform)) ||
        !read(item_address, item_again) || item_again != item ||
        !entity_view(entity.pointer, layout, current) || current != entity ||
        !component_address(current, layout, transform_component, current_transform, false) || current_transform != transform_address ||
        !component_address(current, layout, item_component, current_item, false) || current_item != item_address) return false;
    if (!std::all_of(std::begin(transform.rotation), std::end(transform.rotation), [](float value) { return std::isfinite(value); }) ||
        !std::all_of(std::begin(transform.scale), std::end(transform.scale), [](float value) { return std::isfinite(value); })) return false;
    prop = {};
    prop.item_id = item;
    prop.entity_id = entity.id;
    std::copy_n(transform.position, 3, prop.position);
    std::copy_n(transform.rotation, 4, prop.orientation);
    std::copy_n(transform.scale, 3, prop.scale);
    return !include_template_uuid || read_prop_template_uuid(entity, layout, prop);
}

namespace {
struct PropReadOperation {
    std::uint32_t handle{}, entity_id{};
    KfcRuntimePropRecord prop{};
    bool result{};
    std::uint32_t status{5};
};

void read_prop_operation(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<PropReadOperation*>(opaque);
    ComponentType transform{}, item{};
    ResolvedLayout layout{};
    if (!resolve_component("keen::ecs::CurrentTransform", transform) || transform.size != 0x38 ||
        !resolve_component("keen::ecs::UsedItem", item) || item.size != 4 || !layout_snapshot(layout)) return;
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (operation.entity_id && direct && layout.occupancy_address && layout.keys_address) {
        EntityView entity{};
        std::size_t slot{};
        const auto membership = entity_for_id(operation.entity_id, layout, entity, slot);
        if (membership != EntityMembership::Present) {
            if (membership == EntityMembership::Absent) operation.status = 6;
            return;
        }
        if (!read_prop_payload(entity, layout, transform, item, operation.prop, true)) return;
        operation.prop.entity_handle = handle_for(entity, slot);
        operation.result = operation.prop.entity_handle != 0;
        if (operation.result) operation.status = 1;
        return;
    }
    if (operation.handle && direct) {
        HandleRecord record{};
        {
            std::scoped_lock lock(state_mutex);
            const auto found = handles.find(operation.handle);
            if (found == handles.end() || found->second.epoch != layout_epoch) {
                operation.status = 4;
                return;
            }
            record = found->second;
        }
        EntityView identity{};
        identity.pointer = record.pointer;
        identity.id = record.id;
        identity.generation = record.generation;
        if (layout.occupancy_address && layout.keys_address) {
            const auto membership = entity_membership(identity, layout);
            if (membership == EntityMembership::Absent) {
                operation.status = 6;
                return;
            }
            if (membership != EntityMembership::Present) return;
        }
        EntityView entity{};
        if (!entity_for_handle(operation.handle, layout, entity) ||
            !read_prop_payload(entity, layout, transform, item, operation.prop, true)) return;
        operation.prop.entity_handle = operation.handle;
        operation.result = true;
        operation.status = 1;
        return;
    }
    // Legacy handles also retain the original live-table membership check.
    // An ID/generation in a readable, erased allocation is not sufficient.
    HandleRecord record{};
    if (operation.handle) {
        std::scoped_lock lock(state_mutex);
        const auto found = handles.find(operation.handle);
        if (found == handles.end() || found->second.epoch != layout_epoch) return;
        record = found->second;
    }
    std::vector<EntityPointer> pointers;
    if (!entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return;
    if (operation.handle) {
        const auto cached = std::find_if(pointers.begin(), pointers.end(),
            [&](const auto& pointer) { return pointer.pointer == record.pointer; });
        if (cached != pointers.end()) std::iter_swap(pointers.begin(), cached);
    }
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (!entity_view(pointer.pointer, layout, entity) ||
            (operation.handle ? entity.id != record.id || entity.generation != record.generation :
                entity.id != operation.entity_id)) continue;
        if (!read_prop_payload(entity, layout, transform, item, operation.prop, true)) return;
        operation.prop.entity_handle = handle_for(entity, pointer.table_index);
        operation.result = operation.prop.entity_handle &&
            (!operation.handle || operation.prop.entity_handle == operation.handle);
        if (operation.result) operation.status = 1;
        return;
    }
}

bool read_prop(const std::shared_ptr<PropReadOperation>& operation){
    std::scoped_lock operation_lock(operation_mutex);
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (direct) return invoke_read_operation(read_prop_operation, operation, 500) && operation->result;
    read_prop_operation(operation.get());
    return operation->result;
}
}

bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop){
    if (!entity_id || !prop) return false;
    auto operation = std::make_shared<PropReadOperation>();
    operation->entity_id = entity_id;
    if (!read_prop(operation)) return false;
    *prop = operation->prop;
    return true;
}
}

namespace EcsRuntime {
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop) {
    return EcsImplementation::ResolvePropEntityId(entity_id, prop);
}
}

extern "C" bool __cdecl KfcRuntimeWorldEntityGetTransform(std::uint32_t handle, KfcRuntimePropRecord* prop) {
    if (!handle || !prop) return false;
    auto operation = std::make_shared<EcsImplementation::PropReadOperation>();
    operation->handle = handle;
    if (!EcsImplementation::read_prop(operation)) return false;
    *prop = operation->prop;
    return true;
}

extern "C" std::uint32_t __cdecl KfcRuntimeWorldEntityReadTransformForSession(
    std::uint32_t handle, std::uint64_t expected_session, KfcRuntimePropRecord* prop) {
    using namespace EcsImplementation;
    if (!handle || !expected_session || !prop) return 2;
    std::scoped_lock operation_lock(operation_mutex);
    {
        std::scoped_lock lock(state_mutex);
        if (stop_requested.load(std::memory_order_acquire)) return 4;
        if (!read_only_binding || !live_layout.occupancy_address || !live_layout.keys_address) return 3;
        if (write_fault_active_locked()) return 7;
    }
    if (KfcRuntimeWorldSessionId() != expected_session) return 4;
    auto operation = std::make_shared<PropReadOperation>();
    operation->handle = handle;
    const bool completed = invoke_read_operation(read_prop_operation, operation, 500);
    if (stop_requested.load(std::memory_order_acquire) || KfcRuntimeWorldSessionId() != expected_session) return 4;
    {
        std::scoped_lock lock(state_mutex);
        if (write_fault_active_locked()) return 7;
    }
    if (!completed) return 5;
    if (operation->status == 1) *prop = operation->prop;
    return operation->status;
}
