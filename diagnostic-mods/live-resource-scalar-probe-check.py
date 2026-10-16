"""Private Lua 5.4 fixtures for the loaded ResourceService scalar probe.
Uses immutable owned snapshots and a separate fresh Resource getter.
No game process or production native API is opened.
"""
import ctypes as C
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
package = root / "diagnostic-mods/live-resource-scalar-probe"
assert json.loads((package / "mod.json").read_text())["capabilities"] == ["runtime"]
assert json.loads((package / "extended.mod.json").read_text())["targets"] == ["client"]
probe = (package / "src/mod.lua").read_text(encoding="utf-8")
assert 'local PATH = "equipment.gliderConfig.yawAngleSpeed"' in probe
assert not any(token in probe for token in ("slot=35", "slot == 35", "runtime.functions", "runtime.ecs.write"))
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
local logs, ready, attempts, successes, physical_writes = {},false,0,0,0
local gets, inventories, snapshots, write_result_reads, read_tick = 0,0,0,0,0
local session_id, yaw, item, slot = 17,60,123,'Glider'
local second_yaw, apply_attempts, restore_attempts, read_failures = 70,0,0,0
local resource_missing, fault, injected = false,false,false
local selected_guid, selected_part = '11111111-1111-1111-1111-111111111111',2
local second_guid = '99999999-9999-9999-9999-999999999999'
local issued = setmetatable({}, {__mode='k'})
local function has(marker)
 for _,line in ipairs(logs) do if line:find(marker,1,true) then return true end end
 return false
end
local function readonly(values)
 return setmetatable({}, {__index=values,__newindex=function() error('snapshot mutation forbidden') end})
end
local function resource(guid,part,value,actual_slot,actual_item,source,writer_result)
 snapshots=snapshots+1
 local data=readonly({itemId=readonly({value=actual_item}),equipment=readonly({slot=actual_slot,
  gliderConfig=readonly({yawAngleSpeed=value,accelerationForward=1,airResistanceVertical=1.6})})})
 local values={guid=guid,part=part,source=source or 'loaded-runtime',loaded=(source or 'loaded-runtime')=='loaded-runtime',
  type=readonly({qualified_name='keen::ItemInfo'})}
 local obj=setmetatable({}, {__index=function(_,key)
  if key=='data' then
   if writer_result then write_result_reads=write_result_reads+1;error('writer result is not an independent readback') end
   return data
  end
  return values[key]
 end,__newindex=function() error('resource mutation forbidden') end})
 issued[obj]={guid=guid,part=part,snapshot_yaw=value,writer_result=writer_result}
 return obj
end
shroudforge={log={info=function(value) logs[#logs+1]=value end}}
runtime={world={},types={},has=function(name)
 if not ready or fault then return false end
 return not(SCENARIO=='write_gate_unavailable' and name=='runtime.assets.write')
end}
runtime.world.session_id=function() return ready and session_id or 0 end
runtime.types.get=function(name)
 assert(name=='keen::EquipmentSlotType')
 if SCENARIO=='enum_missing' then return nil end
 return readonly({qualified_name=name,primitive_type='Enum',enum_fields=readonly({
  Glider=readonly({name='Glider',value=35}),None=readonly({name='None',value=0}),
  Helmet=readonly({name='Helmet',value=1})})})
end
game={assets={}}
game.assets.get_loaded_resources_by_type=function(name)
 assert(ready and name=='keen::ItemInfo')
 inventories=inventories+1
 local rows={}
 -- Several batches of ordinary ItemInfo records still contain glider defaults.
 for i=1,65 do
  rows[#rows+1]=resource(string.format('ordinary-%03d',i),0,999,'Helmet',1000+i)
 end
 local first_slot=SCENARIO=='no_gliders' and 'Helmet' or slot
 if SCENARIO=='bad_slot_read' then first_slot=35 end
 rows[#rows+1]=resource(second_guid,0,second_yaw,SCENARIO=='no_gliders' and 'None' or 'Glider',999)
 rows[#rows+1]=resource(selected_guid,selected_part,yaw,first_slot,item,SCENARIO=='bad_source' and 'kfc-file' or nil)
 if SCENARIO=='duplicate_resource' then rows[#rows+1]=resource(selected_guid,selected_part,yaw,first_slot,item) end
 if SCENARIO=='empty_inventory' then return {} end
 return rows
end
game.assets.get_loaded_resource=function(guid,name,part)
 assert(ready and guid==selected_guid and name=='keen::ItemInfo' and part==selected_part)
 gets=gets+1
 if resource_missing then return nil,'selected ResourceId is no longer loaded' end
 if SCENARIO=='normal' and successes==1 and read_failures<2 then
  read_failures=read_failures+1
  return nil,'resource snapshot is temporarily busy'
 end
 if SCENARIO=='apply_read_timeout' and successes==1 and not has('PHASE restore') then return nil,'temporary read failure' end
 if SCENARIO=='restore_read_timeout' and successes==2 then return nil,'temporary read failure' end
 return resource(selected_guid,selected_part,yaw,slot,item)
end
game.assets.write_loaded_field=function(input,path,target,expected)
 assert(ready and not fault,'write called after native fault')
 local identity=issued[input]
 assert(identity and identity.guid==selected_guid and identity.part==selected_part)
 assert(path=='equipment.gliderConfig.yawAngleSpeed')
 assert(type(target)=='number' and type(expected)=='number')
 attempts=attempts+1
 assert(successes<2,'more than two successful writes requested')
 local applying=target==61
 if applying then
  apply_attempts=apply_attempts+1
  assert(target==61 and expected==60)
  if SCENARIO=='throw_apply' then error('private throwing write') end
  if SCENARIO=='race_apply' and apply_attempts==1 then yaw=90 end
  if SCENARIO=='normal' and apply_attempts<=2 then return nil,'Resource scalar snapshot changed before update. Read current values and retry' end
 else
  restore_attempts=restore_attempts+1
  assert(target==60 and expected==61)
  if SCENARIO=='race_restore' and restore_attempts==1 then yaw=90 end
  if SCENARIO=='cleanup_busy' or (SCENARIO=='normal' and restore_attempts<=2) then return nil,'resource lifetime gate is busy' end
 end
 if yaw~=expected then return nil,'Resource scalar snapshot changed before update. Read current values and retry' end
 yaw=target
 physical_writes=physical_writes+1
 if (SCENARIO=='uncertain_apply' and applying) or (SCENARIO=='uncertain_restore' and not applying) then
  fault=true
  return nil,'Resource write outcome is uncertain because lifetime or original data-page protection could not be verified. Do not retry blindly'
 end
 successes=successes+1
 return resource(selected_guid,selected_part,yaw,slot,item,nil,true)
end
game.assets.get_resource=function() error('KFC file fallback forbidden') end
game.assets.get_resources_by_type=game.assets.get_resource
runtime.ecs={write=function() error('ECS writes forbidden') end}
runtime.functions={get=function() error('native function calls forbidden') end}
'''

checks = r'''
module.on_load()
assert(inventories==0 and gets==0 and attempts==0,'startup invoked resource APIs')
for _=1,5 do module.on_update() end
assert(inventories==0 and gets==0 and attempts==0,'unready startup invoked resource APIs')
ready=true
if SCENARIO=='bad_baseline' then yaw=1/0 end
for tick=1,2800 do
 read_tick=tick
 module.on_update()
 if SCENARIO=='world_change_during_inventory' and inventories==1 and not injected then session_id=18;injected=true end
 if SCENARIO=='external_before_apply' and has('BASELINE path=') and not injected then yaw=90;injected=true end
 if successes==1 and not injected then
  if SCENARIO=='world_change' then session_id=18;injected=true
  elseif SCENARIO=='external_after_apply' then yaw=90;injected=true
  elseif SCENARIO=='scope_item_changed' then item=999;injected=true
  elseif SCENARIO=='scope_slot_changed' then slot='Helmet';injected=true
  elseif SCENARIO=='resource_unloaded' then resource_missing=true;injected=true
  elseif SCENARIO=='unload_after_apply' then break end
 end
 if has('RESOURCE_APPLY_OBSERVED') and not injected then
  if SCENARIO=='external_before_restore' then yaw=90;injected=true
  elseif SCENARIO=='external_restore' then yaw=60;injected=true end
 end
 if has('SUMMARY') then break end
end
module.on_unload()
local output=table.concat(logs,'\n')
assert(not has('UNEXPECTED_ERROR'),output)
assert(successes<=2 and physical_writes<=2,'successful write cap exceeded')
assert(second_yaw==70,'second Glider was changed')
assert(write_result_reads==0,'writer result was reused as independent readback')
if SCENARIO=='normal' then
 assert(has('result=COMPLETE') and successes==2 and attempts==6 and yaw==60,output)
 assert(has('real_gliders=2') and has('RESOURCE_APPLY_OBSERVED') and has('RESOURCE_RESTORE_OBSERVED'),output)
 assert(gets>=8 and has('gameplay_verified=false'),output)
elseif SCENARIO=='write_gate_unavailable' or SCENARIO=='enum_missing' or SCENARIO=='empty_inventory' then
 assert(has('result=STARTUP_TIMEOUT') and attempts==0,output)
elseif SCENARIO=='duplicate_resource' or SCENARIO=='bad_source' or SCENARIO=='bad_slot_read' or SCENARIO=='bad_baseline' then
 assert(has('result=INVENTORY_REJECTED') and attempts==0,output)
elseif SCENARIO=='no_gliders' then
 assert(has('result=NO_GLIDER') and attempts==0,output)
elseif SCENARIO=='world_change_during_inventory' then
 assert(has('result=SESSION_CHANGED') and attempts==0,output)
elseif SCENARIO=='external_before_apply' or SCENARIO=='race_apply' then
 assert(has('result=APPLY_SCOPE_CHANGED') and successes==0 and physical_writes==0 and yaw==90,output)
elseif SCENARIO=='world_change' then
 assert(has('result=SESSION_CHANGED') and attempts==1 and successes==1 and yaw==61,output)
elseif SCENARIO=='external_after_apply' then
 assert(has('result=READBACK_SCOPE_CHANGED') and attempts==1 and successes==1 and yaw==90,output)
elseif SCENARIO=='scope_item_changed' or SCENARIO=='scope_slot_changed' then
 assert(has('result=READBACK_SCOPE_CHANGED') and has('cleanup_required=true') and attempts==1,output)
elseif SCENARIO=='resource_unloaded' or SCENARIO=='cleanup_busy' then
 assert(has('result=CLEANUP_TIMEOUT') and has('cleanup_required=true') and successes==1,output)
elseif SCENARIO=='unload_after_apply' then
 assert(has('UNLOAD_CLEANUP') and successes==2 and yaw==60 and has('result=UNLOADED'),output)
 assert(not has('result=COMPLETE'),output)
elseif SCENARIO=='external_before_restore' or SCENARIO=='race_restore' then
 assert(has('result=CLEANUP_SCOPE_CHANGED') and successes==1 and yaw==90,output)
elseif SCENARIO=='external_restore' then
 assert(has('result=RESTORED_EXTERNALLY') and successes==1 and yaw==60 and has('RESOURCE_RESTORE_OBSERVED'),output)
elseif SCENARIO=='uncertain_apply' or SCENARIO=='throw_apply' then
 assert(has('result=OUTCOME_UNCERTAIN') and has('cleanup_required=true') and attempts==1 and successes==0,output)
elseif SCENARIO=='uncertain_restore' then
 assert(has('result=OUTCOME_UNCERTAIN') and has('cleanup_required=true') and attempts==2 and successes==1 and physical_writes==2,output)
elseif SCENARIO=='apply_read_timeout' then
 assert(has('result=RESTORED_APPLY_UNCONFIRMED') and successes==2 and yaw==60 and not has('RESOURCE_APPLY_OBSERVED'),output)
elseif SCENARIO=='restore_read_timeout' then
 assert(has('result=RESTORE_READ_TIMEOUT') and successes==2 and yaw==60,output)
else error('unknown scenario') end
'''

scenarios = (
    "normal", "write_gate_unavailable", "enum_missing", "empty_inventory", "duplicate_resource",
    "bad_source", "bad_slot_read", "bad_baseline", "no_gliders", "world_change_during_inventory",
    "external_before_apply", "race_apply", "world_change", "external_after_apply", "scope_item_changed",
    "scope_slot_changed", "resource_unloaded", "unload_after_apply", "external_before_restore",
    "race_restore", "external_restore", "uncertain_apply", "throw_apply", "uncertain_restore",
    "cleanup_busy", "apply_read_timeout", "restore_read_timeout",
)
for scenario in scenarios:
    vm = lua.luaL_newstate()
    assert vm
    try:
        lua.luaL_openlibs(vm)
        source = (f"local SCENARIO='{scenario}'\n" + bootstrap + "\nlocal module=(function()\n" + probe + "\nend)()\n" + checks).encode()
        status = lua.luaL_loadbufferx(vm, source, len(source), b"@live-resource-scalar-probe-fixture", None)
        if not status:
            status = lua.lua_pcallk(vm, 0, 0, 0, 0, None)
        if status:
            raise AssertionError(scenario + ": " + lua.lua_tolstring(vm, -1, None).decode(errors="replace"))
        print("Lua 5.4 ResourceService scalar lifecycle passed: " + scenario)
    finally:
        lua.lua_close(vm)
print(f"{len(scenarios)} private scenarios passed, no game process/native calls.")
