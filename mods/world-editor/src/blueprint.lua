-- Blueprint format, rotation and coordinate calculations.
return function(editor)
    function editor.rotation_turns()
        local configured = math.floor(tonumber(editor.setting("rotationQuarterTurns")) or 0) % 4
        if editor.active_rotation_turns == nil or configured ~= editor.observed_rotation_setting then
            editor.active_rotation_turns = configured
            editor.observed_rotation_setting = configured
        end
        return editor.active_rotation_turns
    end

    function editor.rotate_blueprint()
        if not editor.clipboard then
            editor.set_editor_message("need_a", "No blueprint is active. Press F5 to start a capture or select a saved blueprint.")
            shroudforge.log.warn("World Editor: capture or load a blueprint before rotating it")
            return
        end
        editor.active_rotation_turns = (editor.rotation_turns() + 1) % 4
        editor.placement_preview = nil
        local axis = editor.clipboard.region.rotationAxis or editor.setting("rotationAxis")
        if axis ~= "x" and axis ~= "y" and axis ~= "z" then axis = "y" end
        local degrees = editor.active_rotation_turns * 90
        shroudforge.log.info(string.format(
            "World Editor rotation: %d/3 quarter turns (%d degrees) around %s. F3 rotates again, F7 places it, and Preview checks the cursor target.",
            editor.active_rotation_turns, degrees, string.upper(axis)))
        editor.editor_stage = "selected"
        editor.set_editor_message("selected", string.format("Rotation set to %d°. Aim at the target and press F7 to place the blueprint.", degrees))
    end

    function editor.finite_number(value)
        return type(value) == "number" and value == value and value ~= math.huge and value ~= -math.huge
    end

    function editor.prop_scale_is_unit(scale)
        return type(scale) == "table" and scale[1] == 1 and scale[2] == 1 and scale[3] == 1
    end

    function editor.prop_scale_matches(prop, scale)
        local actual = prop and prop.transform and prop.transform.scale
        if not actual or type(scale) ~= "table" then return false end
        for index, axis in ipairs({"x", "y", "z"}) do
            if not editor.finite_number(actual[axis]) or not editor.finite_number(scale[index]) or
               math.abs(actual[axis] - scale[index]) > 1e-7 then return false end
        end
        return true
    end

    function editor.maximum_copyable_props()
        local configured = tonumber(editor.setting("maximumCopyableProps"))
        if not editor.finite_number(configured) then configured = editor.default_maximum_copyable_props end
        return math.min(editor.maximum_supported_props, math.max(1, math.floor(configured)))
    end

    function editor.grid_to_world(index, axis, spec)
        return spec.origin[axis] + index * spec.cellSize[axis]
    end

    function editor.world_to_grid(position, axis, spec)
        return math.floor((position - spec.origin[axis]) / spec.cellSize[axis])
    end

    function editor.rotate_vector_quarters(x, y, z, axis, turns)
        for _ = 1, turns % 4 do
            if axis == "x" then x, y, z = x, -z, y
            elseif axis == "y" then x, y, z = z, y, -x
            else x, y, z = -y, x, z end
        end
        return x, y, z
    end

    function editor.rotated_box_bounds(minimum, maximum, turns, axis)
        local low = {math.huge, math.huge, math.huge}
        local high = {-math.huge, -math.huge, -math.huge}
        for corner = 0, 7 do
            local x = corner % 2 == 0 and minimum[1] or maximum[1]
            local y = math.floor(corner / 2) % 2 == 0 and minimum[2] or maximum[2]
            local z = math.floor(corner / 4) % 2 == 0 and minimum[3] or maximum[3]
            local rx, ry, rz = editor.rotate_vector_quarters(x, y, z, axis, turns)
            low[1], low[2], low[3] = math.min(low[1], rx), math.min(low[2], ry), math.min(low[3], rz)
            high[1], high[2], high[3] = math.max(high[1], rx), math.max(high[2], ry), math.max(high[3], rz)
        end
        return low, high
    end

    function editor.world_position(position)
        if not position then return nil end
        local x, y, z = tonumber(position.x), tonumber(position.y), tonumber(position.z)
        if not editor.finite_number(x) or not editor.finite_number(y) or not editor.finite_number(z) then return nil end
        -- Prop-query/get-transform Lua API records are already decoded to world units.
        return {x = x, y = y, z = z}
    end

    function editor.cursor_world_position(position)
        if not position then return nil end
        local scale = 4294967296
        local x, y, z = tonumber(position.x), tonumber(position.y), tonumber(position.z)
        if not editor.finite_number(x) or not editor.finite_number(y) or not editor.finite_number(z) then return nil end
        -- The cursor hook exposes the raw Q32.32 fields from the game structure.
        return {x = x / scale, y = y / scale, z = z / scale}
    end

    function editor.rotated_blueprint(source, quarter_turns)
        local region = source.region
        local sx, sy, sz = region.sx or 0, region.sy or 0, region.sz or 0
        local axis = region.rotationAxis or editor.setting("rotationAxis")
        if axis ~= "x" and axis ~= "y" and axis ~= "z" then axis = "y" end
        local cells, coverage, props = source.cells, source.coverage, {}
        local extent = {source.extent[1], source.extent[2], source.extent[3]}
        local voxel_offset = source.voxelOffset and {source.voxelOffset[1], source.voxelOffset[2], source.voxelOffset[3]} or nil
        local cell_size = source.hasVoxels and {source.cellSize[1], source.cellSize[2], source.cellSize[3]} or nil
        local voxel_extent = source.hasVoxels and {
            sx * cell_size[1], sy * cell_size[2], sz * cell_size[3]
        } or nil
        for index, prop in ipairs(source.props or {}) do
            props[index] = {
                itemId = prop.itemId, entityHandle = prop.entityHandle,
                templateUuidHighHex = prop.templateUuidHighHex, templateUuidLowHex = prop.templateUuidLowHex,
                x = prop.x, y = prop.y, z = prop.z,
                qx = prop.qx, qy = prop.qy, qz = prop.qz, qw = prop.qw,
                sx = prop.sx, sy = prop.sy, sz = prop.sz,
            }
        end
        local half_sqrt_two = math.sqrt(0.5)
        for _ = 1, quarter_turns % 4 do
            local next_sx, next_sy, next_sz = sx, sy, sz
            if axis == "x" then next_sy, next_sz = sz, sy
            elseif axis == "y" then next_sx, next_sz = sz, sx
            else next_sx, next_sy = sy, sx end
            if source.hasVoxels then
                local next_cells, next_coverage = {}, coverage and {} or nil
                for z = 0, sz - 1 do
                    for y = 0, sy - 1 do
                        for x = 0, sx - 1 do
                            local source_index = x + sx * (y + sy * z) + 1
                            local next_x, next_y, next_z
                            if axis == "x" then next_x, next_y, next_z = x, sz - 1 - z, y
                            elseif axis == "y" then next_x, next_y, next_z = z, y, sx - 1 - x
                            else next_x, next_y, next_z = sy - 1 - y, x, z end
                            local target_index = next_x + next_sx * (next_y + next_sy * next_z) + 1
                            next_cells[target_index] = cells[source_index]
                            if coverage then next_coverage[target_index] = coverage[source_index] end
                        end
                    end
                end
                cells = next_cells
                if coverage then coverage = next_coverage end
            end
            for _, prop in ipairs(props) do
                local x, y, z = prop.x, prop.y, prop.z
                prop.x, prop.y, prop.z = editor.rotate_vector_quarters(x, y, z, axis, 1)
                local qx, qy, qz, qw = prop.qx, prop.qy, prop.qz, prop.qw
                if axis == "x" then
                    prop.qx, prop.qy, prop.qz, prop.qw = half_sqrt_two * (qx + qw),
                        half_sqrt_two * (qy - qz), half_sqrt_two * (qz + qy), half_sqrt_two * (qw - qx)
                elseif axis == "y" then
                    prop.qx, prop.qy, prop.qz, prop.qw = half_sqrt_two * (qx + qz),
                        half_sqrt_two * (qy + qw), half_sqrt_two * (qz - qx), half_sqrt_two * (qw - qy)
                else
                    prop.qx, prop.qy, prop.qz, prop.qw = half_sqrt_two * (qx - qy),
                        half_sqrt_two * (qy + qx), half_sqrt_two * (qz + qw), half_sqrt_two * (qw - qz)
                end
            end
            if voxel_offset then
                local channel_low = voxel_offset
                local channel_high = {channel_low[1] + voxel_extent[1], channel_low[2] + voxel_extent[2], channel_low[3] + voxel_extent[3]}
                voxel_offset = editor.rotated_box_bounds(channel_low, channel_high, 1, axis)
                voxel_extent = {voxel_extent[1], voxel_extent[2], voxel_extent[3]}
                if axis == "x" then voxel_extent[2], voxel_extent[3] = voxel_extent[3], voxel_extent[2]
                elseif axis == "y" then voxel_extent[1], voxel_extent[3] = voxel_extent[3], voxel_extent[1]
                else voxel_extent[1], voxel_extent[2] = voxel_extent[2], voxel_extent[1] end
                if axis == "x" then cell_size[2], cell_size[3] = cell_size[3], cell_size[2]
                elseif axis == "y" then cell_size[1], cell_size[3] = cell_size[3], cell_size[1]
                else cell_size[1], cell_size[2] = cell_size[2], cell_size[1] end
            end
            local bounds_low, bounds_high = editor.rotated_box_bounds({0, 0, 0}, extent, 1, axis)
            extent = {bounds_high[1] - bounds_low[1], bounds_high[2] - bounds_low[2], bounds_high[3] - bounds_low[3]}
            sx, sy, sz = next_sx, next_sy, next_sz
        end
        local bounds_low, bounds_high = editor.rotated_box_bounds({0, 0, 0}, source.extent, quarter_turns, axis)
        return {sx = sx, sy = sy, sz = sz, cells = cells, props = props,
            hasVoxels = source.hasVoxels, coverage = coverage, voxelOffset = voxel_offset,
            extent = extent, cellSize = cell_size, boundsMin = bounds_low, boundsMax = bounds_high}
    end

    function editor.guid_halves(reference)
        local function find_guid(value, depth)
            if depth > 4 or value == nil then return nil end
            if type(value) == "string" then
                local compact = value:gsub("[^%x]", "")
                if #compact == 32 then return compact end
                return nil
            end
            if type(value) ~= "table" then
                local text = tostring(value)
                local compact = text:gsub("[^%x]", "")
                if #compact == 32 then return compact end
                return nil
            end
            for _, key in ipairs({"value", "guid", "id", "reference", "objectId"}) do
                local found = find_guid(value[key], depth + 1)
                if found then return found end
            end
            return nil
        end
        local value = find_guid(reference, 0)
        if not value then return nil end
        -- KFC GUID strings use the Windows GUID field order; the engine spawn ABI
        -- consumes the two qwords in native little-endian memory order.
        local tail = value:sub(17, 32)
        local low = tail:sub(15, 16) .. tail:sub(13, 14) .. tail:sub(11, 12) .. tail:sub(9, 10) ..
            tail:sub(7, 8) .. tail:sub(5, 6) .. tail:sub(3, 4) .. tail:sub(1, 2)
        local high = value:sub(13, 16) .. value:sub(9, 12) .. value:sub(1, 8)
        return high, low
    end

    function editor.item_id(value)
        local kind = type(value)
        if kind == "userdata" then
            value = value.value
        elseif kind == "table" then
            value = value.value or value.id
        end
        return tonumber(value)
    end

    function editor.voxel_region(source, use_current_cursor, cursor_override)
        if not source then return nil, "paste targets use world-space anchors" end
        local grid, grid_reason = editor.get_voxel_grid_spec()
        if not grid then return nil, grid_reason end
        local names = source and {"sourceX", "sourceY", "sourceZ"} or {"targetX", "targetY", "targetZ"}
        local x, y, z
        local sx, sy, sz
        if source and (editor.selection_a or editor.selection_b) then
            if not editor.selection_a or not editor.selection_b then
                return nil, "mark both selection corners at the cursor, or clear the cursor selection"
            end
            local ax, ay, az = editor.world_to_grid(editor.selection_a.x, 1, grid), editor.world_to_grid(editor.selection_a.y, 2, grid), editor.world_to_grid(editor.selection_a.z, 3, grid)
            local bx, by, bz = editor.world_to_grid(editor.selection_b.x, 1, grid), editor.world_to_grid(editor.selection_b.y, 2, grid), editor.world_to_grid(editor.selection_b.z, 3, grid)
            x, y, z = math.min(ax, bx), math.min(ay, by), math.min(az, bz)
            sx, sy, sz = math.abs(bx - ax) + 1, math.abs(by - ay) + 1, math.abs(bz - az) + 1
        else
            local point, cursor_reason
            if not source then
                if use_current_cursor then
                    if cursor_override then point = cursor_override
                    else point, cursor_reason = editor.cursor_point() end
                else point = editor.selection_target end
            end
            if not source and use_current_cursor and not point then
                return nil, cursor_reason or "the live cursor position is unavailable"
            end
            if point then
                x, y, z = editor.world_to_grid(point.x, 1, grid), editor.world_to_grid(point.y, 2, grid), editor.world_to_grid(point.z, 3, grid)
            else
                x, y, z = tonumber(editor.setting(names[1])), tonumber(editor.setting(names[2])), tonumber(editor.setting(names[3]))
            end
        end
        if not sx then sx, sy, sz = tonumber(editor.setting("sizeX")), tonumber(editor.setting("sizeY")), tonumber(editor.setting("sizeZ")) end
        if not x or not y or not z or not sx or not sy or not sz or
           x % 1 ~= 0 or y % 1 ~= 0 or z % 1 ~= 0 or
           sx % 1 ~= 0 or sy % 1 ~= 0 or sz % 1 ~= 0 or
           sx < 1 or sy < 1 or sz < 1 or
           sx * sy * sz > 65536 then
            return nil, "coordinates must be integers and the region must contain at most 65,536 cells"
        end
        return {x = x, y = y, z = z, sx = sx, sy = sy, sz = sz}
    end

    function editor.target_anchor(use_current_cursor, cursor_override)
        if use_current_cursor then
            if cursor_override then return {cursor_override.x, cursor_override.y, cursor_override.z} end
            local point, reason = editor.cursor_point()
            if not point then return nil, reason end
            return {point.x, point.y, point.z}
        end
        if editor.selection_target then return {editor.selection_target.x, editor.selection_target.y, editor.selection_target.z} end
        if editor.clipboard and not editor.clipboard.hasVoxels then
            local x, y, z = tonumber(editor.setting("targetWorldX")), tonumber(editor.setting("targetWorldY")), tonumber(editor.setting("targetWorldZ"))
            if not editor.finite_number(x) or not editor.finite_number(y) or not editor.finite_number(z) then
                return nil, "props-only world target coordinates must be finite numbers"
            end
            return {x, y, z}
        end
        local grid, reason = editor.get_voxel_grid_spec()
        if not grid then return nil, reason end
        local x, y, z = tonumber(editor.setting("targetX")), tonumber(editor.setting("targetY")), tonumber(editor.setting("targetZ"))
        if not x or not y or not z or x % 1 ~= 0 or y % 1 ~= 0 or z % 1 ~= 0 then
            return nil, "manual target cell coordinates must be integers"
        end
        return {editor.grid_to_world(x, 1, grid), editor.grid_to_world(y, 2, grid), editor.grid_to_world(z, 3, grid)}
    end

    function editor.plan_world_bounds(plan, anchor)
        local low, high = plan.boundsMin, plan.boundsMax
        low = {low[1], low[2], low[3]}
        high = {high[1], high[2], high[3]}
        if plan.hasVoxels and plan.voxelOffset then
            local channel_high = {plan.voxelOffset[1] + plan.sx * plan.cellSize[1],
                plan.voxelOffset[2] + plan.sy * plan.cellSize[2], plan.voxelOffset[3] + plan.sz * plan.cellSize[3]}
            for axis = 1, 3 do
                low[axis] = math.min(low[axis], plan.voxelOffset[axis])
                high[axis] = math.max(high[axis], channel_high[axis])
            end
        end
        return {anchor[1] + low[1], anchor[2] + low[2], anchor[3] + low[3],
            anchor[1] + high[1], anchor[2] + high[2], anchor[3] + high[3]}
    end

    function editor.voxel_target_region(plan, anchor)
        if not plan.hasVoxels then return nil end
        local grid, reason = editor.get_voxel_grid_spec()
        if not grid then return nil, reason end
        for axis = 1, 3 do
            if math.abs(plan.cellSize[axis] - grid.cellSize[axis]) > 1e-6 then
                return nil, "blueprint voxel cell size does not match the active world grid"
            end
        end
        local region = {sx = plan.sx, sy = plan.sy, sz = plan.sz}
        local low = {anchor[1] + plan.voxelOffset[1], anchor[2] + plan.voxelOffset[2], anchor[3] + plan.voxelOffset[3]}
        local sizes = {plan.sx, plan.sy, plan.sz}
        for axis = 1, 3 do
            local coordinate = (low[axis] - grid.origin[axis]) / grid.cellSize[axis]
            local nearest = math.floor(coordinate + 0.5)
            if not editor.finite_number(coordinate) or math.abs(coordinate - nearest) > 1e-6 then
                return nil, "paste anchor would place the voxel channel off-grid. Choose a cursor point aligned to the voxel grid"
            end
            region[({"x", "y", "z"})[axis]] = nearest
        end
        region.queryBounds = {low[1], low[2], low[3],
            low[1] + sizes[1] * grid.cellSize[1], low[2] + sizes[2] * grid.cellSize[2], low[3] + sizes[3] * grid.cellSize[3]}
        return region, nil, grid
    end

    function editor.blueprint_path(name)
        name = name or editor.setting("blueprintName")
        if name == "" or not name:match("^[%w_-]+$") then
            return nil, "blueprint name may contain only letters, digits, underscores, and hyphens"
        end
        return "world-editor/blueprints/" .. name .. ".sfbp"
    end

    function editor.encode_blueprint(source)
        if not source then return nil, "no blueprint data is active" end
        local region, values = source.region, source.cells
        local extent = source.extent
        local body = {"SHROUDFORGE_WORLD_BLUEPRINT_V7",
            region.rotationAxis or "y", table.concat(extent, ","), source.hasVoxels and "voxel" or "props",
            source.hasVoxels and table.concat({region.sx, region.sy, region.sz}, ",") or "-",
            source.hasVoxels and table.concat(source.voxelOffset, ",") or "-",
            source.hasVoxels and table.concat(source.cellSize, ",") or "-",
            source.hasVoxels and table.concat(values, ",") or "-",
            source.hasVoxels and table.concat(source.coverage, ",") or "-",
            tostring(#(source.props or {}))}
        for _, prop in ipairs(source.props or {}) do
            body[#body + 1] = table.concat({prop.itemId, prop.x, prop.y, prop.z,
                prop.qx, prop.qy, prop.qz, prop.qw, prop.sx, prop.sy, prop.sz,
                prop.templateUuidHighHex, prop.templateUuidLowHex}, ",")
        end
        local content = table.concat(body, "\n")
        if #content > editor.maximum_blueprint_bytes then
            return nil, "blueprint exceeds the 256 MiB file limit"
        end
        return content
    end

    function editor.load_blueprint_content(name, supplied_content)
        name = name or editor.setting("blueprintName")
        local path, reason = editor.blueprint_path(name)
        if not path then shroudforge.log.warn("World Editor: " .. reason); return end
        local content = supplied_content
        if content == nil then
            local ok, read_content = pcall(io.read_export_to_string, path)
            if not ok then
                shroudforge.log.warn("World Editor could not load blueprint (enable export and verify the file): " .. tostring(read_content))
                return
            end
            content = read_content
        end
        local lines = {}
        for line in (content .. "\n"):gmatch("([^\n]*)\n") do lines[#lines + 1] = line end
        local version = lines[1]
        local rotation_axis, extent_text, kind = lines[2], lines[3], lines[4]
        local dimensions, offset_text, cell_size_text, encoded, coverage_encoded = lines[5], lines[6], lines[7], lines[8], lines[9]
        local has_voxels = kind == "voxel"
        if version ~= "SHROUDFORGE_WORLD_BLUEPRINT_V7" or
           (rotation_axis ~= "x" and rotation_axis ~= "y" and rotation_axis ~= "z") or
           (kind ~= "voxel" and kind ~= "props") or not extent_text or not dimensions or not offset_text or
           not cell_size_text or not encoded or not coverage_encoded then
            shroudforge.log.warn("World Editor: blueprint format is invalid")
            return
        end
        local function parse_triplet(line)
            local a, b, c = line:match("^([^,]+),([^,]+),([^,]+)$")
            a, b, c = tonumber(a), tonumber(b), tonumber(c)
            if not editor.finite_number(a) or not editor.finite_number(b) or not editor.finite_number(c) then return nil end
            return {a, b, c}
        end
        local extent = parse_triplet(extent_text)
        if not extent or extent[1] < 0 or extent[2] < 0 or extent[3] < 0 or
           (kind == "props" and (extent[1] <= 0 or extent[2] <= 0 or extent[3] <= 0)) then
            shroudforge.log.warn("World Editor: blueprint extent is invalid")
            return
        end
        local cells, coverage = {}, {}
        local sx, sy, sz, voxel_offset, cell_size
        if has_voxels then
            local dimensions_values = parse_triplet(dimensions)
            voxel_offset = parse_triplet(offset_text)
            cell_size = parse_triplet(cell_size_text)
            if not dimensions_values or not voxel_offset or not cell_size or
               cell_size[1] <= 0 or cell_size[2] <= 0 or cell_size[3] <= 0 then
                shroudforge.log.warn("World Editor: voxel channel metadata is invalid")
                return
            end
            sx, sy, sz = dimensions_values[1], dimensions_values[2], dimensions_values[3]
            if sx % 1 ~= 0 or sy % 1 ~= 0 or sz % 1 ~= 0 or sx < 1 or sy < 1 or sz < 1 or sx * sy * sz > 65536 then
                shroudforge.log.warn("World Editor: blueprint dimensions exceed supported limits")
                return
            end
            for cell in encoded:gmatch("[^,]+") do
                local value = tonumber(cell)
                if not value or value % 1 ~= 0 or value < 0 or value > 65535 then
                    shroudforge.log.warn("World Editor: blueprint contains an invalid voxel cell")
                    return
                end
                cells[#cells + 1] = value
            end
            if #cells ~= sx * sy * sz then
                shroudforge.log.warn("World Editor: blueprint cell count does not match its dimensions")
                return
            end
            for state in coverage_encoded:gmatch("[^,]+") do
                local value = tonumber(state)
                if not value or value % 1 ~= 0 or value < 0 or value > 2 then
                    shroudforge.log.warn("World Editor: blueprint contains invalid voxel coverage")
                    return
                end
                coverage[#coverage + 1] = value
            end
            if #coverage ~= #cells then
                shroudforge.log.warn("World Editor: blueprint coverage count does not match its dimensions")
                return
            end
            for index, state in ipairs(coverage) do
                if state == 2 and cells[index] == 0 then
                    shroudforge.log.warn("World Editor: occupied blueprint cells must have a non-zero value")
                    return
                elseif state ~= 2 then
                    cells[index] = 0
                end
            end
        elseif dimensions ~= "-" or offset_text ~= "-" or cell_size_text ~= "-" or encoded ~= "-" or coverage_encoded ~= "-" then
            shroudforge.log.warn("World Editor: props-only blueprint has unexpected voxel data")
            return
        end
        local props = {}
        local count_line = 10
        local count = tonumber(lines[count_line])
        local maximum = editor.maximum_copyable_props()
        if not count or count % 1 ~= 0 or count < 0 or count > maximum or count > editor.maximum_supported_props or #lines ~= count_line + count then
            shroudforge.log.warn("World Editor: blueprint prop count is invalid")
            return
        end
        local recipes = count > 0 and editor.resolve_placeable_items() or {}
        for index = 1, count do
            local fields = {}
            local field_count, invalid_field = 0, false
            for field in (lines[index + count_line] .. ","):gmatch("(.-),") do
                field_count = field_count + 1
                if field_count <= 11 then
                    fields[field_count] = tonumber(field)
                    if not editor.finite_number(fields[field_count]) then invalid_field = true end
                else
                    fields[field_count] = field
                end
            end
            if field_count ~= 13 or invalid_field or not fields[1] or fields[1] < 1 or
               fields[1] % 1 ~= 0 or not recipes or not recipes[fields[1]] then
                shroudforge.log.warn("World Editor: blueprint references an item without a current placement recipe")
                return
            end
            for field = 2, 11 do
                if not editor.finite_number(fields[field]) then
                    shroudforge.log.warn("World Editor: blueprint has invalid prop transform data")
                    return
                end
            end
            local rotation_norm = fields[5]^2 + fields[6]^2 + fields[7]^2 + fields[8]^2
            if not editor.finite_number(rotation_norm) or rotation_norm < 1e-12 then
                shroudforge.log.warn("World Editor: blueprint contains a zero-length prop rotation")
                return
            end
            local template_high, template_low = fields[12], fields[13]
            if type(template_high) ~= "string" or type(template_low) ~= "string" or
               not template_high:match("^%x+$") or not template_low:match("^%x+$") or
               #template_high > 16 or #template_low > 16 or
               (template_high:match("^0+$") and template_low:match("^0+$")) then
                shroudforge.log.warn("World Editor: blueprint prop has an invalid native template UUID")
                return
            end
            template_high, template_low = template_high:lower(), template_low:lower()
            props[#props + 1] = {itemId = fields[1], templateUuidHighHex = template_high,
                templateUuidLowHex = template_low, x = fields[2], y = fields[3], z = fields[4],
                qx = fields[5], qy = fields[6], qz = fields[7], qw = fields[8],
                sx = fields[9], sy = fields[10], sz = fields[11]}
        end
        editor.clipboard = {region = {sx = sx, sy = sy, sz = sz, rotationAxis = rotation_axis},
            cells = has_voxels and cells or nil, props = props, hasVoxels = has_voxels,
            coverage = has_voxels and coverage or nil,
            extent = extent, voxelOffset = voxel_offset,
            cellSize = cell_size}
        editor.selection_a, editor.selection_b = nil, nil
        editor.placement_preview = nil
        editor.active_blueprint_name = name
        editor.editor_stage = "selected"
        editor.set_editor_message("selected", "Blueprint " .. name .. " loaded and selected. Press F7 to place it. Open Manage to add a cover from F12 screenshots.")
        shroudforge.log.info(string.format("World Editor loaded blueprint '%s' (%d cells, %d props). It is selected and ready for F7 placement.", editor.active_blueprint_name, #cells, #props))
        return true
    end
end
