-- Temporary controlled developer probe. Only the existing typed API may write.
-- At most one successful +1 change and one successful restoration per load.
local PREFIX = "[SF_TYPED_WRITE_PROBE_V1] "
local WAIT_LIMIT = 2400 -- 100 ms scheduling, about four minutes plus call time.
local NETWORK_LIMIT = 100
local RESTORE_LIMIT = 300
local S = {phase="wait", tick=0, phase_tick=0, successes=0, attempts=0, done=false}
local ROOT_FIELDS = {"Stamina", "Stamina_Min", "Stamina_Max", "Stamina_RechargeRampup",
    "Stamina_RechargeTimeout", "Stamina_Recharge", "Stamina_Max_Base", "Stamina_Max_Adder"}
local SCOPE_FIELDS = {"Stamina_Min", "Stamina_Max_Base", "Stamina_RechargeRampup",
    "Stamina_RechargeTimeout", "Stamina_Recharge"}

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
local function finish(result, reason)
    if S.done then return end
    S.done = true
    log("SUMMARY", string.format("result=%s reason=%s successful_writes=%d attempts=%d session=%s entity=%s cleanup_required=%s baseline_adder=%s applied_adder=%s network_apply=%s network_restore=%s",
        result, tostring(reason), S.successes, S.attempts, tostring(S.session), tostring(S.entity),
        tostring(S.owned == true or S.uncertain == true), tostring(S.baseline and S.baseline.Stamina_Max_Adder),
        tostring(S.target_adder), tostring(S.network_apply == true), tostring(S.network_restore == true)))
end
local function session()
    local value, reason = call(runtime.world.session_id)
    if type(value) ~= "number" or value == 0 then return nil, reason or "nonzero session is not ready" end
    return value
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
local function valid_root(root)
    if type(root) ~= "table" or type(root.values) ~= "table" or
        root.scalar_type ~= "keen::sint32" or not root.component or
        root.component.qualified_name ~= "keen::ecs::Stamina" then
        return false, "unexpected reflected Stamina root or storage type"
    end
    local v, count = root.values, 0
    for _ in pairs(v) do count = count + 1 end
    if count ~= #ROOT_FIELDS then return false, "Stamina root field count differs" end
    for _, name in ipairs(ROOT_FIELDS) do
        local n = v[name]
        if type(n) ~= "number" or n ~= n or n % 1 ~= 0 or n < -2147483648 or n > 2147483647 then
            return false, "invalid signed scalar " .. name
        end
    end
    if v.Stamina_Max <= v.Stamina_Min or v.Stamina_Max > 1000000 or
        v.Stamina_Max_Base + v.Stamina_Max_Adder ~= v.Stamina_Max or
        v.Stamina < v.Stamina_Min or v.Stamina > v.Stamina_Max then
        return false, "Stamina Base/Adder/Maximum/current relationship is outside probe scope"
    end
    return true
end
local function read_root()
    local root, reason = call(runtime.ecs.read_attributes, S.entity, "Stamina")
    if not root then return nil, reason end
    local valid, why = valid_root(root)
    if not valid then return nil, why, true end
    return root
end
local function unchanged_scope(root)
    if root.root_hash ~= S.root_hash then return false, "attribute root identity changed" end
    for _, name in ipairs(SCOPE_FIELDS) do
        if root.values[name] ~= S.baseline[name] then return false, "external change to " .. name end
    end
    return true
end
local function read_network()
    return call(runtime.ecs.read_fields, S.entity, "keen::ecs::NetworkPlayerAttributes", {"stamina", "staminaMax"})
end
local function trace_log(result, marker)
    for _, step in ipairs(result.trace or {}) do
        if step.name == "Stamina_Max" then
            log(marker, string.format("current_bits=%s->%s maximum_bits=%s->%s steps=%d",
                tostring(step.before_bits[S.current_index + 1]), tostring(step.after_bits[S.current_index + 1]),
                tostring(step.before_bits[S.maximum_index + 1]), tostring(step.after_bits[S.maximum_index + 1]),
                #(result.trace or {})))
            return
        end
    end
    log("TRACE_MISSING", "successful update returned no Stamina_Max trace")
end
local function update(target)
    if S.successes >= 2 then finish("WRITE_LIMIT", "two successful changes already reached") return nil end
    local ready = feature("runtime.ecs.write")
    if not ready then wait("guarded writes are not ready") return nil end
    S.attempts = S.attempts + 1
    local result, reason, threw = call(runtime.ecs.update_attribute, S.entity, "Stamina_Max_Adder", target)
    if not result then
        local detail = tostring(reason)
        if threw or detail:lower():find("uncertain", 1, true) then
            -- A failed/throwing call with an uncertain outcome is never retried
            -- as if it had definitely done nothing.
            S.uncertain = true
            finish("OUTCOME_UNCERTAIN", detail .. ", inspect root before manual cleanup")
        else wait(detail) end
        return nil
    end
    S.successes = S.successes + 1
    return result
end
local function restore_once()
    local stable, reason = same_session()
    if stable == false then finish("SESSION_CHANGED", reason .. ", no restore into another world") return end
    if not stable then wait(reason) return end
    local root, why, invalid = read_root()
    if not root then
        if invalid then finish("CLEANUP_SCOPE_CHANGED", why) else wait(why) end
        return
    end
    local scope, detail = unchanged_scope(root)
    if not scope then finish("CLEANUP_SCOPE_CHANGED", detail .. ", original values are not overwritten") return end
    local v = root.values
    if v.Stamina_Max_Adder == S.baseline.Stamina_Max_Adder and v.Stamina_Max == S.baseline.Stamina_Max then
        S.owned = false
        log("RESTORED_EXTERNALLY", "baseline Adder/Maximum already present, no second write")
        phase("restore_observe")
        return
    end
    if v.Stamina_Max_Adder ~= S.target_adder or v.Stamina_Max ~= S.target_max then
        finish("CLEANUP_SCOPE_CHANGED", "external Adder/Maximum change, no blind restoration")
        return
    end
    local result = update(S.baseline.Stamina_Max_Adder)
    if not result then return end
    S.owned = false
    log("RESTORE_APPLIED", string.format("adder=%s max=%s current=%s successful_writes=%d",
        tostring(result.values and result.values.Stamina_Max_Adder), tostring(result.values and result.values.Stamina_Max),
        tostring(result.values and result.values.Stamina), S.successes))
    trace_log(result, "RESTORE_CALCULATION_TRACE")
    local valid, error = valid_root(result)
    if not valid or result.values.Stamina_Max_Adder ~= S.baseline.Stamina_Max_Adder or
        result.values.Stamina_Max ~= S.baseline.Stamina_Max then
        S.owned = true
        finish("RESTORE_RESULT_UNEXPECTED", error or "restore result differs from baseline scope")
        return
    end
    phase("restore_observe")
end

local function step()
    if S.done then return end
    S.tick = S.tick + 1
    if S.phase == "wait" then
        if age() > WAIT_LIMIT then finish("STARTUP_TIMEOUT", "no write was performed") return end
        if not feature("runtime.ecs.read") or not feature("runtime.ecs.write") then
            wait("typed reads and guarded writes must both be ready") return
        end
        local id, reason = session()
        if not id then wait(reason) return end
        local entities, why = call(runtime.ecs.query, "keen::ecs::PlayerInput", "keen::ecs::Stamina", "keen::ecs::NetworkPlayerAttributes")
        if not entities or #entities ~= 1 then wait(why or "exactly one matching player is required") return end
        local id_again = session()
        if id_again ~= id then wait("session changed during target selection") return end
        S.session, S.entity = id, entities[1]
        log("TARGET_BOUND", "session=" .. tostring(id) .. " entity=" .. tostring(S.entity))
        phase("baseline")
    elseif S.phase == "baseline" then
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then wait(reason) return end
        if age() > WAIT_LIMIT then finish("BASELINE_TIMEOUT", "no write was performed") return end
        local root, why, invalid = read_root()
        if not root then
            if invalid then finish("BASELINE_REJECTED", why) else wait(why) end
            return
        end
        if root.values.Stamina ~= root.values.Stamina_Max then wait("wait for full Stamina so +1 rescaling is observable") return end
        local network, error = read_network()
        if not network or network.staminaMax ~= root.values.Stamina_Max then wait(error or "network maximum must agree before apply") return end
        local current = call(runtime.ecs.get_attribute, "Stamina")
        local maximum = call(runtime.ecs.get_attribute, "Stamina_Max")
        if not current or not maximum or current.root_hash ~= root.root_hash or maximum.root_hash ~= root.root_hash then
            wait("attribute indices are not resolved") return
        end
        S.current_index, S.maximum_index = current.index, maximum.index
        S.baseline, S.root_hash = {}, root.root_hash
        for name, value in pairs(root.values) do S.baseline[name] = value end
        S.target_adder, S.target_max = root.values.Stamina_Max_Adder + 1, root.values.Stamina_Max + 1
        if S.target_adder > 2147483647 then finish("BASELINE_REJECTED", "Adder+1 outside signed domain") return end
        log("BASELINE", string.format("root=%s base=%d adder=%d max=%d current=%d network_max=%s",
            tostring(root.root_hash), S.baseline.Stamina_Max_Base, S.baseline.Stamina_Max_Adder,
            S.baseline.Stamina_Max, S.baseline.Stamina, tostring(network.staminaMax)))
        phase("apply")
    elseif S.phase == "apply" then
        if age() > RESTORE_LIMIT then finish("APPLY_TIMEOUT", "no successful change") return end
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then wait(reason) return end
        local root, why, invalid = read_root()
        if not root then if invalid then finish("APPLY_SCOPE_CHANGED", why) else wait(why) end return end
        local scope, error = unchanged_scope(root)
        if not scope or root.values.Stamina_Max_Adder ~= S.baseline.Stamina_Max_Adder or root.values.Stamina_Max ~= S.baseline.Stamina_Max then
            finish("APPLY_SCOPE_CHANGED", error or "external Adder/Maximum change") return
        end
        if root.values.Stamina ~= root.values.Stamina_Max then wait("wait for full Stamina before apply") return end
        local result = update(S.target_adder)
        if not result then return end
        S.owned = true
        log("APPLIED", string.format("adder=%s max=%s current=%s successful_writes=%d",
            tostring(result.values and result.values.Stamina_Max_Adder), tostring(result.values and result.values.Stamina_Max),
            tostring(result.values and result.values.Stamina), S.successes))
        trace_log(result, "APPLY_CALCULATION_TRACE")
        local valid, detail = valid_root(result)
        if not valid or not unchanged_scope(result) or result.values.Stamina_Max_Adder ~= S.target_adder or result.values.Stamina_Max ~= S.target_max then
            finish("APPLY_RESULT_UNEXPECTED", detail or "result scope differs, inspect before cleanup") return
        end
        phase("observe")
    elseif S.phase == "observe" or S.phase == "restore_observe" then
        local restoring = S.phase == "restore_observe"
        local stable, reason = same_session()
        if stable == false then finish("SESSION_CHANGED", reason) return end
        if not stable then
            if age() > NETWORK_LIMIT then
                if restoring then finish("RESTORE_READ_TIMEOUT", reason) else phase("restore") end
            else wait(reason) end
            return
        end
        local root, why, invalid = read_root()
        if not root then
            if invalid then finish("READBACK_SCOPE_CHANGED", why)
            elseif age() > NETWORK_LIMIT then
                if restoring then finish("RESTORE_READ_TIMEOUT", why) else phase("restore") end
            else wait(why) end
            return
        end
        local scope, error = unchanged_scope(root)
        local expected_adder = restoring and S.baseline.Stamina_Max_Adder or S.target_adder
        local expected_max = restoring and S.baseline.Stamina_Max or S.target_max
        if not scope or root.values.Stamina_Max_Adder ~= expected_adder or root.values.Stamina_Max ~= expected_max then
            finish("READBACK_SCOPE_CHANGED", error or "Adder/Maximum changed externally, no blind cleanup") return
        end
        if not S[S.phase .. "_root"] then
            S[S.phase .. "_root"] = true
            log(restoring and "RESTORE_ROOT_OBSERVED" or "APPLY_ROOT_OBSERVED",
                string.format("adder=%d max=%d current=%d", root.values.Stamina_Max_Adder, root.values.Stamina_Max, root.values.Stamina))
        end
        local network, error = read_network()
        if network and network.staminaMax == expected_max then
            if restoring then
                S.network_restore = true
                log("NETWORK_RESTORE_OBSERVED", "staminaMax=" .. tostring(network.staminaMax))
                finish(S.network_apply and "COMPLETE" or "RESTORED_NETWORK_APPLY_UNCONFIRMED", "typed root restored and network maximum agrees")
            else
                S.network_apply = true
                log("NETWORK_APPLY_OBSERVED", "staminaMax=" .. tostring(network.staminaMax))
                phase("restore")
            end
        elseif age() > NETWORK_LIMIT then
            log(restoring and "NETWORK_RESTORE_TIMEOUT" or "NETWORK_APPLY_TIMEOUT", tostring(error or (network and network.staminaMax)))
            if restoring then finish("RESTORED_NETWORK_UNCONFIRMED", "root restored, network maximum did not agree in time") else phase("restore") end
        else wait(error or "waiting for independent network maximum") end
    elseif S.phase == "restore" then
        if age() > RESTORE_LIMIT then finish("CLEANUP_TIMEOUT", "original Adder could not be restored within bounded retries") return end
        restore_once()
    end
end

return {
    update_interval_ms = 100,
    on_load = function() log("START", "controlled Adder+1 then restore, no startup require or writes") end,
    on_update = function()
        local ok, reason = pcall(step)
        if not ok then
            log("UNEXPECTED_ERROR", tostring(reason))
            if S.owned and not S.done then phase("restore") else finish("ERROR", reason) end
        end
    end,
    on_unload = function()
        if S.owned and not S.done then
            log("UNLOAD_CLEANUP", "one scoped restore attempt before unload")
            local ok, error = pcall(restore_once)
            if not ok then log("UNLOAD_CLEANUP_ERROR", tostring(error)) end
        end
        if not S.done then finish(S.owned and "UNLOADED_CLEANUP_REQUIRED" or "UNLOADED", "no further update callbacks after unload") end
    end,
}
