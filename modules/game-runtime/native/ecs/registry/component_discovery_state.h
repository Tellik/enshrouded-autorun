#pragma once
#include "abi/runtime_api.h"
#include "engine-context/engine_owners.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <mutex>
#include <string>
#include <unordered_map>
#include <unordered_set>
#include <vector>
#include "ecs/entity_storage.h"
#include "ecs/registry/read_component_registry.h"
#include "ecs/registry/find_component_registry.h"
#include <nlohmann/json.hpp>

namespace EcsImplementation {
struct ComponentSlot { std::uint16_t index{}, stride{}, offset{}; };

struct TemplateLayoutSample {
    std::uint64_t uuid[2]{};
    std::uintptr_t layout{};
    std::string name;
    std::uint64_t entity_count{};
    std::vector<ComponentSlot> components;
};

extern std::unordered_map<std::uint32_t, std::unordered_set<std::uint16_t>> observed_indices_by_size;

extern std::unordered_map<std::uintptr_t, std::vector<ComponentSlot>> discovered_layout_components;

extern std::unordered_map<std::string, TemplateLayoutSample> template_layout_samples;

extern std::uint64_t template_layout_samples_dropped;

extern std::vector<std::uintptr_t> discovery_entities;

extern std::size_t discovery_cursor;

extern std::uintptr_t discovery_table;

extern std::uint64_t discovery_last_restart;

extern std::uint64_t discovery_scanned_entities;

extern std::uint64_t discovery_layout_count;

extern std::uint64_t discovery_restarts;

extern bool discovery_complete;

extern std::mutex discovery_mutex;

extern ComponentRegistry::Discovery registry_discovery;

void reset_component_discovery();

bool collect_layout_component_candidates(const EntityView& entity, const ResolvedLayout& layout,
                                         std::vector<ComponentSlot>& components);

void collect_template_layout_sample(const EntityView& entity,
                                    const std::vector<ComponentSlot>& components);

void publish_discovered_component_indices();

nlohmann::json registration_type_json(const ComponentRegistry::Type& type);

void read_component_registry(std::uintptr_t manager);

void component_discovery_tick(std::uintptr_t manager);
}
