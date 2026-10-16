local last_state
local last_detail

local function report(state, detail)
    if state == last_state and detail == last_detail then return end
    last_state, last_detail = state, detail
    runtime.report_effect(state, detail)
end

local function update()
    if shroudforge.settings.get("autoRefill") == false then
        report("no-change", "Automatic stamina refill is disabled")
        return
    end
    if not runtime.has("runtime.ecs.read") or not runtime.has("runtime.ecs.write") then
        report("waiting", "Waiting for typed ECS reads and guarded writes in an authoritative world")
        return
    end
    local players, reason = runtime.ecs.query("keen::ecs::PlayerInput", "keen::ecs::Stamina")
    if not players then
        report("waiting", tostring(reason or "Player component query is pending"))
        return
    end
    if #players == 0 then
        report("no-target", "No player with initialized Stamina storage is currently present")
        return
    end
    local written = 0
    for _, entity in ipairs(players) do
        local stamina, why = runtime.ecs.read_attributes(entity, "Stamina")
        if not stamina then
            report("waiting", tostring(why or "Stamina snapshot is not stable yet"))
            return
        end
        local current, maximum = stamina.values.Stamina, stamina.values.Stamina_Max
        if type(current) ~= "number" or type(maximum) ~= "number" then
            report("write-failed", "The reflected Stamina root has no numeric current/maximum pair")
            return
        end
        if current < maximum then
            -- The API binds the initialized root on this entity, recalculates
            -- its original programs, and compares the complete root snapshot
            -- before the guarded native write and change notification.
            local applied, error = runtime.ecs.update_attribute(entity, "Stamina", maximum)
            if not applied then
                report("waiting", tostring(error or "Stamina update must be retried"))
                return
            end
            written = written + 1
        end
    end
    if written > 0 then
        report("write-confirmed", "Stamina refilled through the reflected attribute root and guarded ECS API")
    else
        report("no-change", "All observed player Stamina roots are already at maximum")
    end
end

return {
    update_interval_ms = 250,
    on_load = function()
        report("waiting", "Waiting for the authoritative world and typed ECS API")
    end,
    on_update = update,
    on_unload = function()
        report("no-change", "Automatic stamina refill stopped")
    end,
}
