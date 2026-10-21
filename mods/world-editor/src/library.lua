-- Persistent blueprint library and file operations.
return function(editor)
    function editor.finish_save_progress()
        local capture_save = editor.process_progress.operation == "Capture and save blueprint"
        editor.process_step("Refreshing blueprint library", capture_save and 5 or 3,
            capture_save and 5 or 3, "Updating the blueprint list.")
        if editor.refresh_blueprint_library and editor.refresh_blueprint_library() then
            editor.save_step(4, "Blueprint saved", "complete")
            editor.finish_process("complete", "The blueprint is saved and ready to use.")
        else
            editor.save_step(3, "Saved, library refresh failed", "error")
            editor.finish_process("error", "The file was saved, but the blueprint list could not be refreshed.", "warn")
        end
    end

    function editor.export_ready_for_capture(save_and_select)
        if not save_and_select then return true end
        local ok, result = pcall(function()
            return io.export_exists("world-editor/blueprints/capture-1.sfbp")
        end)
        if not ok then
            shroudforge.log.error("World Editor cannot start F8 capture because blueprint export is unavailable: " .. tostring(result))
            return false
        end
        return true
    end

    -- Both local and server captures use one non-overwriting save path.
    function editor.save_capture()
        local name
        for index = 1, 999999 do
            local candidate = "capture-" .. tostring(index)
            local ok, exists = pcall(io.export_exists, "world-editor/blueprints/" .. candidate .. ".sfbp")
            if not ok then
                editor.set_editor_message("ready", "Capture stopped. The blueprint folder could not be checked. Check the export folder and try F8 again.")
                shroudforge.log.error("World Editor cannot safely choose a capture name because export storage could not be inspected: " .. tostring(exists))
                editor.finish_process("error", "Could not inspect the blueprint folder: " .. tostring(exists), "error")
                return false
            end
            if exists == false then name = candidate; break end
        end
        if not name or not editor.save_blueprint_named(name) then
            editor.set_editor_message("ready", "Capture is ready, but the blueprint could not be saved. Check the export folder and try F8 again.")
            shroudforge.log.error("World Editor captured the region but could not save a unique persistent blueprint")
            if editor.process_progress.state == "running" then
                editor.finish_process("error", "The capture is ready, but the blueprint file could not be saved.", "error")
            end
            return false
        end
        editor.active_blueprint_name = name
        editor.editor_stage = "selected"
        editor.set_editor_message("selected", "Blueprint " .. name .. " saved and selected. Press F7 to place it. Open Manage to add a cover from F12 screenshots.")
        editor.finish_save_progress()
        shroudforge.log.debug("World Editor help: blueprint saved as '" .. name .. "' and selected. Click to select or double-click to manage it. Press F7 to place it.")
        return editor.save_progress.state == "complete"
    end

    editor.save_blueprint_named = function(name)
        if not editor.clipboard then
            editor.set_editor_message("need_a", "There is no capture to save. Mark both corners with F5 first.")
            shroudforge.log.warn("World Editor: capture a region before saving a blueprint")
            return
        end
        local prop_count = #(editor.clipboard.props or {})
        local maximum = editor.maximum_copyable_props()
        if prop_count > maximum then
            editor.set_editor_message("ready", string.format("Save stopped. This capture has %d props, above your limit of %d. Raise the limit in mod settings or capture a smaller area.", prop_count, maximum))
            shroudforge.log.warn(string.format("World Editor: cannot save %d props because the configured blueprint maximum is %d", prop_count, maximum))
            return false
        end
        local path, reason = editor.blueprint_path(name)
        if not path then
            editor.set_editor_message("ready", "Save stopped. " .. tostring(reason) .. ". Use letters, numbers, underscores, or hyphens in the blueprint name.")
            shroudforge.log.warn("World Editor: " .. reason)
            return
        end
        if editor.process_progress.state ~= "running" then
            editor.begin_process("Save blueprint", "Encoding blueprint", 3,
                "Preparing " .. tostring(name) .. " for export.")
        end
        local capture_save = editor.process_progress.operation == "Capture and save blueprint"
        editor.process_step("Encoding blueprint", capture_save and 3 or 1, capture_save and 5 or 3,
            "Encoding " .. tostring(name) .. ".")
        -- A direct save already has a complete capture in the clipboard.
        if editor.save_progress.state ~= "saving" then
            editor.editor_stage = "capturing"
            editor.save_step(0, "Preparing blueprint")
        end
        editor.save_step(1, "Encoding blueprint")
        local content, encode_reason = editor.encode_blueprint(editor.clipboard)
        if not content then
            editor.set_editor_message("ready", "Save stopped. " .. tostring(encode_reason))
            shroudforge.log.warn("World Editor: " .. tostring(encode_reason))
            editor.finish_process("error", "Blueprint encoding failed: " .. tostring(encode_reason), "warn")
            return
        end
        editor.process_step("Writing blueprint file", capture_save and 4 or 2, capture_save and 5 or 3,
            string.format("Writing %s (%d bytes).", path, #content))
        editor.save_step(2, "Writing blueprint")
        local ok, err = pcall(io.export, path, content)
        if not ok then
            editor.set_editor_message("ready", "Save failed. Check that export storage is enabled, then try again.")
            shroudforge.log.error("World Editor could not save persistent blueprint (enable export): " .. tostring(err))
            editor.finish_process("error", "Could not write the blueprint file: " .. tostring(err), "error")
            return false
        end
        if type(content) ~= "string" or #content > editor.maximum_blueprint_bytes then
            shroudforge.log.warn("World Editor: blueprint is not text or exceeds the 256 MiB file limit")
            editor.finish_process("error", "The encoded blueprint exceeds the supported file limit.", "warn")
            return false
        end
        editor.save_step(3, "Updating library")
        shroudforge.log.info("World Editor saved persistent blueprint: " .. path)
        editor.process_step("Blueprint file written", capture_save and 4 or 2, capture_save and 5 or 3,
            "The file was written. Refreshing the blueprint library.")
        return true
    end

    function editor.save_blueprint()
        local name = editor.setting("blueprintName")
        if editor.save_blueprint_named(name) then
            editor.active_blueprint_name = name
            editor.set_editor_message("selected", "Blueprint " .. name .. " saved and selected. Press F7 to place it. Open Manage to add a cover from F12 screenshots.")
            editor.finish_save_progress()
        end
    end

    function editor.load_blueprint(name, supplied_content)
        local remote_paste = editor.process_progress.state == "running" and
            editor.process_progress.operation == "Place blueprint from client"
        if remote_paste then
            editor.process_step("Validating uploaded blueprint", 1, 5,
                "Checking the blueprint format and its current placement recipes.")
        else
            editor.begin_process("Load blueprint", "Reading blueprint", 0,
                "Reading and checking the selected blueprint.")
        end
        local ok, loaded = pcall(editor.load_blueprint_content, name, supplied_content)
        if not ok then
            editor.finish_process("error", "Blueprint loading stopped: " .. tostring(loaded), "error")
            return false
        end
        if not loaded then
            editor.finish_process("error", "Blueprint could not be loaded. See the world-editor log for the validation reason.", "warn")
            return false
        end
        if remote_paste then
            editor.process_step("Blueprint validated", 2, 5,
                "Blueprint data and prop recipes are valid on the server.")
        else
            editor.process_step("Blueprint activated", 1, 1,
                "Blueprint is active and ready for preview or placement.")
            editor.finish_process("complete", "Blueprint loaded and activated.")
        end
        return true
    end

    editor.refresh_blueprint_library = function()
        local ok, paths = pcall(io.export_list, "world-editor/blueprints")
        if not ok then
            shroudforge.log.warn("World Editor could not scan the blueprint folder: " .. tostring(paths))
            return false
        end
        local entries = {}
        for _, path in ipairs(paths) do
            local name = path:match("^world%-editor/blueprints/([^/]+)%.sfbp$")
            if name and name:match("^[%w_-]+$") then
                entries[#entries + 1] = {
                    name = name,
                    path = path,
                    image = "world-editor/blueprints/" .. name .. ".png",
                }
            end
        end
        table.sort(entries, function(left, right) return left.name:lower() < right.name:lower() end)
        editor.blueprint_library = entries
        shroudforge.log.debug(string.format("World Editor blueprint library refreshed: %d blueprint(s)", #entries))
        for index, entry in ipairs(entries) do
            shroudforge.log.debug(string.format("World Editor blueprint %d: %s%s", index, entry.name,
                entry.name == editor.active_blueprint_name and " [selected]" or ""))
        end
        return true
    end

    function editor.library_name_setting(key)
        local name = editor.setting(key)
        local path, reason = editor.blueprint_path(name)
        if not path then
            shroudforge.log.warn("World Editor: " .. tostring(reason))
            return nil
        end
        return name, path
    end

    function editor.action_names(value)
        if type(value) == "string" then
            local old_name, new_name = value:match("^([^\t]+)\t([^\t]+)$")
            if old_name and new_name then return old_name, new_name end
        end
        return editor.setting("blueprintName"), editor.setting("blueprintNewName")
    end

    function editor.rename_library_blueprint(value)
        local old_name, new_name = editor.action_names(value)
        local old_path = editor.blueprint_path(old_name)
        local new_path = editor.blueprint_path(new_name)
        if not old_path or not new_path then
            shroudforge.log.warn("World Editor: use blueprint names containing only letters, digits, underscores, and hyphens")
            return
        end
        if not old_name or not new_name then return end
        if old_name == new_name then
            editor.set_editor_message(nil, "Rename stopped. Enter a different blueprint name.")
            shroudforge.log.warn("World Editor: source and new blueprint names are identical")
            return
        end
        if io.export_exists(new_path) then
            editor.set_editor_message(nil, "Rename stopped. A blueprint with that name already exists.")
            shroudforge.log.warn("World Editor: a blueprint with that name already exists")
            return
        end
        local old_image, new_image = "world-editor/blueprints/" .. old_name .. ".png",
            "world-editor/blueprints/" .. new_name .. ".png"
        local old_thumb, new_thumb = "world-editor/blueprints/" .. old_name .. ".thumb.png",
            "world-editor/blueprints/" .. new_name .. ".thumb.png"
        local has_image = io.export_exists(old_image)
        local has_thumb = io.export_exists(old_thumb)
        editor.begin_process("Rename blueprint", "Renaming blueprint file", 3,
            "Renaming the blueprint and its cover files.")
        local ok, reason = pcall(io.export_rename, old_path, new_path)
        if not ok then
            editor.set_editor_message(nil, "Rename failed. Check the export folder and try again.")
            shroudforge.log.error("World Editor could not rename blueprint: " .. tostring(reason))
            editor.finish_process("error", "The blueprint file could not be renamed.", "error")
            return
        end
        editor.process_step("Renaming cover image", 1, 3, "Updating the blueprint cover image.")
        if has_image then
            local image_ok, image_reason = pcall(io.export_rename, old_image, new_image)
            if not image_ok then
                pcall(io.export_rename, new_path, old_path)
                shroudforge.log.error("World Editor could not rename the blueprint image. The rename was rolled back: " .. tostring(image_reason))
                editor.finish_process("error", "The cover image could not be renamed. The blueprint rename was rolled back.", "error")
                return
            end
        end
        editor.process_step("Renaming thumbnail", 2, 3, "Updating the blueprint thumbnail.")
        if has_thumb then
            local thumb_ok, thumb_reason = pcall(io.export_rename, old_thumb, new_thumb)
            if not thumb_ok then
                if has_image then pcall(io.export_rename, new_image, old_image) end
                pcall(io.export_rename, new_path, old_path)
                shroudforge.log.error("World Editor could not rename the blueprint thumbnail. The rename was rolled back: " .. tostring(thumb_reason))
                editor.finish_process("error", "The thumbnail could not be renamed. The blueprint rename was rolled back.", "error")
                return
            end
        end
        if editor.active_blueprint_name == old_name then
            editor.active_blueprint_name = new_name
            editor.publish_editor_state()
        end
        editor.refresh_blueprint_library()
        editor.set_editor_message(nil, "Blueprint renamed to " .. new_name .. ".")
        shroudforge.log.info("World Editor renamed blueprint " .. old_name .. " to " .. new_name)
        editor.process_step("Blueprint library refreshed", 3, 3, "The renamed blueprint is ready in the library.")
        editor.finish_process("complete", "Blueprint renamed to " .. new_name .. ".")
    end

    function editor.duplicate_library_blueprint(value)
        local source_name, new_name = editor.action_names(value)
        local source_path = editor.blueprint_path(source_name)
        local destination = editor.blueprint_path(new_name)
        if not source_path or not destination then
            shroudforge.log.warn("World Editor: use blueprint names containing only letters, digits, underscores, and hyphens")
            return
        end
        if not source_name or not new_name then return end
        if source_name == new_name then
            editor.set_editor_message(nil, "Duplicate stopped. Enter a different name for the copy.")
            shroudforge.log.warn("World Editor: choose a different name for the duplicate")
            return
        end
        if io.export_exists(destination) then
            editor.set_editor_message(nil, "Duplicate stopped. A blueprint with that name already exists.")
            shroudforge.log.warn("World Editor: a blueprint with that name already exists")
            return
        end
        editor.begin_process("Duplicate blueprint", "Copying blueprint file", 3,
            "Copying the blueprint and its cover files.")
        local ok, reason = pcall(io.export_copy, source_path, destination)
        if not ok then
            editor.set_editor_message(nil, "Duplicate failed. Check the export folder and try again.")
            shroudforge.log.error("World Editor could not duplicate blueprint: " .. tostring(reason))
            editor.finish_process("error", "The blueprint file could not be copied.", "error")
            return
        end
        local source_image, destination_image = "world-editor/blueprints/" .. source_name .. ".png",
            "world-editor/blueprints/" .. new_name .. ".png"
        local source_thumb, destination_thumb = "world-editor/blueprints/" .. source_name .. ".thumb.png",
            "world-editor/blueprints/" .. new_name .. ".thumb.png"
        if io.export_exists(source_image) then
            editor.process_step("Copying cover image", 1, 3, "Copying the blueprint cover image.")
            local image_ok, image_reason = pcall(io.export_copy, source_image, destination_image)
            if not image_ok then
                pcall(io.export_delete, destination)
                shroudforge.log.error("World Editor could not duplicate the blueprint image. The duplicate was rolled back: " .. tostring(image_reason))
                editor.finish_process("error", "The cover image could not be copied. The duplicate was rolled back.", "error")
                return
            end
        end
        if io.export_exists(source_thumb) then
            editor.process_step("Copying thumbnail", 2, 3, "Copying the blueprint thumbnail.")
            local thumb_ok, thumb_reason = pcall(io.export_copy, source_thumb, destination_thumb)
            if not thumb_ok then
                pcall(io.export_delete, destination)
                pcall(io.export_delete, destination_image)
                shroudforge.log.error("World Editor could not duplicate the blueprint thumbnail. The duplicate was rolled back: " .. tostring(thumb_reason))
                editor.finish_process("error", "The thumbnail could not be copied. The duplicate was rolled back.", "error")
                return
            end
        end
        editor.refresh_blueprint_library()
        editor.set_editor_message(nil, "Blueprint duplicated as " .. new_name .. ".")
        shroudforge.log.info("World Editor duplicated blueprint " .. source_name .. " as " .. new_name)
        editor.process_step("Blueprint library refreshed", 3, 3, "The duplicate is ready in the library.")
        editor.finish_process("complete", "Blueprint duplicated as " .. new_name .. ".")
    end

    function editor.delete_library_blueprint(name)
        name = name or editor.setting("blueprintName")
        local path = editor.blueprint_path(name)
        if not path then
            editor.set_editor_message(nil, "Delete stopped because the blueprint name is invalid.")
            shroudforge.log.warn("World Editor: invalid blueprint name")
            return
        end
        if not name then return end
        if not io.export_exists(path) then
            editor.set_editor_message(nil, "Delete stopped because this blueprint file no longer exists.")
            shroudforge.log.warn("World Editor: blueprint file does not exist: " .. path)
            return
        end
        local image = "world-editor/blueprints/" .. name .. ".png"
        local thumbnail = "world-editor/blueprints/" .. name .. ".thumb.png"
        editor.begin_process("Delete blueprint", "Deleting blueprint file", 3,
            "Deleting the blueprint and its cover files.")
        local ok, reason = pcall(io.export_delete, path)
        if not ok then
            editor.set_editor_message(nil, "Delete failed. Check the export folder and try again.")
            shroudforge.log.error("World Editor could not delete blueprint: " .. tostring(reason))
            editor.finish_process("error", "The blueprint file could not be deleted.", "error")
            return
        end
        editor.process_step("Deleting cover image", 1, 3, "Removing the blueprint cover image if present.")
        if io.export_exists(image) then
            local image_ok, image_reason = pcall(io.export_delete, image)
            if not image_ok then shroudforge.log.warn("World Editor deleted the blueprint but could not delete its image: " .. tostring(image_reason)) end
        end
        if io.export_exists(thumbnail) then
            editor.process_step("Deleting thumbnail", 2, 3, "Removing the blueprint thumbnail if present.")
            local thumb_ok, thumb_reason = pcall(io.export_delete, thumbnail)
            if not thumb_ok then shroudforge.log.warn("World Editor deleted the blueprint but could not delete its thumbnail: " .. tostring(thumb_reason)) end
        end
        if editor.active_blueprint_name == name then
            editor.clipboard, editor.active_blueprint_name, editor.placement_preview = nil, nil, nil
            editor.editor_stage = "need_a"
            editor.publish_editor_state()
        end
        editor.refresh_blueprint_library()
        editor.set_editor_message(nil, "Blueprint " .. name .. " deleted.")
        shroudforge.log.info("World Editor deleted blueprint " .. name)
        editor.process_step("Blueprint library refreshed", 3, 3, "The blueprint was removed from the library.")
        editor.finish_process("complete", "Blueprint " .. name .. " deleted.")
    end

    function editor.refresh_blueprint_library_with_progress()
        editor.begin_process("Refresh blueprint library", "Reading blueprint files", 1,
            "Refreshing the saved blueprint list.")
        local ok = editor.refresh_blueprint_library()
        if ok then
            editor.process_step("Blueprint library refreshed", 1, 1,
                "The saved blueprint list is up to date.")
            editor.finish_process("complete", "Blueprint library refreshed.")
        else
            editor.finish_process("error", "The blueprint list could not be refreshed. Check the World Editor log.", "warn")
        end
        return ok
    end
end
