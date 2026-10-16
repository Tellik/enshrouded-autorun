-- Temporary developer probe. Every engine/resource operation below is a read.
-- One query is resumed until complete. No runtime.require during startup.
local PREFIX = "[SF_LIVE_PROBE_V1] "
local CHUNKS_PER_UPDATE = 4
local ASSETS_PER_UPDATE = 4
local MAX_SCAN_UPDATES = 20000 -- 25 ms scheduling, about 8m20s plus call time.
local state = {
    stage = "registry", tick = 0, active_updates = 0, jobs = {}, job_index = 1,
    asset_index = 1, assets_ok = 0, assets_failed = 0, samples = 0,
    counts = {observed = 0, unobserved = 0, presence_unreadable = 0,
        typed_partial = 0, query_failed = 0},
    pending_calls = 0, unexpected = 0, finished = false,
}

local function log(marker, detail)
    shroudforge.log.info(PREFIX .. marker .. " " .. (detail or ""))
end

local function safe(fn, ...)
    local ok, result, reason = pcall(fn, ...)
    if not ok then return nil, tostring(result) end
    return result, reason
end

local function waiting(detail)
    if state.tick % 200 == 1 then
        log("WAIT", "stage=" .. state.stage .. " reason=" .. tostring(detail))
    end
end

local function scalar(value)
    if type(value) == "table" or type(value) == "userdata" then
        return tonumber(value.value)
    end
    return tonumber(value)
end

local function finite(value)
    return type(value) == "number" and value == value and math.abs(value) < math.huge
end

local function summarize(reason)
    if state.finished then return end
    state.finished = true
    log("SUMMARY", string.format(
        "reason=%s registry=%s storage=%d checked=%d observed=%d unobserved=%d presence_but_unreadable=%d typed_partial=%d query_failed=%d untested=%d assets_ok=%d assets_failed=%d player_roots=%s pending_chunks=%d",
        reason, tostring(state.registry_count), #state.jobs, state.job_index - 1,
        state.counts.observed, state.counts.unobserved, state.counts.presence_unreadable,
        state.counts.typed_partial, state.counts.query_failed,
        math.max(0, #state.jobs - state.job_index + 1), state.assets_ok,
        state.assets_failed, tostring(state.player_roots == true), state.pending_calls))
end

local scalar_kinds = {Bool=true, UInt8=true, UInt16=true, UInt32=true, UInt64=true,
    SInt8=true, SInt16=true, SInt32=true, SInt64=true, Float32=true, Float64=true,
    Enum=true, Bitmask8=true, Bitmask16=true, Bitmask32=true, Bitmask64=true, Guid=true}

local function inline_selections(ty)
    local selected = {}
    local function visit(current, path, tokens, depth)
        if #selected >= 3 or depth > 5 then return end
        local kind = current.primitive_type
        if scalar_kinds[kind] and path ~= "" then
            selected[#selected + 1] = {path=path, tokens=tokens}
        elseif kind == "Typedef" and current.inner_type then
            visit(current.inner_type, path, tokens, depth + 1)
        elseif kind == "StaticArray" and current.inner_type and current.field_count > 0 then
            local next_tokens = {}
            for _, token in ipairs(tokens) do next_tokens[#next_tokens + 1] = token end
            next_tokens[#next_tokens + 1] = 1
            visit(current.inner_type, path .. "[1]", next_tokens, depth + 1)
        elseif kind == "Struct" then
            for _, field in ipairs(current.fields) do
                if #selected >= 3 then break end
                local next_tokens = {}
                for _, token in ipairs(tokens) do next_tokens[#next_tokens + 1] = token end
                next_tokens[#next_tokens + 1] = field.name
                local next_path = path == "" and field.name or (path .. "." .. field.name)
                visit(field.type, next_path, next_tokens, depth + 1)
            end
        end
    end
    visit(ty, "", {}, 0)
    return selected
end

local function touch(value, selections)
    for _, selection in ipairs(selections) do
        local current = value
        for _, token in ipairs(selection.tokens) do current = current[token] end
        if current == nil then error("nil reflected value at " .. selection.path) end
    end
    return true
end

local function query(names)
    local reason
    for _ = 1, CHUNKS_PER_UPDATE do
        local entities
        entities, reason = safe(runtime.ecs.query, table.unpack(names))
        if entities ~= nil then return entities end
        state.pending_calls = state.pending_calls + 1
        if not tostring(reason):find("still scanning", 1, true) then break end
    end
    return nil, reason
end

local function read_item(resource, index)
    local ok, result = pcall(function()
        if resource.source ~= "loaded-runtime" or resource.loaded ~= true then
            error("resource is not a loaded-runtime snapshot")
        end
        -- Access all lazy wrappers inside the protected block.
        local data = resource.data
        local id = scalar(data.itemId)
        local equipment = data.equipment
        local glider = equipment.gliderConfig
        local acceleration = glider.accelerationForward
        local turn = glider.yawAngleSpeed
        local min_x = equipment.placementAABBmin.x
        local max_x = equipment.placementAABBmax.x
        if not id or id < 0 or id > 4294967295 or id % 1 ~= 0 then error("invalid ItemInfo.itemId") end
        if not finite(acceleration) or not finite(turn) or not finite(min_x) or not finite(max_x) then
            error("nested reflected scalar is not finite")
        end
        return string.format("index=%d guid=%s item=%s source=%s acceleration=%s yaw=%s bounds_x=%s,%s",
            index, tostring(resource.guid), tostring(id), resource.source,
            tostring(acceleration), tostring(turn), tostring(min_x), tostring(max_x))
    end)
    if ok then
        state.assets_ok = state.assets_ok + 1
        if state.assets_ok <= 3 then log("ASSET_DATA_OK", result) end
    else
        state.assets_failed = state.assets_failed + 1
        if state.assets_failed <= 5 then
            log("ASSET_DATA_FAIL", "index=" .. index .. " reason=" .. tostring(result))
        end
    end
    return ok
end

local function registry_step()
    local registrations, reason = safe(runtime.ecs.get_registry)
    if not registrations then waiting(reason) return end
    local seen = {}
    for _, registration in ipairs(registrations.entries) do
        local ty = registration.runtime_type
        if ty and not seen[ty.qualified_name] then
            seen[ty.qualified_name] = true
            state.jobs[#state.jobs + 1] = {name=ty.qualified_name, ty=ty,
                registration=registration.qualified_name, size=ty.size}
        end
    end
    table.sort(state.jobs, function(a, b) return a.name < b.name end)
    state.registry_count = registrations.count
    log("REGISTRY_OK", string.format("registrations=%d storage=%d source=%s",
        registrations.count, #state.jobs, tostring(registrations.source)))
    local attributes, attribute_reason = safe(runtime.ecs.get_attributes)
    if attributes then
        log("ATTRIBUTE_CATALOG_OK", "count=" .. attributes.count ..
            " layouts=" .. #(attributes.component_layouts or {}))
    else log("ATTRIBUTE_CATALOG_FAIL", tostring(attribute_reason)) end
    if runtime.functions and runtime.functions.list_native then
        local functions, function_reason = safe(runtime.functions.list_native, 0, 1)
        if functions then
            local coverage = functions.dependency_resolution or {}
            log("FUNCTION_CATALOG_OK", string.format(
                "inventory=%s systems=%s references=%s types=%s reflection_resolved=%s registration_resolved=%s access_modes_resolved=%s native_signatures_resolved=%s",
                tostring(functions.inventory_count), tostring(coverage.engine_descriptors),
                tostring(coverage.dependency_references), tostring(coverage.distinct_dependency_types),
                tostring(coverage.reflection_resolved), tostring(coverage.registration_resolved),
                tostring(coverage.access_modes_resolved), tostring(coverage.native_signatures_resolved)))
        else log("FUNCTION_CATALOG_FAIL", tostring(function_reason)) end
    end
    state.stage = "asset_list"
end

local function asset_list_step()
    local resources, reason = safe(game.assets.get_loaded_resources_by_type, "keen::ItemInfo")
    if not resources or #resources == 0 then
        state.asset_list_attempts = (state.asset_list_attempts or 0) + 1
        waiting(reason or "loaded ItemInfo list is empty")
        if state.asset_list_attempts >= 400 then
            state.assets_failed = state.assets_failed + 1
            log("ASSET_LIST_FAIL", tostring(reason or "empty after 400 updates"))
            state.stage = "players"
        end
        return
    end
    state.resources = resources
    log("ASSET_LIST_OK", "type=keen::ItemInfo count=" .. #resources)
    read_item(resources[1], 1)
    state.asset_index = 2
    state.stage = "players"
end

local function player_step()
    local players, reason = query({"keen::ecs::PlayerInput", "keen::ecs::Stamina", "keen::ecs::Health"})
    if not players then
        if not tostring(reason):find("still scanning", 1, true) then
            state.player_query_errors = (state.player_query_errors or 0) + 1
            if state.player_query_errors >= 40 then
                log("PLAYER_QUERY_FAIL", tostring(reason))
                state.stage = "asset_data"
                return
            end
        end
        waiting(reason)
        return
    end
    if #players == 0 then
        state.empty_player_queries = (state.empty_player_queries or 0) + 1
        if state.empty_player_queries < 4 then return end
        log("PLAYER_UNOBSERVED", "completed_queries=4 selection=PlayerInput+Stamina+Health")
        state.stage = "asset_data"
        return
    end
    local successes = 0
    for index = 1, math.min(#players, 8) do
        local entity = players[index]
        local stamina, stamina_reason = safe(runtime.ecs.read_attributes, entity, "Stamina")
        local health, health_reason = safe(runtime.ecs.read_attributes, entity, "Health")
        if stamina and health then
            successes = successes + 1
            log("PLAYER_ROOT_OK", string.format(
                "handle=%s health=%s/%s health_base=%s health_add=%s stamina=%s/%s stamina_base=%s stamina_add=%s stamina_component=%s health_component=%s",
                tostring(entity), tostring(health.values.Health), tostring(health.values.Health_Max),
                tostring(health.values.Health_Max_Base), tostring(health.values.Health_Max_Adder),
                tostring(stamina.values.Stamina), tostring(stamina.values.Stamina_Max),
                tostring(stamina.values.Stamina_Max_Base), tostring(stamina.values.Stamina_Max_Adder),
                tostring(stamina.component.qualified_name), tostring(health.component.qualified_name)))
        else
            log("PLAYER_ROOT_RETRY", "handle=" .. tostring(entity) .. " stamina=" ..
                tostring(stamina_reason) .. " health=" .. tostring(health_reason))
        end
        local fields, fields_reason = safe(runtime.ecs.read_fields, entity,
            "keen::ecs::NetworkPlayerAttributes", {"health", "healthMax", "stamina", "staminaMax", "movementSprint"})
        if fields then
            log("PLAYER_FIELDS_OK", string.format("handle=%s health=%s/%s stamina=%s/%s sprint=%s",
                tostring(entity), tostring(fields.health), tostring(fields.healthMax),
                tostring(fields.stamina), tostring(fields.staminaMax), tostring(fields.movementSprint)))
        else log("PLAYER_FIELDS_UNAVAILABLE", tostring(fields_reason)) end
    end
    if successes == 0 then waiting("player root reads pending or unavailable") return end
    state.player_roots = true
    log(state.assets_ok > 0 and "CORE_READ_OK" or "CORE_READ_PARTIAL",
        "registry=true asset_sample=" .. tostring(state.assets_ok > 0) ..
        " player_roots=true writes=false native_engine_calls=false")
    state.stage = "asset_data"
end

local function asset_data_step()
    if state.resources then
        for _ = 1, ASSETS_PER_UPDATE do
            local resource = state.resources[state.asset_index]
            if not resource then break end
            read_item(resource, state.asset_index)
            state.asset_index = state.asset_index + 1
        end
        if state.asset_index <= #state.resources then return end
        log("ASSET_SCAN_DONE", string.format("listed=%d decoded=%d failed=%d",
            #state.resources, state.assets_ok, state.assets_failed))
        state.resources = nil
    end
    state.stage = "types"
end

local function finish_type(status, detail)
    state.counts[status] = state.counts[status] + 1
    local job = state.jobs[state.job_index]
    log("TYPE_RESULT", string.format("%d/%d type=%s status=%s %s", state.job_index,
        #state.jobs, job.name, status == "presence_unreadable" and "presence-but-unreadable" or status, detail or ""))
    state.job_index = state.job_index + 1
    state.current_entities = nil
    state.query_errors = 0
end

local function type_step()
    local job = state.jobs[state.job_index]
    if not job then summarize("complete") return end
    if not state.current_entities then
        local entities, reason = query({job.name})
        if not entities then
            if not tostring(reason):find("still scanning", 1, true) then
                state.query_errors = (state.query_errors or 0) + 1
                if state.query_errors >= 16 then finish_type("query_failed", tostring(reason)) return end
            end
            waiting(job.name .. ": " .. tostring(reason))
            return
        end
        if #entities == 0 then finish_type("unobserved", "query_complete=true") return end
        state.current_entities = entities
        state.entity_attempt = 1
    end
    local entities = state.current_entities
    local entity = entities[state.entity_attempt]
    local raw, raw_reason = safe(runtime.ecs.read_bytes, entity, job.name)
    if not raw then
        state.entity_attempt = state.entity_attempt + 1
        if state.entity_attempt <= math.min(#entities, 3) then return end
        finish_type("presence_unreadable", "matches=" .. #entities .. " reason=" .. tostring(raw_reason))
        return
    end
    if #raw ~= job.size then
        finish_type("presence_unreadable", "unexpected_byte_count=" .. #raw .. " expected=" .. job.size)
        return
    end
    local selections = inline_selections(job.ty)
    local paths = {}
    for _, selection in ipairs(selections) do paths[#paths + 1] = selection.path end
    local value, value_reason = safe(runtime.ecs.read, entity, job.name)
    local owned, owned_reason = safe(runtime.values.decode, job.name, raw)
    local touched, touch_reason = false, "read returned no value"
    if value then touched, touch_reason = pcall(touch, value, selections) end
    local owned_touched = false
    if owned then owned_touched = pcall(touch, owned, selections) end
    local fields, fields_reason
    if #paths > 0 then fields, fields_reason = safe(runtime.ecs.read_fields, entity, job.name, paths) end
    local complete = value ~= nil and touched and owned ~= nil and owned_touched and (#paths == 0 or fields ~= nil)
    finish_type(complete and "observed" or "typed_partial", string.format(
        "matches=%d bytes=%d read=%s owned_decode=%s fields=%d fields_read=%s reason=%s",
        #entities, #raw, tostring(value ~= nil and touched), tostring(owned ~= nil and owned_touched),
        #paths, tostring(#paths == 0 or fields ~= nil), tostring(value_reason or owned_reason or
            (not touched and touch_reason) or fields_reason or "none")))
end

local function update()
    if state.finished then return end
    state.tick = state.tick + 1
    if type(runtime) ~= "table" or not runtime.ecs then waiting("runtime namespace unavailable") return end
    if state.stage == "registry" then registry_step() return end
    if not runtime.has("runtime.ecs.read") then waiting("live world is not ready") return end
    state.active_updates = state.active_updates + 1
    if state.active_updates > MAX_SCAN_UPDATES then summarize("budget-exhausted") return end
    if state.stage == "asset_list" then asset_list_step()
    elseif state.stage == "players" then player_step()
    elseif state.stage == "asset_data" then asset_data_step()
    elseif state.stage == "types" then type_step() end
end

return {
    update_interval_ms = 25,
    on_load = function()
        log("START", "read_only=true query_chunks_per_update=4 scan_budget_updates=" .. MAX_SCAN_UPDATES)
    end,
    on_update = function()
        local ok, reason = pcall(update)
        if not ok then
            log("UNEXPECTED_ERROR", "stage=" .. state.stage .. " reason=" .. tostring(reason))
            summarize("unexpected-error")
        end
    end,
    on_unload = function()
        if not state.finished then summarize("unloaded") end
        log("UNLOAD", "writes=false")
    end,
}
