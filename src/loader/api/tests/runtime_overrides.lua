local function equal(actual, expected, label)
    assert(actual == expected, label .. ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
end

local fixture_id = 0
local function fixture()
    fixture_id = fixture_id + 1
    local f = {values={first=1, second=4}, writes=0, tracker=overrides.new(), prefix=fixture_id .. "/"}
    function f:adapter(key)
        return {
            key=self.prefix .. key,
            claim={acquire=function() return true end, release=function() end},
            read=function() return {value=self.values[key]} end,
            write=function(snapshot, target)
                equal(self.values[key], snapshot.value, "observed-value guard")
                self.writes = self.writes + 1
                if self.failure then return self.failure(self, key, target) end
                self.values[key] = target
                return true, target
            end,
        }
    end
    function f:apply(key, target)
        self.tracker:apply(key, key, self:adapter(key), function() return target end)
    end
    return f
end

-- Uncertainty discovered during apply also blocks restoration of earlier writes.
for _, failure in ipairs({
    function() error("adapter failed after entering its writer") end,
    function() return true, nil end,
    function() return false, nil, "uncertain native rollback" end,
    function(f, key, target)
        f.values[key] = target + 1
        return true, target + 1
    end,
}) do
    local f = fixture()
    f:apply("first", 2)
    f.failure = failure
    f:apply("second", 5)
    assert(f.tracker.fault, "uncertain or unexpected result must fault the tracker")
    local writes = f.writes
    f.failure = nil
    f.tracker:begin_update()
    f.tracker:restore_disabled(function() return false end)
    f.tracker:restore_all()
    f:apply("first", 3)
    equal(f.writes, writes, "fault prevents later apply and restore writes")
    equal(f.values.first, 2, "earlier confirmed write remains owned for inspection")
    assert(f.tracker:has_ownership(), "fault must retain unresolved ownership")
    assert(f.tracker.waiting_count > 0, "blocked restore must be reported")
end

-- A fault raised while iterating restoration must stop later entries in that loop.
for _, method in ipairs({"restore_all", "restore_disabled"}) do
    local f = fixture()
    f:apply("first", 2)
    f:apply("second", 5)
    f.failure = function() error("restore outcome is uncertain") end
    local writes = f.writes
    f.tracker[method](f.tracker, function() return false end)
    equal(f.writes, writes + 1, "first restore fault stops remaining restore writes")
    assert(f.tracker.owned[f.prefix .. "first"] and f.tracker.owned[f.prefix .. "second"], "fault retains every unresolved baseline")
end

-- False and zero are valid baselines and are not confused with missing values.
for _, baseline in ipairs({false, 0}) do
    local f = fixture()
    f.values.first = baseline
    f:apply("first", 2)
    f.tracker:restore_all()
    equal(f.values.first, baseline, "exact baseline restored")
    equal(f.tracker:has_ownership(), false, "successful restoration releases ownership")
end

-- User calculations can fail before a write. Their freshly acquired claim must
-- still be releasable by the automatic lifecycle cleanup.
for _, phase in ipairs({"desired", "normalize"}) do
    local f = fixture()
    local active = false
    local adapter = f:adapter("first")
    adapter.claim = {
        acquire=function() active = true; return true end,
        release=function() active = false end,
    }
    local desired = function() return 2 end
    if phase == "desired" then desired = function() error("foreign calculation failed") end
    else adapter.normalize = function() error("foreign normalization failed") end end
    pcall(function() f.tracker:apply("first", "setting", adapter, desired) end)
    f.tracker:restore_all()
    equal(f.writes, 0, "failed calculation performs no mutation")
    equal(active, false, "failed calculation releases its fresh claim")
    equal(f.tracker:has_ownership(), false, "failed calculation leaves no unresolved mutation")
end

-- A validated original calculator can reject the requested effect by returning
-- the unchanged value. This is a confirmed no-change, without retained baseline.
do
    local f = fixture()
    local adapter = f:adapter("first")
    adapter.write = function(snapshot)
        return true, snapshot.value, nil, snapshot.value
    end
    f.tracker:begin_update()
    f.tracker:apply(adapter.key, "clamped", adapter, function() return 20 end)
    equal(f.tracker.writes, 0, "original no-change must not be reported as a mutation")
    equal(f.tracker:has_ownership(), false, "unchanged original value has no temporary override")
end
