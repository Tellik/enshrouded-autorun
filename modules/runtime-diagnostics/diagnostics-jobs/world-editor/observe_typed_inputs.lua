-- Read-only diagnostic job. It never publishes input, invokes callbacks or
-- changes the world. Every session retains independent, bounded observations.
local components = {
    "keen::ecs::PlayerInput", "keen::ecs::ServerConsumedPlayerInput",
    "keen::ecs::CurrentTransform", "keen::ecs::UsedItem", "keen::ecs::Terraforming",
}
local types = {
    "keen::ecs::PlayerInput", "keen::ecs::ServerConsumedPlayerInput",
    "keen::ecs::CurrentTransform", "keen::ecs::UsedItem", "keen::ecs::Terraforming",
    "keen::ecs::ClientPlayerInputData", "keen::ecs::AdminClientPlayerInputData",
    "keen::ecs::AdminSpawnCommand", "keen::ecs::ClientCursorInput",
    "keen::ecs::CursorSelectObjectAction", "keen::ecs::CursorChangeGrowthLimitAction",
    "keen::ecs::CreateBuildingItemAction", "keen::ecs::BuildingStockCycleAction",
    "keen::ecs::BaseServerConsumedPlayerInput", "keen::ecs::ServerInputConsumer",
    "keen::VersionedData", "keen::VersionedDataVersion", "keen::WorldTransform",
}
local version_pairs = {
    {key="admin_spawn", input="fromAdminClient.adminSpawnCommand.versionData.version", consumed="consumedAdminSpawnAction.version"},
    {key="cursor_select", input="fromClient.cursorInput.selectObjectAction.versionData.version", consumed="consumedCursorSelectObjectAction.version"},
    {key="cursor_growth", input="fromClient.cursorInput.changeGrowthLimitAction.versionData.version", consumed="consumedCursorChangeGrowthLimitAction.version"},
    {key="create_building_item", input="fromClient.createBuildingItemAction.versionData.version", consumed="consumedCreateBuildingItemAction.version"},
    {key="building_stock", input="fromClient.buildingStockCycleAction.versionData.version", consumed="consumedBuildingStockCycleAction.version"},
}
local input_paths, consumed_paths = {}, {}
for _, pair in ipairs(version_pairs) do
    input_paths[#input_paths+1], consumed_paths[#consumed_paths+1] = pair.input, pair.consumed
end
local cursor_paths = {
    "fromClient.cursorInput.primaryTransform.position.x", "fromClient.cursorInput.primaryTransform.position.y",
    "fromClient.cursorInput.primaryTransform.position.z", "fromClient.cursorInput.primaryFlags",
    "fromClient.cursorInput.secondaryFlags", "fromClient.cursorInput.hoveredVoxelMaterialId",
    "fromClient.digitalInput", "fromAdminClient.adminSpawnCommand.count", "fromAdminClient.adminSpawnCommand.level",
}
local consumer_paths = {"inputConsumer.consumedInputs", "inputConsumer.logicalStates", "inputConsumer.totalPreviousInputs"}
local report = {schema=1, kind="typed-input-api-read-only", state="waiting-world", accumulated_callback_delta=0,
    observation_clock="accumulated_callback_delta",
    sessions={}, session_events={}, discarded_session_reads=0, errors={},
    bounds={samples_per_session=16, player_samples_per_round=2, retained_sessions=4, successful_rounds=8},
    findings={existence="unresolved", registration="unresolved", reading="unobserved",
        writability="capability-snapshot-at-reported-time", consume_contract="unverified-read-only-observation"},
    writes_attempted=0, native_callbacks_invoked=0, atomic_snapshot=false}
local elapsed, export_updates, current, observed_session = 0, 0, nil, nil

local function bounded_push(array, value, limit)
    array[#array+1] = value
    if #array > limit then table.remove(array, 1) end
end

local function publish()
    report.accumulated_callback_delta = elapsed
    diagnostic.publish(report)
end

local function invoke(callback, ...)
    local ok, value, reason = pcall(callback, ...)
    if not ok then return nil, tostring(value) end
    if value == nil then return nil, reason or "API returned nil without a reason, availability is unresolved" end
    return value, reason
end

local function flag(value)
    if type(value) == "boolean" then return value end
    return "unknown"
end

local function describe_type(name)
    local ty, reason = invoke(runtime.types.resolve, name)
    if not ty then return {existence="unresolved", reason=reason} end
    local row = {existence="resolved", qualified_name=ty.qualified_name, size=ty.size,
        fields={}, fields_state="unavailable"}
    if ty.qualified_name ~= name then
        return {existence="unresolved", reason="resolved type name does not match the full requested name"}
    end
    if ty.inner_type then row.base_or_element_type = ty.inner_type.qualified_name end
    if ty.fields then
        row.fields_state = "resolved"
        for index, field in ipairs(ty.fields) do
            if index > 96 then row.fields_truncated = true; break end
            row.fields[#row.fields+1] = {name=field.name, offset=field.data_offset,
                type=field.type and field.type.qualified_name or "unresolved"}
        end
    end
    return row
end

local function describe_registration(name)
    local entry, reason = invoke(runtime.ecs.get_component, name)
    if not entry then return {registration="unresolved", reported_at=elapsed, reason=reason} end
    return {registration="resolved", qualified_name=entry.qualified_name, index=entry.index,
        reported_at=elapsed, capability_scope="observation-time-only",
        runtime_size=entry.runtime_size, storage=entry.storage,
        read_available=flag(entry.read_available), write_available=flag(entry.write_available),
        read_fields_available=flag(entry.read_fields_available), write_fields_available=flag(entry.write_fields_available),
        write_fields_reason=entry.write_fields_reason, value_reason=entry.value_reason, reason=entry.reason}
end

local function switch_session(session)
    if observed_session == session then return end
    observed_session = session
    if current then
        current.ended_at = elapsed
        if current.state ~= "observed" then current.state = "session-ended" end
    end
    bounded_push(report.session_events, {session_id=session, at=elapsed}, 12)
    current = nil
    report.current_session_id = session
    report.findings = {existence="unresolved", registration="unresolved", reading="unobserved",
        writability="capability-snapshot-at-reported-time", consume_contract="unverified-read-only-observation"}
    if session == 0 then report.state="waiting-world"; return end
    current = {session_id=session, started_at=elapsed, state="waiting-registry", types={}, registrations={},
        samples={}, rounds=0, successful_rounds=0, comparisons={}, type_attempts=0, registry_attempts=0,
        registry_fetches=0}
    bounded_push(report.sessions, current, 4)
    report.state = "waiting-registry"
end

local function read_fields(handle, component, paths)
    if not current or runtime.world.session_id() ~= current.session_id then
        return {state="session-changed", reason="world changed before field read"}
    end
    local values, reason = invoke(runtime.ecs.read_fields, handle, component, paths)
    if not values then return {state="unavailable", reason=reason} end
    local result = {state="read", values={}}
    for _, path in ipairs(paths) do
        local value = values[path]
        if value == nil then return {state="unavailable", reason="reader omitted selected field " .. path} end
        if type(value) ~= "number" and type(value) ~= "boolean" and type(value) ~= "string" then
            return {state="unavailable", reason="reader returned a non-scalar for " .. path}
        end
        if type(value) == "number" and (value ~= value or value == math.huge or value == -math.huge) then
            return {state="unavailable", reason="reader returned a nonfinite number for " .. path}
        end
        result.values[path] = value
    end
    return result
end

local function equal_fields(first, second, paths)
    if first.state ~= "read" or second.state ~= "read" then return false end
    for _, path in ipairs(paths) do if first.values[path] ~= second.values[path] then return false end end
    return true
end

local function round(session)
    current.type_attempts = current.type_attempts + 1
    local resolved = true
    local described = {}
    for _, name in ipairs(types) do
        local previous = current.types[name]
        described[name] = previous and previous.existence == "resolved" and previous or describe_type(name)
        resolved = resolved and described[name].existence == "resolved"
    end
    current.registry_attempts = current.registry_attempts + 1
    local registry_ready, registrations = true, {}
    for _, name in ipairs(components) do
        -- get_component currently materializes the full registry. Repeating
        -- successful lookups here can outlive a pending query's scan lease.
        -- These are dated descriptions, never ongoing write permissions.
        local entry = current.registrations[name]
        if not entry or entry.registration ~= "resolved" or entry.read_fields_available ~= true then
            entry = describe_registration(name)
            current.registry_fetches = current.registry_fetches + 1
        end
        registrations[name] = entry
        registry_ready = registry_ready and entry.registration == "resolved" and entry.read_fields_available == true
    end
    if runtime.world.session_id() ~= session then return false end
    current.types, current.registrations = described, registrations
    report.findings.existence = resolved and "resolved" or "partially-unresolved"
    if not registry_ready then report.state="waiting-registry"; return true end
    report.findings.registration, report.state, current.state = "resolved", "sampling", "sampling"
    local handles, reason = invoke(runtime.ecs.query, components[1], components[2])
    local query = {state=handles and "read" or "unavailable", reason=reason,
        matching_count=handles and #handles or nil}
    if not handles then
        if runtime.world.session_id() ~= session then return false end
        current.query = query
        return true
    end
    current.rounds = current.rounds + 1
    local sample = {round=current.rounds, at=elapsed, players={}, atomic=false}
    for index=1,math.min(#handles,2) do
        local handle = handles[index]
        local input = read_fields(handle, components[1], input_paths)
        local consumed = read_fields(handle, components[2], consumed_paths)
        local player = {handle=handle, input_versions=input, consumed_versions=consumed,
            cursor_input=read_fields(handle, components[1], cursor_paths),
            consumer=read_fields(handle, components[2], consumer_paths),
            terraforming=read_fields(handle, components[5], {"snappingQueryId.id", "temporaryMiningQueryId.id"})}
        local input_again = read_fields(handle, components[1], input_paths)
        local consumed_again = read_fields(handle, components[2], consumed_paths)
        player.stable_versions = equal_fields(input,input_again,input_paths) and equal_fields(consumed,consumed_again,consumed_paths)
        player.input_versions_again, player.consumed_versions_again = input_again, consumed_again
        sample.players[#sample.players+1] = player
    end
    if runtime.world.session_id() ~= session then return false end
    local props, prop_reason = invoke(runtime.ecs.query, components[3], components[4])
    sample.props = {query_state=props and "read" or "unavailable", reason=prop_reason,
        matching_count=props and #props or nil}
    if props and #props > 0 then
        sample.props.handle = props[1]
        sample.props.transform = read_fields(props[1], components[3], {"transform.position.x", "transform.position.y",
            "transform.position.z", "transform.scale.x", "transform.scale.y", "transform.scale.z"})
        sample.props.item = read_fields(props[1], components[4], {"itemId.value"})
    end
    -- No handles, samples or observations from a changing session are retained.
    if runtime.world.session_id() ~= session then return false end
    current.query = query
    local successful = false
    for _, player in ipairs(sample.players) do
        if player.stable_versions then
            successful = true
            for _, pair in ipairs(version_pairs) do
                local observation = {handle=player.handle, pair=pair.key,
                    input=player.input_versions.values[pair.input], consumed=player.consumed_versions.values[pair.consumed]}
                observation.equal = observation.input == observation.consumed
                bounded_push(current.comparisons, observation, 80)
            end
        end
    end
    if successful then
        current.successful_rounds = current.successful_rounds + 1
        report.findings.reading = "stable-version-pairs-observed"
    end
    bounded_push(current.samples, sample, 16)
    if current.successful_rounds >= 8 then current.state, report.state = "observed", "observed" end
    return true
end

return {
    update_interval_ms=250,
    on_load=function()
        shroudforge.log.info("World Editor typed-input diagnostic prepared for read-only observations")
        publish()
    end,
    on_update=function(delta)
        -- The diagnostic host throttles this callback to 250 ms and supplies
        -- the actual monotonic time accumulated since its previous callback.
        elapsed, export_updates = elapsed + delta, export_updates + 1
        local ok, reason = pcall(function()
            local session = runtime.world.session_id()
            if type(session) ~= "number" or session < 0 then error("world session is unavailable") end
            if observed_session ~= session then switch_session(session) end
            if session ~= 0 and current.state ~= "observed" then
                if not round(session) then
                    report.discarded_session_reads = report.discarded_session_reads + 1
                    switch_session(runtime.world.session_id())
                end
            end
        end)
        if not ok then
            bounded_push(report.errors, tostring(reason), 8)
            report.state = "waiting-api"
        end
        if export_updates >= 4 then export_updates=0; publish() end
    end,
    on_unload=publish,
}
