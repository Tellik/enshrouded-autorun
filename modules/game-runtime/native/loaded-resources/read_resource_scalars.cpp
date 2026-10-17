#include "loaded-resources/loaded_resource_state.h"

using namespace ResourceImplementation;

namespace ResourceRuntime {
std::uint32_t ReadScalarBatch(const KfcRuntimeResourceScalarRequest* requests,
    std::size_t count, std::uint32_t type_hash, std::size_t root_size,
    std::size_t offset, std::uint64_t expected_session,
    std::uint32_t* scalar_bits, std::size_t capacity, std::size_t* actual) {
    constexpr std::size_t maximum_batch = 64;
    constexpr std::uint32_t success = 1, invalid_argument = 2, unsupported = 3,
        stale_context = 4, retry_incomplete = 5;
    static_assert(sizeof(KfcRuntimeResourceId) == 24);
    static_assert(sizeof(KfcRuntimeResourceScalarRequest) == 28);
    if (!actual || !set_batch_count(actual, 0)) return invalid_argument;
    if (!requests || !scalar_bits || !count || count > maximum_batch || capacity < count ||
        !expected_session || root_size < sizeof(std::uint32_t) ||
        offset % alignof(std::uint32_t) || offset > root_size - sizeof(std::uint32_t))
        return invalid_argument;

    std::array<KfcRuntimeResourceScalarRequest, maximum_batch> copied_requests{};
    std::array<std::uint32_t, maximum_batch> keys{};
    std::array<Span, maximum_batch> spans{};
    if (!copy_memory(reinterpret_cast<std::uintptr_t>(requests), copied_requests.data(),
        count * sizeof(KfcRuntimeResourceScalarRequest))) return invalid_argument;
    for (std::size_t index = 0; index < count; ++index) {
        const auto& id = copied_requests[index].resource;
        if (id.type_hash != type_hash ||
            std::all_of(std::begin(id.guid), std::end(id.guid), [](auto value) { return value == 0; }))
            return invalid_argument;
    }

    // All allocation and our own mutex acquisitions precede original engine
    // gates. SessionGuard pins the binding epoch, not mutable resource contents.
    const auto result = [&]() -> std::uint32_t {
        std::scoped_lock lock(state_mutex);
        if (!profile_loaded || !EngineContext::ResourceWriteConfigured()) return unsupported;
        const auto& layout = active_layout;
        if (root_size > layout.maximum_resource_bytes) return invalid_argument;
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) {
            // SessionGuard intentionally admits only local authority. A stable
            // client replica still supports the established guarded Read path,
            // so decline this optimization without weakening stale-session errors.
            EngineContext::EcsView replica{}, confirmation{};
            View before{}, after{};
            if (KfcRuntimeWorldSessionId() == expected_session &&
                EngineContext::ResolveEcs(replica) && replica.role == EngineContext::EcsRole::ClientReplica &&
                resolve_view(layout, before, nullptr, &replica.owners) && registry_is_bound(layout, before) &&
                EngineContext::ResolveEcs(confirmation) && confirmation == replica &&
                resolve_view(layout, after, nullptr, &confirmation.owners) && registry_is_bound(layout, after) && same_view(before, after) &&
                KfcRuntimeWorldSessionId() == expected_session) return unsupported;
            return stale_context;
        }
        View initial{};
        if (!resolve_view(layout, initial) || !registry_is_bound(layout, initial)) return retry_incomplete;
        EngineContext::ResourceLifeGuard lifetime;
        if (!lifetime.TryAcquire(initial.service, initial.reader)) return retry_incomplete;
        EngineContext::Owners owners;
        View before{};
        if (!lifetime.Validate(owners) || owners != session.View().owners ||
            !resolve_view(layout, before, nullptr, &owners) || !same_view(initial, before) ||
            !registry_is_bound(layout, before)) return stale_context;

        // Read only the fixed inline key. Full resources are still read by the
        // final candidate lookup and every existing guarded resource writer.
        for (std::size_t index = 0; index < count; ++index) {
            const auto& request = copied_requests[index];
            auto& span = spans[index];
            if (!locate(layout, before, request.resource, request.static_hash, span) ||
                span.size < root_size || !lifetime.Contains(span.data, span.size)) return retry_incomplete;
            std::uintptr_t scalar{};
            if (!add(span.data, offset, scalar) || scalar % alignof(std::uint32_t)) return retry_incomplete;
            if (!read_value(scalar, keys[index])) return retry_incomplete;
        }

        // The lifetime guard does not freeze keys or map entries. Check every
        // selected identity/span and key again before returning sampled rows.
        for (std::size_t index = 0; index < count; ++index) {
            const auto& request = copied_requests[index];
            const auto& span = spans[index];
            Span current{};
            std::uint32_t confirmation{};
            std::uintptr_t scalar{};
            if (!locate(layout, before, request.resource, request.static_hash, current) ||
                current.data != span.data || current.size != span.size ||
                current.handle != span.handle || current.chunk != span.chunk ||
                !lifetime.Contains(current.data, current.size) || !add(current.data, offset, scalar) ||
                !read_value(scalar, confirmation) || confirmation != keys[index]) return retry_incomplete;
        }
        EngineContext::Owners after_owners;
        View after{};
        if (!lifetime.Validate(after_owners) || after_owners != owners ||
            !resolve_view(layout, after, nullptr, &after_owners) || !same_view(before, after) ||
            !registry_is_bound(layout, after)) return stale_context;
        return success;
    }();
    if (result != success) return result;
    return publish_scalar_batch(keys.data(), count, scalar_bits, actual) ? success : invalid_argument;
}
}

extern "C" std::uint32_t __cdecl KfcRuntimeResourceReadScalarBatch(
    const KfcRuntimeResourceScalarRequest* requests, std::size_t count, std::uint32_t type_hash,
    std::size_t root_size, std::size_t offset, std::uint64_t expected_session,
    std::uint32_t* scalar_bits, std::size_t capacity, std::size_t* actual) {
    try { return ResourceRuntime::ReadScalarBatch(requests, count, type_hash, root_size, offset,
        expected_session, scalar_bits, capacity, actual); }
    catch (...) { if (actual) set_batch_count(actual, 0); return 5; }
}
