runtime.require("runtime.lifecycle")

local active = false
local toggle_count = 0
local previous_toggle = false
local previous_cancel = false
local toggle_binding = nil
local cancel_binding = nil
local observation_elapsed = 0
local observed_session = 0
local observation_count = 0
local observation_status = {}

local movement_sources = {
    { name = "local-client", component = "keen::ecs::ClientPlayerInput",
      x = "data.movementStick.x", y = "data.movementStick.y" },
    { name = "world-input", component = "keen::ecs::PlayerInput",
      x = "fromClient.movementStick.x", y = "fromClient.movementStick.y" },
}

local function log(message)
    shroudforge.log.info("[autorun-probe] " .. message)
end

local function state_label()
    return active and "ON" or "OFF"
end

local function read_bindings()
    return shroudforge.settings.get("toggleProbe", ""),
        shroudforge.settings.get("cancelProbe", "")
end

local function sample_keys()
    return shroudforge.input.is_keybind_down("toggleProbe"),
        shroudforge.input.is_keybind_down("cancelProbe")
end

local function report_status()
    log("Diagnostic state=" .. state_label() .. "; toggles=" .. toggle_count
        .. "; toggle key=" .. tostring(shroudforge.input.keybind_label("toggleProbe"))
        .. "; cancel key=" .. tostring(shroudforge.input.keybind_label("cancelProbe"))
        .. ". No movement is applied.")
end

local function observation_result(source, result)
    if observation_status[source.name] ~= result and observation_count < 40 then
        observation_status[source.name] = result
        observation_count = observation_count + 1
        log("Read-only " .. source.name .. " input: " .. result)
    end
end

local function observe_movement(delta_seconds)
    if type(delta_seconds) ~= "number" or delta_seconds < 0 then return end
    observation_elapsed = observation_elapsed + delta_seconds
    if observation_elapsed < 0.25 then return end
    observation_elapsed = 0

    local session = runtime.world.session_id()
    if session ~= observed_session then
        observed_session = session
        observation_count = 0
        observation_status = {}
        log("Read-only world session=" .. tostring(session))
    end
    if session == 0 or observation_count >= 40 then return end

    for _, source in ipairs(movement_sources) do
        local entities, query_reason = runtime.ecs.query(source.component)
        if entities == nil then
            observation_result(source, "query unavailable: " .. tostring(query_reason))
        elseif #entities ~= 1 then
            observation_result(source, "matching entities=" .. tostring(#entities))
        else
            local fields, read_reason = runtime.ecs.read_fields(
                entities[1], source.component, { source.x, source.y })
            if fields == nil then
                observation_result(source, "fields unavailable: " .. tostring(read_reason))
            else
                local x, y = fields[source.x], fields[source.y]
                if type(x) ~= "number" or type(y) ~= "number" or x ~= x or y ~= y
                    or x == math.huge or x == -math.huge
                    or y == math.huge or y == -math.huge then
                    observation_result(source, "fields were not finite numbers")
                else
                    local label = string.format("movementStick=(%.2f, %.2f)", x, y)
                    observation_result(source, label)
                end
            end
        end
    end
end

shroudforge.ui.on_action("reportStatus", report_status)

return {
    update_interval_ms = 16,

    on_load = function()
        active = false
        toggle_count = 0
        toggle_binding, cancel_binding = read_bindings()
        -- An already held key must be released before its next press counts.
        previous_toggle, previous_cancel = sample_keys()
        observation_elapsed = 0
        observed_session = 0
        observation_count = 0
        observation_status = {}
        log("Loaded. Input-only diagnostic; character movement is unchanged.")
        report_status()
    end,

    on_update = function(delta_seconds)
        -- Observations never alter gameplay and are sampled at most four times
        -- per second. Failure is logged as unavailable without stopping hotkeys.
        local observed, reason = pcall(observe_movement, delta_seconds)
        if not observed and observation_status.error ~= tostring(reason) then
            observation_status.error = tostring(reason)
            log("Read-only movement observation unavailable: " .. tostring(reason))
        end
        local new_toggle, new_cancel = read_bindings()
        local toggle_down, cancel_down = sample_keys()

        if new_toggle ~= toggle_binding or new_cancel ~= cancel_binding then
            active = false
            toggle_binding, cancel_binding = new_toggle, new_cancel
            previous_toggle, previous_cancel = toggle_down, cancel_down
            log("Bindings changed. Diagnostic state=OFF; release held keys before testing.")
            report_status()
            return
        end

        if toggle_down ~= previous_toggle then
            log("Toggle input=" .. (toggle_down and "DOWN" or "UP"))
        end
        if cancel_down ~= previous_cancel then
            log("Cancel input=" .. (cancel_down and "DOWN" or "UP"))
        end

        -- Cancellation takes priority even when both keys arrive in one update.
        if cancel_down then
            if active then
                active = false
                log("Diagnostic state=OFF; cancelled by backward key.")
            end
        elseif toggle_down and not previous_toggle then
            active = not active
            toggle_count = toggle_count + 1
            log("Toggle #" .. toggle_count .. ": diagnostic state=" .. state_label()
                .. ". No movement is applied.")
        end

        previous_toggle, previous_cancel = toggle_down, cancel_down
    end,

    on_unload = function()
        active = false
        log("Unloaded. Diagnostic state=OFF; no movement was applied.")
    end,
}
