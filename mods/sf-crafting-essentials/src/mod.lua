local function setting_boolean(name, default)
    local value = shroudforge.settings.get(name, default)
    if type(value) == "boolean" then return value end
    return default
end

local function prepare_production_time()
    if not loader.features.patch then
        error("Crafting Essentials production duration requires the pregame patch capability")
    end
    if not setting_boolean("productionTimeEnabled", true) then return end

    local seconds = shroudforge.settings.get("productionSeconds", 3)
    assert(type(seconds) == "number" and seconds == seconds and seconds >= 1 and seconds <= 900,
        "Crafting Essentials production seconds must be between 1 and 900")
    local duration = math.floor(seconds * 1000000000 + 0.5)
    local recipe_type = assert(game.types.get("keen::RecipeRegistryResource"),
        "API type unavailable: keen::RecipeRegistryResource")
    local resources = game.assets.get_resources_by_type(recipe_type)
    assert(#resources > 0, "Crafting Essentials found no recipe registries")

    local pending, changed = {}, 0
    for _, resource in ipairs(resources) do
        local data = resource.data
        assert(data and data.recipes, "Recipe registry has no recipes")
        local dirty = false
        for _, recipe in ipairs(data.recipes) do
            local time = recipe.craftingDuration
            assert(time and type(time.value) == "number" and time.value == time.value
                and time.value >= 0 and time.value < math.huge, "Invalid recipe craftingDuration")
            if time.value > 0 and time.value ~= duration then
                time.value = duration
                changed, dirty = changed + 1, true
            end
        end
        if dirty then pending[#pending + 1] = {resource = resource, data = data} end
    end
    for _, entry in ipairs(pending) do entry.resource.data = entry.data end
    shroudforge.log.info("Crafting Essentials: updated " .. changed .. " timed recipes to " .. seconds .. " base seconds")
end

-- The same Runtime API is present in both phases. KFC writes must run only
-- during startup preparation, before the game loads its recipe resources.
if type(runtime) == "table" and runtime.phase == "pregame" then
    prepare_production_time()
    return {}
end

if type(runtime) ~= "table" or runtime.phase ~= "ingame" then return {} end

local features = {
    {setting = "noResourceCost", id = "zero_resource_argument", label = "No Resource Costs"},
    {setting = "infiniteItemUse", id = "clear_item_use_argument", label = "Infinite Item Use"},
    {setting = "infiniteItemSplit", id = "preserve_split_source", label = "Keep Source Stack When Splitting"},
}

local modifiers, active, requested, failures = {}, {}, {}, {}

local function record_failure(feature, reason)
    local detail = tostring(reason or "Runtime operation is unavailable for this game build")
    if failures[feature.setting] ~= detail then
        failures[feature.setting] = detail
        shroudforge.log.error("Crafting Essentials, " .. feature.label .. ": " .. detail)
    end
end

local function clear_failure(feature)
    failures[feature.setting] = nil
end

local function set_feature(feature, enabled)
    if active[feature.setting] == enabled then return true end
    if not enabled then
        local modifier = modifiers[feature.setting]
        if not modifier then
            active[feature.setting] = false
            clear_failure(feature)
            return true
        end
        local ok, reason = modifier.set_enabled(false)
        if not ok then
            record_failure(feature, reason)
            return false
        end
        active[feature.setting] = false
        clear_failure(feature)
        return true
    end

    if not runtime.has("runtime.gameplay.patch") then
        local status = runtime.status("runtime.gameplay.patch")
        record_failure(feature, status.reason or "Runtime patch API is unavailable")
        return false
    end
    local ok, modifier, reason = pcall(runtime.functions.bind_modifier, feature.id)
    if not ok then
        record_failure(feature, modifier)
        return false
    end
    if not modifier then
        record_failure(feature, reason)
        return false
    end
    local available_ok, available, available_reason = pcall(modifier.available)
    if not available_ok or not available then
        record_failure(feature, available_ok and available_reason or available)
        return false
    end
    local enabled_ok, enabled_reason = modifier.set_enabled(true)
    if not enabled_ok then
        record_failure(feature, enabled_reason)
        return false
    end
    modifiers[feature.setting] = modifier
    active[feature.setting] = true
    clear_failure(feature)
    return true
end

local function refresh_report()
    local active_count, requested_count = 0, 0
    local details = {}
    for _, feature in ipairs(features) do
        if active[feature.setting] then active_count = active_count + 1 end
        if requested[feature.setting] then requested_count = requested_count + 1 end
        if failures[feature.setting] then
            details[#details + 1] = feature.label .. ": " .. failures[feature.setting]
        end
    end
    if #details > 0 then
        runtime.report_effect("write-failed", table.concat(details, " | "))
    elseif active_count > 0 then
        runtime.report_effect("write-confirmed", active_count .. " Crafting Essentials runtime features active")
    elseif requested_count == 0 then
        runtime.report_effect("no-change", "All Crafting Essentials runtime features are disabled")
    else
        runtime.report_effect("no-change", "No Crafting Essentials runtime feature was changed")
    end
end

local function apply_settings(initial)
    for _, feature in ipairs(features) do
        local wanted = setting_boolean(feature.setting, true)
        requested[feature.setting] = wanted
        if wanted then
            if not active[feature.setting] then set_feature(feature, true) end
        elseif active[feature.setting] ~= false or initial then
            set_feature(feature, false)
        end
    end
    refresh_report()
end

return {
    update_interval_ms = 1000,
    on_load = function()
        apply_settings(true)
    end,
    on_update = function()
        apply_settings(false)
    end,
    on_unload = function()
        for _, feature in ipairs(features) do
            if active[feature.setting] then set_feature(feature, false) end
        end
        refresh_report()
    end,
}
