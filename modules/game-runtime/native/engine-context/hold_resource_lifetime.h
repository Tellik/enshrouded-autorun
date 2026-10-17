#pragma once
#include "engine-context/engine_owners.h"
namespace EngineContext {
class EcsWriteGuard;
// Holds the original thread registry Shared, so the registered content worker
// cannot be unregistered before its Reader and bulk payload are destroyed.
// This guards lifetime only. Scalar atomicity belongs to ResourceRuntime.
class ResourceLifeGuard {
public:
    ResourceLifeGuard();
    ~ResourceLifeGuard();
    ResourceLifeGuard(const ResourceLifeGuard&) = delete;
    ResourceLifeGuard& operator=(const ResourceLifeGuard&) = delete;
    bool TryAcquire(std::uintptr_t expected_service, std::uintptr_t expected_reader);
    // Prepare outside original gates, then borrow the EcsWriteGuard's already
    // held identical thread-registry Shared ownership. No allocation or host
    // mutex acquisition occurs in the second overload. Declare this guard
    // before the Scene guard, then use it only while Scene remains held.
    bool Prepare();
    bool TryAcquire(const EcsWriteGuard& scene, std::uintptr_t expected_service,
        std::uintptr_t expected_reader);
    bool Validate(Owners& owners) const;
    bool Contains(std::uintptr_t address, std::size_t size) const;
    std::uintptr_t PayloadBase() const;
    void Release() noexcept;
private:
    struct State;
    std::unique_ptr<State> state_;
    bool AcquirePrepared(std::uintptr_t expected_service, std::uintptr_t expected_reader,
        std::uintptr_t borrowed_registry);
};
}
