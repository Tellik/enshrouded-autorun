-- Private Lua world fixture. Exercises the actual diagnostic job, never a
-- game process. Its baseline/clear/restore values are independent test oracles.
local output = "report.json"
local baseline_output = "baseline.json"

local function fixture(mode, previous_files)
    local f = {files = previous_files or {}, cells = {}, writes = {}, reads = 0, time = 0, sequence = 0, session = 17}
    for index = 1, 512 do f.cells[index] = 65282 end
    local environment = setmetatable({}, {__index = _G})
    local function write_file(path, contents)
        if mode == "baseline-export-failed" and path == baseline_output then error("private export failure") end
        if mode == "post-clear-export-failed" and #f.writes == 2 and path == output then error("private report failure") end
        f.files[path] = contents
    end
    environment.diagnostic = {
        encode = encode_diagnostic_report,
        exists = function(path) return f.files[path] ~= nil end,
        read_file = function(path) return assert(f.files[path], "missing diagnostic artifact") end,
        write_file = write_file,
        publish = function(report) write_file(output, encode_diagnostic_report(report)) end,
        finish = function(report)
            f.finished = true
            if report then f.status_report = encode_diagnostic_report(report) end
        end,
    }
    local function ignore() end
    environment.shroudforge = {log = {info = ignore, warn = ignore, error = ignore}}
    environment.runtime = {has = function() return mode ~= "feature-unavailable" end, world = {
        session_id = function() return f.session end,
        context_kind = function() return "direct" end,
        cursor = {get = function()
            f.sequence = f.sequence + 1
            return {sequence = f.sequence, value = {primaryFlags = 4096, material = 2, attachedToProp = false,
                primaryTransform = {position = {x = 80 * 2147483648, y = 80 * 2147483648, z = 80 * 2147483648}}}}
        end},
        voxel = {
            get_grid_spec = function() return {origin = {0, 0, 0}, cellSize = {0.5, 0.5, 0.5}} end,
            read = function(x, y, z, sx, sy, sz)
                assert(x == 80 and y == 72 and z == 80 and sx == 8 and sy == 8 and sz == 8,
                    "read must cover the complete expected aligned chunk")
                f.reads = f.reads + 1
                local cells = {}
                for index = 1, 512 do cells[index] = f.cells[index] end
                if mode == "baseline-changed" and f.reads >= 2 then cells[512] = 65283 end
                return cells
            end,
            write = function(x, y, z, sx, sy, sz, cells)
                assert(x == 80 and y == 72 and z == 80 and sx == 1 and sy == 1 and sz == 1 and #cells == 1,
                    "only the selected cell may be written")
                local saved = assert(f.files[baseline_output], "baseline must exist before a write")
                local baseline = assert(saved:match('"baseline":%[([^%]]+)%]'), "baseline array must be exported")
                local count = 0
                for value in baseline:gmatch("%d+") do assert(value == "65282"); count = count + 1 end
                assert(count == 512, "complete immutable baseline must be saved")
                assert(f.files[output]:find('"state":"intent%-persisted"'), "write intent must be persisted first")
                f.writes[#f.writes + 1] = {value = cells[1], at = f.time}
                local number = #f.writes
                if mode == "no-op-uncertain" and number == 1 then
                    f.cells[512] = 0
                    return false, "private uncertain no-op outcome"
                end
                if mode == "no-op-rejected" and number == 1 then return false, "private rejected write" end
                if mode == "clear-rejected" and number == 2 then return false, "private rejected write" end
                if mode == "restore-rejected" and number == 3 then return false, "private rejected restore" end
                f.cells[1] = cells[1]
                if mode == "outside-cell-changed" and number == 2 then f.cells[512] = 65283 end
                if mode == "clear-uncertain" and number == 2 then return false, "private uncertain outcome" end
                if mode == "clear-exception" and number == 2 then error("private native exception") end
                if mode == "session-changed" and number == 2 then f.session = 18 end
                return true
            end,
        },
    }}
    f.mod = assert(load(voxel_write_probe_source, "voxel-write-probe", "t", environment))()
    f.mod.on_load()
    for _ = 1, 2600 do f.time = f.time + 0.1; f.mod.on_update(0.1) end
    return f
end

local function result(f, state, recovery)
    local report = assert(f.files[output], "report must exist")
    assert(report:find('"state":"' .. state .. '"', 1, true), "expected final state " .. state .. ", got " .. report)
    assert(report:find('"recovery_required":' .. tostring(recovery), 1, true), "wrong recovery status")
end

local success = fixture("success")
result(success, "complete", false)
assert(#success.writes == 3 and success.writes[1].value == 65282 and success.writes[2].value == 0
    and success.writes[3].value == 65282, "expected exactly no-op, clear, restore")
assert(success.writes[2].at - success.writes[1].at >= 1.5
    and success.writes[3].at - success.writes[2].at >= 1.5, "readback phases must span later updates")
for _, cell in ipairs(success.cells) do assert(cell == 65282, "complete original chunk must be restored") end
local original_report = success.files[output]
local reload = fixture("success", success.files)
assert(#reload.writes == 0 and reload.reads == 0 and reload.files[output] == original_report,
    "reload must retain a completed report without restarting")

for _, mode in ipairs({"no-op-rejected", "clear-rejected"}) do
    local f = fixture(mode)
    result(f, "failed", false)
    assert(#f.writes == (mode == "no-op-rejected" and 1 or 2), "rejected write must never be retried")
end
for _, mode in ipairs({"clear-uncertain", "clear-exception", "post-clear-export-failed", "outside-cell-changed"}) do
    local f = fixture(mode)
    assert(#f.writes == 2 and f.cells[1] == 0, "unconfirmed mutation must not trigger an automatic second mutation")
    assert(f.files[baseline_output], "immutable recovery baseline must survive failure")
    if mode ~= "post-clear-export-failed" then
        result(f, "recovery-required", true)
    else
        assert(f.finished and f.status_report:find('"state":"recovery-required"', 1, true),
            "failed file publication must still finish with a visible recovery status")
        assert(f.files[output]:find('"state":"intent-persisted"', 1, true),
            "failed file publication must retain the previous persisted write intent")
    end
end
local restore_failure = fixture("restore-rejected")
result(restore_failure, "recovery-required", true)
assert(#restore_failure.writes == 3 and restore_failure.cells[1] == 0, "failed restore must never be retried")
local no_op_uncertain = fixture("no-op-uncertain")
result(no_op_uncertain, "recovery-required", true)
assert(#no_op_uncertain.writes == 1, "an uncertain no-op must also stop every further mutation")
local changed_session = fixture("session-changed")
result(changed_session, "session-changed", true)
assert(#changed_session.writes == 2, "session change must stop further writes")
local baseline_changed = fixture("baseline-changed")
result(baseline_changed, "failed", false)
assert(#baseline_changed.writes == 0, "unstable baseline must prevent every write")
local export_failure = fixture("baseline-export-failed")
assert(#export_failure.writes == 0, "failed baseline export must prevent every write")
local orphan_baseline = fixture("success", {[baseline_output] = '{"state":"baseline-saved"}'})
assert(#orphan_baseline.writes == 0 and orphan_baseline.files[output] == nil,
    "existing recovery baseline without main report must block reload")
local unavailable_feature = fixture("feature-unavailable")
result(unavailable_feature, "timeout", false)
assert(#unavailable_feature.writes == 0 and unavailable_feature.reads == 0,
    "unavailable features must reach the deadline without reading or writing")
