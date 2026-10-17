#pragma once

#include "abi/runtime_api.h"
#include "ecs/entity_session.h"
#include "ecs/component_access.h"
#include "ecs/snapshot_entity_ids.h"
#include "ecs/read_building_prop.h"
#include "engine-context/engine_owners.h"
#include "engine-context/hold_resource_lifetime.h"
#include "engine-context/lock_scene_access.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstring>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>
#include <nlohmann/json.hpp>

// Private implementation contract. State is defined once, in the owning unit.

namespace NamedServiceImplementation {
extern std::mutex operation_mutex;

struct Contract {
    std::string key;
    std::uint32_t hash{}, type_hash{}, size{}, descriptor{}, function{}, metadata{}, metadata_slot{};
    std::size_t records{}, record_count{}, stride{}, record_hash{}, normal{}, alternate{},
        map_bits{}, map_capacity{}, map_keys{}, map_values{}, map_count{},
        systems{}, system_count{}, system_stride{}, normal_bindings{}, alternate_bindings{},
        normal_count{}, alternate_count{}, maximum_records{}, maximum_capacity{}, maximum_systems{};
};

template<class T> bool read(std::uintptr_t at, T& value) {
    if (!at || at > UINTPTR_MAX - sizeof(value)) return false;
    __try { std::memcpy(&value, reinterpret_cast<const void*>(at), sizeof(value)); return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool copy(void* to, const void* from, std::size_t size);

template<class T> bool field(std::uintptr_t base, std::size_t offset, T& value) {
    return base && offset <= UINTPTR_MAX - base && read(base + offset, value);
}

bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& result);

std::uint32_t hash_name(const std::string& name);

Contract contract(const char* name, std::uint32_t type_hash, std::uint32_t size);

bool named(std::uintptr_t string, std::uint64_t length, const Contract& c);

bool resolve(const Contract& c, const EngineContext::EcsView& owner, std::uintptr_t& payload);

bool writable(std::uintptr_t address, std::size_t size);
}
