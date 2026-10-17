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
}
