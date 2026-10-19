-- Mutating diagnostic job for an explicitly authorized disposable world.
-- Aim the Building Hammer's Single Voxel cursor at terrain and hold it still.
-- This action clears exactly one occupied cell, verifies it, then restores it.
-- Keep the reports for manual recovery. Existing reports block every reload.
local output = "report.json"
local baseline_output = "baseline.json"
local report = {schema = 1, kind = "world-editor-voxel-write-ab", state = "waiting",
    elapsed = 0, writes = {}, observations = {}, recovery_required = false,
    checked_scope = "one aligned 8x8x8 chunk, outside the selected cell",
    baseline_export = baseline_output, mutation_attempts = 0}
local elapsed, phase_started, next_sample, next_publish = 0, 0, 0, 0
local finished, preserve_existing = false, false
local recovery_pending = false
local previous_cursor, stable_samples, stable_since, verified = nil, 0, 0, 0

local function persist(path, contents)
    diagnostic.write_file(path, contents)
    assert(diagnostic.read_file(path) == contents, "diagnostic file readback differs: " .. path)
end

local function publish()
    if preserve_existing then return end
    report.elapsed = elapsed
    diagnostic.publish(report)
end

local function stop(state, reason, recovery)
    report.state, report.reason = state, tostring(reason)
    report.recovery_required = recovery == true
    finished = true
    local ok, failure = pcall(publish)
    if not ok then shroudforge.log.error("Voxel write diagnostic report failed: " .. tostring(failure)) end
    shroudforge.log.info("Voxel write diagnostic stopped: " .. state .. ": " .. tostring(reason))
    if preserve_existing or not ok then
        diagnostic.finish(report)
    else
        diagnostic.finish()
    end
end

local function phase(state)
    report.state, phase_started, next_sample, verified = state, elapsed, elapsed + 0.5, 0
    publish()
end

local function session_current()
    return report.session and runtime.world.session_id() == report.session
        and runtime.world.context_kind() == "direct"
end

local function finite(value)
    return type(value) == "number" and value == value and math.abs(value) < math.huge
end

local function same(left, right)
    if type(left) ~= "table" or type(right) ~= "table" or #left ~= 512 or #right ~= 512 then return false end
    for index = 1, 512 do if left[index] ~= right[index] then return false end end
    return true
end

local function read_chunk()
    if not session_current() then return nil, "session changed" end
    local o = report.origin
    local cells, reason = runtime.world.voxel.read(o[1], o[2], o[3], 8, 8, 8)
    if not session_current() then return nil, "session changed during read" end
    if type(cells) ~= "table" or #cells ~= 512 then return nil, reason or "incomplete chunk read" end
    for _, cell in ipairs(cells) do
        if not finite(cell) or cell % 1 ~= 0 or cell < 0 or cell > 65535 then
            return nil, "invalid packed cell"
        end
    end
    return cells
end

local function observe(cells, wanted)
    local differences, outside = 0, 0
    for index = 1, 512 do
        if cells[index] ~= report.baseline[index] then
            differences = differences + 1
            if index ~= report.target_index then outside = outside + 1 end
        end
    end
    local correct = outside == 0 and cells[report.target_index] == wanted
        and differences == (wanted == report.original_value and 0 or 1)
    report.last_observed = cells
    report.observations[#report.observations + 1] = {phase = report.state, at = elapsed,
        differences = differences, outside_differences = outside,
        selected_value = cells[report.target_index], matches = correct}
    if #report.observations > 80 then table.remove(report.observations, 1) end
    return correct, differences
end

local function choose_target(cells)
    local preferred, fallback
    for index, cell in ipairs(cells) do
        local material, density = cell & 255, cell >> 8
        if material == 2 and density == 255 then preferred = preferred or index end
        if material > 0 and density > 0 then fallback = fallback or index end
    end
    local index = preferred or fallback
    if not index then return false end
    local offset = index - 1
    report.target_index, report.original_value = index, cells[index]
    report.target = {report.origin[1] + offset % 8,
        report.origin[2] + (offset // 8) % 8, report.origin[3] + offset // 64}
    return true
end

local function write_once(kind, value, expected)
    local cells, reason = read_chunk()
    if not cells then return stop("recovery-required", reason, recovery_pending) end
    if not observe(cells, expected) then
        return stop("recovery-required", "chunk changed before " .. kind, not same(cells, report.baseline))
    end
    -- Persist complete recovery data and this exact intent before entering the
    -- native writer. Each kind is attempted once, even if Lua or native throws.
    report.mutation_attempts = report.mutation_attempts + 1
    local attempt = {kind = kind, value = value, at = elapsed, state = "intent-persisted"}
    report.writes[#report.writes + 1] = attempt
    report.recovery_required = true
    publish()
    if not session_current() then return stop("session-changed", "session changed before write", recovery_pending) end
    local p = report.target
    recovery_pending = true
    local called, ok, failure = pcall(runtime.world.voxel.write, p[1], p[2], p[3], 1, 1, 1, {value})
    attempt.state, attempt.returned, attempt.reason = "returned", called and ok == true, tostring(failure or "")
    if not called then attempt.reason = tostring(ok) end
    if not session_current() then return stop("session-changed", "session changed during write", recovery_pending) end
    if not called or ok ~= true then
        -- No subsequent world mutation is permitted, including restoration.
        -- Read-only recovery observations may establish an unchanged baseline.
        report.failed_write = kind
        phase("failed-write-readback")
    else
        phase(kind .. "-readback")
    end
end

local function wait_for_cursor()
    local snapshot, reason = runtime.world.cursor.get()
    report.cursor_reason = reason
    local cursor = snapshot and snapshot.value
    local transform = cursor and cursor.primaryTransform
    local position = transform and transform.position
    if not position or not finite(cursor.primaryFlags) or (cursor.primaryFlags & 1) ~= 0
        or cursor.attachedToProp ~= false then
        previous_cursor, stable_samples = nil, 0
        return
    end
    local grid, grid_reason = runtime.world.voxel.get_grid_spec("voxel")
    report.grid_reason = grid_reason
    if not grid or not grid.origin or not grid.cellSize then return end
    local point, key = {}, {tostring(cursor.primaryFlags), tostring(cursor.material)}
    for index, axis in ipairs({"x", "y", "z"}) do
        local raw, size, origin = position[axis], grid.cellSize[index], grid.origin[index]
        if not finite(raw) or not finite(size) or size ~= 0.5 or not finite(origin) then return end
        local cell = (raw / 4294967296 - origin) / size
        if math.abs(cell - math.floor(cell + 0.5)) > 0.00001 then
            previous_cursor, stable_samples = nil, 0
            return
        end
        point[index], key[#key + 1] = math.floor(cell + 0.5), tostring(raw)
    end
    if point[1] == 0 and point[2] == 0 and point[3] == 0 then return end
    local identity = table.concat(key, ":")
    if previous_cursor == identity and report.cursor_sequence ~= snapshot.sequence then
        stable_samples = stable_samples + 1
    elseif previous_cursor ~= identity then
        previous_cursor, stable_samples, stable_since = identity, 1, elapsed
    end
    report.cursor_sequence = snapshot.sequence
    if stable_samples < 3 or elapsed - stable_since < 0.5 then return end
    report.cursor = snapshot
    report.origin = {math.floor(point[1] / 8) * 8,
        math.floor((point[2] - 8) / 8) * 8, math.floor(point[3] / 8) * 8}
    phase("baseline-first")
end

local function update()
    if elapsed >= 240 then return stop("timeout", "diagnostic deadline exceeded", recovery_pending) end
    if report.session and not session_current() then
        return stop("session-changed", "pinned world session is unavailable or changed", recovery_pending)
    end
    if report.state == "waiting" then
        if runtime.world.session_id() == 0 or runtime.world.context_kind() ~= "direct" then return end
        for _, feature in ipairs({"runtime.world.cursor.get", "runtime.world.voxel.read",
            "runtime.world.voxel.write", "runtime.world.voxel.grid_spec"}) do
            if not runtime.has(feature) then report.waiting_for = feature; return end
        end
        report.session, report.waiting_for = runtime.world.session_id(), nil
        phase("cursor-stability")
    elseif report.state == "cursor-stability" then
        wait_for_cursor()
    elseif elapsed >= next_sample then
        next_sample = elapsed + 0.5
        if report.state == "baseline-first" then
            local cells, reason = read_chunk()
            report.read_reason = reason
            if cells then
                report.baseline = cells
                if not choose_target(cells) then return stop("failed", "no occupied voxel in the selected chunk", false) end
                phase("baseline-verify")
            end
        elseif report.state == "baseline-verify" then
            local cells, reason = read_chunk()
            report.read_reason = reason
            if cells then
                if not same(cells, report.baseline) then return stop("failed", "baseline changed before any write", false) end
                report.baseline_verified = true
                persist(baseline_output, diagnostic.encode({schema = 1, kind = report.kind, session = report.session,
                    state = "baseline-saved", origin = report.origin, baseline = report.baseline,
                    target = report.target, target_index = report.target_index, original_value = report.original_value}))
                report.baseline_persisted = true
                phase("no-op-ready")
            end
        elseif report.state == "no-op-ready" then
            write_once("no-op", report.original_value, report.original_value)
        elseif report.state == "clear-ready" then
            write_once("clear", 0, report.original_value)
        elseif report.state == "restore-ready" then
            write_once("restore", report.original_value, 0)
        else
            local cells, reason = read_chunk()
            report.read_reason = reason
            if cells then
                local wanted = report.state == "clear-readback" and 0 or report.original_value
                local matches = observe(cells, wanted)
                verified = matches and verified + 1 or 0
                report.consecutive_readbacks = verified
                publish()
                if report.state == "failed-write-readback" then
                    if verified >= 3 then
                        recovery_pending = false
                        return stop("failed", "writer failed, full original chunk confirmed unchanged", false)
                    end
                elseif verified >= 3 then
                    if report.state == "no-op-readback" then
                        recovery_pending = false
                        report.no_op_verified = true
                        phase("clear-ready")
                    elseif report.state == "clear-readback" then
                        report.one_cell_change_verified = true
                        phase("restore-ready")
                    elseif report.state == "restore-readback" then
                        report.restored_verified = true
                        recovery_pending = false
                        return stop("complete", "one-cell mutation and full-chunk restoration verified three times", false)
                    end
                end
            end
        end
    end
    if report.state ~= "waiting" and report.state ~= "cursor-stability" and elapsed - phase_started >= 25 then
        return stop("recovery-required", "phase readback deadline exceeded", recovery_pending)
    end
end

return {
    update_interval_ms = 100,
    on_load = function()
        -- Block on any earlier report, including failed/incomplete runs. Never
        -- overwrite recovery evidence or restart a successful test on reload.
        for _, path in ipairs({output, baseline_output}) do
            local ok, exists = pcall(diagnostic.exists, path)
            if not ok then
                preserve_existing = true
                return stop("blocked", "cannot inspect earlier report: " .. tostring(exists), false)
            end
            if exists then
                local readable, previous = pcall(diagnostic.read_file, path)
                local state = readable and type(previous) == "string" and previous:match('"state"%s*:%s*"([^"]+)"')
                preserve_existing = true
                return stop("blocked", "earlier report retained, state=" .. tostring(state or "unreadable"), false)
            end
        end
        local ok, reason = pcall(publish)
        if not ok then return stop("blocked", "export is unavailable: " .. tostring(reason), false) end
        shroudforge.log.info("Voxel write diagnostic armed for this disposable world. Hold the Single Voxel hammer cursor still on terrain.")
    end,
    on_update = function(delta)
        if finished then return end
        if finite(delta) and delta > 0 then elapsed = elapsed + delta end
        local ok, reason = pcall(update)
        if not ok then stop("recovery-required", tostring(reason), recovery_pending) end
        if not finished and elapsed >= next_publish then
            next_publish = elapsed + 1
            local saved, failure = pcall(publish)
            if not saved then stop("recovery-required", tostring(failure), recovery_pending) end
        end
    end,
    on_unload = function()
        if not finished then stop("unloaded", "diagnostic unloaded, no automatic mutation or restoration", recovery_pending) end
    end,
}
