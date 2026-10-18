-- These targets use the same public checked writers available to every mod.
-- The native claims coordinate independent mods by original storage identity.
return function(runtime, assets)
    local api = runtime.overrides
    -- Only immutable catalog names are reused. Entity/root snapshots and claims
    -- are still acquired afresh by their existing checked paths.
    local attribute_names, attribute_name_count = {}, 0
    local maximum_attribute_names = 512
    local function scalar(value)
        return type(value) == "boolean" or type(value) == "string"
            or (type(value) == "number" and value == value and math.abs(value) < math.huge)
    end

    local function attribute(entity, selector)
        local name = attribute_names[selector]
        if not name then
            local descriptor, reason = runtime.ecs.get_attribute(selector)
            if not descriptor then return nil, reason end
            name = descriptor.name
            if attribute_name_count < maximum_attribute_names then
                attribute_names[selector] = name
                attribute_name_count = attribute_name_count + 1
            end
        end
        local initial, why = runtime.ecs.read_attributes(entity, name)
        if not initial then return nil, why end
        local claim, failure = api.get_component_claim(entity, initial.component.qualified_name)
        if not claim then return nil, failure end
        local function normalize(snapshot, value)
            local candidate = {}
            for key, current in pairs(snapshot.values) do candidate[key] = current end
            candidate[name] = value
            local calculated, error = runtime.ecs.evaluate_attributes(name, candidate)
            if not calculated then return nil, error end
            return calculated.values[name]
        end
        return {
            key="attribute/" .. runtime.world.session_id() .. "/" .. entity .. "/" .. name,
            claim=claim,
            normalize=normalize,
            read=function()
                local snapshot, error = runtime.ecs.read_attributes(entity, name)
                if not snapshot then return nil, error end
                local value = snapshot.values[name]
                if not scalar(value) then return nil, "Attribute is not a finite scalar: " .. name end
                return {value=value, values=snapshot.values}
            end,
            write=function(snapshot, value)
                local canonical, error = normalize(snapshot, value)
                if canonical == nil then return false, nil, error end
                local updated, failure = runtime.ecs.update_attribute(entity, name, value, snapshot.values)
                if not updated then return false, nil, failure end
                return true, updated.values[name], nil, canonical
            end,
        }
    end

    local function view_target(view, path, expected_paths, claim_factory)
        local claim, reason = claim_factory(view)
        if not claim then return nil, reason end
        local paths = {path}
        for _, extra in ipairs(expected_paths or {}) do
            if extra ~= path then paths[#paths + 1] = extra end
        end
        return {
            key="view/" .. view.session_id .. "/" .. view.key .. "/" .. path,
            claim=claim, shared=true,
            normalize=function(_, value) return runtime.values.normalize_field(view.type, path, value) end,
            read=function()
                local fields, failure = view:read_fields(paths)
                if not fields then return nil, failure end
                if not scalar(fields[path]) then return nil, "Field is not a finite scalar: " .. path end
                return {value=fields[path], values=fields}
            end,
            write=function(snapshot, value)
                local expected
                if expected_paths and #expected_paths > 0 then
                    expected = {}
                    for _, extra in ipairs(expected_paths) do expected[extra] = snapshot.values[extra] end
                end
                local ok, failure
                if expected then
                    ok, failure = view:write_field(path, value, snapshot.value, expected)
                else
                    ok, failure = view:write_field(path, value, snapshot.value)
                end
                if not ok then return false, nil, failure end
                -- write_field confirms the canonical value under the engine gate.
                return true, value
            end,
        }
    end

    local function resource(resource, path)
        local claim, reason = api.get_resource_claim(resource)
        if not claim then return nil, reason end
        local guid, part, ty, session = resource.guid, resource.part, resource.type, resource.session_id
        return {
            key="resource/" .. guid .. "/" .. ty.qualified_name .. "/" .. part .. "/" .. path,
            claim=claim, shared=true,
            normalize=function(snapshot, value)
                local bytes, why = runtime.values.encode(snapshot.type, value)
                if not bytes then return nil, why end
                return runtime.values.decode(snapshot.type, bytes)
            end,
            read=function()
                if runtime.world.session_id() ~= session then return nil, "Resource target belongs to another world session" end
                local fresh, why = assets.get_loaded_resource(guid, ty, part)
                if not fresh then return nil, why end
                if fresh.session_id ~= session then return nil, "Resource snapshot belongs to another world session" end
                local value, failure, field_type = assets.read_loaded_field(fresh, path)
                if value == nil then return nil, failure end
                if not scalar(value) then return nil, "Resource field is not a finite scalar: " .. path end
                return {value=value, resource=fresh, type=field_type}
            end,
            write=function(snapshot, value)
                if runtime.world.session_id() ~= session or snapshot.resource.session_id ~= session then
                    return false, nil, "Resource target belongs to another world session"
                end
                local updated, failure = assets.write_loaded_field(snapshot.resource, path, value, snapshot.value)
                if not updated then return false, nil, failure end
                local actual, why = assets.read_loaded_field(updated, path)
                if actual == nil then return false, nil, "Uncertain resource write: " .. tostring(why) end
                return true, actual
            end,
        }
    end

    local function component(entity, ty, path, expected_paths)
        local claim, reason = api.get_component_claim(entity, ty)
        if not claim then return nil, reason end
        local fields = {path}
        for _, extra in ipairs(expected_paths or {}) do
            if extra ~= path then fields[#fields + 1] = extra end
        end
        return {
            key="component/" .. runtime.world.session_id() .. "/" .. entity .. "/" .. tostring(ty) .. "/" .. path,
            claim=claim,
            normalize=function(_, value) return runtime.values.normalize_field(ty, path, value) end,
            read=function()
                local values, why = runtime.ecs.read_fields(entity, ty, fields)
                if not values then return nil, why end
                if not scalar(values[path]) then return nil, "Component field is not a finite scalar: " .. path end
                return {value=values[path], values=values}
            end,
            write=function(snapshot, value)
                local ok, why, confirmed = runtime.ecs.write_fields(entity, ty, {[path]=value}, snapshot.values)
                if not ok then return false, nil, why end
                if not confirmed or confirmed[path] == nil then
                    return false, nil, "Uncertain component write: confirmed scalar snapshot is unavailable"
                end
                return true, confirmed[path], nil, confirmed[path]
            end,
        }
    end

    return {
        attribute=attribute, resource=resource, component=component,
        service=function(view, path, expected_paths)
            return view_target(view, path, expected_paths, api.get_service_claim)
        end,
        template=function(view, path, expected_paths)
            return view_target(view, path, expected_paths, api.get_template_claim)
        end,
    }
end
