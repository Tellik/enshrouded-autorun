#include "named-services/service_state.h"



using namespace NamedServiceImplementation;

namespace NamedServiceImplementation {
bool copy(void* to, const void* from, std::size_t size){
    __try { std::memcpy(to, from, size); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool writable(std::uintptr_t address, std::size_t size){
    if (!address || address > UINTPTR_MAX - size) return false;
    const auto end = address + size;
    while (address < end) {
        MEMORY_BASIC_INFORMATION info{};
        if (VirtualQuery(reinterpret_cast<const void*>(address), &info, sizeof(info)) != sizeof(info) ||
            info.State != MEM_COMMIT || info.Type != MEM_PRIVATE || info.Protect != PAGE_READWRITE) return false;
        const auto base = reinterpret_cast<std::uintptr_t>(info.BaseAddress);
        if (base > UINTPTR_MAX - info.RegionSize || base + info.RegionSize <= address) return false;
        address = (std::min)(end, base + info.RegionSize);
    }
    return true;
}
}

extern "C" bool __cdecl KfcRuntimeServiceRead(const char* name, std::uint32_t type_hash, std::uint32_t size,
    std::uint64_t expected_session, unsigned char* buffer, std::size_t capacity) {
    if (!buffer || capacity != size || !expected_session) return false;
    try {
        const auto c = contract(name, type_hash, size);
        std::scoped_lock operation(operation_mutex);
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) return false;
        EngineContext::EcsWriteGuard guard;
        if (!guard.TryAcquire(session.View()) || size > guard.MaximumBytes()) return false;
        std::uintptr_t address{}, again{};
        return guard.Validate() && resolve(c, session.View(), address) && writable(address, size) &&
            copy(buffer, reinterpret_cast<const void*>(address), size) &&
            resolve(c, session.View(), again) && address == again && guard.Validate();
    } catch (...) { return false; }
}

extern "C" std::uint32_t __cdecl KfcRuntimeServiceCompareExchange(const char* name, std::uint32_t type_hash,
    std::uint32_t size, std::uint64_t expected_session, const void* expected, const void* mask, const void* desired) {
    if (!expected || !mask || !desired || !expected_session) return 0;
    try {
        const auto c = contract(name, type_hash, size);
        std::vector<std::uint8_t> expected_bytes(size), selected(size), wanted(size);
        if (!copy(expected_bytes.data(), expected, size) || !copy(selected.data(), mask, size) ||
            !copy(wanted.data(), desired, size) ||
            !std::all_of(selected.begin(), selected.end(), [](auto byte) { return byte <= 1; }) ||
            std::none_of(selected.begin(), selected.end(), [](auto byte) { return byte != 0; })) return 0;
        std::vector<std::uint8_t> before(size), after(size), verified(size);
        std::scoped_lock operation(operation_mutex);
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) return 0;
        if (!session.WritesAllowed()) return 0;
        EngineContext::EcsWriteGuard guard;
        if (!guard.TryAcquire(session.View()) || size > guard.MaximumBytes()) return 0;
        std::uintptr_t address{}, again{};
        if (!guard.Validate() || !resolve(c, session.View(), address) || !writable(address, size) ||
            !copy(before.data(), reinterpret_cast<const void*>(address), size)) return 0;
        if (before != expected_bytes) return 2;
        for (std::size_t index = 0; index < size; ++index)
            after[index] = selected[index] ? wanted[index] : before[index];
        if (after == before)
            return resolve(c, session.View(), again) && address == again && guard.Validate() ? 1 : 0;
        // Original admin_player writes this same plain-data service directly.
        // Its consumers read the service on every invocation, without ECS stamps.
        if (!copy(reinterpret_cast<void*>(address), after.data(), size) ||
            !copy(verified.data(), reinterpret_cast<const void*>(address), size) || verified != after) {
            copy(reinterpret_cast<void*>(address), before.data(), size);
            if (!copy(verified.data(), reinterpret_cast<const void*>(address), size) || verified != before ||
                !resolve(c, session.View(), again) || address != again || !guard.Validate()) {
                session.MarkWriteUncertain("named-service-write-outcome-uncertain");
                return 3;
            }
            return 0;
        }
        if (!resolve(c, session.View(), again) || address != again || !guard.Validate()) {
            session.MarkWriteUncertain("named-service-owner-changed-after-write");
            return 3;
        }
        return 1;
    } catch (...) { return 0; }
}
