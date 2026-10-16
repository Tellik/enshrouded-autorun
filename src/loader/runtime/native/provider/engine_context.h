#pragma once

#include <cstddef>
#include <cstdint>
#include <array>
#include <memory>
#include <string>
#include <nlohmann/json_fwd.hpp>

// Original engine ownership, shared by resource and ECS readers. A successful
// read establishes borrowed data ownership, never a game-thread execution phase.
namespace EngineContext {
enum class EcsRole { ClientReplica, LocalAuthority };
const char* RoleName(EcsRole role);
struct Owners {
    std::uintptr_t image{}, root{}, client{}, controller{}, main_owner{};
    bool operator==(const Owners&) const = default;
};
struct EcsLayout {
    std::size_t manager_count{}, manager_table{};
    std::size_t manager_occupancy{}, manager_keys{};
    std::size_t entity_id{}, entity_generation{}, entity_layout{}, entity_storage{}, entity_definition{}, entity_row{};
    std::size_t entity_definition_uuid{};
    bool entity_definition_uuid_validated{};
    std::size_t component_bits{}, component_offsets{}, component_strides{};
    std::size_t maximum_entities{};
    bool operator==(const EcsLayout&) const = default;
};
struct EcsView {
    Owners owners;
    EcsRole role{EcsRole::ClientReplica};
    std::uintptr_t scene_owner{}, scene_entries{}, scene_entry{}, scene_context{};
    std::array<std::uint64_t, 2> scene_uuid{};
    std::uintptr_t execution_owner{}, registry_owner{}, manager{}, entity_table{};
    EcsLayout layout;
    bool operator==(const EcsView&) const = default;
};
bool Configure(const nlohmann::json& exact_profile, std::string& error);
nlohmann::json SystemDependencyLayout();
// Exact-image named DI service layout. Snapshot before acquiring engine gates.
nlohmann::json ServiceLayout();
void Shutdown();
bool ResolveOwners(Owners& owners, std::string* reason = nullptr);
bool ResolveEcs(EcsView& view, std::string* reason = nullptr);
bool EcsWriteConfigured();
bool ResourceWriteConfigured();
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
// Read-only capability snapshot. A write must repeat this policy under the
// original Scene gate before mutating the component payload.
bool EcsComponentWriteSupported(const EcsView& expected, std::uint16_t component_index);
// Owns the original native-thread registration and Scene execution gates.
// Acquire is nonblocking. Callers prepare buffers and take their own mutexes
// before entry, then perform bounded memory operations. TrackChange is the one
// separately validated native tracker. The current policy admits only its
// no-op branch for registrations with both original tracking bits clear.
class EcsWriteGuard {
public:
    EcsWriteGuard();
    ~EcsWriteGuard();
    EcsWriteGuard(const EcsWriteGuard&) = delete;
    EcsWriteGuard& operator=(const EcsWriteGuard&) = delete;
    bool TryAcquire(const EcsView& expected, const char** reason = nullptr);
    // Revalidates the captured complete Scene/owner chain using only the
    // prepared profile, without allocation or acquiring context_mutex.
    bool Validate() const;
    bool ComponentSupported(std::uint16_t component_index) const;
    bool TrackChange(std::uintptr_t entity, std::uint16_t component_index) const;
    void Release() noexcept;
    std::size_t MaximumBytes() const { return maximum_bytes_; }
private:
    friend class ResourceLifeGuard;
    struct State;
    std::unique_ptr<State> state_;
    std::uintptr_t registry_lock_{}, scene_lock_{};
    std::size_t maximum_bytes_{};
    std::uintptr_t manager_{}, tracker_{}, tracking_context_{};
    std::uintptr_t registry_owner_{};
    std::array<std::uint8_t, 33> tracker_bytes_{};
    std::size_t tracking_manager_offset_{};
    std::size_t tracking_registry_a_{}, tracking_registry_b_{};
};
}
