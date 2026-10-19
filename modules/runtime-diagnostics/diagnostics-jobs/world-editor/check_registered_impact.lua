-- Diagnostic A/B job for the original named isSameEntity callback.
-- All callback inputs and VM buffers are owned. No world mutation is requested.
local type_name = "keen::impact_nodes::IsSameEntity"
local cases = {
    {name="equal", a=305419896, b=305419896, expected=true},
    {name="different", a=305419896, b=305419897, expected=false},
    {name="zero", a=0, b=0, expected=true},
    {name="full-uint32", a=4294967295, b=0, expected=false},
}
local session, rounds, finished = 0, 0, false
local report = {schema=1, kind="registered-impact-owned-buffer-ab", type=type_name,
    world_mutations_requested=0, state="waiting-world", results={}}

local function publish(state, detail)
    report.state, report.detail = state, tostring(detail or "")
    report.session, report.rounds = session, rounds
    diagnostic.publish(report)
    if finished then diagnostic.finish() end
end

return {
    update_interval_ms=250,
    on_load=function() publish("waiting-world") end,
    on_update=function()
        if finished then return end
        local current = runtime.world.session_id()
        if current == 0 then return end
        if session ~= 0 and current ~= session then
            finished = true
            publish("aborted", "world session changed")
            return
        end
        session = current
        rounds = rounds + 1
        local ty, reason = runtime.types.resolve(type_name)
        if not ty or ty.qualified_name ~= type_name then
            publish("waiting-type", reason)
            return
        end
        local results = {}
        for _, case in ipairs(cases) do
            local result, error = runtime.impact.call(ty,
                {entityAId={id=case.a},entityBId={id=case.b}})
            if not result then
                if rounds >= 120 then finished=true end
                publish(finished and "failed" or "waiting-registry", error)
                return
            end
            if runtime.world.session_id() ~= session then
                finished=true
                publish("aborted", "world session changed during call")
                return
            end
            if type(result.branch) ~= "boolean" or result.branch ~= case.expected then
                finished=true
                publish("failed", case.name .. " returned unexpected branch " .. tostring(result.branch))
                return
            end
            results[#results+1] = {name=case.name, branch=result.branch, expected=case.expected}
        end
        -- Missing adapters must reject the type rather than execute world writes.
        local denied, denial = runtime.impact.call("keen::impact_nodes::DestroyEntity", {})
        if denied ~= nil or type(denial) ~= "string" or not denial:find("no validated", 1, true) then
            finished=true
            publish("failed", "unadmitted operation was not explicitly rejected")
            return
        end
        report.results = results
        report.unadmitted_operation = "rejected"
        finished=true
        publish("passed", "four original registered callback results matched A/B expectations")
        shroudforge.log.info("Registered Impact A/B passed: equal, different, zero and full uint32. World mutations requested=0.")
    end,
}
