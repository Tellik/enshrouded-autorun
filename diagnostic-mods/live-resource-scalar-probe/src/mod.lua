-- Private developer probe. Loaded Resource.data is read-only throughout.
-- This proves only one original ResourceService scalar path, not gameplay.
local PREFIX = "[SF_RESOURCE_SCALAR_PROBE_V1] "
local ITEM = "keen::ItemInfo"
local SLOT_ENUM, SLOT = "keen::EquipmentSlotType", "Glider"
local PATH = "equipment.gliderConfig.yawAngleSpeed"
local WAIT_LIMIT, OBSERVE_LIMIT, RESTORE_LIMIT, BATCH = 2400, 100, 300, 32
local S = {phase="wait", tick=0, phase_tick=0, successes=0, attempts=0, done=false, candidates={}, keys={}}

local function log(marker, detail)
    shroudforge.log.info(PREFIX .. marker .. " " .. (detail or ""))
end
local function call(fn, ...)
    local ok, value, reason = pcall(fn, ...)
    if not ok then return nil, tostring(value), true end
    return value, reason, false
end
local function phase(name)
    S.phase, S.phase_tick = name, S.tick
    log("PHASE", name)
end
local function age() return S.tick - S.phase_tick end
local function wait(reason)
    if S.tick % 50 == 1 then log("WAIT", "phase=" .. S.phase .. " reason=" .. tostring(reason)) end
end
local function finite(n) return type(n) == "number" and n == n and n > -math.huge and n < math.huge end
local function item_id(value)
    if type(value) == "table" or type(value) == "userdata" then value = value.value end
    if not finite(value) or value % 1 ~= 0 or value <= 0 or value > 4294967295 then return nil end
    return value
end
local function finish(result, reason)
    if S.done then return end
    S.done = true
    log("SUMMARY", string.format("result=%s reason=%s successful_writes=%d attempts=%d session=%s guid=%s part=%s item_id=%s baseline=%s target=%s resource_apply=%s resource_restore=%s cleanup_required=%s gameplay_verified=false",
        result, tostring(reason), S.successes, S.attempts, tostring(S.session), tostring(S.guid), tostring(S.part), tostring(S.item),
        tostring(S.baseline), tostring(S.target), tostring(S.observed_apply == true), tostring(S.observed_restore == true),
        tostring(S.owned == true or S.uncertain == true)))
end
local function session()
    local id, reason = call(runtime.world.session_id)
    if type(id) ~= "number" or id == 0 then return nil, reason or "nonzero session is not ready" end
    return id
end
local function same_session()
    local id, reason = session()
    if not id then return nil, reason end
    if id ~= S.session then return false, "world/ECS session changed" end
    return true
end
local function feature(name)
    local available, reason = call(runtime.has, name)
    return available == true, reason
end
local function inspect(resource)
    -- Reading this actual Resource.data snapshot is the only decoding path.
    if not resource or resource.source ~= "loaded-runtime" or resource.loaded ~= true or
        not resource.type or resource.type.qualified_name ~= ITEM then
        return nil, "not an original loaded ItemInfo snapshot"
    end
    local guid, part = resource.guid, resource.part
    if type(guid) ~= "string" or guid == "" or type(part) ~= "number" or part < 0 or part % 1 ~= 0 then
        return nil, "invalid reflected resource identity"
    end
    local data = resource.data
    if not data or not data.equipment then return nil, "ItemInfo equipment is unavailable" end
    local slot = data.equipment.slot
    if type(slot) ~= "string" or not S.enum.enum_fields[slot] then return nil, "slot is not a known reflected enum name" end
    if slot ~= SLOT then return {glider=false, guid=guid, part=part} end
    local id = item_id(data.itemId)
    local config = data.equipment.gliderConfig
    local yaw = config and config.yawAngleSpeed
    if not id or not finite(yaw) then return nil, "real Glider has no valid ItemId or yawAngleSpeed" end
    return {glider=true, guid=guid, part=part, item=id, yaw=yaw, resource=resource}
end
local function read_current()
    -- Never reuse .data from the writer result or an earlier Resource object.
    local resource, reason = call(game.assets.get_loaded_resource, S.guid, ITEM, S.part)
    if not resource then return nil, reason or "selected ResourceId is no longer loaded" end
    local pair, why = inspect(resource)
    if not pair then return nil, why, true end
    if pair.guid ~= S.guid or pair.part ~= S.part or not pair.glider or pair.item ~= S.item then
        return nil, "selected ResourceId/Glider/ItemId scope changed", true
    end
    local stable, detail = same_session()
    if not stable then return nil, detail, stable == false end
    return pair
end
local function write_scalar(resource, target, expected)
    if S.successes >= 2 then finish("WRITE_LIMIT", "two successful writes already reached") return nil end
    local stable, reason = same_session()
    if stable == false then finish("SESSION_CHANGED", reason) return nil end
    if not stable then wait(reason) return nil end
    if not feature("runtime.assets.write") then wait("resource scalar writes are not ready") return nil end
    S.attempts = S.attempts + 1
    local fresh, why, threw = call(game.assets.write_loaded_field, resource, PATH, target, expected)
    if not fresh then
        local detail = tostring(why)
        if threw or detail:lower():find("uncertain", 1, true) then
            S.uncertain = true
            finish("OUTCOME_UNCERTAIN", detail .. ", no automatic retry or blind restoration")
        else wait(detail) end
        return nil
    end
    S.successes = S.successes + 1
    return true
end
local function restore_once()
    local stable, reason = same_session()
    if stable == false then finish("SESSION_CHANGED", reason .. ", no restore in another world") return end
    if not stable then wait(reason) return end
    local current, why, invalid = read_current()
    if not current then if invalid then finish("CLEANUP_SCOPE_CHANGED", why) else wait(why) end return end
    if current.yaw == S.baseline then
        S.owned = false
        log("RESTORED_EXTERNALLY", "baseline is already present, no second write")
        S.external = true
        phase("restore_observe") return
    end
    if current.yaw ~= S.target then
        S.owned = false
        finish("CLEANUP_SCOPE_CHANGED", "external yawAngleSpeed change is preserved") return
    end
    if not write_scalar(current.resource, S.baseline, S.target) then return end
    S.owned = false
    log("RESTORE_APPLIED", "path=" .. PATH .. " value=" .. tostring(S.baseline) .. " successful_writes=" .. S.successes)
    phase("restore_observe")
end
local function step()
    if S.done then return end
    S.tick = S.tick + 1
    if S.phase == "wait" then
        if age() > WAIT_LIMIT then finish("STARTUP_TIMEOUT", "no write was performed") return end
        if not feature("runtime.assets.write") then wait("resource scalar writes are not ready") return end
        local id, reason = session()
        if not id then wait(reason) return end
        local metadata = call(runtime.types.get, SLOT_ENUM)
        if not metadata or metadata.qualified_name ~= SLOT_ENUM or metadata.primitive_type ~= "Enum" or
            not metadata.enum_fields or not metadata.enum_fields[SLOT] or metadata.enum_fields[SLOT].name ~= SLOT then
            wait("original EquipmentSlotType.Glider enum metadata is not ready") return
        end
        local resources, why = call(game.assets.get_loaded_resources_by_type, ITEM)
        if type(resources) ~= "table" or #resources == 0 then wait(why or "loaded ItemInfo catalog is empty") return end
        if session() ~= id then wait("session changed during inventory start") return end
        S.session, S.enum, S.resources, S.index = id, metadata, resources, 1
        log("INVENTORY_START", "session=" .. tostring(id) .. " resources=" .. #resources .. " slot_name=" .. SLOT .. " reflected_value=" .. tostring(metadata.enum_fields[SLOT].value))
        phase("inventory")
    elseif S.phase == "inventory" then
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then if age() > WAIT_LIMIT then finish("INVENTORY_TIMEOUT", reason) else wait(reason) end return end
        for _=1,BATCH do
            if S.index > #S.resources then break end
            local index = S.index
            local pair, why = inspect(S.resources[index])
            if not pair then finish("INVENTORY_REJECTED", "index=" .. index .. " reason=" .. tostring(why)) return end
            local key = pair.guid .. ":" .. tostring(pair.part)
            if S.keys[key] then finish("INVENTORY_REJECTED", "duplicate ResourceId in inventory") return end
            S.keys[key] = true
            if pair.glider then S.candidates[#S.candidates+1] = pair end
            S.index = index + 1
        end
        if S.index <= #S.resources then return end
        if #S.candidates == 0 then finish("NO_GLIDER", "complete loaded catalog has no real Glider slot") return end
        table.sort(S.candidates, function(a,b) if a.guid == b.guid then return a.part < b.part end return a.guid < b.guid end)
        local selected = S.candidates[1]
        S.guid, S.part, S.item = selected.guid, selected.part, selected.item
        log("RESOURCE_SELECTED", "real_gliders=" .. #S.candidates .. " guid=" .. S.guid .. " part=" .. S.part .. " item_id=" .. S.item .. " selection=sorted-resource-id-first equipped_glider_claim=false")
        S.resources, S.candidates, S.keys = nil, nil, nil
        phase("baseline")
    elseif S.phase == "baseline" or S.phase == "apply" then
        local applying = S.phase == "apply"
        if age() > RESTORE_LIMIT then finish(applying and "APPLY_TIMEOUT" or "BASELINE_TIMEOUT", "no successful change") return end
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then wait(reason) return end
        local current, why, invalid = read_current()
        if not current then if invalid then finish("BASELINE_SCOPE_CHANGED", why) else wait(why) end return end
        if not applying then
            local target = string.unpack("<f", string.pack("<f", current.yaw + 1))
            if current.yaw <= 0 or not finite(target) or target-current.yaw ~= 1 then
                finish("BASELINE_REJECTED", "yawAngleSpeed+1 must be a finite exact Float32 increment") return
            end
            S.baseline, S.target = current.yaw, target
            log("BASELINE", "path=" .. PATH .. " value=" .. tostring(S.baseline) .. " target=" .. tostring(S.target))
            phase("apply") return
        end
        if current.yaw ~= S.baseline then finish("APPLY_SCOPE_CHANGED", "external yawAngleSpeed changed before apply") return end
        if not write_scalar(current.resource, S.target, S.baseline) then return end
        S.owned = true
        log("APPLIED", "path=" .. PATH .. " value=" .. tostring(S.target) .. " successful_writes=" .. S.successes)
        phase("observe")
    elseif S.phase == "observe" or S.phase == "restore_observe" then
        local restoring = S.phase == "restore_observe"
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then
            if age() > OBSERVE_LIMIT then if restoring then finish("RESTORE_READ_TIMEOUT", reason) else phase("restore") end else wait(reason) end
            return
        end
        local current, why, invalid = read_current()
        if not current then
            if invalid then finish("READBACK_SCOPE_CHANGED", why)
            elseif age() > OBSERVE_LIMIT then if restoring then finish("RESTORE_READ_TIMEOUT", why) else phase("restore") end
            else wait(why) end
            return
        end
        local expected = restoring and S.baseline or S.target
        if current.yaw ~= expected then
            S.owned = false
            finish("READBACK_SCOPE_CHANGED", "external yawAngleSpeed is preserved, no blind restoration") return
        end
        if restoring then
            S.observed_restore = true
            log("RESOURCE_RESTORE_OBSERVED", "fresh_get_loaded_resource=true value=" .. tostring(current.yaw))
            finish(S.external and "RESTORED_EXTERNALLY" or (S.observed_apply and "COMPLETE" or "RESTORED_APPLY_UNCONFIRMED"), "independent ResourceService scalar reads agree, no gameplay effect claimed")
        else
            S.observed_apply = true
            log("RESOURCE_APPLY_OBSERVED", "fresh_get_loaded_resource=true value=" .. tostring(current.yaw))
            phase("restore")
        end
    elseif S.phase == "restore" then
        if age() > RESTORE_LIMIT then finish("CLEANUP_TIMEOUT", "bounded scoped restore retries exhausted") return end
        restore_once()
    end
end

return {
    update_interval_ms=100,
    on_load=function() log("START", "one real Glider resource yawAngleSpeed+1 then restore, no startup writes") end,
    on_update=function()
        local ok, reason = pcall(step)
        if not ok then finish("UNEXPECTED_ERROR", reason) end
    end,
    on_unload=function()
        if S.owned and not S.done and not S.uncertain then
            log("UNLOAD_CLEANUP", "one scoped expected-value restore attempt")
            local ok, reason = pcall(restore_once)
            if not ok then log("UNLOAD_CLEANUP_ERROR", tostring(reason)) end
        end
        if not S.done then finish(S.owned and "UNLOADED_CLEANUP_REQUIRED" or "UNLOADED", "no further updates after unload") end
    end,
}
