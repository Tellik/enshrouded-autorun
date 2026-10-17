#include "ecs/registry/component_discovery_state.h"
#include "ecs/ecs_session_state.h"

namespace EcsImplementation {
void reset_component_discovery(){
    std::scoped_lock lock(discovery_mutex);
    registry_discovery.Reset();
    observed_indices_by_size.clear();
    discovered_layouts.clear();
    discovered_layout_components.clear();
    template_layout_samples.clear();
    template_layout_samples_dropped = 0;
    discovery_entities.clear();
    discovery_cursor = 0;
    discovery_table = 0;
    discovery_last_restart = 0;
    discovery_scanned_entities = 0;
    discovery_layout_count = 0;
    discovery_restarts = 0;
    discovery_complete = false;
}

bool collect_layout_component_candidates(const EntityView& entity, const ResolvedLayout& layout,
                                         std::vector<ComponentSlot>& components){
    std::size_t count{};
    {
        std::scoped_lock lock(state_mutex);
        if (component_registry_ready) count = component_registry.entries.size();
        else for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components)
            count = (std::max)(count, static_cast<std::size_t>(component.index) + 1);
    }
    if (!count || count > max_components) return false;
    std::uint64_t component_bits[16]{};
    if (!read_bytes(entity.layout + layout.component_bits, component_bits, ((count + 63) / 64) * 8)) return false;
    for (std::size_t index = 0; index < count; ++index) {
        if (!(component_bits[index / 64] & (std::uint64_t{1} << (index % 64)))) continue;
        std::uint16_t stride{}, offset{};
        if (!read(entity.layout + layout.component_strides + index * sizeof(stride), stride) || !stride ||
            !read(entity.layout + layout.component_offsets + index * sizeof(offset), offset)) continue;
        observed_indices_by_size[stride].insert(static_cast<std::uint16_t>(index));
        components.push_back({static_cast<std::uint16_t>(index), stride, offset});
    }
    ++discovery_layout_count;
    return true;
}

void collect_template_layout_sample(const EntityView& entity,
                                    const std::vector<ComponentSlot>& components){
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    if (!entity.definition) return;
    struct DefinitionHeader {
        std::uint64_t uuid[2]{};
        std::uintptr_t name{};
        std::uint64_t name_size{};
    } definition{};
    if (!read_bytes(entity.definition + definition_uuid, definition.uuid, sizeof(definition.uuid)) ||
        !read(entity.definition + definition_name, definition.name) || !definition.name ||
        !read(entity.definition + definition_name_size, definition.name_size) ||
        !definition.name_size || definition.name_size > 128) return;
    std::string name(static_cast<std::size_t>(definition.name_size), '\0');
    if (!read_bytes(definition.name, name.data(), name.size())) return;
    if (const auto end = name.find('\0'); end != std::string::npos) name.resize(end);
    if (name.empty()) return;

    std::string key(sizeof(definition.uuid) + sizeof(entity.layout), '\0');
    std::memcpy(key.data(), definition.uuid, sizeof(definition.uuid));
    std::memcpy(key.data() + sizeof(definition.uuid), &entity.layout, sizeof(entity.layout));
    auto found = template_layout_samples.find(key);
    if (found == template_layout_samples.end()) {
        constexpr std::size_t max_template_layout_samples = 4096;
        if (template_layout_samples.size() >= max_template_layout_samples) {
            ++template_layout_samples_dropped;
            return;
        }
        TemplateLayoutSample sample{};
        std::memcpy(sample.uuid, definition.uuid, sizeof(sample.uuid));
        sample.layout = entity.layout;
        sample.name = std::move(name);
        sample.components = components;
        found = template_layout_samples.emplace(std::move(key), std::move(sample)).first;
    }
    ++found->second.entity_count;
}

void publish_discovered_component_indices(){
    std::unordered_map<std::string, std::uint32_t> contract;
    std::unordered_map<std::string, ComponentType> previously_resolved;
    {
        std::scoped_lock lock(state_mutex);
        if (component_registry_seen) return;
        contract = configured_types;
        previously_resolved = types;
    }
    std::unordered_map<std::string, ComponentType> discovered;
    for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components) {
        const auto known = contract.find(component.qualified_name);
        if (known != contract.end() && known->second == component.size)
            discovered.emplace(known->first, ComponentType{component.index, component.size});
    }
    for (const auto& [name, component] : previously_resolved) {
        const auto configured = contract.find(name);
        if (configured != contract.end() && configured->second == component.size)
            discovered.emplace(name, component);
    }
    // Strides remain useful evidence, but a matching size cannot prove identity.
    // Only an engine registration or an exact build profile may publish a mapping.
    std::scoped_lock lock(state_mutex);
    if (configured_types != contract) return;
    types = std::move(discovered);
}

void component_discovery_tick(std::uintptr_t manager){
    std::scoped_lock discovery_lock(discovery_mutex);
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    const auto count_address = manager + KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count;
    const auto table_address = manager + KfcRuntimeCompatibility::EnshroudedClient::entity_manager_table;
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!read(count_address, count) || !count || count > (1u << 20) ||
        !read(table_address, table) || !table || !readable(table, count * sizeof(std::uintptr_t))) return;

    const auto now = GetTickCount64();
    if (table != discovery_table || discovery_entities.size() != count ||
        (discovery_complete && now - discovery_last_restart >= 5000)) {
        discovery_entities.resize(static_cast<std::size_t>(count));
        if (!read_bytes(table, discovery_entities.data(), discovery_entities.size() * sizeof(std::uintptr_t))) {
            discovery_entities.clear();
            return;
        }
        discovery_table = table;
        discovery_cursor %= static_cast<std::size_t>(count);
        discovery_last_restart = now;
        discovery_scanned_entities = 0;
        ++discovery_restarts;
        discovery_complete = false;
    }
    if (discovery_complete || discovery_entities.empty()) return;

    constexpr std::size_t entities_per_tick = 128;
    const auto begin = discovery_cursor;
    const auto end = (std::min)(discovery_cursor + entities_per_tick, discovery_entities.size());
    for (; discovery_cursor < end; ++discovery_cursor) {
        EntityView entity{};
        if (!entity_view(discovery_entities[discovery_cursor], layout, entity)) continue;
        auto components = discovered_layout_components.find(entity.layout);
        if (components == discovered_layout_components.end()) {
            std::vector<ComponentSlot> discovered_components;
            if (!collect_layout_component_candidates(entity, layout, discovered_components)) continue;
            discovered_layouts.insert(entity.layout);
            components = discovered_layout_components.emplace(entity.layout, std::move(discovered_components)).first;
        }
        collect_template_layout_sample(entity, components->second);
    }
    discovery_scanned_entities += end - begin;
    if (discovery_cursor >= discovery_entities.size()) {
        discovery_complete = true;
    }
    // Publish every batch so a changing entity table cannot withhold mappings
    // until an entire snapshot has been traversed.
    publish_discovered_component_indices();
}



std::unordered_map<std::uint32_t, std::unordered_set<std::uint16_t>> observed_indices_by_size;

std::unordered_set<std::uintptr_t> discovered_layouts;

std::unordered_map<std::uintptr_t, std::vector<ComponentSlot>> discovered_layout_components;

std::unordered_map<std::string, TemplateLayoutSample> template_layout_samples;

std::uint64_t template_layout_samples_dropped{};

std::vector<std::uintptr_t> discovery_entities;

std::size_t discovery_cursor{};

std::uintptr_t discovery_table{};

std::uint64_t discovery_last_restart{};

std::uint64_t discovery_scanned_entities{};

std::uint64_t discovery_layout_count{};

std::uint64_t discovery_restarts{};

bool discovery_complete{};

std::mutex discovery_mutex;

ComponentRegistry::Discovery registry_discovery;

}
