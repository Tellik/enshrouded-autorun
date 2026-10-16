"""Private Lua 5.4 lifecycle fixtures. No game process is opened.

The mock uses the source-checked production Enum shape: named string values.
It does not replace an execution of the production Rust codec or a live probe.
"""
import ctypes as C
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
package = root / "diagnostic-mods/live-engine-flight-probe"
assert json.loads((package / "mod.json").read_text())["capabilities"] == ["runtime"]
assert json.loads((package / "extended.mod.json").read_text())["targets"] == ["client"]
probe = (package / "src/mod.lua").read_text(encoding="utf-8")
converter = (root / "src/loader/api/src/env/game/assets/value/converter.rs").read_text()
validator = (root / "src/loader/api/src/env/game/assets/value/validator.rs").read_text()
assert "MappedValue::Enum(v) => v" in converter and ".name()" in converter
assert 'Ok(Value::String(field.name.clone()))' in converter
assert 'r#type.enum_fields.get(&s.to_string_lossy()).is_some()' in validator
assert 'return Ok(LuaValue::String(s.clone()))' in validator
assert 'state=target}, {state=expected}' in probe
assert not any(token in probe for token in ('"velocity"', '"transform"', 'state=11'))

lua = C.CDLL(r"C:\Users\mail\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin\lua54.dll")
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
local logs, ready, queries, attempts, successes, physical_writes = {},false,0,0,0,0
local session_id, handle, current_tick = 17,55,0
local state, gravity = 'Ground',true
local apply_attempts, restore_attempts = 0,0
local pending_gravity, stale = nil,false
local enum={qualified_name='keen::ecs::LocomotionState',primitive_type='Enum',enum_fields={
 Ground={name='Ground',value=1},Falling={name='Falling',value=2},
 BuildingFlying={name='BuildingFlying',value=11}}}
local function has(marker)
 for _,line in ipairs(logs) do if line:find(marker,1,true) then return true end end
 return false
end
local function engine_update()
 if pending_gravity then
  pending_gravity.remaining=pending_gravity.remaining-1
  if pending_gravity.remaining<=0 then gravity=pending_gravity.value;pending_gravity=nil end
 end
 if SCENARIO=='gravity_flapping' and successes==1 then gravity=current_tick%2==0 end
end
shroudforge={log={info=function(value) logs[#logs+1]=value end}}
runtime={ecs={},world={},types={},has=function(name)
 if not ready then return false end
 return not(SCENARIO=='write_gate_unavailable' and name=='runtime.ecs.write')
end}
runtime.types.get=function(name)
 assert(name=='keen::ecs::LocomotionState')
 if SCENARIO=='enum_missing' then return nil end
 return enum
end
runtime.world.session_id=function() return ready and session_id or 0 end
runtime.ecs.query=function(...)
 queries=queries+1
 local names={...}
 assert(#names==3 and names[1]=='keen::ecs::PlayerInput' and names[2]=='keen::ecs::DynamicLocomotion' and names[3]=='keen::ecs::Gravity')
 if queries<=3 then return nil,'Live ECS query is still scanning. Retry on the next update' end
 if SCENARIO=='multiple_players' then return {handle,77} end
 return {handle}
end
runtime.ecs.read_fields=function(entity,component,fields)
 assert(entity==handle,'probe changed its bound opaque handle')
 if stale then return nil,'entity generation changed, handle is stale' end
 assert(#fields==1)
 if component=='keen::ecs::DynamicLocomotion' then
  assert(fields[1]=='state')
  if SCENARIO=='bad_enum_read' then return {state=11} end
  return {state=state}
 end
 assert(component=='keen::ecs::Gravity' and fields[1]=='isActive')
 return {isActive=gravity}
end
runtime.ecs.write_fields=function(entity,component,changes,expected)
 assert(entity==handle and component=='keen::ecs::DynamicLocomotion')
 assert(type(changes)=='table' and type(changes.state)=='string' and enum.enum_fields[changes.state])
 assert(type(expected)=='table' and type(expected.state)=='string' and enum.enum_fields[expected.state])
 local count=0 for _ in pairs(changes) do count=count+1 end assert(count==1,'non-state field write')
 count=0 for _ in pairs(expected) do count=count+1 end assert(count==1,'unexpected expected-field scope')
 attempts=attempts+1
 assert(successes<2,'more than two successful writes requested')
 local applying=changes.state=='BuildingFlying'
 if applying then
  apply_attempts=apply_attempts+1
  assert(changes.state=='BuildingFlying' and expected.state=='Ground')
  if SCENARIO=='throw_apply' then error('private throwing write') end
  if SCENARIO=='race_apply' and apply_attempts==1 then state='Falling' end
  if SCENARIO=='normal' and apply_attempts<=2 then return false,'ECS snapshot changed before compare_exchange' end
 else
  restore_attempts=restore_attempts+1
  assert(changes.state=='Ground' and expected.state=='BuildingFlying')
  if SCENARIO=='race_restore' and restore_attempts==1 then
   state='Falling';pending_gravity={remaining=2,value=true}
  end
  if SCENARIO=='cleanup_busy' or (SCENARIO=='normal' and restore_attempts<=2) then return false,'engine scene guard is busy' end
 end
 -- Model the new fourth argument in the same snapshot consumed by native CAS.
 if state~=expected.state then return false,'expected field differs from current component snapshot: state' end
 state=changes.state
 physical_writes=physical_writes+1
 if (SCENARIO=='uncertain_apply' and applying) or (SCENARIO=='uncertain_restore' and not applying) then
  return false,'ECS write outcome is uncertain because rollback could not be verified'
 end
 successes=successes+1
 if not((SCENARIO=='gravity_apply_timeout' or SCENARIO=='gravity_flapping') and applying) and
  not(SCENARIO=='gravity_restore_timeout' and not applying) then
  pending_gravity={remaining=3,value=not applying}
 end
 return true
end
runtime.ecs.write=function() error('raw component writes forbidden') end
runtime.ecs.write_attribute_storage=runtime.ecs.write
runtime.ecs.update_attribute=runtime.ecs.write
runtime.functions={get=function() error('native function calls forbidden') end}
'''

checks = r'''
module.on_load()
assert(queries==0 and attempts==0,'startup invoked ECS')
for _=1,5 do module.on_update() end
assert(queries==0 and attempts==0,'unready startup queried or wrote')
ready=true
if SCENARIO=='initially_inactive' or SCENARIO=='always_inactive' then gravity=false end
if SCENARIO=='already_flying' then state='BuildingFlying';gravity=false end
local injected=false
for tick=1,2800 do
 current_tick=tick
 engine_update()
 module.on_update()
 if SCENARIO=='initially_inactive' and tick==30 then
  assert(attempts==0,'probe wrote while baseline gravity was inactive')
  gravity=true
 end
 if SCENARIO=='external_before_apply' and not injected and has('BASELINE state=') then
  state='Falling';injected=true
 end
 if successes==1 and not injected then
  if SCENARIO=='world_change' then session_id=18;injected=true
  elseif SCENARIO=='generation_changed' then stale=true;injected=true
  elseif SCENARIO=='external_after_apply' then
   state='Falling';pending_gravity={remaining=2,value=true};injected=true
  elseif SCENARIO=='unload_after_apply' then break end
 end
 if SCENARIO=='external_before_restore' and not injected and has('ENGINE_GRAVITY_APPLY_OBSERVED') then
  state='Falling';pending_gravity={remaining=2,value=true};injected=true
 end
 if SCENARIO=='external_reactivation' and successes==2 and not injected then
  state='BuildingFlying';gravity=false;pending_gravity=nil;injected=true
 end
 if has('SUMMARY') then break end
end
module.on_unload()
local output=table.concat(logs,'\n')
assert(not has('UNEXPECTED_ERROR'),output)
assert(successes<=2 and physical_writes<=2,'successful write limit exceeded')
if SCENARIO=='normal' or SCENARIO=='initially_inactive' then
 assert(has('result=COMPLETE') and successes==2 and state=='Ground' and gravity,output)
 assert(has('ENGINE_SAMPLE state=BuildingFlying gravity_active=false consecutive=3'),output)
 assert(has('RESTORE_SAMPLE state=Ground gravity_active=true consecutive=3'),output)
 if SCENARIO=='normal' then assert(attempts==6,output) end
elseif SCENARIO=='always_inactive' or SCENARIO=='already_flying' then
 assert(has('result=BASELINE_TIMEOUT') and attempts==0 and physical_writes==0,output)
elseif SCENARIO=='multiple_players' or SCENARIO=='write_gate_unavailable' or SCENARIO=='enum_missing' then
 assert(has('result=STARTUP_TIMEOUT') and attempts==0,output)
elseif SCENARIO=='bad_enum_read' then
 assert(has('result=BASELINE_REJECTED') and attempts==0,output)
elseif SCENARIO=='external_before_apply' or SCENARIO=='race_apply' then
 assert(has('result=APPLY_SCOPE_CHANGED') and successes==0 and physical_writes==0 and state=='Falling',output)
elseif SCENARIO=='world_change' then
 assert(has('result=SESSION_CHANGED') and successes==1 and attempts==1,output)
elseif SCENARIO=='generation_changed' then
 assert(has('result=CLEANUP_TIMEOUT') and successes==1 and attempts==1 and has('cleanup_required=true'),output)
elseif SCENARIO=='external_after_apply' or SCENARIO=='external_before_restore' or SCENARIO=='race_restore' then
 assert(has('result=EXTERNAL_STATE_PRESERVED') and successes==1 and state=='Falling' and gravity,output)
 assert(not has('RESTORE_APPLIED'),output)
elseif SCENARIO=='uncertain_apply' or SCENARIO=='throw_apply' then
 assert(has('result=OUTCOME_UNCERTAIN') and has('cleanup_required=true') and attempts==1 and successes==0,output)
elseif SCENARIO=='uncertain_restore' then
 assert(has('result=OUTCOME_UNCERTAIN') and has('cleanup_required=true') and attempts==2 and successes==1 and physical_writes==2,output)
elseif SCENARIO=='unload_after_apply' then
 assert(has('UNLOAD_CLEANUP') and successes==2 and state=='Ground' and has('result=UNLOADED'),output)
 assert(not has('result=COMPLETE'),output)
elseif SCENARIO=='cleanup_busy' then
 assert(has('result=CLEANUP_TIMEOUT') and has('cleanup_required=true') and successes==1,output)
elseif SCENARIO=='gravity_apply_timeout' or SCENARIO=='gravity_flapping' then
 assert(has('result=RESTORED_ENGINE_APPLY_UNCONFIRMED') and successes==2 and state=='Ground' and gravity,output)
elseif SCENARIO=='gravity_restore_timeout' then
 assert(has('result=RESTORED_ENGINE_GRAVITY_UNCONFIRMED') and successes==2 and state=='Ground' and not gravity,output)
elseif SCENARIO=='external_reactivation' then
 assert(has('result=EXTERNAL_REACTIVATION') and successes==2 and attempts==2 and state=='BuildingFlying',output)
else error('unknown scenario') end
'''

scenarios = (
    "normal", "initially_inactive", "always_inactive", "already_flying", "multiple_players",
    "write_gate_unavailable", "enum_missing", "bad_enum_read", "external_before_apply",
    "race_apply", "world_change", "generation_changed", "external_after_apply",
    "external_before_restore", "race_restore", "uncertain_apply", "throw_apply",
    "uncertain_restore", "unload_after_apply", "cleanup_busy", "gravity_apply_timeout",
    "gravity_flapping", "gravity_restore_timeout", "external_reactivation",
)
for scenario in scenarios:
    vm = lua.luaL_newstate()
    assert vm
    try:
        lua.luaL_openlibs(vm)
        source = (f"local SCENARIO='{scenario}'\n" + bootstrap + "\nlocal module=(function()\n" + probe + "\nend)()\n" + checks).encode()
        status = lua.luaL_loadbufferx(vm, source, len(source), b"@live-engine-flight-probe-fixture", None)
        if not status:
            status = lua.lua_pcallk(vm, 0, 0, 0, 0, None)
        if status:
            raise AssertionError(scenario + ": " + lua.lua_tolstring(vm, -1, None).decode(errors="replace"))
        print("Lua 5.4 engine-state lifecycle passed: " + scenario)
    finally:
        lua.lua_close(vm)
print(f"{len(scenarios)} private scenarios passed, no game process/native calls. Enum shape source-checked.")
