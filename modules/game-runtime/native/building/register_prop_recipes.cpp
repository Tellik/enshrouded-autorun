#include "ecs/building_prop_query_state.h"
#include "ecs/ecs_session_state.h"

using namespace EcsImplementation;

extern "C" bool __cdecl KfcRuntimeWorldEntityRegisterPropRecipes(
    const KfcRuntimePropRecipe* recipes, std::size_t count) {
    if ((!recipes && count) || count > 1'000'000) return false;
    std::unordered_map<std::uint32_t, RegisteredPropRecipe> resolved;
    resolved.reserve(count);
    for (std::size_t index = 0; index < count; ++index) {
        const auto& recipe = recipes[index];
        if (!recipe.item_id || !std::all_of(std::begin(recipe.bounds), std::end(recipe.bounds),
                [](float value) { return std::isfinite(value); }) ||
            recipe.bounds[0] > recipe.bounds[3] || recipe.bounds[1] > recipe.bounds[4] ||
            recipe.bounds[2] > recipe.bounds[5]) return false;
        if (!resolved.emplace(recipe.item_id,
                RegisteredPropRecipe{std::to_array(recipe.bounds), recipe.feedback}).second) return false;
    }
    {
        std::scoped_lock lock(prop_recipe_mutex);
        prop_recipes = std::move(resolved);
        prop_recipe_catalog_ready = true;
    }
    return true;
}

extern "C" bool __cdecl KfcRuntimeWorldEntityGetPropRecipe(
    std::uint32_t item_id, KfcRuntimePropRecipe* recipe) {
    if (!item_id || !recipe) return false;
    std::scoped_lock lock(prop_recipe_mutex);
    const auto found = prop_recipes.find(item_id);
    if (found == prop_recipes.end()) return false;
    recipe->item_id = item_id;
    std::copy(found->second.bounds.begin(), found->second.bounds.end(), recipe->bounds);
    recipe->feedback = found->second.feedback;
    return true;
}

namespace EcsImplementation {
std::mutex prop_recipe_mutex;

std::unordered_map<std::uint32_t, RegisteredPropRecipe> prop_recipes;

bool prop_recipe_catalog_ready{};

}
