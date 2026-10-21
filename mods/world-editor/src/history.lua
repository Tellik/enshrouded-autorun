-- Undo journals, rollback and world-session recovery.
return function(editor)
    function editor.update_world_session()
        local session = editor.current_session()
        -- Zero also occurs briefly during loading. Do not discard an undo journal
        -- or call it a different world until a new nonzero generation is known.
        if session == 0 then return end
        if editor.observed_session == nil then editor.observed_session = session; return end
        if session == editor.observed_session then return end
        local previous = editor.observed_session
        editor.observed_session = session
        if editor.undo_state then
            editor.retired_sessions[#editor.retired_sessions + 1] = {session = previous, undo = editor.undo_state}
            shroudforge.log.warn("World Editor retained the journal for ended session " .. tostring(previous) ..
                ", it cannot be replayed in session " .. tostring(session))
        end
        editor.undo_state = nil
        if editor.editor_is_server then
            for peer, transaction in pairs(editor.server_remote_transactions) do
                if transaction.undo then
                    editor.retired_sessions[#editor.retired_sessions+1] = {session=transaction.session,
                        peer=peer,transaction=transaction.transaction,undo=transaction.undo}
                end
            end
            editor.server_remote_transactions = {}
            editor.server_remote_transaction = nil
        end
        editor.pending_cursor_action, editor.pending_prop_capture, editor.pending_world_action = nil, nil, nil
        editor.selection_a, editor.selection_b, editor.selection_target, editor.placement_preview = nil, nil, nil, nil
        editor.live_prop_cache, editor.voxel_grid_spec = {}, nil
        editor.readiness_logged, editor.readiness_wait_logged = false, false
        if editor.save_progress.state == "saving" then
            editor.save_step(editor.save_progress.completed, "Capture cancelled: world session changed", "error")
        end
        if editor.process_progress.state == "running" then
            editor.finish_process("error", "The world changed before this operation could finish.", "warn")
        end
        editor.set_editor_message(editor.clipboard and "selected" or "need_a",
            "World changed. Blueprint retained. Mark a new selection or placement target. Previous-world undo is archived.")
    end

    function editor.count_prop_at(item, position)
        local unavailable = editor.unavailable_world_reason("runtime.world.entity.query_props")
        if unavailable then return nil, unavailable end
        local epsilon = 0.01
        local props, reason = runtime.world.entity.query_props({position[1] - epsilon, position[2] - epsilon, position[3] - epsilon,
            position[1] + epsilon, position[2] + epsilon, position[3] + epsilon}, 0)
        if not props then return nil, reason end
        if #props > 1000000 then return nil, "native prop query exceeded the 1,000,000-prop safety limit" end
        local handles = {}
        for _, prop in ipairs(props) do
            local transform = prop.transform
            local actual_position = transform and editor.world_position(transform.position)
            if actual_position and editor.item_id(prop.itemId) == item and
               math.abs(actual_position.x - position[1]) < 0.01 and
               math.abs(actual_position.y - position[2]) < 0.01 and
               math.abs(actual_position.z - position[3]) < 0.01 then
                handles[#handles + 1] = prop.handle
            end
        end
        return #handles, handles
    end

    function editor.same_cells(left, right)
        if type(left) ~= "table" or type(right) ~= "table" or #left ~= #right then return false end
        for index = 1, #left do if left[index] ~= right[index] then return false end end
        return true
    end

    function editor.transform_matches(actual, expected)
        if type(actual) ~= "table" or type(expected) ~= "table" then return false end
        local actual_transform = actual.transform or actual
        local expected_transform = expected.transform or expected
        local actual_position, expected_position = actual_transform.position, expected_transform.position
        local actual_rotation = actual_transform.orientation or actual_transform.rotation
        local expected_rotation = expected_transform.orientation or expected_transform.rotation
        local actual_scale, expected_scale = actual_transform.scale, expected_transform.scale
        if not actual_position or not expected_position or not actual_rotation or not expected_rotation or
           not actual_scale or not expected_scale then return false end
        local function near(a, b)
            a, b = tonumber(a), tonumber(b)
            return editor.finite_number(a) and editor.finite_number(b) and math.abs(a - b) <= 1e-7
        end
        for _, axis in ipairs({"x", "y", "z"}) do
            if not near(actual_position[axis], expected_position[axis]) or
               not near(actual_scale[axis], expected_scale[axis]) then return false end
        end
        for _, axis in ipairs({"x", "y", "z", "w"}) do
            if not near(actual_rotation[axis], expected_rotation[axis]) then return false end
        end
        return (actual.itemId == nil or expected.itemId == nil or actual.itemId == expected.itemId) and
            (actual.templateUuidHighHex == nil or expected.templateUuidHighHex == nil or
                actual.templateUuidHighHex == expected.templateUuidHighHex) and
            (actual.templateUuidLowHex == nil or expected.templateUuidLowHex == nil or
                actual.templateUuidLowHex == expected.templateUuidLowHex)
    end

    function editor.intended_spawn_transform(position, rotation, scale, item_id_value, template_high, template_low)
        return {
            itemId = item_id_value,
            templateUuidHighHex = template_high,
            templateUuidLowHex = template_low,
            transform = {
                position = {x = position[1], y = position[2], z = position[3]},
                orientation = {x = rotation[1], y = rotation[2], z = rotation[3], w = rotation[4]},
                scale = {x = scale[1], y = scale[2], z = scale[3]},
            },
        }
    end

    editor.undo_voxels = function()
        local backend = editor.execution_backend()
        if not editor.require_execution_backend(backend, "F4 undo") then return end
        if backend == "p2p" then
            if editor.remote_undo and editor.undo_remote then
                editor.undo_remote()
            elseif editor.remote_undo then
                editor.set_editor_message("recovery", "The server undo token is retained, but Steam P2P is unavailable.")
            else
                editor.set_editor_message(nil, "There is no server placement to undo. Place a blueprint on the server with F7 first.")
            end
            return
        end
        -- A recovered server token belongs to a different process/world. Let a
        -- newer local singleplayer placement keep its normal F4 priority.
        if not editor.undo_state and editor.remote_undo then
            editor.set_editor_message("recovery", "A server undo token is retained. Rejoin its Dedicated Server world and press F4 to resume that undo.")
            return
        end
        local expected = editor.undo_state and editor.undo_state.session
        if editor.undo_state then
            local session = runtime.world.session_id and runtime.world.session_id() or 0
            if not expected or session == 0 or session ~= expected then
                editor.set_editor_message("recovery", "Undo belongs to a different or unavailable world session. The journal is retained. No changes were sent.")
                return
            end
        end
        if not editor.undo_state then
            editor.set_editor_message(nil, "Nothing to undo. Place a blueprint with F7 first.")
            shroudforge.log.warn("World Editor: there is no verified blueprint placement to undo")
            return
        end
        local recovering_paste = editor.undo_state.automaticRollback == true
        if recovering_paste then
            editor.process_step("Restoring changes from the failed placement", 0, 0,
                "Checking which world changes can be safely reversed.")
        elseif editor.process_progress.state ~= "running" or editor.process_progress.operation ~= "Undo placement" then
            editor.begin_process("Undo placement", "Checking undo journal", 4,
                "Checking the target world before restoring the previous state.")
        end
        local restore_scale_required = false
        for _, prop in ipairs(editor.undo_state.removed_props or {}) do
            if prop.scale and not editor.prop_scale_is_unit(prop.scale) then restore_scale_required = true end
        end
        if #(editor.undo_state.removed_props or {}) > 0 and
           (not editor.feature("runtime.world.entity.spawn") or
            (restore_scale_required and not editor.feature("runtime.world.entity.set_scale"))) then
            editor.finish_process("error", "Required native prop restoration features are unavailable.", "warn")
            return
        end
        if editor.undo_state.hasVoxels and not editor.require_world_feature("runtime.world.voxel.write", editor.undo_voxels) then
            local unavailable = editor.native_mutation_unavailable_reason("runtime.world.voxel.write")
            if unavailable then
                editor.set_editor_message("recovery", "Undo paused. The journal is retained. " .. unavailable)
                editor.finish_process("error", "The native terrain writer is unavailable, so undo made no changes.", "warn")
            else
                editor.process_step("Waiting for terrain access", 0, 0, "Waiting for the native terrain API.", "debug")
            end
            return
        end
        if editor.undo_state.hasVoxels and not editor.undo_state.automaticRollback and
           not editor.require_world_feature("runtime.world.voxel.read", editor.undo_voxels) then
            editor.process_step("Waiting for terrain verification", 0, 0, "Waiting for the native terrain read API.", "debug")
            return
        end
        local region = editor.undo_state.region
        if editor.undo_state.hasVoxels then
            if not editor.undo_state.automaticRollback then
                local current_cells, read_reason = runtime.world.voxel.read(region.x, region.y, region.z,
                    region.sx, region.sy, region.sz)
                if not current_cells then
                    editor.set_editor_message("recovery", "Undo paused. The terrain snapshot could not be read. Press F4 to retry.")
                    shroudforge.log.warn("World Editor paused undo because the pasted voxel region could not be checked: " .. tostring(read_reason))
                    editor.finish_process("error", "Terrain could not be checked. Retry undo after the runtime is available.", "warn")
                    return
                end
                local expected_cells = editor.undo_state.voxel_written and editor.undo_state.expected_cells or editor.undo_state.cells
                if not editor.same_cells(current_cells, expected_cells) then
                    editor.set_editor_message("recovery", "Undo paused because the terrain changed after placement. Restore those changes manually, then press F4 to retry.")
                    shroudforge.log.warn("World Editor paused undo because the target voxels changed after the paste. No later changes were overwritten.")
                    editor.finish_process("error", "Terrain changed after the paste, so undo paused to protect later edits.", "warn")
                    return
                end
            end
        end
        local entities = editor.undo_state.entities or {}
        local index = editor.undo_state.entity_index or #entities
        local removed_index = editor.undo_state.removed_prop_index or 0
        -- Lock the journal immediately before each mutation so a partial undo
        -- cannot be replaced by F7. Read-only preflight refusals need no new lock.
        -- Restore voxel cells before props so one unreadable prop cannot block the
        -- independent terrain snapshot. The journal preserves any remaining work.
        if editor.undo_state.hasVoxels and editor.undo_state.voxel_written then
            editor.process_step("Restoring terrain", 1, 4, "Restoring and verifying the saved terrain snapshot.")
            editor.undo_state.recovery_required = true
            local ok, reason = runtime.world.voxel.write(region.x, region.y, region.z,
                region.sx, region.sy, region.sz, editor.undo_state.cells)
            if not ok then
                editor.set_editor_message("recovery", "Undo paused because the previous terrain could not be restored. Press F4 to retry.")
                runtime.report_effect("write-failed", reason or "voxel undo failed")
                shroudforge.log.error("World Editor voxel undo failed: " .. tostring(reason))
                editor.finish_process("error", "Terrain restoration failed: " .. tostring(reason), "error")
                return
            end
            editor.undo_state.voxel_written = false
            editor.undo_state.expected_cells = editor.undo_state.cells
        else
            editor.process_step("Terrain checked", 1, 4, "No terrain restoration is pending.")
        end
        -- A queued/paused prop dismantle must never block independent terrain
        -- restoration. F4 may therefore restore the snapshot while that one input
        -- remains pending; it still must not queue a duplicate dismantle.
        if (#entities > 0 or #(editor.undo_state.removed_props or {}) > 0) and
           not editor.feature("runtime.world.entity.get_transform") then
            editor.finish_process("error", "The native prop inspection API is unavailable.", "warn")
            return
        end
        for entity_index = 1, index do
            local entity = entities[entity_index]
            local current, read_reason
            current, read_reason = editor.read_prop(entity.entityHandle, expected)
            if not current and read_reason ~= "entity-absent" then
                editor.set_editor_message("recovery", "Undo paused because the prop state could not be read. Terrain has been restored. Press F4 to retry the prop.")
                shroudforge.log.warn("World Editor paused prop undo because the live transform read failed: " .. tostring(read_reason))
                editor.finish_process("error", "A placed prop could not be checked. Terrain restoration is retained.", "warn")
                return
            end
            if current and not editor.undo_state.automaticRollback and not editor.transform_matches(current, entity.expected_transform) then
                editor.set_editor_message("recovery", "Undo paused because a placed prop was moved or changed. Restore it to its pasted state, then press F4 to retry.")
                runtime.report_effect("write-failed", "a pasted prop changed after paste")
                shroudforge.log.warn("World Editor refused undo because a pasted prop changed after paste")
                editor.finish_process("error", "A placed prop changed after paste, so undo paused to protect that change.", "warn")
                return
            end
        end
        for prop_index = 1, removed_index do
            local prop = editor.undo_state.removed_props[prop_index]
            prop.removal_skipped = nil
            local current, read_reason
            current, read_reason = editor.read_prop(prop.entityHandle, expected)
            if not current and read_reason ~= "entity-absent" then
                editor.set_editor_message("recovery", "Undo paused because the replaced prop state could not be read. Terrain has been restored. Press F4 to retry.")
                shroudforge.log.warn("World Editor paused replaced-prop undo because the live transform read failed: " .. tostring(read_reason))
                editor.finish_process("error", "A replaced prop could not be checked. Retry undo when the runtime is ready.", "warn")
                return
            end
            if current and prop.removal_pending and editor.transform_matches(current, prop.expected_transform) then
                -- This attempted removal did not take effect. The exact original
                -- still exists, so restoring it would create a duplicate.
                prop.removal_skipped = true
            elseif current then
                editor.set_editor_message("recovery", "Undo paused because a replaced prop is already present. Check the world, then press F4 to retry.")
                runtime.report_effect("write-failed", "a replaced prop reappeared before undo")
                shroudforge.log.warn("World Editor refused undo because a replaced prop is already live again")
                editor.finish_process("error", "A replaced prop is already present. Undo paused to avoid a duplicate.", "warn")
                return
            end
        end
        editor.set_editor_message("placing", editor.undo_state.automaticRollback and "Restoring the world after the failed placement…" or "Undo in progress. Removing placed props and restoring the previous world state…")
        editor.process_step("Removing placed props", 2, 4,
            string.format("Removing or verifying %d placed props.", index))
        while index >= 1 do
            local entity = entities[index]
            if not entity.entityHandle then
                editor.set_editor_message("recovery", "Undo paused because a prop handle is missing. Press F4 to retry recovery.")
                runtime.report_effect("write-failed", "pasted prop has no live entity handle")
                shroudforge.log.warn("World Editor paused undo because live prop state could not be inspected")
                editor.finish_process("error", "A placed prop has no verifiable live handle.", "warn")
                return
            end
            local current, read_reason = editor.read_prop(entity.entityHandle, expected)
            if not current and read_reason ~= "entity-absent" then
                editor.set_editor_message("recovery", "Undo paused because the prop state could not be read. Terrain has been restored. Press F4 to retry the prop.")
                shroudforge.log.warn("World Editor paused prop removal because the live transform read failed: " .. tostring(read_reason))
                editor.finish_process("error", "A placed prop could not be inspected. Retry undo later.", "warn")
                return
            end
            if not current then
                index = index - 1
                editor.undo_state.entity_index = index
                shroudforge.log.debug("World Editor undo accepted an already absent pasted prop handle " .. tostring(entity.entityHandle))
            else
            -- Remove the exact server/local prop handle from the same runtime that
            -- created it; handles never leave this process.
            editor.undo_state.recovery_required = true
            local removed, remove_reason = runtime.world.entity.destroy(entity.entityHandle)
            if runtime.world.session_id() ~= editor.undo_state.session then
                editor.set_editor_message("recovery", "World session changed during removal. Undo progress is retained for inspection."); return
            end
            local remaining, read_reason = editor.read_prop(entity.entityHandle, expected)
            if not remaining and read_reason == "entity-absent" then
                index = index - 1
                editor.undo_state.entity_index = index
            else
                local detail = read_reason or remove_reason or "the exact pasted ECS handle is still present after the destroy call"
                editor.set_editor_message("recovery", "Undo paused because a placed prop could not be removed. " .. tostring(detail) .. ". Press F4 to retry.")
                runtime.report_effect("write-failed", detail)
                shroudforge.log.error("World Editor paused undo and preserved its progress: " .. tostring(detail))
                editor.finish_process("error", "A placed prop could not be removed: " .. tostring(detail), "error")
                return
            end
            end
        end
        if removed_index > 0 then
            editor.process_step("Restoring replaced props", 3, 4,
                string.format("Restoring %d replaced props.", removed_index))
        end
        while removed_index >= 1 do
            editor.process_step("Restoring replaced props", 3, 4,
                string.format("Restoring %d replaced props.", removed_index))
            local prop = editor.undo_state.removed_props[removed_index]
            if prop.removal_skipped then
                local current, read_reason = editor.read_prop(prop.entityHandle, expected)
                if not current and read_reason == "entity-absent" then
                    prop.removal_skipped = nil
                elseif not current or not editor.transform_matches(current, prop.expected_transform) then
                    editor.set_editor_message("recovery", "Undo paused because a pending removal changed. Press F4 to retry.")
                    editor.finish_process("error", "The original prop could not be verified before completing recovery.", "warn")
                    return
                end
            end
            if not prop.removal_skipped then
            if not editor.feature("runtime.world.entity.spawn") then return end
            local entity_handle, spawn_reason = prop.restoreHandle
            if not entity_handle then
                editor.undo_state.recovery_required = true
                entity_handle, spawn_reason = runtime.world.entity.spawn(prop.templateUuidHighHex or prop.recipe.uuidHigh,
                    prop.templateUuidLowHex or prop.recipe.uuidLow,
                    prop.position, prop.rotation, prop.recipe.id, 0)
                prop.restoreHandle = entity_handle
            end
            if entity_handle and not editor.prop_scale_is_unit(prop.scale) then
                editor.undo_state.recovery_required = true
                local scaled, scale_reason = runtime.world.entity.set_scale(entity_handle, prop.scale)
                if not scaled then
                    editor.set_editor_message("recovery", "Undo paused while restoring a replaced prop's scale. Press F4 to retry.")
                    shroudforge.log.error("World Editor paused undo while restoring a replaced prop's scale: " ..
                        tostring(scale_reason or "native scale update failed"))
                    editor.finish_process("error", "A replaced prop's scale could not be restored.", "error")
                    return
                end
            end
            local restored = entity_handle and editor.read_prop(entity_handle, expected)
            if not restored or not editor.transform_matches(restored, prop.expected_transform) then
                editor.set_editor_message("recovery", "Undo paused while restoring a replaced prop. Press F4 to retry.")
                shroudforge.log.error("World Editor paused undo while restoring a replaced target prop: " ..
                    tostring(spawn_reason or "spawn could not be verified by entity handle"))
                editor.finish_process("error", "A replaced prop could not be verified after restoration.", "error")
                return
            end
            prop.restoreHandle = nil
            end
            removed_index = removed_index - 1
            editor.undo_state.removed_prop_index = removed_index
        end
        local had_voxels = editor.undo_state.hasVoxels
        editor.undo_state = nil
        editor.set_editor_message("selected", had_voxels and "Undo completed successfully. The previous terrain and prop state was restored." or
            "Undo completed successfully. The placed props were removed and replaced props were restored.")
        runtime.report_effect("write-confirmed", had_voxels and
            "Restored the previous voxel snapshot and removed/restored verified props" or
            "Removed pasted props and restored replaced props")
        shroudforge.log.info(had_voxels and "World Editor restored the previous voxel snapshot and affected props" or
            "World Editor undid the props-only placement")
        editor.process_step("Undo verified", 4, 4, "The previous live world state has been restored.")
        editor.finish_process("complete", "Undo is confirmed in the live world. Save persistence is not verified.")
    end

    editor.rollback_partial_paste = function()
        if not editor.undo_state then return end
        editor.begin_process("Recover failed placement", "Rolling back completed changes", 4,
            "The paste failed. Restoring the world changes that were completed.")
        shroudforge.log.warn("World Editor is rolling back the changes completed before the paste failure")
        editor.undo_state.automaticRollback = true
        editor.undo_voxels()
        if editor.undo_state then
            editor.undo_state.automaticRollback = nil
            shroudforge.log.error("World Editor rollback is incomplete. Press F4 again after resolving the recovery issue.")
            editor.set_editor_message("recovery", "Rollback is incomplete. Press F4 to continue restoring the previous world state.")
            editor.finish_process("error", "Automatic rollback is incomplete. Press F4 to continue recovery.", "error")
        else
            shroudforge.log.info("World Editor automatically restored the pre-paste snapshot")
            editor.set_editor_message("selected", "Placement failed, and the previous world state was restored.")
            editor.finish_process("complete", "The failed placement was rolled back. The previous live state was restored.")
        end
    end

    function editor.reset_editor()
        editor.cancel_pending_queries("Capture cancelled by editor reset.", nil, "Save cancelled")
        if editor.remote_active_request and editor.remote_active_request.operation == "capture" and editor.p2p_bridge then
            editor.p2p_bridge.cancel_capture()
        elseif editor.abandon_remote and (editor.remote_recovery_required or editor.remote_release_pending or editor.remote_request_pending or
           editor.remote_active_request or (editor.p2p_bridge and editor.p2p_bridge.pending())) then
            editor.abandon_remote()
        end
        editor.selection_a, editor.selection_b, editor.selection_target = nil, nil, nil
        -- Cancelling a capture must not discard the journal of an earlier paste.
        editor.clipboard, editor.active_blueprint_name = nil, nil
        editor.live_prop_cache = {}
        editor.placement_preview = nil
        editor.editor_stage = editor.undo_state and editor.undo_state.recovery_required and "recovery" or "need_a"
        editor.editor_hint = editor.remote_release_pending and
            "F6 asked the dedicated server to abandon the previous operation. The server must confirm before F7 can start. Press F6 again to resend if the reply is lost." or
            editor.remote_recovery_required and
            "The dedicated server has an unfinished operation. Press F6 to abandon it; partial terrain or props may remain. F7 unlocks only after server confirmation." or
            editor.undo_state and editor.undo_state.recovery_required and
            "Editor reset. The incomplete placement recovery is still retained. Press F4 to retry it. You can continue using capture and blueprint tools, but F7 stays paused until recovery completes." or
            "Editor reset. Select the Building Hammer, choose a Single Voxel, aim at the first corner, then press F5."
        editor.publish_editor_state()
        shroudforge.log.info("World Editor reset: capture cancelled and selection cleared, undo history preserved")
    end

    function editor.persist_server_undo_token()
        if not editor.editor_ui_available or not io.export then return false end
        local peer, transaction = "none", "none"
        if editor.remote_undo then
            peer, transaction = tostring(editor.remote_undo.peer or ""), tostring(editor.remote_undo.transaction or "")
            if not editor.valid_peer_id(peer) or not transaction:match("^tx_[%w_-]+$") then
                shroudforge.log.error("World Editor refused to persist an invalid server undo token")
                return false
            end
        end
        local contents = table.concat({editor.server_undo_export_header, peer, transaction, ""}, "\n")
        local ok, reason = pcall(io.export, editor.server_undo_export_path, contents)
        if not ok then
            shroudforge.log.error("World Editor could not persist the server undo token: " .. tostring(reason))
            return false
        end
        return true
    end

    function editor.recover_server_undo_token()
        if not editor.editor_ui_available or not io.read_export_to_string then return false end
        local ok, contents = pcall(io.read_export_to_string, editor.server_undo_export_path)
        if not ok or type(contents) ~= "string" then return false end
        contents = contents:gsub("\r\n", "\n")
        local header, peer, transaction = contents:match("^([^\n]+)\n([^\n]+)\n([^\n]+)\n?$")
        if header ~= editor.server_undo_export_header or peer == "none" or transaction == "none" then return false end
        if not editor.valid_peer_id(peer) or not transaction:match("^tx_[%w_-]+$") then
            shroudforge.log.warn("World Editor ignored an invalid persisted server undo token")
            return false
        end
        editor.remote_undo = {peer = peer, transaction = transaction}
        return true
    end

    editor.undo_remote = function()
        if not editor.remote_undo then
            editor.set_editor_message(nil, "There is no confirmed server placement to undo. Place a blueprint on the server with F7 first.")
            return
        end
        if not editor.p2p_bridge then editor.set_editor_message("recovery", "Steam P2P is unavailable. The server undo token is retained."); return end
        if editor.remote_request_pending or editor.p2p_bridge.pending() then
            editor.set_editor_message("building", "The server is still processing the previous World Editor request. No duplicate was sent.")
            return
        end
        editor.begin_process("Undo placement on dedicated server", "Sending undo request", 0,
            "Asking the dedicated server to restore its saved world state.")
        local ok, request = editor.p2p_bridge.start_undo(editor.remote_undo.peer, editor.remote_undo.transaction)
        if not ok then
            editor.set_editor_message("recovery", tostring(request))
            editor.finish_process("error", "The undo request could not be sent: " .. tostring(request), "warn")
            return
        end
        editor.remote_request_pending = true
        editor.remote_active_request = {peer = editor.remote_undo.peer, request_id = request, operation = "undo"}
        editor.set_editor_message("building", "F4 sent the server-owned undo request. Waiting for its native readback result…")
        shroudforge.log.debug("World Editor queued P2P undo request " .. tostring(request))
        shroudforge.log.info("World Editor: undo request sent to the dedicated server. Waiting for confirmation.")
    end

    function editor.finish_server_remote_undo(job, ok, reason)
        if editor.server_remote_undo ~= job then return end
        if editor.pending_world_action then editor.cancel_pending_queries(reason, "recovery") end
        local retained = editor.server_remote_transactions[job.peer]
        if retained and retained.transaction == job.transaction then
            if ok then editor.server_remote_transactions[job.peer] = nil
            elseif editor.undo_state then retained.undo = editor.undo_state end
        end
        editor.server_remote_transaction, editor.server_remote_undo, editor.undo_state = nil, nil, nil
        editor.remote_progress_callback = nil
        job.done(ok, reason)
    end

    function editor.update_server_remote_undo(delta_seconds)
        local job = editor.server_remote_undo
        if not job then return end
        job.elapsed = job.elapsed + math.max(0, tonumber(delta_seconds) or 0)
        local transaction = editor.server_remote_transaction
        if not transaction or transaction.peer ~= job.peer or transaction.transaction ~= job.transaction or
           transaction.session ~= editor.current_session() then
            editor.finish_process("error", "The server undo token belongs to another or ended world session.", "warn")
            editor.finish_server_remote_undo(job, false, "server-world-session-changed-or-undo-token-expired")
            return
        end
        if not editor.undo_state then
            editor.finish_server_remote_undo(job, true, "undo-already-complete")
            return
        end
        if not job.started then
            -- Use the same guarded restoration and readiness continuation as
            -- local F4, without another pair of non-reserving Scene samples.
            editor.undo_voxels()
            job.started = true
        end
        if not editor.undo_state then
            editor.finish_server_remote_undo(job, true, "undo-confirmed")
        elseif editor.pending_world_action then
            if job.elapsed >= 45 then
                editor.finish_process("error", "The server world context did not become ready for undo.", "warn")
                editor.finish_server_remote_undo(job, false, "server-undo-world-context-did-not-become-ready")
            end
        else
            if editor.process_progress.state == "running" then
                editor.finish_process("error", "Server undo paused: " .. tostring(editor.editor_hint), "warn")
            end
            editor.finish_server_remote_undo(job, false, editor.editor_hint or "server-undo-needs-recovery. Retry F4")
        end
    end
end
