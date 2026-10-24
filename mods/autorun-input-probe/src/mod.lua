runtime.require("runtime.lifecycle")

local active = false
local toggle_count = 0
local previous_toggle = false
local previous_cancel = false
local toggle_binding = nil
local cancel_binding = nil

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

shroudforge.ui.on_action("reportStatus", report_status)

return {
    update_interval_ms = 16,

    on_load = function()
        active = false
        toggle_count = 0
        toggle_binding, cancel_binding = read_bindings()
        -- An already held key must be released before its next press counts.
        previous_toggle, previous_cancel = sample_keys()
        log("Loaded. Input-only diagnostic; character movement is unchanged.")
        report_status()
    end,

    on_update = function()
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
