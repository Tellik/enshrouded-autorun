#include "ecs/ecs_state.h"



using namespace EcsImplementation;

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

nlohmann::json registration_type_json(const ComponentRegistry::Type& type){
    if (type.name.empty()) return nullptr;
    return {{"qualified_name",type.name},{"qualified_hash",type.hash},
        {"size",type.size},{"metadata_rva",type.metadata_rva}};
}

void read_component_registry(std::uintptr_t manager){
    const auto now = GetTickCount64();
    if (!manager) {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (component_registry_manager || layout_ready ||
            !handles.empty() || !reverse_handles.empty()) {
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
        component_registry_world_bound = false;
        read_only_binding = false;
        read_only_view = {};
        component_registry_manager = 0;
        live_layout = {};
        layout_ready = false;
        live_table = 0;
    }
    std::unordered_map<std::string, std::uint32_t> contract;
    {
        std::scoped_lock lock(state_mutex);
        if (manager && manager == component_registry_manager && now - component_registry_last_read < 1000) return;
        if (manager != component_registry_manager) component_registry_world_bound = false;
        component_registry_manager = manager;
        component_registry_last_read = now;
        contract = configured_types;
    }
    if (contract.empty()) return;
    ComponentRegistry::Snapshot snapshot;
    std::string error;
    const auto validate_contract = [&](const ComponentRegistry::Snapshot& candidate, std::string& reason) {
        for (const auto& entry : candidate.entries) {
            for (const auto* type : {&entry.storage, &entry.configuration}) {
                if (type->name.empty()) continue;
                const auto found = contract.find(type->name);
                if (found == contract.end() || found->second != type->size) {
                    reason = "engine registration disagrees with the parser type contract: " + type->name;
                    return false;
                }
            }
        }
        return true;
    };
    bool valid{};
    if (manager) {
        valid = ComponentRegistry::Read(manager, image_base, image_size, read_bytes, snapshot, error);
    } else {
        // Registration metadata has its own original owner. Resolving it does
        // not require an execution hook or establish an active ECS world.
        std::scoped_lock discovery_lock(discovery_mutex);
        valid = registry_discovery.Tick(image_base, image_size, read_bytes,
            validate_contract, snapshot);
        const auto& progress = registry_discovery.Status();
        if (!valid && progress.available) return; // cached catalog, revalidation is throttled
        if (!valid) error = progress.reason;
    }
    std::unordered_map<std::string, ComponentType> resolved;
    nlohmann::json entries = nlohmann::json::array();
    if (valid) valid = validate_contract(snapshot, error);
    if (valid) for (const auto& entry : snapshot.entries) {
        if (!entry.storage.name.empty())
            resolved.emplace(entry.storage.name, ComponentType{entry.index, entry.size});
        nlohmann::json callbacks = nlohmann::json::array();
        for (const auto& callback : entry.callbacks) {
            std::array<std::uint8_t, 512> code{};
            std::string code_hex;
            const auto code_address = image_base + callback.function_rva;
            if (image_size >= code.size() && callback.function_rva <= image_size - code.size() &&
                read_bytes(code_address, code.data(), code.size())) {
                static constexpr char digits[] = "0123456789abcdef";
                for (const auto byte : code) { code_hex += digits[byte >> 4]; code_hex += digits[byte & 15]; }
                std::array<std::uint8_t, 512> again{};
                if (!read_bytes(code_address, again.data(), again.size()) || again != code) code_hex.clear();
            }
            callbacks.push_back({{"slot_offset",callback.slot_offset},{"function_rva",callback.function_rva},
                {"origin",callback.in_record ? "registration-record" : "parallel-callback-table"},
                {"code_hex",std::move(code_hex)},
                {"callable",false},{"reason","native callback ABI and effects are not resolved"}});
        }
        entries.push_back({{"index",entry.index},{"qualified_name",entry.name},
            {"qualified_hash",entry.hash},{"runtime_type",registration_type_json(entry.storage)},
            {"template_type",registration_type_json(entry.configuration)}, {"runtime_size",entry.size},
            {"flags_bits",entry.flags},{"storage_flags_bits",entry.storage_flags},
            {"storage",entry.storage.name.empty() ? "template-only" : "entity"},
            {"callbacks",std::move(callbacks)}});
    }
    std::scoped_lock operation_lock(operation_mutex);
    std::scoped_lock lock(state_mutex);
    if (configured_types != contract) return;
    component_registry_error = error;
    if (!valid) {
        if (component_registry_ready) types.clear();
        component_registry_ready = false;
        component_registry_world_bound = false;
        component_registry_json.clear();
        return;
    }
    const bool changed = component_registry.owner != snapshot.owner ||
        component_registry.records != snapshot.records || !component_registry_ready || types != resolved;
    types = std::move(resolved);
    component_registry = std::move(snapshot);
    component_registry_ready = true;
    component_registry_world_bound = manager != 0;
    component_registry_seen = true;
    component_registry_json = nlohmann::json({{"schema_version",1},{"layout_version",1},
        {"available",true},{"source","live-engine-registration"},{"count",entries.size()},
        {"discovery",manager ? "active-world-manager" : "resident-private-memory"},
        {"manager_bound",component_registry_world_bound},
        {"runtime_type_count",types.size()},{"entries",std::move(entries)}}).dump();
    if (changed) {
        ++layout_epoch;
        handles.clear();
        reverse_handles.clear();
        // Dispatcher scans invalidate their own epoch lazily. Clearing these
        // maps from this discovery thread races an active engine callback.
    }
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
}

namespace EcsRuntime {
std::string Status() {
    std::scoped_lock discovery_lock(discovery_mutex);
    std::scoped_lock lock(state_mutex);
    std::ostringstream text;
    text << "types=" << types.size() << '/' << configured_types.size()
         << "{reflected_ecs_types=" << configured_types.size()
         << ",registered_storage_types=" << types.size()
         << ",registrations=" << (component_registry_ready ? component_registry.entries.size() : 0) << '}'
         << " component_index_discovery=" << (discovery_complete ? "complete" : "scanning")
         << '(' << discovery_cursor << '/' << discovery_entities.size()
         << ",layouts=" << discovered_layouts.size()
         << ",templates=" << template_layout_samples.size()
         << ",restarts=" << discovery_restarts << ')'
         << " profile=" << KfcRuntimeCompatibility::EnshroudedClient::status
         << " game_thread=" << GameThreadDispatcher::Status()
         << " entity_hooks{" << WorldRuntime::EntityHookStatus() << '}';
    for (const auto& operation : KfcRuntimeCompatibility::EnshroudedClient::runtime_operations)
        text << " world{" << operation.name << '=' << (operation.available ? operation.status : "unavailable") << '}';
    text << " voxel_context=" << (WorldRuntime::ActiveContextAvailable() ? "ready" : "waiting");
    text << " context_probe{" << WorldRuntime::ContextStatus() << '}';
    if (types.empty()) return text.str() + " registry=unresolved layout=unavailable";
    text << " registry=ready";
    if (!layout_ready || (component_registry_ready && !component_registry_world_bound))
        return text.str() + " layout=unavailable entity_access=unavailable(dispatcher-or-world-layout)";
    if (read_only_binding) {
        EngineContext::EcsView current;
        if (!EngineContext::ResolveEcs(current) || current != read_only_view)
            return text.str() + " layout=unavailable entity_access=unavailable(engine-owner-changed)";
        text << " entity_access=" << (EngineContext::EcsWriteConfigured() &&
            read_only_view.role == EngineContext::EcsRole::LocalAuthority ? "guarded-read-write" : "read-only")
            << " entity_realm=" << EngineContext::RoleName(read_only_view.role);
    } else text << " entity_access=" << (GameThreadDispatcher::Ready() ? "read-write" : "unavailable")
        << " entity_realm=engine-observed";
    text << " layout=ready"
         << " entity{id=0x" << std::hex << live_layout.entity_id
         << ",generation=0x" << live_layout.entity_generation
         << ",layout=0x" << live_layout.entity_layout
         << ",storage=0x" << live_layout.entity_storage
         << ",row=0x" << live_layout.entity_row << std::dec << '}'
         << " components{bits=0x" << std::hex << live_layout.component_bits
         << ",offsets=0x" << live_layout.component_offsets
         << ",strides=0x" << live_layout.component_strides << std::dec << '}'
         << " epoch=" << layout_epoch;
    return text.str();
}

std::string Diagnostics() {
    std::scoped_lock discovery_lock(discovery_mutex);
    std::scoped_lock lock(state_mutex);
    EngineContext::EcsView current_read_view;
    const bool current_read_binding = !read_only_binding ||
        (EngineContext::ResolveEcs(current_read_view) && current_read_view == read_only_view);
    std::size_t observed_indices{};
    for (const auto& entry : observed_indices_by_size) observed_indices += entry.second.size();
    nlohmann::json resolved_mappings = nlohmann::json::array();
    nlohmann::json unresolved_types = nlohmann::json::array();
    nlohmann::json stride_candidates = nlohmann::json::array();
    nlohmann::json template_layouts = nlohmann::json::array();
    nlohmann::json world_operations = nlohmann::json::object();
    for (const auto& operation : KfcRuntimeCompatibility::EnshroudedClient::runtime_operations)
        world_operations[operation.name] = {{"available", operation.available}, {"status", operation.status},
            {"abi", operation.abi}, {"thread", operation.thread}, {"context", operation.context}};
    std::unordered_map<std::uint32_t, std::vector<std::string>> names_by_size;
    std::unordered_set<std::string> profile_names;
    std::size_t dynamic_candidates{};
    std::size_t template_types{}, template_only{};
    if (component_registry_ready) for (const auto& entry : component_registry.entries) {
        template_types += !entry.configuration.name.empty();
        template_only += entry.storage.name.empty();
    }
    for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components)
        profile_names.insert(component.qualified_name);
    for (const auto& [name, size] : configured_types) {
        names_by_size[size].push_back(name);
        if (name.starts_with("keen::ecs::Dynamic")) ++dynamic_candidates;
    }
    for (const auto& [name, component] : types) {
        resolved_mappings.push_back({{"name",name},{"index",component.index},{"size",component.size},
            {"source",component_registry_ready ? "live-engine-registration" : "exact-build-profile"}});
    }
    for (const auto& [size, names] : names_by_size) {
        const auto candidate_set = observed_indices_by_size.find(size);
        nlohmann::json indices = nlohmann::json::array();
        if (candidate_set != observed_indices_by_size.end())
            for (const auto index : candidate_set->second) indices.push_back(index);
        stride_candidates.push_back({{"size",size},{"indices",std::move(indices)},
            {"reflectedTypeCount",names.size()}});
        for (const auto& name : names) {
            if (types.contains(name)) continue;
            const auto candidates = candidate_set == observed_indices_by_size.end() ? 0 : candidate_set->second.size();
            const auto reason = component_registry_ready ? "reflected-type-without-entity-storage" :
                names.size() != 1 ? "shared-reflected-size" :
                candidates == 0 ? "no-live-archetype-observed" :
                candidates != 1 ? "multiple-live-indices-for-size" : "index-conflict";
            unresolved_types.push_back({{"name",name},{"size",size},{"reason",reason}});
        }
    }
    for (const auto& [key, sample] : template_layout_samples) {
        (void)key;
        nlohmann::json component_slots = nlohmann::json::array();
        for (const auto& component : sample.components)
            component_slots.push_back({{"index", component.index}, {"stride", component.stride},
                {"offset", component.offset}});
        template_layouts.push_back({{"templateUuidQwords", {sample.uuid[0], sample.uuid[1]}},
            {"templateName", sample.name},
            {"entitiesSeen", sample.entity_count}, {"componentSlots", std::move(component_slots)}});
    }
    return nlohmann::json({
        {"schemaVersion",1}, {"providerAbi",KFC_RUNTIME_ABI_VERSION},
        {"profile",KfcRuntimeCompatibility::EnshroudedClient::status},
        {"kfcFormat",KfcRuntimeCompatibility::EnshroudedClient::kfc_format},
        {"candidateTypeBreakdown",nlohmann::json{
            {"total",configured_types.size()},
            {"otherEcsTypes",configured_types.size() - dynamic_candidates},
            {"dynamicNameTypes",dynamic_candidates},
            {"engineRegistrations",component_registry_ready ? component_registry.entries.size() : 0},
            {"registeredStorageTypes",component_registry_ready ? types.size() : 0},
            {"registeredTemplateTypes",template_types}, {"templateOnlyRegistrations",template_only},
            {"resolved",types.size()},
            {"unresolved",component_registry_ready ? 0 : configured_types.size() - types.size()},
            {"reflectedWithoutEntityStorage",configured_types.size() - types.size()}
        }},
        {"worldOperations",std::move(world_operations)},
        {"voxelContextActive",WorldRuntime::ActiveContextAvailable()},
        {"worldContext",nlohmann::json::parse(WorldRuntime::ContextDiagnostics())},
        {"entityContextReady",WorldRuntime::EntityContextReady()},
        {"entityHookStatus",WorldRuntime::EntityHookStatus()},
        {"configuredTypes",configured_types.size()}, {"resolvedTypes",types.size()},
        {"layoutReady",layout_ready}, {"layoutEpoch",layout_epoch},
        {"registryAvailable",component_registry_ready}, {"registryManagerBound",component_registry_world_bound},
        {"entityReadSource",read_only_binding ? "original-engine-owner-chain" : "game-thread-dispatcher"},
        {"entityReadRole",read_only_binding ? EngineContext::RoleName(read_only_view.role) : "engine-observed"},
        {"originalWriteGateConfigured",EngineContext::EcsWriteConfigured()},
        {"originalWriteGateLastReason",last_write_reason.load(std::memory_order_relaxed)},
        {"writeFault",write_fault_active_locked()},
        {"writeFaultReason",write_fault_active_locked() ? write_fault_reason : "none"},
        {"registryDiscovery",nlohmann::json{
            {"source","resident-private-memory"},
            {"scanning",registry_discovery.Status().scanning},
            {"available",registry_discovery.Status().available},
            {"reason",registry_discovery.Status().reason},
            {"scannedBytes",registry_discovery.Status().scanned_bytes},
            {"skippedNonresidentBytes",registry_discovery.Status().skipped_nonresident_bytes},
            {"validationBytes",registry_discovery.Status().validation_bytes},
            {"budgetLimitedSteps",registry_discovery.Status().budget_limited_steps},
            {"completedPasses",registry_discovery.Status().completed_passes},
            {"distinctRegistries",registry_discovery.Status().distinct_registries},
            {"distinctCatalogs",registry_discovery.Status().distinct_catalogs},
            {"entityAccessAuthorized",false}
        }},
        {"entityAccessReady",layout_ready && !types.empty() &&
            (read_only_binding ? current_read_binding && component_registry_ready && component_registry_world_bound :
                GameThreadDispatcher::Ready() && (!component_registry_ready || component_registry_world_bound))},
        {"componentDiscovery",nlohmann::json{
            {"complete",discovery_complete},
            {"scannedEntities",discovery_scanned_entities},
            {"totalEntities",discovery_entities.size()},
            {"archetypes",discovery_layout_count},
            {"distinctLayoutsSeen",discovered_layouts.size()},
            {"snapshotRestarts",discovery_restarts},
            {"observedStrideBuckets",observed_indices_by_size.size()},
            {"observedIndexCandidates",observed_indices},
            {"templateLayoutSamples",std::move(template_layouts)},
            {"templateLayoutSamplesDropped",template_layout_samples_dropped},
            {"mappingMethod",component_registry_ready ? "live engine registration, reflection and parallel arrays" :
                KfcRuntimeCompatibility::EnshroudedClient::runtime_components.empty() ? "waiting for live engine registration" : "exact build profile"},
            {"registrationReady",component_registry_ready}, {"registrationReason",component_registry_error},
            {"registrationCount",component_registry.entries.size()},
            {"resolved",std::move(resolved_mappings)},
            {"unresolved",component_registry_ready ? nlohmann::json::array() : unresolved_types},
            {"reflectedWithoutEntityStorage",component_registry_ready ? std::move(unresolved_types) : nlohmann::json::array()},
            {"strideCandidates",std::move(stride_candidates)}
        }},
        {"activeQueryScan",nlohmann::json{
            {"cursor",active_query_cursor.load(std::memory_order_relaxed)},
            {"total",active_query_total.load(std::memory_order_relaxed)},
            {"matches",active_query_matches.load(std::memory_order_relaxed)}
        }},
        {"operations", nlohmann::json{
            {"queries", operation_counters.queries.load()},
            {"querySuccesses", operation_counters.query_successes.load()},
            {"queryIncomplete", operation_counters.query_incomplete.load()},
            {"queryFailures", operation_counters.query_failures.load()},
            {"resolves", operation_counters.resolves.load()},
            {"resolveSuccesses", operation_counters.resolve_successes.load()},
            {"resolveFailures", operation_counters.resolve_failures.load()},
            {"reads", operation_counters.reads.load()},
            {"readSuccesses", operation_counters.read_successes.load()},
            {"readFailures", operation_counters.read_failures.load()},
            {"readBatches", operation_counters.read_batches.load()},
            {"readBatchSuccesses", operation_counters.read_batch_successes.load()},
            {"readBatchFailures", operation_counters.read_batch_failures.load()},
            {"readBatchComponents", operation_counters.read_batch_components.load()},
            {"readBatchAbsent", operation_counters.read_batch_absent.load()},
            {"writes", operation_counters.writes.load()},
            {"writeSuccesses", operation_counters.write_successes.load()},
            {"writeFailures", operation_counters.write_failures.load()},
            {"runtimePatches", nlohmann::json::parse(PatchRuntime::Diagnostics())}
        }},
        {"dispatcher",nlohmann::json::parse(GameThreadDispatcher::Diagnostics())}
    }).dump();
}
}

extern "C" bool __cdecl KfcRuntimeEcsConfigure(const char* const* names,
                                                const std::uint32_t* sizes,
                                                std::size_t count) {
    if (!names || !sizes || !count || count > 20'000) return false;
    std::unordered_map<std::string, std::uint32_t> contract;
    for (std::size_t index = 0; index < count; ++index) {
        if (!names[index] || !sizes[index]) return false;
        const std::string name{names[index]};
        if (!name.starts_with("keen::ecs::")) return false;
        contract.emplace(name, sizes[index]);
    }
    std::scoped_lock operation_lock(operation_mutex);
    {
        std::scoped_lock lock(state_mutex);
        if (configured_types == contract) return true;
        configured_types = std::move(contract);
        component_registry = {};
        component_registry_ready = false;
        component_registry_world_bound = false;
        component_registry_seen = false;
        component_registry_last_read = 0;
        component_registry_json.clear();
        types.clear();
        for (const auto& component : KfcRuntimeCompatibility::EnshroudedClient::runtime_components) {
            const auto configured = configured_types.find(std::string(component.qualified_name));
            if (configured != configured_types.end() && configured->second == component.size)
                types.emplace(configured->first, ComponentType{component.index, component.size});
        }
        live_layout = {};
        layout_ready = false;
        read_only_binding = false;
        read_only_view = {};
        ++layout_epoch;
        handles.clear();
        reverse_handles.clear();
    }
    reset_component_discovery();
    return true;
}

extern "C" bool __cdecl KfcRuntimeEcsDescribe(const char* name, std::uint32_t* size) {
    ComponentType component{};
    if (!size || !resolve_component(name, component)) return false;
    *size = component.size;
    return true;
}

extern "C" std::size_t __cdecl KfcRuntimeEcsRegistry(char* buffer, std::size_t capacity) {
    try {
    std::scoped_lock lock(state_mutex);
    const auto json = component_registry_ready ? component_registry_json : nlohmann::json({
        {"schema_version",1},{"available",false},{"reason",component_registry_error},
        {"entries",nlohmann::json::array()}}).dump();
    const auto required = json.size() + 1;
    if (buffer && capacity >= required) std::memcpy(buffer, json.c_str(), required);
    return required;
    } catch (...) { return 0; }
}

extern "C" std::size_t __cdecl KfcRuntimeEcsRegistration(const char* name, char* buffer, std::size_t capacity) {
    if (!name || !*name || strnlen_s(name, 512) == 512) return 0;
    try {
        std::scoped_lock lock(state_mutex);
        nlohmann::json report;
        if (!component_registry_ready) {
            report = {{"schema_version",1},{"available",false},{"reason",component_registry_error},
                {"entries",nlohmann::json::array()}};
        } else {
            auto entries = nlohmann::json::array();
            for (const auto& entry : component_registry.entries) {
                if (entry.name != name && entry.storage.name != name && entry.configuration.name != name) continue;
                entries.push_back({{"index",entry.index},{"qualified_name",entry.name},
                    {"qualified_hash",entry.hash},{"runtime_type",registration_type_json(entry.storage)},
                    {"template_type",registration_type_json(entry.configuration)},{"runtime_size",entry.size},
                    {"flags_bits",entry.flags},{"storage_flags_bits",entry.storage_flags},
                    {"storage",entry.storage.name.empty() ? "template-only" : "entity"}});
            }
            report = {{"schema_version",1},{"layout_version",1},{"layout_epoch",layout_epoch},
                {"available",true},{"source","live-engine-registration"},{"count",entries.size()},
                {"total_count",component_registry.entries.size()},
                {"discovery",component_registry_world_bound ? "active-world-manager" : "resident-private-memory"},
                {"manager_bound",component_registry_world_bound},{"runtime_type_count",types.size()},
                {"entries",std::move(entries)}};
        }
        const auto serialized = report.dump();
        const auto required = serialized.size() + 1;
        if (buffer && capacity >= required) std::memcpy(buffer, serialized.c_str(), required);
        return required;
    } catch (...) { return 0; }
}

extern "C" bool __cdecl KfcRuntimeEcsResolveType(const char* name, char* runtime_name, std::size_t capacity) {
    if (!name || !runtime_name || !capacity) return false;
    std::scoped_lock lock(state_mutex);
    const std::string* resolved{};
    if (component_registry_ready) {
        for (const auto& entry : component_registry.entries) {
            if (entry.name == name || entry.storage.name == name) {
                resolved = &entry.storage.name;
                break;
            }
        }
    } else {
        const auto found = types.find(name);
        if (found != types.end()) resolved = &found->first;
    }
    if (!resolved || resolved->empty() || resolved->size() + 1 > capacity) return false;
    std::memcpy(runtime_name, resolved->c_str(), resolved->size() + 1);
    return true;
}
