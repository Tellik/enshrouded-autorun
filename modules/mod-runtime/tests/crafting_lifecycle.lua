-- Exercise the shipped callbacks and override adapters against loaded-resource
-- snapshots. Native engine timing is verified separately in the running game.
local second = 1000000000
local function equal(actual, expected, label)
    assert(actual == expected, label .. ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
end

local function fixture(factor)
    local f = {
        session=41, factor=factor, writes=0, original={0}, values={0},
        settings={productionTimeMode="original", workshopTimeFactorPercent=100, infiniteItemUse=false},
    }
    for index = 1, 18 do
        f.original[index + 1], f.values[index + 1] = index * 10 * second, index * 10 * second
    end
    local ty = {qualified_name="keen::RecipeRegistryResource"}
    local function snapshot()
        local recipes = {}
        for index, value in ipairs(f.values) do recipes[index] = {craftingDuration={value=value}} end
        return {guid="test-recipes", part=0, type=ty, session_id=f.session, data={recipes=recipes}}
    end
    local function ordinal(path)
        return assert(tonumber(path:match("^recipes%[(%d+)%]%.craftingDuration%.value$")), "unexpected field path")
    end
    game = {assets={
        get_loaded_resources_by_type=function(name)
            equal(name, ty.qualified_name, "loaded recipe type")
            return {snapshot()}
        end,
        get_loaded_resource=function(guid, name, part)
            equal(guid, "test-recipes", "resource GUID")
            equal(type(name) == "table" and name.qualified_name or name, ty.qualified_name, "resource type")
            equal(part, 0, "resource part")
            return snapshot()
        end,
        read_loaded_field=function(resource, path)
            return resource.data.recipes[ordinal(path)].craftingDuration.value, nil, "i64"
        end,
        write_loaded_field=function(resource, path, desired, expected)
            local index = ordinal(path)
            equal(resource.session_id, f.session, "write session")
            equal(f.values[index], expected, "guarded live value")
            equal(resource.data.recipes[index].craftingDuration.value, expected, "immutable snapshot")
            assert(desired > 0 and desired == math.floor(desired), "positive integer nanoseconds required")
            f.values[index], f.writes = desired, f.writes + 1
            return snapshot()
        end,
    }}
    shroudforge = {settings={get=function(key, fallback)
        local value = f.settings[key]
        if value == nil then return fallback end
        return value
    end}}
    local api = assert(load(overrides_source, "runtime.overrides"))()
    local new_tracker = api.new
    api.new = function()
        f.tracker = new_tracker(function() return f.session end)
        return f.tracker
    end
    api.get_resource_claim = function()
        return {acquire=function() return true end, release=function() end}
    end
    runtime = {
        phase="ingame", overrides=api,
        has=function() return true end,
        world={session_id=function() return f.session end},
        report_effect=function(state, detail) f.state, f.detail = state, detail end,
        values={
            encode=function(_, value) return string.pack("<i8", value) end,
            decode=function(_, bytes) return (string.unpack("<i8", bytes)) end,
        },
        services={get=function(name)
            equal(name, "ecs::GameSettings", "world service")
            return {
                read_fields=function() return {factoryProductionSpeedFactor=f.factor} end,
                write_field=function() error("the world factor must remain unchanged") end,
            }
        end},
    }
    api.resource = override_targets_factory(runtime, game.assets).resource
    f.mod = assert(load(crafting_source, "sf-crafting-essentials"))()
    f.mod.on_load(0.1)
    function f:settle()
        for _ = 1, 6 do self.mod.on_update(0.1) end
        assert(not self.tracker.fault, self.tracker.fault)
        equal(self.tracker.waiting_count, 0, self.detail or "no pending writes")
    end
    function f:expect(ratio)
        for index, original in ipairs(self.original) do
            equal(self.values[index], math.floor(original * ratio + 0.5), "recipe " .. index)
        end
    end
    return f
end

for _, factor in ipairs({1, 0.5, 2}) do
    local f = fixture(factor)
    f:settle()
    f:expect(1)
    equal(f.writes, 0, "100 percent performs no initial writes")
    f.settings.workshopTimeFactorPercent = 0
    f:settle()
    f:expect(factor / 10)
    equal(f.values[2] / second / factor, 1, "shortest actual recipe at zero percent")
    local writes = f.writes
    f:settle()
    equal(f.writes, writes, "scaling does not compound")
    f.settings.workshopTimeFactorPercent = 50
    f:settle()
    f:expect(0.5 + 0.5 * factor / 10)
    f.settings.workshopTimeFactorPercent = 100
    f:settle()
    f:expect(1)
    f.settings.workshopTimeFactorPercent = 0
    f:settle()
    f.factor = factor * 2
    f:settle()
    f:expect(f.factor / 10)
    equal(f.values[2] / second / f.factor, 1, "live world-factor change")
    f.settings.productionTimeMode = "fixed"
    f:settle()
    f:expect(1)
    f.settings.productionTimeMode = "original"
    f:settle()
    f:expect(f.factor / 10)
    f.mod.on_unload()
    f:expect(1)
    equal(f.tracker:has_ownership(), false, "unload releases restored fields")
end
