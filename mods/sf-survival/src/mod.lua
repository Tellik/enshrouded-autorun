local overrides = runtime.overrides.new()
local last_state, last_detail

local attributes = {
    {setting="infiniteStamina", root="Stamina", field="Stamina_Min", maximum="Stamina_Max"},
    {setting="infiniteMana", root="Mana", field="Mana_Min", maximum="Mana_Max"},
    {setting="infiniteUnderwaterBreath", root="Oxygen", field="Oxygen_Min", maximum="Oxygen_Max"},
    {setting="infiniteFrostTimer", root="BodyHeat", field="BodyHeat_Min", maximum="BodyHeat_Max"},
    {setting="fastHealthRegen", root="Health", field="Health_Recharge"},
    {setting="fastManaRegen", root="Mana", field="Mana_Recharge"},
}

local function enabled(key)
    return shroudforge.settings.get(key, false) == true
end

local function report(state, detail)
    if last_state == state and last_detail == detail then return end
    last_state, last_detail = state, detail
    runtime.report_effect(state, detail)
end

local function finite(value)
    return type(value) == "number" and value == value and math.abs(value) < math.huge
end

local function attribute_target(feature, snapshot, baseline)
    if feature.maximum then
        local maximum = snapshot.values[feature.maximum]
        if not finite(maximum) or maximum <= 0 then return nil, "A positive original maximum is required" end
        return maximum
    end
    if baseline <= 0 then return nil, "Natural " .. feature.root .. " regeneration is not active" end
    -- The original Recharge programs clamp this input to 0..65536.
    return math.min(baseline * 100, 65536)
end

local function any_enabled()
    for _, feature in ipairs(attributes) do if enabled(feature.setting) then return true end end
    return enabled("noFallDamage") or enabled("infiniteWeaponDurability")
end

local function update()
    local session = runtime.world.session_id()
    if session == 0 then report("waiting", "Waiting for an authoritative world") return end
    overrides:begin_update()
    overrides:restore_disabled(enabled)
    if overrides.fault then report("write-failed", overrides.fault) return end
    if not any_enabled() then
        report(overrides:has_ownership() and "waiting" or "no-change", overrides:summary("All Survival settings are off"))
        return
    end
    if not runtime.has("runtime.ecs.read") or not runtime.has("runtime.ecs.write") then
        report("waiting", "Waiting for typed ECS reads and the authoritative write scope") return
    end

    local players, reason = runtime.ecs.query("keen::ecs::PlayerInput")
    if not players then report("waiting", tostring(reason or "Player query is pending")) return end
    if #players == 0 then report("no-target", "No live player entity is available") return end

    for _, entity in ipairs(players) do
        for _, feature in ipairs(attributes) do
            if enabled(feature.setting) then
                local target, failure = runtime.overrides.attribute(entity, feature.field)
                if target then
                    overrides:apply(target.key, feature.setting, target, function(snapshot, baseline)
                        return attribute_target(feature, snapshot, baseline)
                    end)
                else overrides:waiting(feature.setting .. ": " .. tostring(failure)) end
            end
        end
        if enabled("noFallDamage") then
            local view, why = runtime.ecs.get_template_component(entity, "keen::ecs::FallDamage")
            if view then
                local target, failure = runtime.overrides.template(view,
                    "fallDamageStartDistance", {"fallDamageLethalDistance"})
                if target then overrides:apply(target.key, "noFallDamage", target, function(snapshot)
                    local lethal = snapshot.values.fallDamageLethalDistance
                    if not finite(lethal) then return nil, "FallDamage lethal distance is not finite" end
                    -- The original calculation disables fall damage at Start >= Lethal.
                    return math.max(snapshot.value, lethal)
                end)
                else overrides:waiting("No Fall Damage: " .. tostring(failure)) end
            else overrides:waiting("No Fall Damage: " .. tostring(why)) end
        end
    end

    if enabled("infiniteWeaponDurability") then
        local service, why = runtime.services.get("ecs::GameSettings")
        if service then
            local target, failure = runtime.overrides.service(service, "enableDurability")
            if target then
                overrides:apply(target.key, "infiniteWeaponDurability", target, function() return false end)
            else overrides:waiting("Infinite Weapon Durability: " .. tostring(failure)) end
        else overrides:waiting("Infinite Weapon Durability: " .. tostring(why)) end
    end

    local state = overrides.fault and "write-failed" or
        (overrides.waiting_count > 0 and "waiting" or (overrides.writes > 0 and "write-confirmed" or "no-change"))
    report(state, overrides:summary("Original Survival inputs are active"))
end

return {
    update_interval_ms = 100,
    on_load = function()
        report("waiting", "SF Survival loaded. Waiting for typed engine access")
    end,
    on_update = update,
}
