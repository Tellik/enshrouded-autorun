-- Read-only diagnostic job using the active World Editor's public mod APIs.
local report = {schema = 1, state = "waiting", samples = {}, failures = {}, elapsed = 0,
    cursor_reads = 0, valid_cursor_reads = 0, client_authority_matches = 0,
    stable_prop_matches = 0, stable_voxel_matches = 0}
local elapsed, since_export, finished = 0, 0, false
local player_handles, prop_handles

local function publish()
    report.elapsed = elapsed
    diagnostic.publish(report)
end

local function positions_equal(left, right)
    for _, axis in ipairs({"x", "y", "z"}) do
        if left[axis] ~= right[axis] then return false end
    end
    return true
end

local function sample()
    local session = runtime.world.session_id()
    report.session = session
    report.context = runtime.world.context_kind()
    if session == 0 then return end
    if report.observed_session and report.observed_session ~= session then
        report.state = "session-changed"
        finished = true
        return
    end
    report.observed_session = session
    local cursor, cursor_reason = runtime.world.cursor.get()
    report.cursor_reason = cursor_reason
    if not cursor then return end
    report.state = "sampling"
    report.cursor_reads = report.cursor_reads + 1
    local current = {cursor = cursor}
    local position = cursor.value.primaryTransform.position
    local valid_cursor = (cursor.value.primaryFlags & 1) == 0
        and (position.x ~= 0 or position.y ~= 0 or position.z ~= 0)
    if valid_cursor then report.valid_cursor_reads = report.valid_cursor_reads + 1 end
    if not player_handles then
        player_handles, report.player_query_reason = runtime.ecs.query("keen::ecs::PlayerInput")
    end
    if player_handles and #player_handles == 1 then
        local base = "fromClient.cursorInput.primaryTransform.position."
        local paths = {base .. "x", base .. "y", base .. "z"}
        local values, reason = runtime.ecs.read_fields(player_handles[1], "keen::ecs::PlayerInput", paths)
        current.authority_reason = reason
        if values then
            current.authority_position = {x = values[paths[1]], y = values[paths[2]], z = values[paths[3]]}
            -- Client input can advance before the authority consumes it. A
            -- difference is recorded, never called proof of a bad field layout.
            current.client_authority_equal = positions_equal(
                cursor.value.primaryTransform.position, current.authority_position)
            if valid_cursor and current.client_authority_equal then
                report.client_authority_matches = report.client_authority_matches + 1
            end
        end
    end
    if valid_cursor and runtime.has("runtime.world.voxel.read") then
        local x, y, z = math.floor(position.x / 2147483648),
            math.floor(position.y / 2147483648), math.floor(position.z / 2147483648)
        -- Include terrain below the hit point instead of comparing only air
        -- in the building preview cell above the surface.
        x, y, z = x - 4, y - 8, z - 4
        local first, reason = runtime.world.voxel.read(x, y, z, 1, 1, 1)
        current.voxel_reason = reason
        if first then
            local block, block_reason = runtime.world.voxel.read(x, y, z, 8, 8, 8)
            local second, second_reason = runtime.world.voxel.read(x, y, z, 1, 1, 1)
            current.voxel_block_reason, current.voxel_second_reason = block_reason, second_reason
            if block and second and first[1] == second[1] then
                current.voxel = {origin = {x, y, z}, cell = first[1], block_cell = block[1],
                    block_count = #block, equal = first[1] == block[1]}
                local nonzero, maximum = 0, 0
                for _, cell in ipairs(block) do
                    if cell ~= 0 then nonzero = nonzero + 1 end
                    maximum = math.max(maximum, cell)
                end
                current.voxel.nonzero_cells, current.voxel.maximum_value = nonzero, maximum
                if current.voxel.equal and #block == 512 then
                    report.stable_voxel_matches = report.stable_voxel_matches + 1
                else
                    report.failures[#report.failures + 1] = "stable voxel differs between single-cell and 8-cube reads"
                end
            end
        end
    end
    if not prop_handles then
        prop_handles, report.prop_query_reason = runtime.ecs.query(
            "keen::ecs::CurrentTransform", "keen::ecs::UsedItem")
    end
    if prop_handles and #prop_handles > 0 then
        local handle = prop_handles[1]
        local first, reason = runtime.world.entity.get_transform(handle)
        current.prop_reason = reason
        if first then
            local paths = {"transform.position.x", "transform.position.y", "transform.position.z",
                "transform.scale.x", "transform.scale.y", "transform.scale.z"}
            local fields, fields_reason = runtime.ecs.read_fields(handle, "keen::ecs::CurrentTransform", paths)
            local item, item_reason = runtime.ecs.read_fields(handle, "keen::ecs::UsedItem", {"itemId.value"})
            local second = runtime.world.entity.get_transform(handle)
            current.prop_fields_reason, current.item_reason = fields_reason, item_reason
            if fields and item and second then
                local expected = {x = fields[paths[1]] / 4294967296,
                    y = fields[paths[2]] / 4294967296, z = fields[paths[3]] / 4294967296}
                local stable = positions_equal(first.transform.position, second.transform.position)
                current.prop = {handle = handle, itemId = first.itemId,
                    templateUuidHighHex = first.templateUuidHighHex,
                    templateUuidLowHex = first.templateUuidLowHex,
                    position = expected, stable = stable}
                if stable then
                    current.prop.equal = positions_equal(first.transform.position, expected)
                        and first.itemId == item["itemId.value"]
                    for index = 1, 3 do
                        current.prop.equal = current.prop.equal
                            and first.transform.scale[index] == fields[paths[index + 3]]
                    end
                    if not current.prop.equal then
                        report.failures[#report.failures + 1] = "stable prop differs between world and ECS readers"
                    else
                        report.stable_prop_matches = report.stable_prop_matches + 1
                    end
                end
            end
        else
            prop_handles = nil
        end
    end
    if runtime.world.session_id() ~= session then return end
    report.samples[#report.samples + 1] = current
    if #report.samples > 20 then table.remove(report.samples, 1) end
    if report.valid_cursor_reads >= 20 and report.stable_prop_matches >= 3
        and report.stable_voxel_matches >= 3 then
        report.state = #report.failures == 0 and "complete" or "failed"
        finished = true
    end
end

return {
    update_interval_ms = 100,
    on_load = function()
        shroudforge.log.info("World Editor native API read-only A/B diagnostic loaded")
        publish()
    end,
    on_update = function(delta)
        if finished then return end
        elapsed, since_export = elapsed + delta, since_export + delta
        local ok, reason = pcall(sample)
        if not ok then
            report.failures[#report.failures + 1] = tostring(reason)
            report.state, finished = "error", true
        elseif elapsed >= 180 then
            report.state, finished = "timeout", true
        end
        if finished or since_export >= 1 then
            since_export = 0
            publish()
            if finished then
                shroudforge.log.info("World Editor native API diagnostic: " .. report.state)
                diagnostic.finish()
            end
        end
    end,
    on_unload = function() publish() end,
}
