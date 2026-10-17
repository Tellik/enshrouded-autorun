#include "loaded-resources/loaded_resource_state.h"

using namespace ResourceImplementation;

namespace ResourceImplementation {
bool compare_exchange_32(std::uintptr_t address, std::uint32_t expected,
    std::uint32_t desired, std::uint32_t& observed) noexcept{
    __try {
        observed = static_cast<std::uint32_t>(InterlockedCompareExchange(
            reinterpret_cast<volatile LONG*>(address), static_cast<LONG>(desired), static_cast<LONG>(expected)));
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

// Caller-owned full-resource expectations are captured before original gates.
// Native wrappers reject invalid pointers without leaving an exception across
// the ABI. These bounded helpers allocate nothing and do not query host locks.
bool copy_expected_snapshot(const void* source, unsigned char* destination, std::size_t size) noexcept{
    const auto address = reinterpret_cast<std::uintptr_t>(source);
    if (!source || !destination || !size || size > UINTPTR_MAX - address) return false;
    __try { std::memcpy(destination, source, size); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool snapshot_matches(std::uintptr_t address, const unsigned char* expected, std::size_t size) noexcept{
    if (!size) return true;
    if (!address || !expected || size > UINTPTR_MAX - address) return false;
    __try { return std::memcmp(reinterpret_cast<const void*>(address), expected, size) == 0; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

class ScalarDataProtection {
public:
    ~ScalarDataProtection() { Restore(); }
    bool Prepare(std::uintptr_t address, std::uintptr_t allocation_base) {
        MEMORY_BASIC_INFORMATION page{};
        if (VirtualQuery(reinterpret_cast<const void*>(address), &page, sizeof(page)) != sizeof(page) ||
            page.State != MEM_COMMIT || page.Type != MEM_PRIVATE ||
            reinterpret_cast<std::uintptr_t>(page.AllocationBase) != allocation_base ||
            (page.Protect != PAGE_READONLY && page.Protect != PAGE_READWRITE)) return false;
        const auto base = reinterpret_cast<std::uintptr_t>(page.BaseAddress);
        if (address < base || address - base > page.RegionSize || sizeof(LONG) > page.RegionSize - (address - base))
            return false;
        address_ = address;
        allocation_base_ = allocation_base;
        old_ = page.Protect;
        if (page.Protect == PAGE_READWRITE) return true;
        DWORD previous{};
        if (!VirtualProtect(reinterpret_cast<void*>(address_), sizeof(LONG), PAGE_READWRITE, &previous)) return false;
        old_ = previous;
        changed_ = true;
        // A changed protection is not permission to write a different kind of
        // page. Restore even if the original observation no longer matches.
        if (previous != PAGE_READONLY) return false;
        MEMORY_BASIC_INFORMATION after{};
        return VirtualQuery(reinterpret_cast<const void*>(address_), &after, sizeof(after)) == sizeof(after) &&
            after.State == MEM_COMMIT && after.Type == MEM_PRIVATE && after.Protect == PAGE_READWRITE &&
            after.AllocationBase == page.AllocationBase;
    }
    bool Restore() noexcept {
        if (!address_) return !restore_failed_;
        if (changed_) {
            DWORD ignored{};
            if (!VirtualProtect(reinterpret_cast<void*>(address_), sizeof(LONG), old_, &ignored)) {
                restore_failed_ = true;
                // The destructor can make one last cleanup attempt. This sticky
                // failure still prevents reporting a confirmed outcome.
                return false;
            }
            changed_ = false;
        }
        MEMORY_BASIC_INFORMATION restored{};
        if (VirtualQuery(reinterpret_cast<const void*>(address_), &restored, sizeof(restored)) != sizeof(restored) ||
            restored.Protect != old_ || restored.State != MEM_COMMIT || restored.Type != MEM_PRIVATE ||
            reinterpret_cast<std::uintptr_t>(restored.AllocationBase) != allocation_base_)
            restore_failed_ = true;
        return !restore_failed_;
    }
private:
    std::uintptr_t address_{}, allocation_base_{};
    DWORD old_{};
    bool changed_{}, restore_failed_{};
};
}

namespace ResourceRuntime {
static std::uint32_t compare_exchange_scalar(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width,
    std::uint64_t expected_session) {
    if (width != sizeof(LONG) || !expected || !desired || !resource_size ||
        offset > resource_size || width > resource_size - offset) return 0;
    std::uint32_t expected_bits{}, desired_bits{};
    if ((!expected_session && !copy_memory(reinterpret_cast<std::uintptr_t>(expected), &expected_bits, sizeof(expected_bits))) ||
        !copy_memory(reinterpret_cast<std::uintptr_t>(desired), &desired_bits, sizeof(desired_bits))) return 0;
    // Serialize our temporary data-page protections. Take this mutex before the
    // original lifetime guard, never while holding an engine gate.
    std::scoped_lock lock(state_mutex);
    if (!profile_loaded || scalar_write_fault) return 0;
    const auto& layout = active_layout;
    std::vector<unsigned char> expected_resource;
    if (expected_session) {
        if (resource_size > layout.maximum_resource_bytes) return 0;
        expected_resource.resize(resource_size);
        if (!copy_expected_snapshot(expected, expected_resource.data(), expected_resource.size())) return 0;
        std::memcpy(&expected_bits, expected_resource.data() + offset, sizeof(expected_bits));
    }
    // The ECS binding mutex is acquired before original engine gates and held
    // until after them. No artificial session is generated by this operation.
    EcsRuntime::SessionGuard session;
    if (expected_session && (!session.TryAcquire(expected_session) || !session.WritesAllowed())) return 0;
    const auto uncertain = [&](const char* reason) -> std::uint32_t {
        scalar_write_fault = reason;
        if (expected_session) session.MarkWriteUncertain(reason);
        return 3;
    };
    View initial{};
    if (!resolve_view(layout, initial) || !registry_is_bound(layout, initial)) return 0;
    const auto session_owners_match = [&](const EngineContext::Owners& owners) {
        return !expected_session || owners == session.View().owners;
    };
    // Prepare the content-worker snapshot before entering Scene. Its lifetime
    // guard then borrows Scene's identical original registry Shared ownership.
    // Declaration order releases Scene before destroying prepared heap storage.
    EngineContext::ResourceLifeGuard lifetime;
    EngineContext::EcsWriteGuard scene;
    if (expected_session) {
        if (!lifetime.Prepare() || !scene.TryAcquire(session.View()) ||
            !lifetime.TryAcquire(scene, initial.service, initial.reader)) return 0;
    } else if (!lifetime.TryAcquire(initial.service, initial.reader)) return 0;
    EngineContext::Owners owners;
    View before{};
    Span span{};
    if (!lifetime.Validate(owners) || !session_owners_match(owners) ||
        (expected_session && !scene.Validate()) || !resolve_view(layout, before, nullptr, &owners) ||
        !same_view(initial, before) || !registry_is_bound(layout, before) ||
        !locate(layout, before, resource, static_hash, span) || span.size != resource_size ||
        !lifetime.Contains(span.data, span.size)) return 0;
    std::uintptr_t scalar{};
    if (!add(span.data, offset, scalar) || scalar % alignof(LONG)) return 0;
    const auto stable = [&]() {
        EngineContext::Owners current_owners;
        View after{};
        Span current{};
        return lifetime.Validate(current_owners) && session_owners_match(current_owners) &&
            (!expected_session || scene.Validate()) && resolve_view(layout, after, nullptr, &current_owners) &&
            same_view(before, after) && registry_is_bound(layout, after) &&
            locate(layout, after, resource, static_hash, current) && current.data == span.data &&
            current.size == span.size && current.handle == span.handle && current.chunk == span.chunk &&
            lifetime.Contains(current.data, current.size);
    };
    ScalarDataProtection protection;
    if (!protection.Prepare(scalar, lifetime.PayloadBase()))
        return protection.Restore() ? 0 : uncertain("resource-page-protection-restore-uncertain");
    if (!stable())
        return protection.Restore() ? 0 : uncertain("resource-page-protection-restore-uncertain");
    // A reflected BlobVariant field offset is meaningful only for the complete
    // resource image used by the Lua codec, including every descriptor. Equal
    // size, ResourceId and scalar bits alone are insufficient after a reload.
    if (expected_session && !snapshot_matches(span.data, expected_resource.data(), expected_resource.size()))
        return protection.Restore() ? 2 : uncertain("resource-page-protection-restore-uncertain");
    std::uint32_t observed{};
    const bool executed = compare_exchange_32(scalar, expected_bits, desired_bits, observed);
    bool unchanged_payload = true;
    if (expected_session && executed) {
        if (observed == expected_bits) {
            std::memcpy(expected_resource.data() + offset, &desired_bits, sizeof(desired_bits));
            unchanged_payload = snapshot_matches(span.data, expected_resource.data(), expected_resource.size());
        } else {
            // A scalar conflict has no mutation by us. All other bytes,
            // especially the variant descriptors, must still match.
            unchanged_payload = snapshot_matches(span.data, expected_resource.data(), offset) &&
                snapshot_matches(span.data + offset + width, expected_resource.data() + offset + width,
                    expected_resource.size() - offset - width);
        }
    }
    const bool unchanged_owner = stable();
    const bool restored = protection.Restore();
    if (!restored) return uncertain("resource-page-protection-restore-uncertain");
    if (!executed) return uncertain("resource-scalar-cas-uncertain");
    if (!unchanged_owner) return uncertain("resource-scalar-owner-changed");
    if (!unchanged_payload) return uncertain("resource-scalar-payload-changed-after-cas");
    return observed == expected_bits ? 1 : 2;
}

std::uint32_t CompareExchangeScalar(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width) {
    return compare_exchange_scalar(resource, static_hash, resource_size, offset, expected, desired, width, 0);
}

std::uint32_t CompareExchangeScalarForSession(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width,
    std::uint64_t expected_session) {
    if (!expected_session) return 0;
    return compare_exchange_scalar(resource, static_hash, resource_size, offset, expected, desired, width, expected_session);
}
}

extern "C" std::uint32_t __cdecl KfcRuntimeResourceCompareExchangeScalar(const KfcRuntimeResourceId* resource,
    std::uint32_t static_hash, std::size_t resource_size, std::size_t offset,
    const void* expected, const void* desired, std::size_t width) {
    if (!resource) return 0;
    try { return ResourceRuntime::CompareExchangeScalar(*resource, static_hash, resource_size, offset, expected, desired, width); }
    catch (...) { return 0; }
}

extern "C" std::uint32_t __cdecl KfcRuntimeResourceCompareExchangeScalarForSession(const KfcRuntimeResourceId* resource,
    std::uint32_t static_hash, std::size_t resource_size, std::size_t offset,
    const void* expected, const void* desired, std::size_t width, std::uint64_t expected_session) {
    if (!resource || !expected_session) return 0;
    try { return ResourceRuntime::CompareExchangeScalarForSession(*resource, static_hash, resource_size, offset,
        expected, desired, width, expected_session); }
    catch (...) { return 0; }
}
