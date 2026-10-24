runtime.require("runtime.lifecycle")

local active = false
local current_session = 0
local toggle_binding = nil
local cancel_binding = nil
local previous_toggle = false
local previous_cancel = false
local discovery_elapsed = 0
local status_elapsed = 0
local status_reports = 0
local trace_elapsed = 0
local trace_reports = 20

local function log(message)
    shroudforge.log.info("[autorun] " .. message)
end

local function bindings()
    return shroudforge.settings.get("toggleAutorun", ""),
        shroudforge.settings.get("cancelAutorun", "")
end

local function keys()
    return shroudforge.input.is_keybind_down("toggleAutorun"),
        shroudforge.input.is_keybind_down("cancelAutorun")
end

local function stop(reason)
    if active and runtime.world.player_input and runtime.world.player_input.autorun_status then
        local status = runtime.world.player_input.autorun_status()
        if status then
            log(string.format("Final native counts: hits=%d local=%d writes=%d axes=(%.2f,%.2f)",
                status.hits, status.localHits, status.writes, status.x, status.y))
        end
    end
    if runtime.world.player_input and runtime.world.player_input.autorun_set then
        runtime.world.player_input.autorun_set(false)
    end
    if active then log("OFF: " .. reason) end
    active = false
end

local function ready()
    if current_session == 0 or not runtime.world.player_input then
        return false, "no player world"
    end
    local input, reason = runtime.world.player_input.get()
    if not input then return false, reason end
    if input.session ~= current_session then
        return false, "local input belongs to another world session"
    end
    return true
end

return {
    update_interval_ms = 16,

    on_load = function()
        stop("mod loaded")
        current_session = runtime.world.session_id()
        toggle_binding, cancel_binding = bindings()
        previous_toggle, previous_cancel = keys()
        discovery_elapsed = 0
        status_elapsed = 0
        status_reports = 0
        trace_elapsed = 0
        trace_reports = 20
        log("Loaded. Toggle=" .. tostring(shroudforge.input.keybind_label("toggleAutorun"))
            .. "; cancel=" .. tostring(shroudforge.input.keybind_label("cancelAutorun")))
    end,

    on_update = function(delta_seconds)
        local session = runtime.world.session_id()
        local toggle_setting, cancel_setting = bindings()
        local toggle_down, cancel_down = keys()
        if session ~= current_session or toggle_setting ~= toggle_binding
            or cancel_setting ~= cancel_binding then
            stop(session ~= current_session and "world session changed" or "bindings changed")
            current_session = session
            toggle_binding, cancel_binding = toggle_setting, cancel_setting
            previous_toggle, previous_cancel = toggle_down, cancel_down
            discovery_elapsed = 0
            status_elapsed = 0
            status_reports = 0
            trace_elapsed = 0
            trace_reports = 20
            return
        end

        if type(delta_seconds) == "number" and delta_seconds > 0 and session ~= 0 then
            discovery_elapsed = discovery_elapsed + delta_seconds
            if discovery_elapsed >= 0.25 then
                discovery_elapsed = 0
                -- The native hook accepts only a recently verified unique
                -- local input. Refresh that identity outside its callback.
                local ok, reason = pcall(runtime.world.player_input.get)
                if not ok then stop("local input refresh failed: " .. tostring(reason)) end
            end
        end

        if cancel_down then
            stop("backward input")
        elseif toggle_down and not previous_toggle then
            if active then
                stop("toggle key")
            else
                trace_elapsed = 0
                trace_reports = 0
                local available, reason = ready()
                if available then
                    local success, why = runtime.world.player_input.autorun_set(true)
                    if success then
                        active = true
                        status_elapsed = 0
                        status_reports = 0
                        log("ON")
                    else
                        log("Cannot start: " .. tostring(why))
                    end
                else
                    log("Cannot start: " .. tostring(reason))
                end
            end
        elseif active then
            local status, reason = runtime.world.player_input.autorun_status()
            if not status or not status.active then
                stop("native request ended: " .. tostring(reason))
            else
                local success, why = runtime.world.player_input.autorun_set(true)
                if not success then stop("native request failed: " .. tostring(why)) end
            end
        end

        if trace_reports < 20 and type(delta_seconds) == "number" and delta_seconds > 0 then
            trace_elapsed = trace_elapsed + delta_seconds
            if trace_elapsed >= 0.5 then
                trace_elapsed = 0
                trace_reports = trace_reports + 1
                local status = runtime.world.player_input.autorun_status()
                if status then
                    log(string.format("Read-only trace: hook=%d first=%d other=%d axes=(%.2f,%.2f)",
                        status.hits, status.localHits, status.hits - status.localHits,
                        status.x, status.y))
                end
            end
        end

        if active and type(delta_seconds) == "number" and delta_seconds > 0 then
            status_elapsed = status_elapsed + delta_seconds
            if status_elapsed >= 1 and status_reports < 30 then
                status_elapsed = 0
                status_reports = status_reports + 1
                local status = runtime.world.player_input.autorun_status()
                if status then
                    log(string.format("Native counts: hits=%d local=%d writes=%d axes=(%.2f,%.2f)",
                        status.hits, status.localHits, status.writes, status.x, status.y))
                end
            end
        end

        previous_toggle, previous_cancel = toggle_down, cancel_down
    end,

    on_unload = function()
        stop("mod unloaded")
    end,
}
