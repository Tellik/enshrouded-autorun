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

namespace EcsImplementation {
extern std::mutex prop_recipe_mutex;

struct RegisteredPropRecipe { std::array<float, 6> bounds{}; std::uint32_t feedback{}; };

extern std::unordered_map<std::uint32_t, RegisteredPropRecipe> prop_recipes;

extern bool prop_recipe_catalog_ready;

extern std::mutex prop_query_call_mutex;

struct PropQueryResultCache {
    std::array<double, 6> bounds{};
    std::uint64_t epoch{};
    std::vector<KfcRuntimePropRecord> props;
    std::size_t validation_cursor{};
    bool ready{};
};

extern PropQueryResultCache prop_query_result_cache;

struct PropQueryOperation {
    double bounds[6]{};
    double padding{};
    bool exact_recipe_bounds{};
    std::vector<KfcRuntimePropRecord> output;
    std::size_t result{SIZE_MAX};
    std::vector<EntityPointer> pointers;
    std::unordered_set<std::uint64_t> seen;
    std::unordered_map<std::uint32_t, RegisteredPropRecipe> recipes;
    std::size_t cursor{};
    std::size_t validation_cursor{};
    std::uint64_t epoch{}, started_ms{}, touched_ms{}, progressed_ms{};
    bool initialized{};
    bool validating_cached{};
};

extern std::shared_ptr<PropQueryOperation> pending_prop_query;

bool recipe_bounds_intersect(const KfcRuntimePropRecord& prop,
                             const std::array<float, 6>& bounds,
                             const double* query_bounds);

void query_props_native(PropQueryOperation& operation);

}
