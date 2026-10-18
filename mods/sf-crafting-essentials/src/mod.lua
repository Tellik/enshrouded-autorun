local function setting_boolean(name, default)
    local value = shroudforge.settings.get(name, default)
    if type(value) == "boolean" then return value end
    return default
end

local function setting_string(name, default)
    local value = shroudforge.settings.get(name, default)
    if type(value) == "string" then return value end
    return default
end

local function finite(value)
    return type(value) == "number" and value == value and math.abs(value) < math.huge
end

local function prepare_recipe_data()
    if not loader.features.patch then
        error("Crafting Essentials recipe changes require the pregame patch capability")
    end

    local mode = setting_string("productionTimeMode", "original")
    assert(mode == "fixed" or mode == "original",
        "Crafting Essentials production time mode must be 'fixed' or 'original'")
    local duration, seconds
    if mode == "fixed" then
        seconds = shroudforge.settings.get("productionSeconds", 3)
        assert(finite(seconds) and seconds >= 1 and seconds <= 900,
            "Crafting Essentials production seconds must be between 1 and 900")
        duration = math.floor(seconds * 1000000000 + 0.5)
    end
    local recipe_type = assert(game.types.get("keen::RecipeRegistryResource"),
        "API type unavailable: keen::RecipeRegistryResource")
    local resources = game.assets.get_resources_by_type(recipe_type)
    assert(#resources > 0, "Crafting Essentials found no recipe registries")

    local no_resource_cost = setting_boolean("noResourceCost", true)
    local pending, changed, timed_recipes, free_recipes, direct_inputs, category_inputs, water_charges = {}, 0, 0, 0, 0, 0, 0
    for _, resource in ipairs(resources) do
        local data = resource.data
        assert(data and data.recipes, "Recipe registry has no recipes")
        local dirty = false
        for _, recipe in ipairs(data.recipes) do
            local time = recipe.craftingDuration
            assert(time and finite(time.value) and time.value >= 0,
                "Invalid recipe craftingDuration")
            if time.value > 0 then timed_recipes = timed_recipes + 1 end
            if mode == "fixed" and time.value > 0 and time.value ~= duration then
                time.value = duration
                changed, dirty = changed + 1, true
            end
            if no_resource_cost then
                assert(recipe.input, "Recipe has no input list")
                local recipe_cost_changed = false
                for _, input in ipairs(recipe.input) do
                    local item_stack = input.itemStack
                    local category = input.inputItemCategory
                    assert(item_stack and finite(item_stack.count) and item_stack.count >= 0,
                        "Invalid recipe itemStack.count")
                    assert(category and finite(category.count) and category.count >= 0,
                        "Invalid recipe inputItemCategory.count")
                    if item_stack.count > 0 then
                        item_stack.count = 0
                        direct_inputs, dirty, recipe_cost_changed = direct_inputs + 1, true, true
                    end
                    if category.count > 0 then
                        category.count = 0
                        category_inputs, dirty, recipe_cost_changed = category_inputs + 1, true, true
                    end
                end
                assert(finite(recipe.waterCharges) and recipe.waterCharges >= 0,
                    "Invalid recipe waterCharges")
                if recipe.waterCharges > 0 then
                    recipe.waterCharges = 0
                    water_charges, dirty, recipe_cost_changed = water_charges + 1, true, true
                end
                if recipe_cost_changed then free_recipes = free_recipes + 1 end
            end
        end
        if dirty then pending[#pending + 1] = {resource = resource, data = data} end
    end
    for _, entry in ipairs(pending) do entry.resource.data = entry.data end
    if mode == "fixed" then
        shroudforge.log.info("Crafting Essentials: updated " .. changed ..
            " of " .. timed_recipes .. " timed recipes to " .. seconds .. " base seconds")
    else
        shroudforge.log.info("Crafting Essentials: preserved original durations for " ..
            timed_recipes .. " timed recipes")
    end
    if no_resource_cost then
        shroudforge.log.info("Crafting Essentials: removed recipe costs from " .. free_recipes ..
            " recipes (" .. direct_inputs .. " direct inputs, " .. category_inputs ..
            " category inputs, " .. water_charges .. " water requirements)")
    end
end

-- Asset edits must happen before the game loads its recipe resources.
if type(runtime) == "table" and runtime.phase == "pregame" then
    prepare_recipe_data()
    return {}
end

if type(runtime) ~= "table" or runtime.phase ~= "ingame" then return {} end

local overrides = runtime.overrides.new()
local item_use_cache, production_cache
local last_report_state, last_report_detail
local function report(state, detail)
    if last_report_state == state and last_report_detail == detail then return end
    last_report_state, last_report_detail = state, detail
    runtime.report_effect(state, detail)
end

local function apply_workshop_time_factor()
    local mode = setting_string("productionTimeMode", "original")
    if mode ~= "original" and mode ~= "fixed" then
        overrides:waiting("Production Time Mode must be 'original' or 'fixed'")
        return
    end
    if mode == "fixed" then
        return "Production Duration Scale is disabled in shared-time mode"
    end

    local percent = shroudforge.settings.get("workshopTimeFactorPercent", 100)
    if not finite(percent) or percent < 0 or percent > 200 or percent % 5 ~= 0 then
        overrides:waiting("Production Duration Scale must be between 0% and 200% in 5% steps")
        return
    end

    local shortest_seconds
    if percent < 100 then
        local cache = production_cache
        if not cache or cache.mode ~= "original" or not cache.scan_complete or
           not finite(cache.minimum_duration_ns) or cache.minimum_duration_ns <= 0 then
            overrides:waiting("Production Duration Scale is waiting for the shortest recipe time")
            return
        end
        shortest_seconds = cache.minimum_duration_ns / 1000000000
    end

    local service, service_reason = runtime.services.get("ecs::GameSettings")
    if not service then
        overrides:waiting("Production Duration Scale: " .. tostring(service_reason or "authoritative world is unavailable"))
        return
    end
    local target, target_reason = runtime.overrides.service(service, "factoryProductionSpeedFactor")
    if not target then
        overrides:waiting("Production Duration Scale: " .. tostring(target_reason or "typed setting is unavailable"))
        return
    end

    -- At 0%, make the shortest recipe take one second. At 100%, preserve the
    -- world's factor. Above 100%, scale normal duration proportionally.
    overrides:apply(target.key, "workshopTimeFactorPercent", target, function(_, baseline)
        if not finite(baseline) or baseline <= 0 then
            return nil, "The world's production factor must be positive and finite"
        end
        local desired
        if percent <= 100 then
            if percent == 100 then
                desired = baseline
            else
                local normal_shortest_seconds = shortest_seconds / baseline
                local target_shortest_seconds = 1 +
                    (normal_shortest_seconds - 1) * (percent / 100)
                desired = shortest_seconds / target_shortest_seconds
            end
        else
            desired = baseline / (percent / 100)
        end
        if not finite(desired) or desired <= 0 then
            return nil, "The scaled production factor must be positive and finite"
        end
        return desired
    end, true)

    local fields, read_reason = service:read_fields({"factoryProductionSpeedFactor"})
    if fields and finite(fields.factoryProductionSpeedFactor) then
        return "Production Duration Scale requested " .. percent ..
            "% (observed GameSettings factor " .. tostring(fields.factoryProductionSpeedFactor) .. ")"
    end
    return "Production Duration Scale requested " .. percent .. "% (current value unavailable: " ..
        tostring(read_reason or "typed read failed") .. ")"
end

local item_use_events = {
    ["keen::actor::PayUsageCost"] = true,
    ["keen::actor::ApplyBuffEvent"] = true,
}

local function resource_cache(session, qualified_type, label)
    return {
        session = session,
        qualified_type = qualified_type, label = label,
        pending = {}, pending_keys = {}, head = 1, tail = 0,
        scanned = {}, scanned_count = 0, targets = {}, target_keys = {},
        target_cursor = 1, refresh_in = 0, minimum_duration_ns = nil,
        scan_complete = false, scan_failed = false,
    }
end

local function resource_key(resource)
    return resource.guid .. "/" .. resource.part
end

local function refresh_resources(cache, delta_seconds)
    cache.refresh_in = cache.refresh_in - delta_seconds
    if cache.refresh_in > 0 then return end
    local ok, resources = pcall(game.assets.get_loaded_resources_by_type, cache.qualified_type)
    cache.refresh_in = ok and 10 or 1
    if not ok then
        overrides:waiting(cache.label .. ": " .. tostring(resources))
        return
    end
    cache.scan_failed = false
    for _, resource in ipairs(resources) do
        local key = resource_key(resource)
        if not cache.scanned[key] and not cache.pending_keys[key] then
            cache.tail = cache.tail + 1
            cache.pending[cache.tail] = resource
            cache.pending_keys[key] = true
            cache.scan_complete = false
        end
    end
end

local function next_resource(cache)
    if cache.head > cache.tail then return end
    local resource = cache.pending[cache.head]
    cache.pending[cache.head] = nil
    cache.head = cache.head + 1
    if cache.head > cache.tail then cache.pending, cache.head, cache.tail = {}, 1, 0 end
    local key = resource_key(resource)
    local ok, data = pcall(function() return resource.data end)
    if not ok or not data then
        cache.pending_keys[key] = nil
        cache.scan_failed = true
        cache.scan_complete = false
        overrides:waiting(cache.label .. ": resource snapshot is unavailable: " .. tostring(data))
        return
    end
    cache.current = {resource = resource, key = key, data = data, sequence = 1, event = 1, recipe = 1}
    return cache.current
end

local function finish_resource(cache, current)
    cache.scanned[current.key] = true
    cache.scanned_count = cache.scanned_count + 1
    cache.pending_keys[current.key], cache.current = nil, nil
end

local function add_resource_target(cache, current, path)
    local key = current.key .. "/" .. path
    if cache.target_keys[key] then return end
    cache.target_keys[key] = true
    cache.targets[#cache.targets + 1] = {
        guid = current.resource.guid, part = current.resource.part, path = path,
    }
end

local function apply_resource_targets(cache, setting, desired)
    -- Each public target uses a fresh snapshot, an original storage claim,
    -- expected scalar and automatic restoration. The work remains bounded.
    for _ = 1, math.min(4, #cache.targets) do
        if cache.target_cursor > #cache.targets then cache.target_cursor = 1 end
        local entry = cache.targets[cache.target_cursor]
        cache.target_cursor = cache.target_cursor + 1
        local ok, resource = pcall(game.assets.get_loaded_resource, entry.guid, cache.qualified_type, entry.part)
        if not ok or not resource then
            overrides:waiting(cache.label .. ": a discovered resource snapshot is unavailable")
        else
            local target, reason = runtime.overrides.resource(resource, entry.path)
            if target then
                overrides:apply(target.key, setting, target, desired, true)
            else overrides:waiting(cache.label .. ": " .. tostring(reason)) end
        end
    end
end

local function resource_writes_ready(label)
    local session = runtime.world.session_id()
    if session == 0 or not runtime.has("runtime.assets.write") or not runtime.has("runtime.ecs.write") then
        overrides:waiting(label .. ": waiting for guarded resource writes in the authoritative world")
        return
    end
    return session
end

local function discover_item_use_targets()
    local cache = item_use_cache
    local resource_budget, event_budget = 4, 128
    while resource_budget > 0 and event_budget > 0 do
        local current = cache.current
        if not current then
            if cache.head > cache.tail then break end
            resource_budget = resource_budget - 1
            current = next_resource(cache)
        end
        if current then
            local sequence = current.data.subSequences[current.sequence]
            if not sequence then
                finish_resource(cache, current)
            else
                local event = sequence.events[current.event]
                if not event then
                    current.sequence, current.event = current.sequence + 1, 1
                else
                    -- Variant.type is the original qualified name, not a Type object.
                    if item_use_events[event.type] then
                        local path = "subSequences[" .. current.sequence .. "].events[" .. current.event .. "].value.consume"
                        add_resource_target(cache, current, path)
                    end
                    current.event = current.event + 1
                    event_budget = event_budget - 1
                end
            end
        end
    end
end

local function apply_item_use(delta_seconds)
    if not setting_boolean("infiniteItemUse", true) then return "Keep Used Items is off" end
    local session = resource_writes_ready("Keep Used Items")
    if not session then return end
    if not item_use_cache or item_use_cache.session ~= session then
        item_use_cache = resource_cache(session, "keen::actor::ActorSequenceResource", "Keep Used Items")
    end
    refresh_resources(item_use_cache, delta_seconds)
    discover_item_use_targets()
    local cache = item_use_cache
    apply_resource_targets(cache, "infiniteItemUse", function() return false end)
    if cache.current or cache.head <= cache.tail then
        overrides:waiting("Keep Used Items: scanning loaded action sequences")
    end
    return "Keep Used Items: " .. cache.scanned_count .. " action sequences checked, " ..
        #cache.targets .. " applicable actions found"
end

local function discover_production_targets()
    local cache = production_cache
    local resource_budget, recipe_budget = 4, 128
    while resource_budget > 0 and recipe_budget > 0 do
        local current = cache.current
        if not current then
            if cache.head > cache.tail then break end
            resource_budget = resource_budget - 1
            current = next_resource(cache)
        end
        if current then
            local recipe = current.data.recipes[current.recipe]
            if not recipe then
                finish_resource(cache, current)
            else
                local duration = recipe.craftingDuration.value
                if finite(duration) and duration > 0 then
                    if not cache.minimum_duration_ns or duration < cache.minimum_duration_ns then
                        cache.minimum_duration_ns = duration
                    end
                elseif not finite(duration) or duration < 0 then
                    cache.scan_failed = true
                    overrides:waiting("Production Time: a recipe duration is invalid")
                end
                current.recipe = current.recipe + 1
                recipe_budget = recipe_budget - 1
            end
        end
    end
    if not cache.current and cache.head > cache.tail and not cache.scan_failed then
        cache.scan_complete = true
    end
end

local function apply_production_time(delta_seconds)
    local mode = setting_string("productionTimeMode", "original")
    if mode ~= "original" and mode ~= "fixed" then
        overrides:waiting("Production Time Mode must be 'original' or 'fixed'")
        return
    end

    local seconds = shroudforge.settings.get("productionSeconds", 3)
    if mode == "fixed" and (not finite(seconds) or seconds < 1 or seconds > 900) then
        overrides:waiting("Production Time must be between 1 and 900 seconds")
        return
    end
    if mode == "fixed" then
        return "Shared recipe time " .. seconds .. " seconds is set for startup asset preparation"
    end

    local session = resource_writes_ready("Production Time")
    if not session then return end
    if not production_cache or production_cache.session ~= session or production_cache.mode ~= mode then
        production_cache = resource_cache(session, "keen::RecipeRegistryResource", "Production Time")
        production_cache.mode = mode
    end
    refresh_resources(production_cache, delta_seconds)
    discover_production_targets()
    local cache = production_cache
    if cache.current or cache.head <= cache.tail then overrides:waiting("Production Time: scanning loaded recipes") end
    if not cache.scan_complete or not cache.minimum_duration_ns then
        overrides:waiting("Production Time: finding the shortest timed recipe")
        return
    end
    return "Original recipe times preserved, shortest timed recipe " ..
        tostring(cache.minimum_duration_ns / 1000000000) .. " seconds"
end

local function update(delta_seconds)
    overrides:begin_update()
    overrides:restore_disabled(function(setting)
        if setting == "workshopTimeFactorPercent" then
            return setting_string("productionTimeMode", "original") == "original"
        end
        if setting == "productionSeconds" then return false end
        return setting_boolean(setting, true)
    end)
    local item_use_status, production_status, workshop_status
    if not overrides.fault then
        local elapsed = finite(delta_seconds) and math.max(0, delta_seconds) or 0.1
        production_status = apply_production_time(elapsed)
        workshop_status = apply_workshop_time_factor()
        item_use_status = apply_item_use(elapsed)
    end

    local details = {}
    if production_status then details[#details + 1] = production_status end
    if workshop_status then details[#details + 1] = workshop_status end
    if item_use_status then details[#details + 1] = item_use_status end
    if overrides.waiting_count > 0 and not overrides.fault then
        details[#details + 1] = overrides:summary("Crafting Essentials changes pending")
    end
    if overrides.fault then details[#details + 1] = overrides.fault end

    local state = overrides.fault and "write-failed" or
        (overrides.waiting_count > 0 and "waiting" or
            (overrides.writes > 0 and "write-confirmed" or "no-change"))
    report(state, #details > 0 and table.concat(details, " | ") or
        "Production duration scale is controlled through the typed GameSettings service")
end

return {
    update_interval_ms = 100,
    on_load = update,
    on_update = update,
}
