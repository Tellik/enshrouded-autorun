#pragma once

#include "ecs/entities.h"
#include "ecs/components.h"
#include "ecs/queries.h"
#include "ecs/props.h"
#include "engine-context/session.h"
#include "voxels/voxel_api.h"
#include "building/building_api.h"
#include "player-input/cursor.h"
#include "code-inventory/function_inventory.h"
#include "engine-context/owners.h"
#include "engine-context/resource_lifetime.h"
#include "engine-context/scene_gate.h"
#include "abi/runtime.h"
#include "loaded-resources/resource_api.h"
#include <algorithm>
#include <atomic>
#include <cstring>
#include <string>
#include <nlohmann/json.hpp>

// Private implementation contract. State is defined once, in the owning unit.

namespace ProviderImplementation {
extern std::atomic<bool> provider_initialized;
}
