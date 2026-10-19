-- Exercise shipped Lua code with in-memory exports and denied native operations.
-- These checks do not establish that a native operation works in Enshrouded.
local function equal(actual, expected, label)
    assert(actual == expected, label .. ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
end

local function near(actual, expected, label)
    assert(math.abs(actual - expected) < 1e-12, label .. ": expected " .. expected .. ", got " .. actual)
end

local function sequence(actual, expected, label)
    equal(#actual, #expected, label .. " length")
    for index, value in ipairs(expected) do equal(actual[index], value, label .. "[" .. index .. "]") end
end

-- Independent snapshot of the retained setting values, including manual fallback
-- coordinates. The fixture reads the actual shipped extended.mod.json values.
local expected_settings = {
    panelWidth=1180, panelHeight=190, panelPositionX=-1, panelPositionY=92,
    maximumCopyableProps=60000, sourceX=0, sourceY=0, sourceZ=0,
    sizeX=8, sizeY=8, sizeZ=8, blueprintName="my_blueprint", blueprintNewName="my_blueprint_copy",
    pasteVoxelMode="replace", targetPropMode="keep", targetX=0, targetY=0, targetZ=0,
    targetWorldX=0, targetWorldY=0, targetWorldZ=0, rotationQuarterTurns=0, rotationAxis="y",
    entityHandle=0, templateUuidHigh="0000000000000000", templateUuidLow="0000000000000000",
    trackingId=0, feedbackId=0, entityX=0, entityY=0, entityZ=0,
    boundsMinX=0, boundsMinY=0, boundsMinZ=0, boundsMaxX=0, boundsMaxY=0, boundsMaxZ=0,
}
for name, value in pairs(expected_settings) do equal(world_editor_settings[name], value, "setting " .. name) end
for name in pairs(world_editor_settings) do assert(expected_settings[name] ~= nil, "unexpected setting " .. name) end

-- A literal V7 file is the compatibility oracle. The UUID halves exceed exact
-- double precision, and nonuniform scale must survive without numeric UUID casts.
local blueprint_file = table.concat({
    "SHROUDFORGE_WORLD_BLUEPRINT_V7", "y", "1,0.5,1.5", "voxel", "2,1,3",
    "-0.5,0,0.5", "0.5,0.5,0.5", "10,20,0,40,0,60", "2,2,1,2,0,2", "1",
    "42,0.25,0.5,0.75,0,0,0,1,0.5,1.25,2,fedcba9876543210,0123456789abcdef",
}, "\n")

local unavailable = {
    ["runtime.world.voxel.read"]=true, ["runtime.world.voxel.write"]=true,
    ["runtime.world.entity.spawn"]=true, ["runtime.world.entity.destroy"]=true,
    ["runtime.world.entity.place"]=true, ["runtime.world.entity.finish_building"]=true,
}

local function fixture(context, session, deny_unverified_features)
    local f = {context=context, session=session, settings={}, actions={}, files={}, native_calls=0, sends=0, effects={}}
    for name, value in pairs(world_editor_settings) do f.settings[name] = value end
    f.files["world-editor/blueprints/fixture.sfbp"] = blueprint_file
    local function forbidden(operation)
        return function()
            f.native_calls = f.native_calls + 1
            error("blocked native operation was invoked: " .. operation)
        end
    end
    local environment = setmetatable({}, {__index=_G})
    environment.io = {
        export=function(path, contents) f.files[path] = contents end,
        export_list=function(path)
            equal(path, "world-editor/blueprints", "library export directory")
            return {"world-editor/blueprints/fixture.sfbp"}
        end,
        read_export_to_string=function(path) return assert(f.files[path], "export does not exist: " .. path) end,
        export_exists=function(path) return f.files[path] ~= nil end,
    }
    local function ignore_log() end
    environment.shroudforge = {
        settings={get=function(name) return f.settings[name] end},
        ui={on_action=function(name, action) f.actions[name] = action end},
        input={is_key_down=function() return false end},
        log={debug=ignore_log, info=ignore_log, warn=ignore_log, error=ignore_log, trace=ignore_log},
    }
    environment.game = {assets={get_loaded_resources_by_type=function(name)
        equal(name, "keen::ItemInfo", "placement recipe resource")
        return {{data={itemId=42, equipment={
            placedEntity="01234567-89ab-cdef-0123-456789abcdef",
            placementAABBmin={x=0, y=0, z=0}, placementAABBmax={x=1, y=1, z=1},
            placementColliders={{dataArray={{value={materialFeedbackId=7}}}}},
        }}}}
    end}}
    environment.runtime = {
        is_server=false,
        require=function(name) equal(name, "runtime.lifecycle", "required runtime feature") end,
        has=function(name) return not (deny_unverified_features and unavailable[name]) end,
        status=function() return {reason="native operation is not verified for Enshrouded 1.0"} end,
        report_effect=function(state, detail)
            assert(state ~= "write-confirmed", "a blocked action must not report a successful world change")
            f.effects[#f.effects + 1] = {state=state, detail=detail}
        end,
        network={
            status=function() return {available=true, configured_server_steam_id="76561198000000001"} end,
            send_mod=function() f.sends = f.sends + 1; error("blocked action sent a P2P request") end,
            receive_mod=function() return {} end,
        },
        world={
            session_id=function() return f.session end,
            context_kind=function() return f.context end,
            voxel={read=forbidden("voxel.read"), write=forbidden("voxel.write"),
                get_grid_spec=forbidden("voxel.get_grid_spec")},
            cursor={get=forbidden("cursor.get")},
            entity={
                spawn=forbidden("entity.spawn"), destroy=forbidden("entity.destroy"),
                place=forbidden("entity.place"), finish_building=forbidden("entity.finish_building"),
                get_transform=forbidden("entity.get_transform"), set_scale=forbidden("entity.set_scale"),
                query_props=forbidden("entity.query_props"), query_props_in_bounds=forbidden("entity.query_props_in_bounds"),
                register_prop_recipes=function(recipes)
                    equal(#recipes, 1, "registered recipe count")
                    equal(recipes[1].itemId, 42, "registered recipe item")
                    sequence(recipes[1].bounds, {0, 0, 0, 1, 1, 1}, "registered placement bounds")
                    equal(recipes[1].feedback, 7, "registered material feedback")
                    return true
                end,
            },
        },
    }
    f.runtime, f.io = environment.runtime, environment.io
    local loaded = {}
    environment.require = function(name)
        if loaded[name] then return loaded[name] end
        local source = assert(world_editor_sources[name], "unexpected module " .. name)
        local module = assert(load(source, "world-editor/" .. name, "t", environment))()
        if name == "blueprint" then
            -- Capture the real shared editor state at its existing module boundary.
            local initialize = module
            module = function(editor) f.editor = editor; initialize(editor) end
        end
        loaded[name] = module
        return module
    end
    f.mod = assert(load(world_editor_sources.mod, "world-editor/mod", "t", environment))()
    f.mod.on_load()
    f.actions.selectBlueprint("fixture")
    equal(f.editor.active_blueprint_name, "fixture", "blueprint loaded through UI action")
    assert(f.editor.clipboard, "the literal V7 blueprint must be accepted")
    for name, value in pairs(world_editor_settings) do equal(f.editor.setting(name), value, "editor setting " .. name) end
    return f
end

local f = fixture("direct", 81, true)
local captured = f.editor.clipboard
sequence(captured.cells, {10, 20, 0, 40, 0, 60}, "decoded cells")
sequence(captured.coverage, {2, 2, 1, 2, 0, 2}, "decoded coverage")
sequence(captured.extent, {1, 0.5, 1.5}, "decoded extent")
sequence(captured.voxelOffset, {-0.5, 0, 0.5}, "decoded voxel offset")
sequence(captured.cellSize, {0.5, 0.5, 0.5}, "decoded cell size")
local prop = captured.props[1]
equal(#captured.props, 1, "decoded prop count")
equal(prop.itemId, 42, "decoded prop item")
equal(prop.templateUuidHighHex, "fedcba9876543210", "UUID high")
equal(prop.templateUuidLowHex, "0123456789abcdef", "UUID low including leading zero")
sequence({prop.sx, prop.sy, prop.sz}, {0.5, 1.25, 2}, "decoded prop scale")
equal(f.editor.encode_blueprint(captured), blueprint_file, "V7 wire compatibility")
f.actions.saveBlueprint()
equal(f.files["world-editor/blueprints/my_blueprint.sfbp"], blueprint_file, "UI save preserves V7 content")

-- Seen from above, +90 degrees around Y maps the three original X=1 cells
-- into the first new Z row, followed by the three original X=0 cells.
f.actions.rotateBlueprint()
equal(f.editor.rotation_turns(), 1, "rotate action advances one quarter turn")
local rotated = f.editor.rotated_blueprint(captured, f.editor.rotation_turns())
sequence({rotated.sx, rotated.sy, rotated.sz}, {3, 1, 2}, "rotated dimensions")
sequence(rotated.cells, {20, 40, 60, 10, 0, 0}, "90 degree cell order")
sequence(rotated.coverage, {2, 2, 2, 2, 1, 0}, "90 degree coverage order")
sequence(rotated.extent, {1.5, 0.5, 1}, "rotated extent")
sequence(rotated.voxelOffset, {0.5, 0, -0.5}, "rotated voxel offset")
sequence(rotated.boundsMin, {0, 0, -1}, "rotated minimum bounds")
sequence(rotated.boundsMax, {1.5, 0.5, 0}, "rotated maximum bounds")
local rotated_prop = rotated.props[1]
sequence({rotated_prop.x, rotated_prop.y, rotated_prop.z}, {0.75, 0.5, -0.25}, "rotated prop position")
near(rotated_prop.qy, math.sqrt(0.5), "rotated prop quaternion Y")
near(rotated_prop.qw, math.sqrt(0.5), "rotated prop quaternion W")
sequence({rotated_prop.sx, rotated_prop.sy, rotated_prop.sz}, {0.5, 1.25, 2}, "rotation preserves prop scale")
equal(rotated_prop.templateUuidHighHex, prop.templateUuidHighHex, "rotation preserves UUID high")
equal(rotated_prop.templateUuidLowHex, prop.templateUuidLowHex, "rotation preserves UUID low")
equal(f.editor.encode_blueprint(captured), blueprint_file, "rotation does not alter the captured blueprint")

local mutation_actions = {"pasteVoxels", "pasteAtCursor", "spawnEntity", "placeEntity",
    "destroyEntity", "destroySelectedProp", "undoVoxels"}
local function expect_blocked(f, label)
    f.settings.trackingId, f.settings.feedbackId, f.settings.entityHandle = 42, 7, 91
    -- A retained journal makes F4 exercise the gate before any restoration.
    local journal = {session=81, hasVoxels=true, voxel_written=true,
        region={x=0, y=0, z=0, sx=2, sy=1, sz=3}, cells={0, 0, 0, 0, 0, 0},
        expected_cells={10, 20, 0, 40, 0, 60}, entities={}, removed_props={}}
    f.editor.undo_state = journal
    for _, action in ipairs(mutation_actions) do
        assert(f.actions[action], "missing UI action " .. action)()
        equal(f.native_calls, 0, label .. " blocks native operations for " .. action)
        equal(f.sends, 0, label .. " sends no P2P request for " .. action)
        equal(f.editor.p2p_bridge.pending(), false, label .. " queues no P2P request for " .. action)
        equal(f.editor.undo_state, journal, label .. " retains the undo journal for " .. action)
    end
end

expect_blocked(f, "local context with unavailable write features")

-- A stale status snapshot must not turn an absent native writer into a ten
-- second world-readiness wait. The action remains a no-op and reports the
-- missing mutation API directly.
local missing_writer = fixture("direct", 81, false)
missing_writer.runtime.has = function() return false end
missing_writer.runtime.status = function() return {available=true} end
missing_writer.actions.pasteVoxels()
equal(missing_writer.editor.pending_world_action, nil, "missing voxel writer does not start a retry")
assert(missing_writer.editor.editor_hint:find("No verified native ECS terrain write operation", 1, true),
    "missing voxel writer is reported directly")
equal(missing_writer.native_calls, 0, "missing voxel writer performs no native write")
missing_writer.mod.on_update(15)
equal(missing_writer.native_calls, 0, "missing voxel writer cannot retry into a mutation")
missing_writer.mod.on_unload()

local missing_spawn = fixture("direct", 81, false)
missing_spawn.runtime.has = function(name) return name ~= "runtime.world.entity.spawn" end
missing_spawn.runtime.status = function() return {available=true} end
equal(missing_spawn.editor.feature("runtime.world.entity.spawn"), false,
    "missing spawn command rejects stale status availability")
assert(missing_spawn.editor.editor_hint:find("No verified native ECS prop spawn operation", 1, true),
    "missing prop spawn is reported directly")
equal(missing_spawn.native_calls, 0, "missing prop spawn invokes no native operation")
missing_spawn.mod.on_unload()

local missing_undo_writer = fixture("direct", 81, false)
missing_undo_writer.runtime.has = function(name) return name ~= "runtime.world.voxel.write" end
missing_undo_writer.runtime.status = function() return {available=true} end
local retained_journal = {session=81, hasVoxels=true, voxel_written=true,
    region={x=0, y=0, z=0, sx=2, sy=1, sz=3}, cells={0, 0, 0, 0, 0, 0},
    expected_cells={10, 20, 0, 40, 0, 60}, entities={}, removed_props={}}
missing_undo_writer.editor.undo_state = retained_journal
missing_undo_writer.actions.undoVoxels()
equal(missing_undo_writer.editor.pending_world_action, nil, "missing undo writer does not start a retry")
equal(missing_undo_writer.editor.undo_state, retained_journal, "missing undo writer retains recovery journal")
equal(missing_undo_writer.editor.process_progress.state, "error", "missing undo writer ends with an error")
assert(missing_undo_writer.editor.editor_hint:find("journal is retained", 1, true),
    "missing undo writer explains recovery retention")
equal(missing_undo_writer.native_calls, 0, "missing undo writer performs no mutation")
missing_undo_writer.mod.on_unload()

-- Props-only paste must also respect the unavailable native spawn feature.
f.editor.clipboard = {region={rotationAxis="y"}, extent={1, 1, 1}, hasVoxels=false, props={prop}}
f.actions.pasteVoxels()
equal(f.native_calls, 0, "props-only paste does not invoke unavailable native operations")
equal(f.editor.process_progress.state, "error", "blocked prop placement is not reported complete")
f.mod.on_update(0.1)
equal(f.native_calls, 0, "update does not retry unverified operations")
equal(f.sends, 0, "update sends no local placement to P2P")
f.mod.on_unload()

-- Unknown context must stop before native feature readiness, even when every
-- feature claims availability. A zero generation must do the same for 'direct'.
expect_blocked(fixture("unknown", 81, false), "unknown world context")
expect_blocked(fixture("direct", 0, false), "unavailable world generation")

-- A transient transform failure must retain the exact journal and must never
-- invoke destruction or restoration. Only an explicit membership absence can
-- complete the removal side of undo.
local pending = fixture("direct", 81, false)
local pending_journal = {session=81, hasVoxels=false, voxel_written=false,
    entities={{entityHandle=91}}, entity_index=1, removed_props={}, removed_prop_index=0}
pending.editor.undo_state = pending_journal
pending.runtime.world.entity.get_transform = function() return nil, "entity-state-unavailable-retry" end
pending.actions.undoVoxels()
equal(pending.native_calls, 0, "unreadable prop cannot trigger a mutation")
equal(pending.editor.undo_state, pending_journal, "unreadable prop retains undo journal")
equal(pending.editor.undo_state.entity_index, 1, "unreadable prop retains undo progress")
pending.runtime.world.entity.get_transform = function() return nil, "entity-absent" end
pending.runtime.report_effect = function(state, detail)
    pending.effects[#pending.effects + 1] = {state=state, detail=detail}
end
pending.actions.undoVoxels()
equal(pending.native_calls, 0, "confirmed absent prop does not trigger another mutation")
equal(pending.editor.undo_state, nil, "confirmed absent prop completes props-only undo")

local replaced = fixture("direct", 81, false)
local original = replaced.editor.intended_spawn_transform({1, 2, 3}, {0, 0, 0, 1}, {1, 1, 1}, 42)
local replaced_journal = {session=81, hasVoxels=false, voxel_written=false,
    entities={}, entity_index=0, removed_props={{entityHandle=91, removal_pending=true,
        expected_transform=original}}, removed_prop_index=1}
replaced.editor.undo_state = replaced_journal
replaced.runtime.world.entity.get_transform = function() return nil, "entity-state-unavailable-retry" end
replaced.actions.undoVoxels()
equal(replaced.editor.undo_state, replaced_journal, "unconfirmed target removal keeps its restore record")
equal(replaced.native_calls, 0, "unconfirmed target removal cannot spawn a duplicate")
replaced.runtime.world.entity.get_transform = function() return original end
replaced.runtime.report_effect = function() end
replaced.actions.undoVoxels()
equal(replaced.native_calls, 0, "still-present original is not restored twice")
equal(replaced.editor.undo_state, nil, "verified original completes pending-removal recovery")

-- F8 can wait for a temporarily unreadable terrain context. Cancelling that
-- continuation must also close the public progress state and permit a new F8.
local function waiting_capture()
    local f = fixture("direct", 81, false)
    f.editor.selection_a, f.editor.selection_b = {x=0, y=0, z=0}, {x=1, y=1, z=1}
    f.runtime.has = function(name) return name ~= "runtime.world.voxel.read" end
    f.runtime.status = function() return {reason="world operation is not ready"} end
    f.actions.captureAndSave()
    assert(f.editor.pending_world_action, "F8 must retain its pending terrain continuation")
    equal(f.editor.process_progress.state, "running", "pending F8 progress")
    equal(f.editor.save_progress.state, "saving", "pending F8 save")
    return f
end

local function capture_stopped(f, label)
    equal(f.editor.pending_world_action, nil, label .. " clears the terrain retry")
    equal(f.editor.pending_cursor_action, nil, label .. " clears the cursor continuation")
    equal(f.editor.pending_prop_capture, nil, label .. " clears the prop continuation")
    equal(f.editor.process_progress.state, "error", label .. " ends operation progress")
    assert(f.editor.save_progress.state ~= "saving", label .. " releases the save busy state")
    assert(f.editor.editor_stage ~= "capturing", label .. " leaves the capture stage")
    assert(f.files["world-editor/editor-state.txt"]:find("processState=error\n", 1, true),
        label .. " publishes terminal progress")
    equal(f.files["world-editor/blueprints/capture-1.sfbp"], nil, label .. " saves no incomplete blueprint")
    equal(f.native_calls, 0, label .. " invokes no unavailable native operation")
end

local timed_out = waiting_capture()
local previous_blueprint = timed_out.editor.clipboard
timed_out.mod.on_update(15)
capture_stopped(timed_out, "terrain retry timeout")
equal(timed_out.editor.clipboard, previous_blueprint, "timeout retains the previous complete blueprint")
local terminal_sequence = timed_out.editor.process_progress.sequence
timed_out.mod.on_update(410)
capture_stopped(timed_out, "later updates after timeout")
equal(timed_out.editor.process_progress.sequence, terminal_sequence, "cancelled F8 cannot restart itself")
timed_out.actions.captureAndSave()
assert(timed_out.editor.pending_world_action, "a new F8 is accepted after cancellation")
equal(timed_out.editor.process_progress.sequence, terminal_sequence + 1, "new F8 starts a new operation")
timed_out.actions.resetEditor()
capture_stopped(timed_out, "F6 during a terrain retry")

local missing_session = waiting_capture()
previous_blueprint = missing_session.editor.clipboard
missing_session.session = 0
missing_session.mod.on_update(0.1)
capture_stopped(missing_session, "world session unavailable")
equal(missing_session.editor.clipboard, previous_blueprint, "session loss retains the previous complete blueprint")

local denied_after_wait = waiting_capture()
denied_after_wait.runtime.status = function() return {reason="native operation is not verified"} end
denied_after_wait.mod.on_update(0.1)
capture_stopped(denied_after_wait, "terrain feature becomes unavailable")

local readiness_race = waiting_capture()
readiness_race.runtime.status = function() return {available=true} end
readiness_race.mod.on_update(0.1)
assert(readiness_race.editor.pending_world_action, "has/status readiness race retains a pending terrain capture")
readiness_race.mod.on_update(15)
capture_stopped(readiness_race, "terrain readiness race timeout")

local denied_capture = fixture("direct", 81, true)
denied_capture.editor.selection_a, denied_capture.editor.selection_b = {x=0, y=0, z=0}, {x=1, y=1, z=1}
denied_capture.actions.captureAndSave()
capture_stopped(denied_capture, "terrain feature is unavailable immediately")

local denied_export = fixture("direct", 81, false)
denied_export.editor.selection_a, denied_export.editor.selection_b = {x=0, y=0, z=0}, {x=1, y=1, z=1}
denied_export.io.export_exists = function() error("export unavailable") end
denied_export.actions.captureAndSave()
capture_stopped(denied_export, "blueprint export is unavailable")

local prop_capture = fixture("direct", 81, false)
local prop_queries = 0
prop_capture.editor.selection_a, prop_capture.editor.selection_b = {x=0, y=0, z=0}, {x=1, y=1, z=1}
prop_capture.runtime.world.entity.query_props_in_bounds = function()
    prop_queries = prop_queries + 1
    return nil, "still scanning"
end
prop_capture.actions.capturePropsOnly()
assert(prop_capture.editor.pending_prop_capture, "props-only capture retains its continuation")
local retained_journal = {session=81, recovery_required=true}
prop_capture.editor.undo_state = retained_journal
prop_capture.actions.resetEditor()
capture_stopped(prop_capture, "F6 during a prop scan")
equal(prop_capture.editor.undo_state, retained_journal, "capture reset retains previous placement recovery")
prop_capture.mod.on_update(410)
equal(prop_queries, 1, "cancelled prop capture never resumes its scan or save callback")
capture_stopped(prop_capture, "later updates after prop capture reset")

-- A bounded original cache request returns pending while the engine fills it.
-- F8 retries that same capture and preserves the overall timeout across retries.
local function cache_capture(complete_after, grid_ready_after, pending_reason)
    local f = fixture("direct", 81, false)
    f.editor.selection_a, f.editor.selection_b = {x=0, y=0, z=0}, {x=1, y=1, z=1}
    local reads, grid_reads = 0, 0
    f.runtime.world.voxel.get_grid_spec = function()
        grid_reads = grid_reads + 1
        if grid_reads < (grid_ready_after or 0) then return nil, "voxel-grid-pending" end
        return {id="voxel", origin={0,0,0}, cellSize={0.5,0.5,0.5}, maximum={20480,20480,20480}}
    end
    f.runtime.world.voxel.read = function(_, _, _, sx, sy, sz)
        reads = reads + 1
        if reads < complete_after then return nil, pending_reason or "voxel-read-pending" end
        local cells = {}
        for index=1,sx*sy*sz do cells[index]=256 end
        return cells
    end
    f.runtime.world.entity.query_props_in_bounds = function() return {} end
    f.actions.captureAndSave()
    return f, function() return reads end, function() return grid_reads end
end

local cache_ready, cache_reads = cache_capture(3)
assert(cache_ready.editor.pending_world_action, "cache-pending F8 retains its retry")
equal(cache_ready.files["world-editor/blueprints/capture-1.sfbp"], nil, "pending cache saves no partial blueprint")
cache_ready.mod.on_update(0.1)
assert(cache_ready.editor.pending_world_action, "second cache miss remains pending")
cache_ready.mod.on_update(0.1)
equal(cache_reads(), 3, "original cache is polled on later updates")
equal(cache_ready.editor.pending_world_action, nil, "completed cache clears retry")
equal(cache_ready.editor.process_progress.state, "complete", "complete cache finishes F8")
assert(cache_ready.files["world-editor/blueprints/capture-1.sfbp"], "complete cache saves the V7 blueprint")

local compatible_cache = cache_capture(2, nil, "voxel read is waiting for the Scene gate or original cache readiness")
assert(compatible_cache.editor.pending_world_action, "installed ABI-12 bridge pending remains compatible")
compatible_cache.mod.on_update(0.1)
equal(compatible_cache.editor.process_progress.state, "complete", "installed bridge pending completes F8")

local cache_timeout, timeout_reads = cache_capture(math.huge)
cache_timeout.mod.on_update(5)
equal(cache_timeout.editor.pending_world_action.elapsed, 5, "first cache retry preserves elapsed")
cache_timeout.mod.on_update(5)
equal(cache_timeout.editor.pending_world_action.elapsed, 10, "later cache retry preserves elapsed")
cache_timeout.mod.on_update(5)
capture_stopped(cache_timeout, "original cache timeout")
equal(timeout_reads(), 3, "timeout stops rather than resetting after each cache retry")

-- Grid metadata uses the same session-bound reader, so a busy Scene must wait
-- before coordinates or cells are consumed, with one overall F8 deadline.
local grid_ready, grid_cell_reads, grid_reads = cache_capture(1, 3)
assert(grid_ready.editor.pending_world_action, "pending grid metadata retains F8")
equal(grid_cell_reads(), 0, "pending metadata cannot read terrain with guessed coordinates")
equal(grid_ready.files["world-editor/blueprints/capture-1.sfbp"], nil, "pending grid saves no partial blueprint")
grid_ready.mod.on_update(0.1)
equal(grid_cell_reads(), 0, "second metadata miss still performs no terrain read")
grid_ready.mod.on_update(0.1)
equal(grid_reads(), 3, "grid is retried on later updates and then cached")
equal(grid_cell_reads(), 1, "terrain is read once after metadata becomes ready")
equal(grid_ready.editor.pending_world_action, nil, "ready grid completes its retry")
equal(grid_ready.editor.process_progress.state, "complete", "ready grid finishes F8")
assert(grid_ready.files["world-editor/blueprints/capture-1.sfbp"], "ready grid saves the V7 blueprint")

local grid_timeout, grid_timeout_cells = cache_capture(1, math.huge)
grid_timeout.mod.on_update(5)
grid_timeout.mod.on_update(5)
grid_timeout.mod.on_update(5)
capture_stopped(grid_timeout, "grid metadata timeout")
equal(grid_timeout_cells(), 0, "grid timeout never reads terrain")

local grid_invalid, grid_invalid_cells = cache_capture(1, math.huge)
grid_invalid.runtime.world.voxel.get_grid_spec = function() return nil, "native voxel grid returned invalid metadata" end
grid_invalid.mod.on_update(0.1)
capture_stopped(grid_invalid, "invalid grid metadata")
equal(grid_invalid_cells(), 0, "invalid metadata never reads terrain")

local grid_race, grid_race_cells = cache_capture(1, math.huge)
grid_race.runtime.has = function(name) return name ~= "runtime.world.voxel.grid_spec" end
grid_race.runtime.status = function() return {available=true} end
grid_race.mod.on_update(0.1)
assert(grid_race.editor.pending_world_action, "has/status readiness race retains the grid retry")
equal(grid_race_cells(), 0, "readiness race cannot consume guessed grid coordinates")
grid_race.runtime.has = function() return true end
grid_race.runtime.world.voxel.get_grid_spec = function()
    return {id="voxel", origin={0,0,0}, cellSize={0.5,0.5,0.5}, maximum={20480,20480,20480}}
end
grid_race.mod.on_update(0.1)
equal(grid_race.editor.process_progress.state, "complete", "ready metadata completes after the readiness race")

local bool_grid = cache_capture(1, math.huge)
bool_grid.runtime.world.voxel.get_grid_spec = function()
    return nil, "Grid is unavailable. This backend currently exposes 'voxel'"
end
bool_grid.mod.on_update(0.1)
assert(bool_grid.editor.pending_world_action, "installed bool-only grid bridge remains pending")
bool_grid.mod.on_update(15)
capture_stopped(bool_grid, "installed bool-only grid bridge timeout")
