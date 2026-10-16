local wanted = true
local stopped
local owned_session
local owned = {}
local last_state, last_detail

local function report(state, detail)
    if state == last_state and detail == last_detail then return end
    last_state, last_detail = state, detail
    runtime.report_effect(state, detail)
end

local function stop(reason)
    stopped = tostring(reason)
    report("write-failed", "No Fall Damage stopped: " .. stopped)
end

local function session_id()
    return runtime.world and runtime.world.session_id and runtime.world.session_id() or 0
end

local function finite(value)
    return type(value) == "number" and value == value and value > -math.huge and value < math.huge
end

local function fall_config(resource)
    local found
    for index, component in ipairs(resource.data.components) do
        if component.type == "keen::ecs::FallDamage" then
            if found then return nil, "Template has multiple FallDamage variants" end
            local value = component.value
            if not finite(value.fallDamageStartDistance) or not finite(value.fallDamageLethalDistance) then
                return nil, "FallDamage distances are not finite reflected scalars"
            end
            found = {index = index, start = value.fallDamageStartDistance,
                lethal = value.fallDamageLethalDistance,
                path = "components[" .. index .. "].value.fallDamageStartDistance"}
        end
    end
    return found, found and nil or "Player template has no FallDamage configuration"
end

local function read_current(resource)
    local ok, fresh = pcall(game.assets.get_loaded_resource, resource.guid, resource.type, resource.part)
    if not ok or not fresh then return nil, tostring(fresh or "Loaded template is unavailable") end
    local decoded, config, reason = pcall(fall_config, fresh)
    if not decoded then return nil, tostring(config) end
    if not config then return nil, reason end
    return fresh, config
end

local function key(resource)
    return tostring(resource.guid) .. "/" .. resource.type.qualified_name .. "/" .. tostring(resource.part)
end

local function write(resource, path, value, expected)
    local ok, updated, reason = pcall(game.assets.write_loaded_field, resource, path, value, expected)
    if not ok then stop("uncertain Lua write outcome: " .. tostring(updated)); return nil end
    if not updated then
        if tostring(reason):lower():find("uncertain", 1, true) then stop(reason)
        else report("waiting", tostring(reason or "Resource scalar update is not ready")) end
        return nil
    end
    return updated
end

local function restore()
    if stopped then return false end
    if next(owned) == nil then owned_session = nil; return true end
    if session_id() ~= owned_session then
        stop("World session changed before restoration. No other session was modified")
        return false
    end
    if not runtime.has("runtime.assets.write") then
        report("waiting", "Waiting for resource access to restore the original fall configuration")
        return false
    end
    for id, entry in pairs(owned) do
        local fresh, current = read_current(entry.resource)
        if not fresh then report("waiting", current); return false end
        if current.index ~= entry.index or current.lethal ~= entry.lethal or current.start ~= entry.applied then
            stop("Fall configuration changed outside this mod. Restoration did not overwrite it")
            return false
        end
        if session_id() ~= owned_session then stop("World session changed before restoration"); return false end
        if not write(fresh, entry.path, entry.baseline, entry.applied) then return false end
        local verified, actual = read_current(entry.resource)
        if not verified or session_id() ~= owned_session or actual.index ~= entry.index or
            actual.start ~= entry.baseline or actual.lethal ~= entry.lethal then
            stop("uncertain restoration readback: " .. tostring(actual))
            return false
        end
        owned[id] = nil
    end
    owned_session = nil
    report("write-confirmed", "Original player-template fall configuration restored and read back")
    return true
end

local function update()
    if stopped then return end
    if owned_session and session_id() ~= owned_session then
        stop("World session changed with an owned resource update. No new session was modified")
        return
    end
    if not wanted then
        local had_owned = next(owned) ~= nil
        if restore() and not had_owned then report("no-change", "No Fall Damage is disabled") end
        return
    end
    if session_id() == 0 or not runtime.has("runtime.ecs.read") or not runtime.has("runtime.assets.write") then
        report("waiting", "Waiting for a world, typed player reads and guarded resource scalar writes")
        return
    end
    local session = session_id()
    local players, reason = runtime.ecs.query("keen::ecs::PlayerInput", "keen::ecs::DynamicFallDamage")
    if not players then report("waiting", tostring(reason or "Player query is pending")); return end
    if #players == 0 then report("no-target", "No player with DynamicFallDamage is present"); return end
    for _, entity in ipairs(players) do
        local template, why = runtime.ecs.get_template(entity)
        if not template then report("waiting", tostring(why or "Player template is pending")); return end
        local fresh, config = read_current(template)
        if not fresh then report("waiting", config); return end
        local id = key(fresh)
        local entry = owned[id]
        if entry then
            if config.index ~= entry.index or config.lethal ~= entry.lethal or config.start ~= entry.applied then
                stop("Owned fall configuration changed outside this mod")
                return
            end
        elseif config.start < config.lethal then
            entry = {resource = fresh, index = config.index, path = config.path,
                baseline = config.start, lethal = config.lethal, applied = config.lethal}
            if session_id() ~= session then report("waiting", "World changed during template lookup"); return end
            -- Only the original Start >= Lethal disable branch is requested.
            -- The native API compares this exact scalar again before writing.
            owned[id], owned_session = entry, session
            if not write(fresh, config.path, config.lethal, config.start) then
                if not stopped then owned[id] = nil; if next(owned) == nil then owned_session = nil end end
                return
            end
            local verified, actual = read_current(fresh)
            if not verified or session_id() ~= session or actual.index ~= entry.index or
                actual.start ~= entry.applied or actual.lethal ~= entry.lethal then
                stop("uncertain fall configuration readback: " .. tostring(actual))
                return
            end
            report("write-confirmed", "Original player-template FallDamage disable branch set and read back. Shared templates affect every user of that template")
        end
    end
    if next(owned) == nil then report("no-change", "Observed player templates already disable fall damage") end
end

shroudforge.ui.on_action("toggleFallDamagePatch", function()
    if stopped then report("write-failed", "No Fall Damage remains stopped: " .. stopped); return end
    wanted = not wanted
    if wanted then update()
    else
        local had_owned = next(owned) ~= nil
        if restore() and not had_owned then report("no-change", "No Fall Damage is disabled") end
    end
end)

return {
    update_interval_ms = 250,
    on_load = function() report("waiting", "Waiting for the actual player template and resource API") end,
    on_update = update,
    on_unload = function()
        wanted = false
        if stopped then report("write-failed", "No automatic restoration after an uncertain or changed resource outcome: " .. stopped)
        elseif not restore() then report("write-failed", "Original fall configuration restoration was not confirmed") end
    end,
}
