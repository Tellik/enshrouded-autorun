-- Private API fixture. Executes the actual read-only action without a game.
local output = "report.json"
local function fixture(mode)
    local f = {session=0, files={}, type_calls=0, registry_calls=0, queries=0, reads=0, forbidden_calls=0, version=7,
        unreadable_calls=0}
    local environment = setmetatable({}, {__index=_G})
    local function forbidden()
        f.forbidden_calls = f.forbidden_calls + 1
        error("a non-read API was accessed")
    end
    environment.diagnostic = {
        encode=encode_diagnostic_report,
        publish=function(report)
            local text = encode_diagnostic_report(report)
            f.files[output] = text
            f.report = parse_typed_input_report(text)
        end,
        finish=function() error("typed-input observations must remain active across sessions") end,
    }
    environment.shroudforge = {log={info=function() end}}
    local ecs = {
        get_component=function(name)
            f.registry_calls = f.registry_calls + 1
            assert(name:match("^keen::ecs::"), "component must use its full original type name")
            if mode=="registry-wait" and f.registry_calls<=10 then return nil, "registry is starting" end
            local readable = true
            if mode=="readability-wait" and name=="keen::ecs::PlayerInput" then
                f.unreadable_calls = f.unreadable_calls + 1
                readable = f.unreadable_calls > 2
            end
            return {qualified_name=name, index=41, runtime_size=8, storage="entity",
                read_available=readable, write_available=false, read_fields_available=readable,
                write_fields_available=false, write_fields_reason="original write phase is unavailable"}
        end,
        query=function(...)
            f.queries = f.queries + 1
            if mode=="empty-query" then return {} end
            if mode=="query-unavailable" then return nil, "owner is not ready" end
            if mode=="query-pending" and f.queries<=3 then return nil, "query scan is pending" end
            local first = (...)
            return first=="keen::ecs::CurrentTransform" and {301} or {101,102,103}
        end,
        read_fields=function(handle, component, paths)
            f.reads = f.reads + 1
            assert(handle==101 or handle==102 or handle==301, "samples must be bounded to two players and one prop")
            if mode=="read-nil" then return nil end
            if mode=="missing-field" then return {} end
            if mode=="mid-session" and f.reads==2 then f.session=18 end
            local fields = {}
            for _, path in ipairs(paths) do
                fields[path] = path:match("%.version$") and f.version or 0
                if path:match("Flags$") or path:match("digitalInput$") or path:match("^inputConsumer%.") then
                    if mode=="named-bitmasks" then
                        fields[path] = fixture_named_mask(path:match("primaryFlags$") and {} or {"BuildingUndo"})
                    elseif mode=="invalid-bitmask" then
                        fields[path] = {17}
                    elseif mode=="oversized-bitmask" then
                        fields[path] = {}
                        for index=1,65 do fields[path][index]="BuildingUndo" end
                    end
                end
                if mode=="consumed-different" and component=="keen::ecs::ServerConsumedPlayerInput" and path:match("%.version$") then
                    fields[path] = f.version-1
                end
            end
            return fields
        end,
    }
    environment.runtime = setmetatable({types={resolve=function(name)
        f.type_calls = f.type_calls + 1
        if mode=="type-nil" and name=="keen::ecs::AdminSpawnCommand" then return nil end
        return {qualified_name=name, size=8, fields={{name="version", data_offset=0,
            type={qualified_name="keen::VersionedDataVersion"}}}}
    end}, ecs=setmetatable(ecs,{__index=forbidden}), world=setmetatable({session_id=function() return f.session end},{__index=forbidden})},
        {__index=forbidden})
    f.mod = assert(load(typed_inputs_probe_source,"typed-input-probe","t",environment))()
    f.mod.on_load()
    function f:step(count, delta)
        for _=1,count do self.mod.on_update(delta or 0.25) end
        self.mod.on_unload()
    end
    return f
end

local waiting = fixture("normal")
waiting:step(12)
assert(waiting.report.state=="waiting-world" and waiting.registry_calls==0 and waiting.reads==0,
    "world absence must wait without entity access")
waiting.session=17
waiting:step(20)
assert(waiting.report.state=="observed", "stable input and consumed snapshots should be observed")
local session = waiting.report.sessions[1]
assert(session.successful_rounds==8 and #session.samples==8 and #session.samples[1].players==2)
assert(waiting.registry_calls==5 and session.registry_fetches==5,
    "readable registrations must be fetched only once per session")
assert(session.registrations["keen::ecs::PlayerInput"].reported_at==session.started_at
    and session.registrations["keen::ecs::PlayerInput"].reported_at<waiting.report.accumulated_callback_delta
    and session.registrations["keen::ecs::PlayerInput"].capability_scope=="observation-time-only",
    "cached capabilities must retain their original observation time")
assert(session.registrations["keen::ecs::PlayerInput"].write_fields_available==false,
    "successful reads must retain independently denied writability")
assert(waiting.report.findings.consume_contract=="unverified-read-only-observation",
    "equal versions do not prove command consumption")
local named = fixture("named-bitmasks")
named.session=17
named:step(20)
local named_player = named.report.sessions[1].samples[1].players[1]
assert(named.report.state=="observed" and named.report.schema==2)
assert(named_player.cursor_input.values["fromClient.digitalInput"].kind=="named-bitmask"
    and named_player.cursor_input.values["fromClient.digitalInput"].names[1]=="BuildingUndo")
assert(named_player.consumer.values["inputConsumer.consumedInputs"].names[1]=="BuildingUndo"
    and next(named_player.cursor_input.values["fromClient.cursorInput.primaryFlags"].names)==nil,
    "named and empty bitmask userdata must be copied without numeric coercion")
for _, mode in ipairs({"invalid-bitmask","oversized-bitmask"}) do
    local invalid=fixture(mode)
    invalid.session=17
    invalid:step(20)
    assert(invalid.report.sessions[1].successful_rounds==0 and invalid.report.state~="observed",
        "invalid input masks must never count as a completed input observation")
end
local reads_before = waiting.reads
waiting:step(8)
assert(waiting.reads==reads_before, "a completed session must stop sampling")
waiting.session=0
waiting:step(4)
waiting.session=19
waiting:step(20)
assert(#waiting.report.sessions==2 and waiting.report.sessions[2].session_id==19,
    "a new world must get a new segment and fresh entity queries")
assert(waiting.report.sessions[2].successful_rounds==8)
assert(waiting.registry_calls==10 and waiting.report.sessions[2].registry_fetches==5,
    "a new session must invalidate every cached registration")

local retry = fixture("registry-wait")
retry.session=17
retry:step(2)
assert(retry.report.state=="waiting-registry" and retry.queries==0,
    "nil registration must mean waiting, not absence")
retry:step(24)
assert(retry.report.state=="observed", "registry availability must be retried")
assert(retry.registry_calls==15, "only unresolved descriptors should be retried until the catalog is ready")

local unreadable = fixture("readability-wait")
unreadable.session=17
unreadable:step(26)
assert(unreadable.report.state=="observed" and unreadable.unreadable_calls==3 and unreadable.registry_calls==7,
    "only non-readable registrations should be retried while readable siblings stay cached")

local pending = fixture("query-pending")
pending.session=17
for callback=1,3 do
    pending:step(1,0.016)
    assert(pending.queries==callback and pending.registry_calls==5,
        "a pending query must advance on every throttled callback even with a small global tick delta")
end
pending:step(1,0.016)
assert(pending.report.sessions[1].rounds==1 and pending.reads>0 and pending.registry_calls==5,
    "the fourth callback must collect the pending query without another catalog fetch")
assert(pending.report.observation_clock=="accumulated_callback_delta"
    and math.abs(pending.report.accumulated_callback_delta-0.064)<0.000001,
    "the report must label summed callback delta rather than imply wall-clock elapsed time")

for _, mode in ipairs({"query-unavailable","read-nil","missing-field","empty-query","type-nil","consumed-different","mid-session"}) do
    local f = fixture(mode)
    f.session=17
    f:step(44)
    assert(f.forbidden_calls==0 and f.report.writes_attempted==0 and f.report.native_callbacks_invoked==0)
    if mode=="query-unavailable" then
        assert(f.report.sessions[1].query.state=="unavailable")
        assert(f.registry_calls==5 and f.queries>1,
            "a pending entity query must progress without another registry fetch")
    elseif mode=="read-nil" or mode=="missing-field" or mode=="empty-query" then
        assert(f.report.sessions[1].successful_rounds==0, "missing values must never count as read success")
        assert(#f.report.sessions[1].samples<=16, "unavailable observations must stay bounded")
    elseif mode=="type-nil" then
        local ty=f.report.sessions[1].types["keen::ecs::AdminSpawnCommand"]
        assert(ty.existence=="unresolved" and ty.reason:find("nil",1,true))
    elseif mode=="consumed-different" then
        assert(f.report.sessions[1].comparisons[1].equal==false)
        assert(f.report.findings.consume_contract=="unverified-read-only-observation")
    elseif mode=="mid-session" then
        assert(f.report.discarded_session_reads==1 and #f.report.sessions[1].samples==0)
        assert(f.report.sessions[2].session_id==18, "mixed-session observations must be discarded")
    end
end
assert(waiting.forbidden_calls==0 and retry.forbidden_calls==0)
