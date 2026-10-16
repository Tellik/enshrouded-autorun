-- Temporary read-only observation through the public Lua API.
local previous = {}
local item_resources, item_cursor, item_session, gliders
local function emit(key, value)
    value = tostring(value)
    if previous[key] == value then return end
    previous[key] = value
    shroudforge.log.info("[SF_SURVIVAL_READ_PROBE] " .. key .. " " .. value)
end
local function fields(values)
    local result = {}
    for key, value in pairs(values or {}) do result[#result + 1] = key .. "=" .. tostring(value) end
    table.sort(result)
    return table.concat(result, ",")
end
local function observe_gliders(session)
    if item_session ~= session then item_resources, item_cursor, gliders = nil, 1, {} item_session = session end
    if not item_resources then
        local ok, resources = pcall(game.assets.get_loaded_resources_by_type, "keen::ItemInfo")
        if not ok then emit("item-list", resources) return end
        item_resources = resources
        emit("item-count", #resources)
    end
    for _ = 1, 64 do
        local resource = item_resources[item_cursor]
        if not resource then return end
        item_cursor = item_cursor + 1
        local ok, data = pcall(function() return resource.data end)
        if ok and data.equipment.slot == "Glider" then
            local value = data.equipment.gliderConfig
            local item = data.itemId
            if type(item) == "table" or type(item) == "userdata" then item = item.value end
            gliders[item] = true
            emit("glider/" .. item, tostring(resource.guid) .. " " .. fields({
                accelerationForward=value.accelerationForward,
                airResistanceLongitudinal=value.airResistanceLongitudinal,
                airResistanceVertical=value.airResistanceVertical,
                yawAngleSpeed=value.yawAngleSpeed,
                updraftStrengthChargeStartValue=value.updraftStrengthChargeStartValue}))
        elseif not ok then emit("item-read/" .. item_cursor, data) end
    end
end
return {
    update_interval_ms = 250,
    on_load = function() emit("loaded", "read-only public API observer, no writes") end,
    on_update = function()
        local session = runtime.world.session_id()
        emit("session", session)
        if session == 0 then return end
        observe_gliders(session)
        local players, reason = runtime.ecs.query("keen::ecs::PlayerInput")
        if not players then emit("query", reason) return end
        emit("players", table.concat(players, ","))
        for _, player in ipairs(players) do
            local actor, actor_reason = runtime.ecs.read_fields(player, "keen::ecs::Actor", {"currentAction.triggerContext.usedItem.itemId"})
            if actor then
                local item = actor["currentAction.triggerContext.usedItem.itemId"]
                if type(item) == "table" or type(item) == "userdata" then item = item.value end
                emit(player .. "/action-item", "item=" .. tostring(item) .. " known-glider=" .. tostring(gliders[item] == true))
            else emit(player .. "/action-item", actor_reason) end
            for _, root in ipairs({"Health", "Stamina", "Mana", "Oxygen", "BodyHeat"}) do
                local snapshot, why = runtime.ecs.read_attributes(player, root)
                emit(player .. "/" .. root, snapshot and
                    ("scalar=" .. snapshot.scalar_type .. " component=" .. snapshot.component.qualified_name .. " " .. fields(snapshot.values)) or why)
            end
            local template, why = runtime.ecs.get_template_component(player, "keen::ecs::FallDamage")
            if template then
                local values, failure = template:read_fields({"fallDamageStartDistance", "fallDamageLethalDistance"})
                emit(player .. "/FallDamage", values and (template.key .. " " .. fields(values)) or failure)
            else emit(player .. "/FallDamage", why) end
        end
        local service, why = runtime.services.get("ecs::GameSettings")
        if service then
            local values, failure = service:read_fields({"enableDurability"})
            emit("GameSettings", values and (service.key .. " " .. fields(values)) or failure)
        else emit("GameSettings", why) end
    end,
}
