-- Private developer probe. Requires write_fields' optional expected_fields argument.
-- At most two successful writes, both to the original reflected state field.
local PREFIX = "[SF_ENGINE_FLIGHT_PROBE_V1] "
local LOCOMOTION = "keen::ecs::DynamicLocomotion"
local GRAVITY = "keen::ecs::Gravity"
local ENUM = "keen::ecs::LocomotionState"
local TARGET = "BuildingFlying"
local WAIT_LIMIT, OBSERVE_LIMIT, RESTORE_LIMIT = 2400, 100, 300
local REQUIRED_SAMPLES = 3
local S = {phase="wait", tick=0, phase_tick=0, successes=0, attempts=0, samples=0, done=false}

local function log(marker, detail)
    shroudforge.log.info(PREFIX .. marker .. " " .. (detail or ""))
end
local function call(fn, ...)
    local ok, value, reason = pcall(fn, ...)
    if not ok then return nil, tostring(value), true end
    return value, reason, false
end
local function phase(name)
    S.phase, S.phase_tick, S.samples = name, S.tick, 0
    log("PHASE", name)
end
local function age() return S.tick - S.phase_tick end
local function wait(reason)
    if S.tick % 50 == 1 then log("WAIT", "phase=" .. S.phase .. " reason=" .. tostring(reason)) end
end
local function finish(result, reason)
    if S.done then return end
    S.done = true
    log("SUMMARY", string.format("result=%s reason=%s successful_writes=%d attempts=%d session=%s entity=%s baseline_state=%s engine_gravity_apply=%s engine_gravity_restore=%s cleanup_required=%s",
        result, tostring(reason), S.successes, S.attempts, tostring(S.session), tostring(S.entity),
        tostring(S.baseline), tostring(S.engine_apply == true), tostring(S.engine_restore == true),
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
local function valid_state(value)
    -- Production Enum decoding returns the name string, not a numeric/table wrapper.
    return type(value) == "string" and S.enum and S.enum.enum_fields[value] ~= nil
end
local function read_state()
    local value, reason = call(runtime.ecs.read_fields, S.entity, LOCOMOTION, {"state"})
    if not value then return nil, reason end
    if type(value) ~= "table" or not valid_state(value.state) then
        return nil, "state is not a known reflected enum name", true
    end
    return value.state
end
local function read_pair()
    local state, reason, invalid = read_state()
    if not state then return nil, reason, invalid end
    local gravity, why = call(runtime.ecs.read_fields, S.entity, GRAVITY, {"isActive"})
    if not gravity then return nil, why end
    if type(gravity) ~= "table" or type(gravity.isActive) ~= "boolean" then
        return nil, "Gravity.isActive is not a reflected boolean", true
    end
    -- These independent snapshots are bracketed by state and session readbacks.
    local again, error, bad = read_state()
    if not again then return nil, error, bad end
    local stable, detail = same_session()
    if not stable then return nil, detail, stable == false end
    if state ~= again then return nil, "state changed during independent Gravity read" end
    return {state=state, gravity=gravity.isActive}
end
local function write_state(target, expected)
    if S.successes >= 2 then finish("WRITE_LIMIT", "two successful writes already reached") return nil end
    local stable, reason = same_session()
    if stable == false then finish("SESSION_CHANGED", reason) return nil end
    if not stable then wait(reason) return nil end
    if not feature("runtime.ecs.write") then wait("guarded writes are not ready") return nil end
    if not valid_state(target) or not valid_state(expected) then
        finish("ENUM_REJECTED", "write and expected values must be known enum names") return nil
    end
    S.attempts = S.attempts + 1
    local ok, why, threw = call(runtime.ecs.write_fields, S.entity, LOCOMOTION,
        {state=target}, {state=expected})
    if ok ~= true then
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
    if stable == false then finish("SESSION_CHANGED", reason .. ", no restore into another world") return end
    if not stable then wait(reason) return end
    local state, why, invalid = read_state()
    if not state then
        if invalid then finish("CLEANUP_SCOPE_CHANGED", why) else wait(why) end
        return
    end
    if state ~= TARGET then
        S.owned = false
        log("EXTERNAL_STATE_OBSERVED", "state=" .. state .. ", no baseline write over external state")
        S.external = true
        phase("restore_observe")
        return
    end
    if not write_state(S.baseline, TARGET) then return end
    S.owned = false
    log("RESTORE_APPLIED", "state=" .. S.baseline .. " successful_writes=" .. S.successes)
    phase("restore_observe")
end
local function step()
    if S.done then return end
    S.tick = S.tick + 1
    if S.phase == "wait" then
        if age() > WAIT_LIMIT then finish("STARTUP_TIMEOUT", "no write was performed") return end
        if not feature("runtime.ecs.read") or not feature("runtime.ecs.write") then wait("typed reads and guarded writes must be ready") return end
        if not S.enum then
            local metadata = call(runtime.types.get, ENUM)
            if not metadata or metadata.qualified_name ~= ENUM or metadata.primitive_type ~= "Enum" or
                not metadata.enum_fields or not metadata.enum_fields[TARGET] or
                metadata.enum_fields[TARGET].name ~= TARGET then
                wait("original BuildingFlying enum metadata is not ready") return
            end
            S.enum = metadata
            log("ENUM_RESOLVED", "type=" .. ENUM .. " member=" .. TARGET .. " value=" .. tostring(metadata.enum_fields[TARGET].value))
        end
        local id, reason = session()
        if not id then wait(reason) return end
        local entities, why = call(runtime.ecs.query, "keen::ecs::PlayerInput", LOCOMOTION, GRAVITY)
        if not entities or #entities ~= 1 then wait(why or "exactly one matching player is required") return end
        if session() ~= id then wait("session changed during target selection") return end
        S.session, S.entity = id, entities[1]
        log("TARGET_BOUND", "session=" .. tostring(id) .. " entity=" .. tostring(S.entity))
        phase("baseline")
    elseif S.phase == "baseline" then
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then wait(reason) return end
        if age() > WAIT_LIMIT then finish("BASELINE_TIMEOUT", "no write was performed") return end
        local pair, why, invalid = read_pair()
        if not pair then if invalid then finish("BASELINE_REJECTED", why) else wait(why) end return end
        if pair.state == TARGET then wait("baseline is already BuildingFlying, no change is owned") return end
        if not pair.gravity then wait("original Gravity.isActive must be true before apply") return end
        S.baseline = pair.state
        log("BASELINE", "state=" .. S.baseline .. " gravity_active=true")
        phase("apply")
    elseif S.phase == "apply" then
        if age() > RESTORE_LIMIT then finish("APPLY_TIMEOUT", "no successful change") return end
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then wait(reason) return end
        local pair, why, invalid = read_pair()
        if not pair then if invalid then finish("APPLY_SCOPE_CHANGED", why) else wait(why) end return end
        if pair.state ~= S.baseline then finish("APPLY_SCOPE_CHANGED", "external state changed before apply") return end
        if not pair.gravity then wait("Gravity baseline is no longer active") return end
        if not write_state(TARGET, S.baseline) then return end
        S.owned = true
        log("APPLIED", "state=" .. TARGET .. " successful_writes=" .. S.successes)
        phase("observe")
    elseif S.phase == "observe" or S.phase == "restore_observe" then
        local restoring = S.phase == "restore_observe"
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then
            if age() > OBSERVE_LIMIT then
                if restoring then finish("RESTORE_READ_TIMEOUT", reason) else phase("restore") end
            else wait(reason) end
            return
        end
        local pair, why, invalid = read_pair()
        if not pair then
            S.samples = 0
            if invalid then finish("READBACK_REJECTED", why)
            elseif age() > OBSERVE_LIMIT then
                if restoring then finish("RESTORE_READ_TIMEOUT", why) else phase("restore") end
            else wait(why) end
            return
        end
        if restoring and pair.state == TARGET then
            -- A later activation belongs to another actor. Never keep fighting it.
            finish("EXTERNAL_REACTIVATION", "BuildingFlying observed after restoration, no third write") return
        elseif not restoring and pair.state ~= TARGET then
            S.owned, S.external = false, true
            log("EXTERNAL_STATE_OBSERVED", "state=" .. pair.state .. ", no baseline write over external state")
            phase("restore_observe") return
        end
        local expected_gravity = restoring
        if pair.gravity == expected_gravity then
            S.samples = S.samples + 1
            log(restoring and "RESTORE_SAMPLE" or "ENGINE_SAMPLE", "state=" .. pair.state .. " gravity_active=" .. tostring(pair.gravity) .. " consecutive=" .. S.samples)
            if S.samples >= REQUIRED_SAMPLES then
                if restoring then
                    S.engine_restore = true
                    log("ENGINE_GRAVITY_RESTORE_OBSERVED", "normal state and Gravity.isActive=true across multiple updates")
                    finish(S.external and "EXTERNAL_STATE_PRESERVED" or (S.engine_apply and "COMPLETE" or "RESTORED_ENGINE_APPLY_UNCONFIRMED"),
                        "only state was written, independent original Gravity state observed")
                else
                    S.engine_apply = true
                    log("ENGINE_GRAVITY_APPLY_OBSERVED", "BuildingFlying and Gravity.isActive=false across multiple updates")
                    phase("restore")
                end
            end
        else
            S.samples = 0
            wait("waiting for independent original Gravity transition")
        end
        if not S.done and S.phase == (restoring and "restore_observe" or "observe") and age() > OBSERVE_LIMIT then
            log(restoring and "ENGINE_GRAVITY_RESTORE_TIMEOUT" or "ENGINE_GRAVITY_APPLY_TIMEOUT", "state=" .. pair.state .. " gravity_active=" .. tostring(pair.gravity))
            if restoring then finish("RESTORED_ENGINE_GRAVITY_UNCONFIRMED", "state restored, Gravity reactivation not observed") else phase("restore") end
        end
    elseif S.phase == "restore" then
        if age() > RESTORE_LIMIT then finish("CLEANUP_TIMEOUT", "scoped state restoration exhausted bounded retries") return end
        restore_once()
    end
end

return {
    update_interval_ms = 100,
    on_load = function() log("START", "original BuildingFlying state then scoped restore, expected_fields required, no startup writes") end,
    on_update = function()
        local ok, reason = pcall(step)
        if not ok then finish("UNEXPECTED_ERROR", reason) end
    end,
    on_unload = function()
        if S.owned and not S.done and not S.uncertain then
            log("UNLOAD_CLEANUP", "one scoped expected-state restore attempt")
            local ok, reason = pcall(restore_once)
            if not ok then log("UNLOAD_CLEANUP_ERROR", tostring(reason)) end
        end
        if not S.done then finish(S.owned and "UNLOADED_CLEANUP_REQUIRED" or "UNLOADED", "no further updates after unload") end
    end,
}
