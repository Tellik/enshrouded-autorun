#include "ecs/registry/component_discovery_state.h"
#include "ecs/ecs_session_state.h"

using namespace EcsImplementation;

namespace EcsImplementation {

namespace {
struct CallbackCode {
    std::array<std::uint8_t, 512> bytes{};
    bool stable{};

    bool operator==(const CallbackCode& other) const {
        return stable == other.stable && (!stable || bytes == other.bytes);
    }
};

// Protected by state_mutex, alongside the snapshot and JSON it describes.
std::vector<CallbackCode> published_callback_code;
}

nlohmann::json registration_type_json(const ComponentRegistry::Type& type){
    if (type.name.empty()) return nullptr;
    return {{"qualified_name",type.name},{"qualified_hash",type.hash},
        {"size",type.size},{"metadata_rva",type.metadata_rva}};
}

void read_component_registry(std::uintptr_t manager){
    const auto now = GetTickCount64();
    if (!manager) {
        std::unordered_map<std::uint32_t, HandleRecord> retired_handles;
        std::unordered_map<std::uint64_t, std::uint32_t> retired_reverse_handles;
        {
            std::scoped_lock operation_lock(operation_mutex);
            std::scoped_lock lock(state_mutex);
            if (component_registry_manager || layout_ready ||
                !handles.empty() || !reverse_handles.empty()) {
                ++layout_epoch;
                handles.swap(retired_handles);
                reverse_handles.swap(retired_reverse_handles);
            }
            component_registry_world_bound = false;
            read_only_binding = false;
            read_only_view = {};
            component_registry_manager = 0;
            live_layout = {};
            layout_ready = false;
            live_table = 0;
        }
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
    std::vector<CallbackCode> callback_code;
    if (valid) valid = validate_contract(snapshot, error);
    if (valid) {
        resolved.reserve(snapshot.entries.size());
        std::size_t callback_count{};
        for (const auto& entry : snapshot.entries) callback_count += entry.callbacks.size();
        callback_code.reserve(callback_count);
    }
    if (valid) for (const auto& entry : snapshot.entries) {
        if (!entry.storage.name.empty())
            resolved.emplace(entry.storage.name, ComponentType{entry.index, entry.size});
        for (const auto& callback : entry.callbacks) {
            CallbackCode code;
            const auto code_address = image_base + callback.function_rva;
            if (image_size >= code.bytes.size() && callback.function_rva <= image_size - code.bytes.size() &&
                read_bytes(code_address, code.bytes.data(), code.bytes.size())) {
                std::array<std::uint8_t, 512> again{};
                code.stable = read_bytes(code_address, again.data(), again.size()) && again == code.bytes;
            }
            callback_code.push_back(std::move(code));
        }
    }
    if (valid) {
        std::scoped_lock lock(state_mutex);
        if (configured_types != contract) return;
        // Validation still runs on every refresh, including callback code. Skip
        // only the diagnostic allocation/serialization when its complete input
        // and the active world binding are unchanged.
        if (component_registry_ready && component_registry_world_bound == (manager != 0) &&
            types == resolved && component_registry == snapshot &&
            published_callback_code == callback_code) {
            component_registry_error = error;
            return;
        }
    }
    std::string serialized;
    if (valid) {
        nlohmann::json entries = nlohmann::json::array();
        std::size_t code_index{};
        for (const auto& entry : snapshot.entries) {
            nlohmann::json callbacks = nlohmann::json::array();
            for (const auto& callback : entry.callbacks) {
                const auto& code = callback_code[code_index++];
                std::string code_hex;
                if (code.stable) {
                    static constexpr char digits[] = "0123456789abcdef";
                    code_hex.resize(code.bytes.size() * 2);
                    for (std::size_t index = 0; index < code.bytes.size(); ++index) {
                        code_hex[index * 2] = digits[code.bytes[index] >> 4];
                        code_hex[index * 2 + 1] = digits[code.bytes[index] & 15];
                    }
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
        serialized = nlohmann::json({{"schema_version",1},{"layout_version",1},
            {"available",true},{"source","live-engine-registration"},{"count",entries.size()},
            {"discovery",manager ? "active-world-manager" : "resident-private-memory"},
            {"manager_bound",manager != 0},
            {"runtime_type_count",resolved.size()},{"entries",std::move(entries)}}).dump();
    }
    // Retired snapshots, strings and handle nodes are freed after both locks.
    std::unordered_map<std::uint32_t, HandleRecord> retired_handles;
    std::unordered_map<std::uint64_t, std::uint32_t> retired_reverse_handles;
    std::scoped_lock operation_lock(operation_mutex);
    std::scoped_lock lock(state_mutex);
    if (configured_types != contract) return;
    component_registry_error = error;
    if (!valid) {
        if (component_registry_ready) types.swap(resolved);
        component_registry_ready = false;
        component_registry_world_bound = false;
        component_registry_json.swap(serialized);
        published_callback_code.swap(callback_code);
        return;
    }
    const bool changed = component_registry.owner != snapshot.owner ||
        component_registry.records != snapshot.records || !component_registry_ready || types != resolved;
    types.swap(resolved);
    std::swap(component_registry, snapshot);
    component_registry_ready = true;
    component_registry_world_bound = manager != 0;
    component_registry_seen = true;
    component_registry_json.swap(serialized);
    published_callback_code.swap(callback_code);
    if (changed) {
        ++layout_epoch;
        handles.swap(retired_handles);
        reverse_handles.swap(retired_reverse_handles);
        // Dispatcher scans invalidate their own epoch lazily. Clearing these
        // maps from this discovery thread races an active engine callback.
    }
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
    if (component_registry_ready) {
        const auto required = component_registry_json.size() + 1;
        if (buffer && capacity >= required) std::memcpy(buffer, component_registry_json.c_str(), required);
        return required;
    }
    const auto json = nlohmann::json({
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

namespace EcsImplementation {
ComponentRegistry::Snapshot component_registry;

bool component_registry_ready{};

bool component_registry_world_bound{};

bool component_registry_seen{};

std::string component_registry_error{"awaiting entity manager"};

std::uint64_t component_registry_last_read{};

std::uintptr_t component_registry_manager{};

std::string component_registry_json;

}
