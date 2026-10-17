#include "loaded-resources/loaded_resource_state.h"

using namespace ResourceImplementation;

namespace ResourceRuntime {
bool Read(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    unsigned char* buffer, std::size_t capacity, std::size_t* actual) {
    if (!actual || (!buffer && capacity)) return false;
    *actual = 0;
    Layout layout;
    {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded) return false;
        layout = active_layout;
    }
    View before{};
    Span span{};
    if (!resolve_view(layout, before) || !registry_is_bound(layout, before) ||
        !locate(layout, before, resource, static_hash, span)) return false;
    *actual = span.size;
    const auto stable_span = [&] {
        View after{};
        Span after_span{};
        return resolve_view(layout, after) && registry_is_bound(layout, after) && same_view(before, after) &&
            locate(layout, after, resource, static_hash, after_span) &&
            span.data == after_span.data && span.size == after_span.size && span.handle == after_span.handle;
    };
    if (!buffer && capacity == 0) {
        // The size query also backs Lua's direct GUID existence check. Preserve
        // its readable-payload filter without querying every metadata scalar.
        RegionCache regions;
        return readable(span.data, span.size, regions) && stable_span();
    }
    if (capacity < span.size || (!buffer && span.size)) return false;
    std::vector<unsigned char> snapshot(span.size), confirmation(span.size);
    if (!copy_memory(span.data, snapshot.data(), span.size) ||
        !copy_memory(span.data, confirmation.data(), span.size) ||
        snapshot != confirmation) return false;
    if (!stable_span()) return false;
    if (!snapshot.empty()) std::memcpy(buffer, snapshot.data(), snapshot.size());
    return true;
}
}

extern "C" bool __cdecl KfcRuntimeResourceRead(const KfcRuntimeResourceId* resource,
    std::uint32_t static_hash, unsigned char* buffer, std::size_t capacity, std::size_t* actual) {
    if (!resource) return false;
    try { return ResourceRuntime::Read(*resource, static_hash, buffer, capacity, actual); }
    catch (...) { if (actual) *actual = 0; return false; }
}
