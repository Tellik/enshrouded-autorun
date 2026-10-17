#include "engine-context/context_state.h"



using namespace EngineContextImplementation;

namespace EngineContextImplementation {
std::mutex context_mutex;

Layout active_layout;

bool configured{};

nlohmann::json system_dependency_layout;

nlohmann::json service_layout;

bool read_span(std::uintptr_t address, void* output, std::size_t size){
    if (!address || address > UINTPTR_MAX - size) return false;
    __try { std::memcpy(output, reinterpret_cast<const void*>(address), size); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& address){
    if (!base || offset > UINTPTR_MAX - base) return false;
    address = base + offset;
    return true;
}

bool vtable(std::uintptr_t object, std::uintptr_t rva, std::uintptr_t image, std::size_t size){
    std::uintptr_t actual{};
    return rva < size && read(object, actual) && actual == image + rva;
}

bool resolve(const Layout& layout, EngineContext::Owners& output, std::string* reason, std::uintptr_t known_image){
    const auto fail = [&](const char* message) { if (reason) *reason = message; return false; };
    EngineContext::Owners view;
    view.image = known_image ? known_image : reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    if (!view.image || !add(view.image, layout.root_rva, view.root) ||
        !vtable(view.root, layout.root_vtable, view.image, layout.image_size)) return fail("root-vtable-mismatch");
    std::uintptr_t field{}, back{};
    if (!add(view.root, layout.client_offset, field) || !read(field, view.client) ||
        !vtable(view.client, layout.client_vtable, view.image, layout.image_size)) return fail("client-owner-unavailable-or-vtable-mismatch");
    if (!add(view.client, layout.controller_offset, field) || !read(field, view.controller) ||
        !vtable(view.controller, layout.controller_vtable, view.image, layout.image_size)) return fail("client-controller-unavailable-or-vtable-mismatch");
    if (!add(view.controller, layout.controller_back_pointer, field) || !read(field, back) || back != view.client)
        return fail("client-controller-owner-mismatch");
    if (!add(view.controller, layout.main_owner_offset, field) || !read(field, view.main_owner) || !view.main_owner)
        return fail("main-owner-unavailable");
    output = view;
    if (reason) reason->clear();
    return true;
}

bool select_execution_owner(const Layout& layout, EngineContext::EcsView& view, std::string* reason, std::uintptr_t image){
    const auto fail = [&](const char* message) { if (reason) *reason = message; return false; };
    if (!resolve(layout, view.owners, reason, image)) return false;
    auto context = view.owners.main_owner;
    std::uintptr_t field{};
    if (layout.authority) {
        std::uint64_t count{}, capacity{};
        if (!add(view.owners.client, layout.client_scene_offset, field) || !read(field, view.scene_owner) || !view.scene_owner ||
            !add(view.scene_owner, layout.scene_entries_offset, field) || !read(field, view.scene_entries) ||
            !add(view.scene_owner, layout.scene_count_offset, field) || !read(field, count) || count > layout.maximum_scenes ||
            !add(view.scene_owner, layout.scene_capacity_offset, field) || !read(field, capacity) || capacity < count || capacity > layout.maximum_scenes)
            return fail("ecs-active-scene-list-unavailable");
        if (count && (!view.scene_entries || view.scene_entries > UINTPTR_MAX - count * layout.entry_stride))
            return fail("ecs-active-scene-list-range-invalid");
        std::size_t ready{};
        for (std::size_t index = 0; index < count; ++index) {
            const auto entry = view.scene_entries + index * layout.entry_stride;
            std::uint8_t state{};
            if (!add(entry, layout.entry_state_offset, field) || !read(field, state)) return fail("ecs-scene-state-unreadable");
            if (state != layout.entry_ready_state) continue;
            if (++ready > 1) return fail("ecs-local-authority-is-ambiguous");
            view.scene_entry = entry;
            if (!add(entry, layout.entry_context_offset, field) || !read(field, view.scene_context) || !view.scene_context ||
                !add(entry, layout.entry_uuid_offset, field) || !read(field, view.scene_uuid) || !(view.scene_uuid[0] || view.scene_uuid[1]))
                return fail("ecs-ready-scene-context-invalid");
        }
        if (count && !ready) return fail("ecs-active-scene-is-transitioning");
        if (ready) {
            view.role = EngineContext::EcsRole::LocalAuthority;
            context = view.scene_context;
        }
        std::uintptr_t entries_again{};
        std::uint64_t count_again{}, capacity_again{};
        if (!add(view.scene_owner, layout.scene_entries_offset, field) || !read(field, entries_again) || entries_again != view.scene_entries ||
            !add(view.scene_owner, layout.scene_count_offset, field) || !read(field, count_again) || count_again != count ||
            !add(view.scene_owner, layout.scene_capacity_offset, field) || !read(field, capacity_again) || capacity_again != capacity)
            return fail("ecs-active-scene-list-changed");
    }
    if (!add(context, layout.execution_owner_offset, field) || !read(field, view.execution_owner) || !view.execution_owner)
        return fail("ecs-execution-owner-unavailable");
    return true;
}

bool resolve_ecs(const Layout& layout, EngineContext::EcsView& output, std::string* reason, std::uintptr_t image){
    const auto fail = [&](const char* message) { if (reason) *reason = message; return false; };
    EngineContext::EcsView view;
    std::uintptr_t manager_owner{}, table{};
    std::uint64_t count{};
    if (!select_execution_owner(layout, view, reason, image)) return false;
    if (!add(view.execution_owner, layout.registry_owner_offset, view.registry_owner) ||
        !add(view.execution_owner, layout.entity_manager_offset, view.manager) ||
        !field(view.manager, layout.manager_owner_offset, manager_owner) || manager_owner != view.registry_owner)
        return fail("ecs-manager-registry-owner-mismatch");
    if (!field(view.manager, layout.entities.manager_count, count) || !count || count > layout.entities.maximum_entities ||
        !field(view.manager, layout.entities.manager_table, table) || !table)
        return fail("ecs-entity-table-unavailable");
    if (table > UINTPTR_MAX - count * sizeof(std::uintptr_t)) return fail("ecs-entity-table-range-overflow");
    view.entity_table = table;
    EngineContext::EcsView again;
    std::uintptr_t owner_again{}, table_again{};
    std::uint64_t count_again{};
    if (!select_execution_owner(layout, again, reason, image) || again.owners != view.owners || again.role != view.role ||
        again.scene_owner != view.scene_owner || again.scene_entries != view.scene_entries || again.scene_entry != view.scene_entry ||
        again.scene_context != view.scene_context || again.scene_uuid != view.scene_uuid || again.execution_owner != view.execution_owner ||
        !field(view.manager, layout.manager_owner_offset, owner_again) || owner_again != view.registry_owner ||
        !field(view.manager, layout.entities.manager_count, count_again) || count_again != count ||
        !field(view.manager, layout.entities.manager_table, table_again) || table_again != table)
        return fail("ecs-engine-owner-changed-during-read");
    view.layout = layout.entities;
    output = view;
    if (reason) reason->clear();
    return true;
}
}

namespace EngineContext {
bool Configure(const nlohmann::json& profile, std::string& error) {
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    try {
        const auto& identity = profile.at("image");
        if (!image_size || image_sha256.empty() || profile.at("target") != image_target ||
            identity.at("sha256") != image_sha256 || identity.at("timestamp") != image_timestamp ||
            identity.at("size") != image_size)
            throw std::runtime_error("engine owner profile does not match the exact loaded image");
        const auto& owners = profile.at("ownerLayout");
        Layout layout;
        layout.image_size = image_size;
        auto offset = [](const nlohmann::json& object, const char* key, std::size_t maximum = 0x4000000) {
            const auto value = object.at(key).get<std::size_t>();
            if (value > maximum) throw std::runtime_error(std::string("engine owner offset out of range: ") + key);
            return value;
        };
        layout.root_rva = offset(owners, "rootObjectRva", image_size - 1);
        layout.root_vtable = offset(owners, "rootVtableRva", image_size - 1);
        layout.client_offset = offset(owners, "clientPointerOffset");
        layout.client_vtable = offset(owners, "clientVtableRva", image_size - 1);
        layout.controller_offset = offset(owners, "controllerPointerOffset");
        layout.controller_vtable = offset(owners, "controllerVtableRva", image_size - 1);
        layout.controller_back_pointer = offset(owners, "controllerClientBackPointerOffset");
        layout.main_owner_offset = offset(owners, "controllerMainOwnerOffset");
        nlohmann::json dependencies;
        if (profile.contains("systemDependencyLayout")) {
            if (!profile.at("provenance").value("systemDependenciesValidated", false))
                throw std::runtime_error("system dependencies lack original scheduler validation");
            const auto& model = profile.at("systemDependencyLayout");
            const auto& slots = model.at("slots");
            const auto source = model.at("source").get<std::string>();
            if (model.at("id") != "ecs-system-dependencies-v1" || source.empty() || !slots.is_array() || slots.size() != 6)
                throw std::runtime_error("invalid system dependency model");
            auto output = nlohmann::json::array();
            std::uint32_t offsets{};
            for (const auto& slot : slots) {
                const auto position = slot.at("offset").get<unsigned>();
                const auto role = slot.at("storageRole").get<std::string>();
                const auto access = slot.at("accessMode").get<std::string>();
                if (position < 24 || position > 104 || (position - 24) % 16 ||
                    (offsets & (1u << ((position - 24) / 16))) ||
                    (role != "runtime" && role != "template") ||
                    (role == "template" ? access != "configuration-read" : (access != "read" && access != "write")))
                    throw std::runtime_error("invalid or duplicate system dependency slot");
                offsets |= 1u << ((position - 24) / 16);
                output.push_back({{"offset",position},{"storage_role",role},{"access_mode",access},
                    {"system_record_mask_offset",offset(slot,"systemRecordMaskOffset",0x10000)}});
            }
            dependencies = {{"id",model.at("id")},{"source",source},{"validated",true},
                {"image",{{"target",image_target},{"sha256",image_sha256},{"timestamp",image_timestamp},{"size",image_size}}},
                {"record_builder_rva",offset(model,"recordBuilderRva",image_size - 1)},
                {"access_set_pack_rva",offset(model,"accessSetPackRva",image_size - 1)},
                {"compatibility_predicate_rva",offset(model,"compatibilityPredicateRva",image_size - 1)},
                {"optionality",nullptr},{"slots",std::move(output)}};
        }
        if (profile.contains("ecsReadLayout")) {
            if (!profile.at("provenance").value("ecsReadValidated", false))
                throw std::runtime_error("ECS owner layout lacks read validation");
            const auto& ecs = profile.at("ecsReadLayout");
            layout.execution_owner_offset = offset(ecs, "mainExecutionOwnerOffset");
            layout.registry_owner_offset = offset(ecs, "executionRegistryOwnerOffset");
            layout.entity_manager_offset = offset(ecs, "executionEntityManagerOffset");
            layout.manager_owner_offset = offset(ecs, "entityManagerOwnerOffset");
            auto& e = layout.entities;
            e.manager_count = offset(ecs, "entityManagerCount"); e.manager_table = offset(ecs, "entityManagerTable");
            if (ecs.contains("entityManagerOccupancy") != ecs.contains("entityManagerKeys"))
                throw std::runtime_error("ECS entity hash membership needs both occupancy and keys offsets");
            if (ecs.contains("entityManagerOccupancy")) {
                e.manager_occupancy = offset(ecs, "entityManagerOccupancy");
                e.manager_keys = offset(ecs, "entityManagerKeys");
                if (!e.manager_occupancy || !e.manager_keys)
                    throw std::runtime_error("ECS entity hash membership offsets are zero");
            }
            e.entity_id = offset(ecs, "entityId"); e.entity_generation = offset(ecs, "entityGeneration");
            e.entity_layout = offset(ecs, "entityLayout"); e.entity_storage = offset(ecs, "entityStorage");
            e.entity_definition = offset(ecs, "entityDefinitionPointer"); e.entity_row = offset(ecs, "entityRow");
            e.entity_definition_uuid_validated = ecs.contains("entityDefinitionUuidOffset");
            if (e.entity_definition_uuid_validated)
                e.entity_definition_uuid = offset(ecs, "entityDefinitionUuidOffset", 65536);
            e.component_bits = offset(ecs, "componentBits"); e.component_offsets = offset(ecs, "componentOffsets");
            e.component_strides = offset(ecs, "componentStrides");
            e.maximum_entities = offset(ecs, "maximumEntities", 1u << 20);
            if (!e.maximum_entities) throw std::runtime_error("engine entity limit is zero");
            if (ecs.contains("clientSceneOwnerOffset")) {
                if (!profile.at("provenance").value("ecsAuthorityReadValidated", false))
                    throw std::runtime_error("local authority owner layout lacks read validation");
                layout.client_scene_offset = offset(ecs, "clientSceneOwnerOffset");
                layout.scene_entries_offset = offset(ecs, "sceneEntriesPointerOffset");
                layout.scene_count_offset = offset(ecs, "sceneEntriesCountOffset");
                layout.scene_capacity_offset = offset(ecs, "sceneEntriesCapacityOffset");
                layout.entry_stride = offset(ecs, "sceneEntryStride", 0x10000);
                layout.entry_context_offset = offset(ecs, "sceneEntryContextOffset");
                layout.entry_uuid_offset = offset(ecs, "sceneEntryUuidOffset");
                layout.entry_state_offset = offset(ecs, "sceneEntryStateOffset");
                layout.entry_ready_state = static_cast<std::uint8_t>(offset(ecs, "sceneEntryReadyState", 255));
                layout.maximum_scenes = offset(ecs, "maximumSceneEntries", 4096);
                if (!layout.maximum_scenes || layout.entry_stride < 24 ||
                    layout.entry_context_offset > layout.entry_stride - 8 || layout.entry_uuid_offset > layout.entry_stride - 16 ||
                    layout.entry_state_offset >= layout.entry_stride) throw std::runtime_error("invalid active scene entry layout");
                layout.authority = true;
            }
            layout.ecs = true;
        }
        if (profile.contains("ecsWriteLayout")) {
            const auto& model = profile.at("ecsWriteLayout");
            const auto& provenance = profile.at("provenance");
            if (!layout.authority || model.at("id") != "ecs-native-thread-scene-gate-v1" ||
                !provenance.value("ecsWriteGateValidated", false))
                throw std::runtime_error("ECS write gate lacks original authority/lifetime validation");
            auto& w = layout.write;
            w.enabled = model.value("enabled", false);
            if (w.enabled && !provenance.value("ecsChangeTrackingValidated", false))
                throw std::runtime_error("ECS writes lack change tracking validation");
            if (w.enabled && !model.value("phaseValidated", false))
                throw std::runtime_error("ECS writes lack original command/epoch phase validation");
            if (model.value("phasePolicy", std::string{}) != "untracked-components")
                throw std::runtime_error("ECS write phase policy must be untracked-components");
            w.registry_lock = offset(model, "threadRegistryLockRva", image_size - sizeof(LONG));
            w.registry_entries = offset(model, "threadRegistryEntriesRva", image_size - 1);
            w.registry_count = offset(model, "threadRegistryCountRva", image_size - sizeof(std::uint16_t));
            w.callback = offset(model, "threadCallbackRva", image_size - 1);
            w.capacity = offset(model, "threadRegistryCapacity", 256);
            w.stride = offset(model, "threadRegistryEntryStride", 256);
            w.row_handle = offset(model, "registryEntryHandleOffset", 256);
            w.row_name = offset(model, "registryEntryNamePointerOffset", 256);
            w.row_length = offset(model, "registryEntryNameLengthOffset", 256);
            w.inline_name = offset(model, "threadInlineNameOffset", 65536);
            w.scene_thread = offset(model, "sceneThreadOffset", 65536);
            w.name_pointer = offset(model, "threadNamePointerOffset", 65536);
            w.name_length = offset(model, "threadNameLengthOffset", 65536);
            w.name_capacity = offset(model, "threadNameCapacityOffset", 65536);
            w.name_capacity_value = offset(model, "threadNameCapacity", w.name.size());
            w.thread_handle = offset(model, "threadHandleOffset", 65536);
            w.thread_context = offset(model, "threadContextOffset", 65536);
            w.thread_stop = offset(model, "threadStopOffset", 65536);
            w.thread_callback = offset(model, "threadCallbackOffset", 65536);
            w.scene_context = offset(model, "sceneContextOffset", 65536);
            w.scene_lock = offset(model, "sceneLockOffset", 65536);
            w.scene_state = offset(model, "sceneStateOffset", 65536);
            w.running_state = static_cast<std::uint8_t>(offset(model, "sceneRunningState", 255));
            w.execution_mode = offset(model, "executionModeOffset");
            w.authority_mode = static_cast<std::uint8_t>(offset(model, "executionAuthorityMode", 255));
            w.maximum_bytes = offset(model, "maximumWriteBytes", 65536);
            w.tracker = offset(model, "changeTrackerRva", image_size - w.tracker_bytes.size());
            w.tracking_context = offset(model, "changeTrackerContextOffset");
            w.tracking_manager = offset(model, "changeTrackerManagerOffset", 65536);
            w.tracking_queue_b = offset(model, "changeTrackerQueueBOffset", 65536);
            w.tracking_registry_a = offset(model, "changeTrackerRegistryAOffset", 65536);
            w.tracking_registry_b = offset(model, "changeTrackerRegistryBOffset", 65536);
            w.maximum_tracking_capacity = offset(model, "maximumTrackingQueueCapacity", 1u << 20);
            const auto tracker_bytes = model.at("changeTrackerBytes").get<std::string>();
            if (tracker_bytes.size() != w.tracker_bytes.size() * 2 || !w.tracker || !w.maximum_tracking_capacity ||
                !w.tracking_registry_a || !w.tracking_registry_b || w.tracking_registry_a == w.tracking_registry_b)
                throw std::runtime_error("invalid original ECS change tracker");
            auto nibble = [](char character) -> unsigned {
                if (character >= '0' && character <= '9') return character - '0';
                if (character >= 'a' && character <= 'f') return character - 'a' + 10;
                throw std::runtime_error("invalid change tracker guard bytes");
            };
            for (std::size_t index = 0; index < w.tracker_bytes.size(); ++index)
                w.tracker_bytes[index] = static_cast<std::uint8_t>((nibble(tracker_bytes[index * 2]) << 4) |
                    nibble(tracker_bytes[index * 2 + 1]));
            const auto name = model.at("threadName").get<std::string>();
            if (!w.capacity || w.stride < 24 || !w.maximum_bytes || !w.registry_lock || w.registry_lock % alignof(LONG) ||
                w.capacity * w.stride > image_size || w.registry_entries > image_size - w.capacity * w.stride ||
                w.row_handle > w.stride - 8 || w.row_name > w.stride - 8 || w.row_length > w.stride - 8 ||
                name.empty() || name.size() > w.name_capacity_value || name.find('\0') != std::string::npos ||
                w.name_capacity_value > w.name.size() || !w.inline_name)
                throw std::runtime_error("invalid native-thread ECS write gate layout");
            std::copy(name.begin(), name.end(), w.name.begin());
            w.name_size = name.size();
        }
        if (profile.contains("resourceWriteLayout")) {
            const auto& model = profile.at("resourceWriteLayout");
            auto& r = layout.resource_write;
            if (model.at("id") != "resource-atomic-scalar-v1" || !profile.contains("ecsWriteLayout") ||
                !profile.at("provenance").value("resourceAtomicScalarWriteValidated", false))
                throw std::runtime_error("resource scalar writes lack original worker lifetime validation");
            r.enabled = model.value("enabled", false);
            r.service_thread = offset(model, "serviceThreadOffset", 65536);
            r.callback = offset(model, "threadCallbackRva", image_size - 1);
            r.initialized = offset(model, "serviceInitializedOffset", 65536);
            r.initialized_value = model.at("serviceInitializedValue").get<std::uint32_t>();
            r.reader = offset(profile.at("resourceLayout"), "resourceServiceReaderOffset", 65536);
            r.payload_pointer = offset(model, "servicePayloadPointerOffset", 65536);
            r.payload_size = offset(model, "servicePayloadSizeOffset", 65536);
            const auto name = model.at("threadName").get<std::string>();
            if (!r.service_thread || !r.callback || r.initialized_value != 1 || name.empty() ||
                name.size() > layout.write.name_capacity_value || name.find('\0') != std::string::npos)
                throw std::runtime_error("invalid resource content worker layout");
            std::copy(name.begin(), name.end(), r.name.begin());
            r.name_size = name.size();
        }
        auto services = profile.value("serviceLayout", nlohmann::json{});
        if (!services.is_null() &&
            (!layout.write.enabled || !profile.at("provenance").value("servicePlainDataValidated", false)))
            throw std::runtime_error("named service data lacks original mutable/lifetime validation");
        std::scoped_lock lock(context_mutex);
        active_layout = layout;
        system_dependency_layout = std::move(dependencies);
        service_layout = std::move(services);
        configured = true;
        error.clear();
        return true;
    } catch (const std::exception& exception) {
        Shutdown();
        error = exception.what();
        return false;
    }
}

void Shutdown() {
    std::scoped_lock lock(context_mutex);
    configured = false;
    active_layout = {};
    system_dependency_layout = nullptr;
    service_layout = nullptr;
}

nlohmann::json SystemDependencyLayout() {
    std::scoped_lock lock(context_mutex);
    return configured ? system_dependency_layout : nlohmann::json{};
}

nlohmann::json ServiceLayout() {
    std::scoped_lock lock(context_mutex);
    return configured ? service_layout : nlohmann::json{};
}

const char* RoleName(EcsRole role) { return role == EcsRole::LocalAuthority ? "local-authority" : "client-replica"; }

bool ResolveOwners(Owners& owners, std::string* reason) {
    Layout layout;
    { std::scoped_lock lock(context_mutex); if (!configured) { if (reason) *reason = "engine-owner-profile-unavailable"; return false; } layout = active_layout; }
    return resolve(layout, owners, reason);
}

bool ResolveEcs(EcsView& output, std::string* reason) {
    Layout layout;
    { std::scoped_lock lock(context_mutex); if (!configured || !active_layout.ecs) { if (reason) *reason = "ecs-read-owner-profile-unavailable"; return false; } layout = active_layout; }
    return resolve_ecs(layout, output, reason);
}

bool EcsWriteConfigured() {
    std::scoped_lock lock(context_mutex);
    return configured && active_layout.write.enabled;
}

bool ResourceWriteConfigured() {
    std::scoped_lock lock(context_mutex);
    return configured && active_layout.resource_write.enabled;
}
}
