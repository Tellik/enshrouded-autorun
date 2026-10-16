#include "engine_context.h"
#include "profile.h"
#include <windows.h>
#include <algorithm>
#include <cstring>
#include <limits>
#include <mutex>
#include <stdexcept>
#include <nlohmann/json.hpp>

namespace {
struct WriteLayout {
    bool enabled{};
    std::uintptr_t registry_lock{}, registry_entries{}, registry_count{}, callback{};
    std::size_t capacity{}, stride{}, row_handle{}, row_name{}, row_length{};
    std::size_t inline_name{}, scene_thread{}, name_pointer{}, name_length{}, name_capacity{}, name_capacity_value{};
    std::size_t thread_handle{}, thread_context{}, thread_stop{}, thread_callback{};
    std::size_t scene_context{}, scene_lock{}, scene_state{}, execution_mode{}, maximum_bytes{};
    std::uint8_t running_state{}, authority_mode{};
    std::array<char, 64> name{};
    std::size_t name_size{};
    std::uintptr_t tracker{};
    std::size_t tracking_context{}, tracking_manager{}, tracking_queue_b{}, maximum_tracking_capacity{};
    std::size_t tracking_registry_a{}, tracking_registry_b{};
    std::array<std::uint8_t, 33> tracker_bytes{};
};
struct ResourceWriteLayout {
    bool enabled{};
    std::size_t service_thread{}, initialized{}, reader{}, payload_pointer{}, payload_size{};
    std::uint32_t initialized_value{};
    std::uintptr_t callback{};
    std::array<char, 64> name{};
    std::size_t name_size{};
};
struct Layout {
    std::size_t image_size{};
    std::uintptr_t root_rva{}, root_vtable{}, client_vtable{}, controller_vtable{};
    std::size_t client_offset{}, controller_offset{}, controller_back_pointer{}, main_owner_offset{};
    bool ecs{};
    std::size_t execution_owner_offset{}, registry_owner_offset{}, entity_manager_offset{}, manager_owner_offset{};
    bool authority{};
    std::size_t client_scene_offset{}, scene_entries_offset{}, scene_count_offset{}, scene_capacity_offset{};
    std::size_t entry_stride{}, entry_context_offset{}, entry_uuid_offset{}, entry_state_offset{}, maximum_scenes{};
    std::uint8_t entry_ready_state{};
    EngineContext::EcsLayout entities;
    WriteLayout write;
    ResourceWriteLayout resource_write;
};
std::mutex context_mutex;
Layout active_layout;
bool configured{};
nlohmann::json system_dependency_layout;
nlohmann::json service_layout;

template<class T> bool read(std::uintptr_t address, T& value) {
    if (!address || address > UINTPTR_MAX - sizeof(value)) return false;
    __try { std::memcpy(&value, reinterpret_cast<const void*>(address), sizeof(value)); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
bool read_span(std::uintptr_t address, void* output, std::size_t size) {
    if (!address || address > UINTPTR_MAX - size) return false;
    __try { std::memcpy(output, reinterpret_cast<const void*>(address), size); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& address) {
    if (!base || offset > UINTPTR_MAX - base) return false;
    address = base + offset;
    return true;
}
template<class T> bool field(std::uintptr_t base, std::size_t offset, T& value) {
    std::uintptr_t address{};
    return add(base, offset, address) && read(address, value);
}
bool untracked_component(std::uintptr_t owner, std::uint16_t index,
                         std::size_t mask_a, std::size_t mask_b) {
    // ComponentRegistry owner layout v1 owns the original registration count at
    // +16. The policy is by current registration index, never a type-name list.
    std::uint64_t count{}, count_again{}, a{}, b{}, a_again{}, b_again{};
    const auto word = static_cast<std::size_t>(index / 64) * sizeof(std::uint64_t);
    if (!owner || !mask_a || !mask_b || !field(owner, 16, count) || !count || count > 1024 || index >= count ||
        !field(owner, mask_a + word, a) || !field(owner, mask_b + word, b) ||
        !field(owner, mask_a + word, a_again) || !field(owner, mask_b + word, b_again) ||
        !field(owner, 16, count_again) || count_again != count || a_again != a || b_again != b)
        return false;
    return ((a | b) & (std::uint64_t{1} << (index % 64))) == 0;
}
bool try_shared(std::uintptr_t address) {
    if (!address || address % alignof(LONG)) return false;
    __try {
        auto* lock = reinterpret_cast<volatile LONG*>(address);
        const LONG current = *lock;
        return current >= 0 && current < 0x7fffffff &&
            InterlockedCompareExchange(lock, current + 1, current) == current;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
bool try_exclusive(std::uintptr_t address) {
    if (!address || address % alignof(LONG)) return false;
    __try {
        return InterlockedCompareExchange(reinterpret_cast<volatile LONG*>(address),
            static_cast<LONG>(0x80000000u), 0) == 0;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
void release_shared(std::uintptr_t address) noexcept {
    __try { InterlockedDecrement(reinterpret_cast<volatile LONG*>(address)); }
    __except (EXCEPTION_EXECUTE_HANDLER) {}
}
void release_exclusive(std::uintptr_t address) noexcept {
    __try { InterlockedExchange(reinterpret_cast<volatile LONG*>(address), 0); }
    __except (EXCEPTION_EXECUTE_HANDLER) {}
}
bool tracking_call(std::uintptr_t function, const std::array<std::uint8_t, 33>& expected,
                   std::uintptr_t manager, std::uintptr_t context, std::uintptr_t entity, std::uint16_t index) {
    std::array<std::uint8_t, 33> bytes{};
    if (!read_span(function, bytes.data(), bytes.size()) || bytes != expected) return false;
    using Callback = void(__fastcall*)(std::uintptr_t, std::uintptr_t, std::uintptr_t, std::uint16_t, std::uint8_t);
    __try { reinterpret_cast<Callback>(function)(manager, context, entity, index, 0); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
bool vtable(std::uintptr_t object, std::uintptr_t rva, std::uintptr_t image, std::size_t size) {
    std::uintptr_t actual{};
    return rva < size && read(object, actual) && actual == image + rva;
}
bool resolve(const Layout& layout, EngineContext::Owners& output, std::string* reason, std::uintptr_t known_image = 0) {
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
bool select_execution_owner(const Layout& layout, EngineContext::EcsView& view, std::string* reason, std::uintptr_t image = 0) {
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
bool resolve_ecs(const Layout& layout, EngineContext::EcsView& output, std::string* reason, std::uintptr_t image = 0) {
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
struct ResourceLifeGuard::State {
    Layout layout;
    Owners owners;
    std::uintptr_t registry_lock{}, service{}, reader{}, thread{}, handle{}, payload{};
    std::uint64_t payload_size{};
    bool owns_registry{};
};
ResourceLifeGuard::ResourceLifeGuard() = default;
ResourceLifeGuard::~ResourceLifeGuard() { Release(); }
void ResourceLifeGuard::Release() noexcept {
    if (state_ && state_->registry_lock && state_->owns_registry) {
        release_shared(state_->registry_lock);
    }
    // Retain prepared storage until destruction or the next Prepare, so a
    // rejected borrowed acquisition never frees heap storage under Scene.
    if (state_) { state_->registry_lock = 0; state_->owns_registry = false; }
}
bool ResourceLifeGuard::Validate(Owners& owners) const {
    if (!state_ || !state_->registry_lock) return false;
    const auto& s = *state_;
    const auto& w = s.layout.write;
    const auto& r = s.layout.resource_write;
    std::uintptr_t name{}, handle{}, callback{}, context{}, reader{}, payload{};
    std::uint64_t length{}, capacity{}, payload_size{};
    std::uint32_t stop{}, initialized{};
    Owners current;
    if (!field(s.thread, w.name_pointer, name) || name != s.thread + w.inline_name ||
        !field(s.thread, w.name_length, length) || length != r.name_size ||
        !field(s.thread, w.name_capacity, capacity) || capacity != w.name_capacity_value ||
        !field(s.thread, w.thread_handle, handle) || handle != s.handle ||
        !field(s.thread, w.thread_callback, callback) || callback != s.owners.image + r.callback ||
        !field(s.thread, w.thread_context, context) || context != s.thread ||
        !field(s.thread, w.thread_stop, stop) || stop != 0 ||
        !field(s.service, r.initialized, initialized) || initialized != r.initialized_value ||
        !field(s.service, r.reader, reader) || reader != s.reader ||
        !field(s.service, r.payload_pointer, payload) || payload != s.payload ||
        !field(s.service, r.payload_size, payload_size) || payload_size != s.payload_size ||
        !resolve(s.layout, current, nullptr, s.owners.image) || current != s.owners) return false;
    owners = current;
    return true;
}
bool ResourceLifeGuard::Contains(std::uintptr_t address, std::size_t size) const {
    return state_ && state_->registry_lock && address >= state_->payload &&
        address - state_->payload <= state_->payload_size &&
        size <= state_->payload_size - (address - state_->payload);
}
std::uintptr_t ResourceLifeGuard::PayloadBase() const { return state_ ? state_->payload : 0; }
bool ResourceLifeGuard::TryAcquire(std::uintptr_t expected_service, std::uintptr_t expected_reader) {
    return Prepare() && AcquirePrepared(expected_service, expected_reader, 0);
}
bool ResourceLifeGuard::Prepare() {
    if (state_ && state_->registry_lock) return false;
    // Allocate/snapshot before the original Shared guard. Its critical section
    // never takes a caller mutex, allocates, waits or invokes engine code.
    auto state = std::make_unique<State>();
    {
        std::scoped_lock lock(context_mutex);
        if (!configured || !active_layout.resource_write.enabled) return false;
        state->layout = active_layout;
    }
    if (!resolve(state->layout, state->owners, nullptr)) return false;
    state_ = std::move(state);
    return true;
}
bool ResourceLifeGuard::TryAcquire(const EcsWriteGuard& scene, std::uintptr_t expected_service,
    std::uintptr_t expected_reader) {
    if (!state_ || !scene.scene_lock_ || !scene.registry_lock_ || !scene.Validate()) return false;
    return AcquirePrepared(expected_service, expected_reader, scene.registry_lock_);
}
bool ResourceLifeGuard::AcquirePrepared(std::uintptr_t expected_service, std::uintptr_t expected_reader,
    std::uintptr_t borrowed_registry) {
    if (!state_ || state_->registry_lock || !expected_service || !expected_reader) return false;
    const auto& w = state_->layout.write;
    const auto& r = state_->layout.resource_write;
    const auto image = state_->owners.image;
    std::uintptr_t lock{};
    if (!add(image, w.registry_lock, lock) ||
        (borrowed_registry ? borrowed_registry != lock : !try_shared(lock))) return false;
    state_->registry_lock = lock;
    state_->owns_registry = !borrowed_registry;
    const auto fail = [&] { Release(); return false; };
    std::uint16_t count{};
    if (!field(image, w.registry_count, count) || count > w.capacity) return fail();
    unsigned matches{};
    for (std::size_t index = 0; index < count; ++index) {
        const auto row = image + w.registry_entries + index * w.stride;
        std::uintptr_t name{}, handle{};
        std::uint64_t length{};
        if (!field(row, w.row_length, length) || !field(row, w.row_name, name) || !field(row, w.row_handle, handle))
            return fail();
        if (length != r.name_size) continue;
        std::array<char, 64> text{};
        if (!read_span(name, text.data(), length)) return fail();
        if (std::memcmp(text.data(), r.name.data(), length)) continue;
        if (++matches != 1 || !handle || name < w.inline_name || name - w.inline_name < r.service_thread)
            return fail();
        state_->thread = name - w.inline_name;
        state_->service = state_->thread - r.service_thread;
        state_->handle = handle;
    }
    if (matches != 1 || state_->service != expected_service) return fail();
    state_->reader = expected_reader;
    if (!field(state_->service, r.payload_pointer, state_->payload) || !state_->payload ||
        !field(state_->service, r.payload_size, state_->payload_size) || !state_->payload_size ||
        state_->payload_size > UINTPTR_MAX - state_->payload) return fail();
    Owners owners;
    if (!Validate(owners)) return fail();
    return true;
}
bool EcsComponentWriteSupported(const EcsView& expected, std::uint16_t index) {
    Layout layout;
    {
        std::scoped_lock lock(context_mutex);
        if (!configured || !active_layout.write.enabled) return false;
        layout = active_layout;
    }
    if (expected.role != EcsRole::LocalAuthority) return false;
    EcsView current, again;
    const auto& w = layout.write;
    return resolve_ecs(layout, current, nullptr) && current == expected &&
        untracked_component(current.registry_owner, index, w.tracking_registry_a, w.tracking_registry_b) &&
        resolve_ecs(layout, again, nullptr) && again == current &&
        untracked_component(again.registry_owner, index, w.tracking_registry_a, w.tracking_registry_b);
}
struct EcsWriteGuard::State {
    Layout layout;
    EcsView expected;
    std::uintptr_t thread{}, handle{};
};
EcsWriteGuard::EcsWriteGuard() = default;
EcsWriteGuard::~EcsWriteGuard() { Release(); }
void EcsWriteGuard::Release() noexcept {
    if (scene_lock_) { release_exclusive(scene_lock_); scene_lock_ = 0; }
    if (registry_lock_) { release_shared(registry_lock_); registry_lock_ = 0; }
    maximum_bytes_ = 0;
    manager_ = tracker_ = tracking_context_ = registry_owner_ = 0;
    state_.reset();
}
bool EcsWriteGuard::Validate() const {
    if (!state_ || !registry_lock_ || !scene_lock_) return false;
    EcsView current;
    std::uint8_t mode{};
    const auto& s = *state_;
    const auto& w = s.layout.write;
    std::uintptr_t name{}, handle{}, callback{}, context{}, wanted_context{};
    std::uint64_t length{}, capacity{};
    std::uint32_t stop{};
    std::uint8_t running{};
    return field(s.thread, w.name_pointer, name) && name == s.thread + w.inline_name &&
        field(s.thread, w.name_length, length) && length == w.name_size &&
        field(s.thread, w.name_capacity, capacity) && capacity == w.name_capacity_value &&
        field(s.thread, w.thread_handle, handle) && handle == s.handle &&
        field(s.thread, w.thread_callback, callback) && callback == s.expected.owners.image + w.callback &&
        add(s.expected.scene_owner, w.scene_context, wanted_context) &&
        field(s.thread, w.thread_context, context) && context == wanted_context &&
        field(s.thread, w.thread_stop, stop) && stop == 0 &&
        field(s.expected.scene_owner, w.scene_state, running) && running == w.running_state &&
        resolve_ecs(s.layout, current, nullptr, s.expected.owners.image) &&
        current == state_->expected &&
        field(current.execution_owner, w.execution_mode, mode) && mode == w.authority_mode;
}
bool EcsWriteGuard::ComponentSupported(std::uint16_t component_index) const {
    if (!registry_lock_ || !scene_lock_ || component_index >= 1024) return false;
    std::uintptr_t owner{};
    return field(manager_, 0, owner) && owner == registry_owner_ &&
        untracked_component(owner, component_index, tracking_registry_a_, tracking_registry_b_);
}
bool EcsWriteGuard::TrackChange(std::uintptr_t entity, std::uint16_t component_index) const {
    std::uintptr_t manager{};
    if (!entity || !ComponentSupported(component_index) ||
        !field(tracking_context_, tracking_manager_offset_, manager) || manager != manager_) return false;
    // Both tracking category bits are absent. The original recorder has no
    // queue, stamp, alias traversal or allocator effects for this component.
    // Repeat the policy after the separately guarded original entry returns.
    return tracking_call(tracker_, tracker_bytes_, manager_, tracking_context_, entity, component_index) &&
        ComponentSupported(component_index);
}
bool EcsWriteGuard::TryAcquire(const EcsView& expected, const char** reason) {
    const auto fail = [&](const char* message) {
        Release();
        if (reason) *reason = message;
        return false;
    };
    if (registry_lock_ || scene_lock_) {
        if (reason) *reason = "ecs-write-guard-already-held";
        return false;
    }
    Layout layout;
    {
        std::scoped_lock lock(context_mutex);
        if (!configured || !active_layout.write.enabled) return fail("ecs-write-gate-disabled");
        layout = active_layout;
    }
    auto prepared = std::make_unique<State>();
    prepared->layout = layout;
    prepared->expected = expected;
    const auto& w = layout.write;
    const auto image = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    if (!image || expected.owners.image != image || expected.role != EcsRole::LocalAuthority)
        return fail("ecs-write-requires-current-local-authority");
    std::uintptr_t address{};
    if (!add(image, w.registry_lock, address) || !try_shared(address))
        return fail("ecs-thread-registry-busy");
    registry_lock_ = address;
    // No allocation, caller mutex acquisition or queue wait from this point
    // until Release. Only TrackChange may enter its validated native recorder.
    // The image-global guard prevents unregister and
    // Scene free before the borrowed Scene lock is touched.
    std::uint16_t count{};
    if (!field(image, w.registry_count, count) || count > w.capacity)
        return fail("ecs-thread-registry-count-invalid");
    std::uintptr_t selected_thread{}, selected_scene{}, selected_handle{};
    unsigned matches{};
    for (std::size_t index = 0; index < count; ++index) {
        const auto row = image + w.registry_entries + index * w.stride;
        std::uintptr_t name{}, handle{};
        std::uint64_t length{};
        if (!field(row, w.row_length, length) || !field(row, w.row_name, name) || !field(row, w.row_handle, handle))
            return fail("ecs-thread-registry-row-unreadable");
        if (length != w.name_size) continue;
        std::array<char, 64> text{};
        if (!read_span(name, text.data(), length)) return fail("ecs-thread-name-unreadable");
        if (std::memcmp(text.data(), w.name.data(), length)) continue;
        if (++matches > 1) return fail("ecs-server-thread-is-ambiguous");
        if (!handle || name < w.inline_name || name - w.inline_name < w.scene_thread)
            return fail("ecs-server-thread-range-invalid");
        selected_thread = name - w.inline_name;
        selected_scene = selected_thread - w.scene_thread;
        selected_handle = handle;
    }
    if (!matches) return fail("ecs-server-thread-unregistered");
    const auto valid_thread = [&]() {
        std::uintptr_t name{}, handle{}, callback{}, context{}, wanted_context{};
        std::uint64_t length{}, capacity{};
        std::uint32_t stop{};
        std::uint8_t state{};
        return field(selected_thread, w.name_pointer, name) && name == selected_thread + w.inline_name &&
            field(selected_thread, w.name_length, length) && length == w.name_size &&
            field(selected_thread, w.name_capacity, capacity) && capacity == w.name_capacity_value &&
            field(selected_thread, w.thread_handle, handle) && handle == selected_handle &&
            field(selected_thread, w.thread_callback, callback) && callback == image + w.callback &&
            add(selected_scene, w.scene_context, wanted_context) &&
            field(selected_thread, w.thread_context, context) && context == wanted_context &&
            field(selected_thread, w.thread_stop, stop) && stop == 0 &&
            field(selected_scene, w.scene_state, state) && state == w.running_state;
    };
    if (!valid_thread()) return fail("ecs-server-thread-owner-invalid-or-stopping");
    if (!add(selected_scene, w.scene_lock, address) || !try_exclusive(address))
        return fail("ecs-scene-frame-busy");
    scene_lock_ = address;
    EcsView current;
    std::uint8_t mode{};
    if (!valid_thread() || !resolve_ecs(layout, current, nullptr, image) ||
        current.scene_owner != selected_scene || current != expected ||
        !field(current.execution_owner, w.execution_mode, mode) || mode != w.authority_mode)
        return fail("ecs-write-owner-changed-or-not-authoritative");
    maximum_bytes_ = w.maximum_bytes;
    manager_ = current.manager;
    registry_owner_ = current.registry_owner;
    tracker_ = image + w.tracker;
    if (!add(manager_, w.tracking_context, tracking_context_)) return fail("ecs-change-tracker-context-invalid");
    tracker_bytes_ = w.tracker_bytes;
    tracking_manager_offset_ = w.tracking_manager;
    tracking_registry_a_ = w.tracking_registry_a;
    tracking_registry_b_ = w.tracking_registry_b;
    prepared->thread = selected_thread;
    prepared->handle = selected_handle;
    state_ = std::move(prepared);
    if (reason) *reason = nullptr;
    return true;
}
}
