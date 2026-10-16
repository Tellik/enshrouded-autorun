"""Local Lua 5.4 syntax/lifecycle fixture, without the game or native provider."""
import ctypes as C
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
package = root / "diagnostic-mods/live-runtime-probe"
manifest = json.loads((package / "mod.json").read_text())
extended = json.loads((package / "extended.mod.json").read_text())
assert manifest["capabilities"] == ["runtime"]
assert extended["enabled"] is True
probe = (package / "src/mod.lua").read_text()
library = Path(r"C:\Users\mail\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin\lua54.dll")
lua = C.CDLL(str(library))
lua.luaL_newstate.restype = C.c_void_p
lua.luaL_openlibs.argtypes = [C.c_void_p]
lua.luaL_loadbufferx.argtypes = [C.c_void_p, C.c_char_p, C.c_size_t, C.c_char_p, C.c_char_p]
lua.luaL_loadbufferx.restype = C.c_int
lua.lua_pcallk.argtypes = [C.c_void_p, C.c_int, C.c_int, C.c_int, C.c_ssize_t, C.c_void_p]
lua.lua_pcallk.restype = C.c_int
lua.lua_tolstring.argtypes = [C.c_void_p, C.c_int, C.POINTER(C.c_size_t)]
lua.lua_tolstring.restype = C.c_char_p
lua.lua_close.argtypes = [C.c_void_p]

bootstrap = r'''
local logs, ready, calls, writes = {}, false, {}, 0
shroudforge={log={info=function(value) logs[#logs+1]=value end}}
local scalar_type={primitive_type='Float32',qualified_name='keen::float32',size=4}
local entries={}
for _,suffix in ipairs({'Observed','Partial','PresenceUnreadable','QueryFailed','Unobserved'}) do
    local name='keen::ecs::Probe'..suffix
    entries[#entries+1]={qualified_name=name,runtime_type={qualified_name=name,size=4,
        primitive_type='Struct',fields={{name='x',type=scalar_type}}}}
end
local resource={source='loaded-runtime',loaded=true,guid='fixture',data={itemId={value=17},equipment={
    gliderConfig={accelerationForward=1,yawAngleSpeed=60},placementAABBmin={x=0},placementAABBmax={x=1}}}}
if FAIL_LAZY then
    resource.data=nil
    setmetatable(resource,{__index=function(_,key) if key=='data' then error('fixture lazy resource decode failed') end end})
end
game={assets={get_loaded_resources_by_type=function() return {resource,resource} end}}
runtime={has=function() return ready end,ecs={},values={}}
runtime.ecs.get_registry=function()
    if not ready then return nil,'fixture world is not ready' end
    return {count=5,entries=entries,source='fixture'}
end
runtime.ecs.get_attributes=function() return {count=307,component_layouts={}} end
runtime.ecs.query=function(...)
    local names={...}; local key=table.concat(names,'|')
    calls[key]=(calls[key] or 0)+1
    if names[1]=='keen::ecs::ProbeQueryFailed' then return nil,'fixture query rejected' end
    if calls[key]<=6 then return nil,'Live ECS query is still scanning. Retry on the next update' end
    if names[1]=='keen::ecs::ProbeUnobserved' then return {} end
    return {10}
end
runtime.ecs.read_attributes=function(_,name)
    if name=='Stamina' then return {values={Stamina=137,Stamina_Max=143,Stamina_Max_Base=110,
        Stamina_Max_Adder=33},component={qualified_name='keen::ecs::Stamina'}} end
    return {values={Health=450,Health_Max=450,Health_Max_Base=150,Health_Max_Adder=300},
        component={qualified_name='keen::ecs::Health'}}
end
runtime.ecs.read_bytes=function(_,name)
    if name=='keen::ecs::ProbePresenceUnreadable' then return nil,'fixture presence has no storage column' end
    return string.char(0,0,0,0)
end
runtime.ecs.read=function() return {x=2} end
runtime.ecs.read_fields=function(_,name)
    if name=='keen::ecs::NetworkPlayerAttributes' then
        return {health=450,healthMax=450,stamina=137,staminaMax=143,movementSprint=9}
    end
    return {x=2}
end
runtime.values.decode=function(name)
    if name=='keen::ecs::ProbePartial' then return nil,'fixture unsupported complete value' end
    return {x=2}
end
runtime.ecs.write=function() writes=writes+1;error('unexpected write') end
runtime.ecs.update_attribute=runtime.ecs.write
runtime.functions={get=function() error('unexpected native function binding') end}
'''

checks = r'''
module.on_load()
assert(next(calls)==nil,'on_load performed a query')
for _=1,3 do module.on_update() end
assert(next(calls)==nil,'unready startup performed a query')
ready=true
for _=1,250 do module.on_update() end
module.on_unload()
local output=table.concat(logs,'\n')
assert(not output:find('UNEXPECTED_ERROR',1,true),output)
assert(output:find('SUMMARY reason=complete',1,true),output)
assert(output:find('observed=1 unobserved=1 presence_but_unreadable=1 typed_partial=1 query_failed=1 untested=0',1,true),output)
assert(output:find('PLAYER_ROOT_OK',1,true),output)
assert(output:find('PLAYER_FIELDS_OK',1,true),output)
if FAIL_LAZY then
    assert(output:find('ASSET_DATA_FAIL',1,true),output)
    assert(not output:find('CORE_READ_OK',1,true),output)
else
    assert(output:find('ASSET_SCAN_DONE listed=2 decoded=2 failed=0',1,true),output)
    assert(output:find('CORE_READ_OK',1,true),output)
end
assert(writes==0,'probe attempted a write')
'''

for fail_lazy in (False, True):
    state = lua.luaL_newstate()
    assert state
    try:
        lua.luaL_openlibs(state)
        source = (f"local FAIL_LAZY={'true' if fail_lazy else 'false'}\n" + bootstrap +
                  "\nlocal module=(function()\n" + probe + "\nend)()\n" + checks).encode()
        status = lua.luaL_loadbufferx(state, source, len(source), b"@live-runtime-probe-fixture", None)
        if not status:
            status = lua.lua_pcallk(state, 0, 0, 0, 0, None)
        if status:
            raise AssertionError(lua.lua_tolstring(state, -1, None).decode(errors="replace"))
        print(f"Lua 5.4 syntax/lifecycle fixture passed, lazy_resource_failure={fail_lazy}, no writes")
    finally:
        lua.lua_close(state)
