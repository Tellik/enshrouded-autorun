#include "loaded-resources/resource_state.h"



using namespace ResourceImplementation;

namespace ResourceImplementation {
bool item_map_view(const ItemMapLayout& layout, const View& owner, ItemMapView& view){
    view = {};
    if (!add(owner.registry, layout.map_offset, view.address) ||
        !copy_memory(view.address, view.header.data(), layout.header_size)) return false;
    const auto field = [&](std::size_t offset, auto& value) {
        std::memcpy(&value, view.header.data() + offset, sizeof(value));
    };
    field(layout.bits_offset, view.bits);
    field(layout.capacity_offset, view.capacity);
    field(layout.keys_offset, view.keys);
    field(layout.values_offset, view.values);
    field(layout.count_offset, view.count);
    if (view.capacity > layout.maximum_capacity || view.count > view.capacity ||
        (view.capacity && (view.capacity & (view.capacity - 1)))) return false;
    // The original getter returns null before using the arrays if count is 0.
    if (!view.count) return true;
    return view.capacity && view.bits && view.keys && view.values &&
        ((view.capacity + 63) / 64) * 8 <= UINTPTR_MAX - view.bits &&
        view.capacity * 4 <= UINTPTR_MAX - view.keys &&
        view.capacity * 8 <= UINTPTR_MAX - view.values;
}

// Original uint32 hash + linear probing, stopping at the first unoccupied
// slot. A bounded prefix without an empty slot is incomplete, never not-found.
std::uint32_t item_map_lookup(const ItemMapLayout& layout, const ItemMapView& view,
    std::uint32_t wanted, std::uintptr_t& payload){
    payload = 0;
    if (!view.count) return 6;
    auto slot = static_cast<std::size_t>(wanted) & (view.capacity - 1);
    const auto maximum = std::min<std::uint64_t>(view.capacity, layout.maximum_probes);
    for (std::uint64_t probe = 0; probe < maximum; ++probe) {
        std::uint64_t bits{};
        std::uint32_t key{};
        if (!read_value(view.bits + (slot / 64) * 8, bits)) return 5;
        if (!(bits & (std::uint64_t{1} << (slot % 64)))) return 6;
        if (!read_value(view.keys + slot * 4, key)) return 5;
        if (key == wanted) {
            return read_value(view.values + slot * 8, payload) && payload ? 1 : 5;
        }
        slot = (slot + 1) & (view.capacity - 1);
    }
    return 5;
}
}

namespace ResourceRuntime {
std::uint32_t ResolveItem(std::uint32_t item_id, std::uint32_t type_hash,
    std::size_t root_size, std::size_t item_id_offset, std::size_t guid_offset,
    std::uint64_t expected_session, KfcRuntimeResourceId* resource) {
    constexpr std::uint32_t found = 1, invalid_argument = 2, unsupported = 3,
        stale_context = 4, retry_incomplete = 5, not_found = 6;
    if (!resource || !publish_resource_id({}, resource) || !item_id || !expected_session ||
        root_size < 20 || item_id_offset > root_size - 4 || item_id_offset % 4 ||
        guid_offset > root_size - 16) return invalid_argument;
    KfcRuntimeResourceId resolved{};
    const auto result = [&]() -> std::uint32_t {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded || !active_layout.item_map.enabled || !EngineContext::ResourceWriteConfigured())
            return unsupported;
        const auto& layout = active_layout;
        const auto& item = layout.item_map;
        if (type_hash != item.type_hash || root_size != item.root_size ||
            item_id_offset != item.item_id_offset || guid_offset != item.guid_offset) return invalid_argument;
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) return stale_context;
        View initial{};
        if (!resolve_view(layout, initial) || !registry_is_bound(layout, initial)) return retry_incomplete;
        EngineContext::ResourceLifeGuard lifetime;
        if (!lifetime.TryAcquire(initial.service, initial.reader)) return retry_incomplete;
        EngineContext::Owners owners;
        View before{};
        if (!lifetime.Validate(owners) || owners != session.View().owners ||
            !resolve_view(layout, before, nullptr, &owners) || !same_view(initial, before) ||
            !registry_is_bound(layout, before)) return stale_context;
        ItemMapView map_before{};
        if (!item_map_view(item, before, map_before)) return retry_incomplete;
        std::uintptr_t payload{};
        const auto lookup = item_map_lookup(item, map_before, item_id, payload);
        if (lookup != found && lookup != not_found) return retry_incomplete;
        Span span{};
        if (lookup == found) {
            std::uintptr_t id_address{}, guid_address{};
            std::uint32_t stored_id{};
            if (!lifetime.Contains(payload, root_size) || !add(payload, item_id_offset, id_address) ||
                !add(payload, guid_offset, guid_address) || !read_value(id_address, stored_id) ||
                stored_id != item_id || !copy_memory(guid_address, resolved.guid, sizeof(resolved.guid)))
                return retry_incomplete;
            resolved.type_hash = type_hash;
            resolved.part_index = item.part_index;
            if (!locate(layout, before, resolved, canonical_resource_hash(resolved), span) ||
                span.data != payload || span.size < root_size || !lifetime.Contains(span.data, span.size))
                return retry_incomplete;
        }

        // The original map is a borrowed, read-only sampled view. Resource
        // lifetime does not freeze its header/arrays, so recheck the original
        // probe and payload identity rather than trusting a retained pointer.
        ItemMapView map_after{};
        std::uintptr_t confirmed_payload{};
        if (!item_map_view(item, before, map_after) || map_after.address != map_before.address ||
            map_after.header != map_before.header ||
            item_map_lookup(item, map_after, item_id, confirmed_payload) != lookup ||
            confirmed_payload != payload) return retry_incomplete;
        if (lookup == found) {
            std::uint32_t confirmed_id{};
            std::array<unsigned char, 16> confirmed_guid{};
            Span current{};
            if (!read_value(payload + item_id_offset, confirmed_id) || confirmed_id != item_id ||
                !copy_memory(payload + guid_offset, confirmed_guid.data(), confirmed_guid.size()) ||
                std::memcmp(confirmed_guid.data(), resolved.guid, confirmed_guid.size()) ||
                !locate(layout, before, resolved, canonical_resource_hash(resolved), current) ||
                current.data != span.data || current.size != span.size || current.handle != span.handle ||
                current.chunk != span.chunk || !lifetime.Contains(current.data, current.size)) return retry_incomplete;
        }
        EngineContext::Owners after_owners;
        View after{};
        if (!lifetime.Validate(after_owners) || after_owners != owners ||
            !resolve_view(layout, after, nullptr, &after_owners) || !same_view(before, after) ||
            !registry_is_bound(layout, after)) return stale_context;
        return lookup;
    }();
    if (result != found) return result;
    return publish_resource_id(resolved, resource) ? found : invalid_argument;
}
}

extern "C" std::uint32_t __cdecl KfcRuntimeResourceResolveItem(
    std::uint32_t item_id, std::uint32_t type_hash, std::size_t root_size,
    std::size_t item_id_offset, std::size_t guid_offset, std::uint64_t expected_session,
    KfcRuntimeResourceId* resource) {
    try { return ResourceRuntime::ResolveItem(item_id, type_hash, root_size,
        item_id_offset, guid_offset, expected_session, resource); }
    catch (...) { if (resource) publish_resource_id({}, resource); return 5; }
}
