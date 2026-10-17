#include "profiles/read_game_profile.h"
#include "profiles/game_profile.h"
#include <windows.h>
#include <array>
#include <cmath>
#include <cstring>
#include <unordered_set>
#include <nlohmann/json.hpp>

namespace KfcRuntimeCompatibility::EnshroudedClient {
void ReadGameProfile(const nlohmann::json& selected, bool exact, const std::uint8_t* base) {
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(base);
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(base + dos->e_lfanew);
    const auto& layout = selected.at("ecsLayout");
    auto offset = [&](const char* key) {
        const auto value = layout.at(key).get<std::size_t>();
        if (value > 0x10000) throw std::runtime_error("layout offset out of range");
        return value;
    };
    entity_manager_count = offset("entityManagerCount");
    entity_manager_table = offset("entityManagerTable");
    component_offsets = offset("componentOffsets"); component_strides = offset("componentStrides");
    entity_id = offset("entityId"); entity_generation = offset("entityGeneration");
    entity_layout = offset("entityLayout"); entity_storage = offset("entityStorage");
    entity_definition = offset("entityDefinitionPointer"); entity_row = offset("entityRow");
    component_bits = offset("componentBits"); lookup_manager = offset("lookupManager");
    const auto& definition_layout = selected.at("entityDefinitionLayout");
    auto definition_offset = [&](const char* key) {
        const auto value = definition_layout.at(key).get<std::size_t>();
        if (value > 0x10000) throw std::runtime_error(std::string("entity definition offset out of range: ") + key);
        return value;
    };
    definition_uuid = definition_offset("uuid");
    definition_name = definition_offset("name");
    definition_name_size = definition_offset("nameSize");
    const auto& hooks = selected.at("hooks");
    game_thread_signature = hooks.at("game_thread").at("signature").get<std::string>();
    entity_manager_signature = hooks.at("entity_manager").at("signature").get<std::string>();
    game_thread_original = hooks.at("game_thread").at("original").get<std::vector<std::uint8_t>>();
    entity_manager_original = hooks.at("entity_manager").at("original").get<std::vector<std::uint8_t>>();
    if (game_thread_original.size() < 5 || game_thread_original.size() > 32 ||
        entity_manager_original.size() < 5 || entity_manager_original.size() > 32) throw std::runtime_error("invalid hook length");
    world_prop_update_signature = hooks.at("world_prop_update").at("signature").get<std::string>();
    world_prop_update_original = hooks.at("world_prop_update").at("original").get<std::vector<std::uint8_t>>();
    world_actor_placement_signature = hooks.at("world_actor_placement").at("signature").get<std::string>();
    world_actor_placement_original = hooks.at("world_actor_placement").at("original").get<std::vector<std::uint8_t>>();
    if (hooks.contains("world_building_dispatch")) {
        world_building_dispatch_signature = hooks.at("world_building_dispatch").at("signature").get<std::string>();
        world_building_dispatch_original = hooks.at("world_building_dispatch").at("original").get<std::vector<std::uint8_t>>();
    } else {
        world_building_dispatch_signature.clear();
        world_building_dispatch_original.clear();
    }
    if (world_prop_update_original.size() < 5 || world_prop_update_original.size() > 32 ||
        world_actor_placement_original.size() < 5 || world_actor_placement_original.size() > 32)
        throw std::runtime_error("invalid world context hook length");
    world_cursor_signature.clear();
    world_cursor_original.clear();
    world_cursor_capture_offset = 0;
    if (hooks.contains("world_cursor")) {
        const auto& cursor_hook = hooks.at("world_cursor");
        world_cursor_signature = cursor_hook.at("signature").get<std::string>();
        world_cursor_original = cursor_hook.at("original").get<std::vector<std::uint8_t>>();
        world_cursor_capture_offset = cursor_hook.at("captureOffset").get<std::size_t>();
        if (world_cursor_signature.empty() || world_cursor_signature.size() > 256 ||
            world_cursor_original.size() != 7 || world_cursor_capture_offset > 0x10000)
            throw std::runtime_error("invalid native world cursor hook profile");
    }
    const auto& entity_context = selected.at("worldContexts").at("entityPlacement");
    auto context_offset = [&](const char* key) {
        const auto value = entity_context.at(key).get<std::size_t>();
        if (value > 0x10000) throw std::runtime_error(std::string("world context offset out of range: ") + key);
        return value;
    };
    world_context_layout.actor_frame_service_view = context_offset("actorFrameServiceViewOffset");
    world_context_layout.service_view_world = context_offset("serviceViewWorldOffset");
    world_context_layout.placement_context = context_offset("placementContextOffset");
    world_context_layout.place_queue = context_offset("placeQueueOffset");
    world_context_layout.remove_queue = context_offset("removeQueueOffset");
    world_context_layout.publish_state = context_offset("publishStateOffset");
    world_context_layout.publish_commands = context_offset("publishCommandsOffset");
    world_context_layout.owner = context_offset("ownerOffset");
    world_context_layout.client_cursor_service_view = 0;
    world_context_layout.client_cursor_service_world = 0;
    if (exact_build_match && image_target == "enshrouded.exe" && selected.at("worldContexts").contains("clientCursorRead")) {
        const auto& cursor = selected.at("worldContexts").at("clientCursorRead");
        const auto frame = cursor.at("frameServiceViewOffset").get<std::size_t>();
        const auto world = cursor.at("serviceWorldOffset").get<std::size_t>();
        if (!frame || !world || frame > 0x10000 || world > 0x10000)
            throw std::runtime_error("invalid client cursor read context layout");
        world_context_layout.client_cursor_service_view = frame;
        world_context_layout.client_cursor_service_world = world;
    }
    runtime_world_grids.clear();
    const auto& grids = selected.at("worldGrids");
    if (!grids.is_object() || grids.empty()) throw std::runtime_error("world grid catalog is missing or empty");
    for (auto item = grids.begin(); item != grids.end(); ++item) {
        RuntimeWorldGrid grid{};
        grid.id = item.key();
        const auto origin = item.value().at("origin").get<std::array<double, 3>>();
        const auto cell_size = item.value().at("cellSize").get<std::array<double, 3>>();
        const auto maximum = item.value().at("maximum").get<std::array<std::uint64_t, 3>>();
        for (int axis = 0; axis < 3; ++axis) {
            if (!std::isfinite(origin[axis]) || !std::isfinite(cell_size[axis]) || cell_size[axis] <= 0 || !maximum[axis])
                throw std::runtime_error("invalid world grid specification: " + grid.id);
            grid.origin[axis] = origin[axis];
            grid.cell_size[axis] = cell_size[axis];
            grid.maximum[axis] = maximum[axis];
        }
        runtime_world_grids.push_back(std::move(grid));
    }
    runtime_components.clear();
    std::unordered_set<std::string> names;
    std::unordered_set<unsigned> indices;
    const bool live_components = selected.value("componentResolution", std::string{}) == "live-registration";
    const auto no_components = nlohmann::json::array();
    const nlohmann::json* component_entries{};
    if (selected.contains("components")) {
        if (live_components) throw std::runtime_error("live registration profile must not duplicate component mappings");
        component_entries = &selected.at("components");
    } else if (live_components) {
        component_entries = &no_components;
    } else {
        throw std::runtime_error("profile must select live registration or an explicit component fallback");
    }
    if (!component_entries || !component_entries->is_array())
        throw std::runtime_error("profile component catalog is not an array");
    for (const auto& component : *component_entries) {
        const auto name = component.at("name").get<std::string>();
        const auto index = component.at("index").get<unsigned>();
        const auto size = component.at("size").get<unsigned>();
        if (!name.starts_with("keen::ecs::") || index >= 1024 || !size || size > 65535 ||
            !names.insert(name).second || !indices.insert(index).second) throw std::runtime_error("invalid component mapping");
        if (exact) runtime_components.push_back({name, static_cast<std::uint16_t>(index), size});
    }
    if (exact && !live_components && runtime_components.empty()) throw std::runtime_error("empty component profile");
    runtime_operations.clear();
    attribute_calculation_model = exact && selected.at("image").contains("sha256")
        ? selected.value("attributeCalculationModel", std::string{}) : std::string{};
    if (!attribute_calculation_model.empty() && attribute_calculation_model != "attribute-command-v1")
        throw std::runtime_error("unknown attribute calculation model");
    runtime_patches.clear();
    world_finish_event_id_rva = 0;
    const auto operations = selected.find("worldOperations");
    if (operations != selected.end()) {
        std::unordered_set<std::string> operation_names;
        for (auto item = operations->begin(); item != operations->end(); ++item) {
            const auto& value = item.value();
            RuntimeOperation operation{};
            operation.name = item.key();
            operation.function_rva = value.value("functionRva", std::uintptr_t{});
            operation.global_rva = value.value("globalRva", std::uintptr_t{});
            operation.guard_rva = value.value("guardRva", std::uintptr_t{});
            operation.validation_offset = value.value("validationOffset", std::uintptr_t{});
            operation.mode = value.value("mode", std::uint32_t{});
            operation.context_pointer_offset = value.value("contextPointerOffset", std::ptrdiff_t{});
            operation.world_offset = value.value("worldOffset", std::ptrdiff_t{});
            if (operation.name == "runtime.world.entity.finish_building")
                world_finish_event_id_rva = value.value("eventIdRva", std::uintptr_t{});
            operation.guard_bytes = value.value("guardBytes", std::vector<std::uint8_t>{});
            operation.abi = value.at("abi").get<std::string>();
            operation.actor_context = operation.name == "runtime.world.context.active" && operation.abi == "actor-world-context";
            operation.thread = value.at("thread").get<std::string>();
            operation.context = value.at("context").get<std::string>();
            const bool explicitly_unresolved = !value.value("validated", true);
            if (!operation.name.starts_with("runtime.world.") ||
                !operation_names.insert(operation.name).second ||
                operation.abi.empty() || operation.abi.size() > 256 ||
                operation.thread != "game" || operation.context.empty() || operation.context.size() > 256 ||
                (!explicitly_unresolved && !operation.actor_context && (operation.function_rva == 0) == (operation.global_rva == 0)) ||
                (operation.actor_context && (operation.function_rva || operation.global_rva || !operation.validation_offset || operation.guard_bytes.empty())) ||
                (operation.function_rva != 0 && operation.global_rva != 0) ||
                operation.guard_bytes.size() > 64)
                throw std::runtime_error("invalid world operation profile entry: " + operation.name);
            if (!explicitly_unresolved && !operation.actor_context && operation.name == "runtime.world.context.active" &&
                (!operation.global_rva || !operation.context_pointer_offset || !operation.world_offset))
                throw std::runtime_error("active world context requires a validated pointer chain");

            const auto selected_image_size = static_cast<std::size_t>(nt->OptionalHeader.SizeOfImage);
            const bool target_in_image = operation.actor_context ? exact : operation.function_rva
                ? operation.function_rva < selected_image_size
                : operation.global_rva < selected_image_size;
            const bool guard_in_image = operation.guard_bytes.empty() ||
                (operation.guard_rva < selected_image_size &&
                 operation.guard_bytes.size() <= selected_image_size - operation.guard_rva);
            const bool guard_matches = guard_in_image &&
                (operation.guard_bytes.empty() ||
                 std::memcmp(base + operation.guard_rva, operation.guard_bytes.data(),
                             operation.guard_bytes.size()) == 0);
            const bool event_id_valid = operation.name != "runtime.world.entity.finish_building" ||
                (world_finish_event_id_rva && world_finish_event_id_rva < selected_image_size &&
                 sizeof(std::uint32_t) <= selected_image_size - world_finish_event_id_rva);
            operation.available = target_in_image && guard_matches && event_id_valid &&
                (exact || !operation.guard_bytes.empty());
            operation.status = !target_in_image ? "target-outside-image" :
                !guard_in_image ? "guard-outside-image" :
                !guard_matches ? "instruction-guard-mismatch" :
                !event_id_valid ? "finish-event-id-outside-image" :
                !exact ? "build-diff-guard-verified" :
                operation.global_rva ? "runtime-pointer-validation-required" : "verified";
            if (explicitly_unresolved) {
                operation.available = false;
                operation.status = "unresolved:" + value.value("unresolvedReason", std::string("native ABI or context is not proven"));
            }
            runtime_operations.push_back(std::move(operation));
        }
    }
    const auto patches = selected.find("runtimePatches");
    if (patches != selected.end()) {
        std::unordered_set<std::string> patch_names;
        for (auto item = patches->begin(); item != patches->end(); ++item) {
            const auto& value = item.value();
            RuntimePatch patch{};
            patch.name = item.key();
            if (value.contains("modifier")) patch.modifier_json = value.at("modifier").dump();
            patch.signature = value.at("signature").get<std::string>();
            patch.kind = value.at("kind").get<std::string>();
            const auto& function = value.at("function");
            patch.function_id = function.at("id").get<std::string>();
            patch.function_begin_rva = function.at("beginRva").get<std::uintptr_t>();
            patch.function_end_rva = function.at("endRva").get<std::uintptr_t>();
            patch.target_offset = function.at("targetOffset").get<std::size_t>();
            patch.overwrite = value.at("overwriteBytes").get<std::size_t>();
            patch.payload = value.at("payload").get<std::vector<std::uint8_t>>();
            patch.return_rel32_offset = value.value("returnRel32Offset", std::size_t{});
            for (const auto& entry : value.value("inlineReferences", nlohmann::json::array())) {
                RuntimePatch::InlineReference ref{entry.at("displacementOffset"), entry.at("nextInstructionOffset"), entry.at("dataOffset"), entry.at("dataSize")};
                const auto length = patch.payload.size();
                if (patch.kind != "detour" || ref.displacement > length || length - ref.displacement < 4 ||
                    ref.next != ref.displacement + 4 || !ref.size || ref.data > length || ref.size > length - ref.data ||
                    ref.data < patch.return_rel32_offset + 4)
                    throw std::runtime_error("invalid inline payload reference: " + patch.name);
                patch.inline_references.push_back(ref);
            }
            if (!patch.name.starts_with("runtime.patch.") || !patch_names.insert(patch.name).second ||
                patch.function_id.empty() || patch.function_id.size() > 128 ||
                patch.function_begin_rva >= patch.function_end_rva ||
                patch.target_offset >= patch.function_end_rva - patch.function_begin_rva ||
                patch.signature.empty() || patch.signature.size() > 256 || patch.overwrite < 3 || patch.overwrite > 32 ||
                patch.payload.empty() || patch.payload.size() > 256 ||
                (patch.kind != "bytes" && patch.kind != "detour") ||
                (patch.kind == "bytes" && patch.payload.size() != patch.overwrite) ||
                (patch.kind == "detour" && (patch.return_rel32_offset < 1 || patch.return_rel32_offset + 4 > patch.payload.size() ||
                    patch.payload[patch.return_rel32_offset - 1] != 0xe9)))
                throw std::runtime_error("invalid runtime patch profile entry: " + patch.name);
            if (exact && (patch.function_end_rva > nt->OptionalHeader.SizeOfImage ||
                patch.function_begin_rva >= nt->OptionalHeader.SizeOfImage))
                throw std::runtime_error("runtime patch function range outside image: " + patch.name);
            runtime_patches.push_back(std::move(patch));
        }
    }
    status = selected.at("id").get<std::string>() + (exact ? ":exact-image" : ":build-diff-warning");
}
}
