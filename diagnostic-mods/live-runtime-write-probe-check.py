"""Private Lua 5.4 lifecycle fixture for the controlled typed write probe.
No game process, production Lua executor or native runtime is involved.
"""
import ctypes as C
import json
from pathlib import Path
root=Path(__file__).resolve().parents[1]
package=root/'diagnostic-mods/live-runtime-write-probe'
manifest=json.loads((package/'mod.json').read_text())
assert manifest['capabilities']==['runtime']
assert json.loads((package/'extended.mod.json').read_text())['enabled'] is True
probe=(package/'src/mod.lua').read_text()
lua=C.CDLL(r'C:\Users\mail\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin\lua54.dll')
lua.luaL_newstate.restype=C.c_void_p
lua.luaL_openlibs.argtypes=[C.c_void_p]
lua.luaL_loadbufferx.argtypes=[C.c_void_p,C.c_char_p,C.c_size_t,C.c_char_p,C.c_char_p]
lua.luaL_loadbufferx.restype=C.c_int
lua.lua_pcallk.argtypes=[C.c_void_p,C.c_int,C.c_int,C.c_int,C.c_ssize_t,C.c_void_p]
lua.lua_pcallk.restype=C.c_int
lua.lua_tolstring.argtypes=[C.c_void_p,C.c_int,C.POINTER(C.c_size_t)]
lua.lua_tolstring.restype=C.c_char_p
lua.lua_close.argtypes=[C.c_void_p]
bootstrap=r'''
local logs, ready, queries, attempts, successes, physical_writes = {},false,0,0,0,0
local session_id=17
local network_max=143
local pending_network
local adder_attempts,restore_attempts=0,0
local root_values={Stamina=143,Stamina_Min=0,Stamina_Max=143,Stamina_RechargeRampup=2000,
 Stamina_RechargeTimeout=250,Stamina_Recharge=95,Stamina_Max_Base=110,Stamina_Max_Adder=33}
local ordered={'Stamina','Stamina_Min','Stamina_Max','Stamina_RechargeRampup',
 'Stamina_RechargeTimeout','Stamina_Recharge','Stamina_Max_Base','Stamina_Max_Adder'}
local function clone(v) local t={} for k,x in pairs(v) do t[k]=x end return t end
local function snapshot()
 return {values=clone(root_values),scalar_type='keen::sint32',root_hash=123,
 component={qualified_name='keen::ecs::Stamina'}}
end
local function has(marker)
 for _,value in ipairs(logs) do if value:find(marker,1,true) then return true end end
 return false
end
shroudforge={log={info=function(value) logs[#logs+1]=value end}}
runtime={ecs={},world={},has=function(name)
 if not ready then return false end
 return not(SCENARIO=='write_gate_unavailable' and name=='runtime.ecs.write')
end}
runtime.world.session_id=function() return ready and session_id or 0 end
runtime.ecs.query=function(...)
 queries=queries+1
 local names={...}
 assert(#names==3 and names[1]=='keen::ecs::PlayerInput' and names[2]=='keen::ecs::Stamina' and names[3]=='keen::ecs::NetworkPlayerAttributes')
 if queries<=3 then return nil,'Live ECS query is still scanning. Retry on the next update' end
 if SCENARIO=='multiple_players' then return {55,77} end
 return {55}
end
runtime.ecs.get_attribute=function(name)
 return {root_hash=123,index=name=='Stamina' and 0 or 2}
end
runtime.ecs.read_attributes=function(entity,name)
 assert(entity==55 and name=='Stamina')
 return snapshot()
end
runtime.ecs.read_fields=function(entity,name,fields)
 assert(entity==55 and name=='keen::ecs::NetworkPlayerAttributes')
 assert(fields[1]=='stamina' and fields[2]=='staminaMax')
 if pending_network then
  pending_network.remaining=pending_network.remaining-1
  if pending_network.remaining<=0 then network_max=pending_network.value;pending_network=nil end
 end
 return {stamina=root_values.Stamina,staminaMax=network_max}
end
runtime.ecs.update_attribute=function(entity,name,target)
 assert(entity==55 and name=='Stamina_Max_Adder')
 attempts=attempts+1
 assert(successes<2,'more than two successful writes requested')
 if target==34 then
  adder_attempts=adder_attempts+1
  if SCENARIO=='normal' and adder_attempts<=2 then return nil,'Attribute snapshot changed before update. Read current values and retry' end
 else
  assert(target==33,'unexpected restore target')
  restore_attempts=restore_attempts+1
  if SCENARIO=='cleanup_busy' or (SCENARIO=='normal' and restore_attempts<=2) then return nil,'guarded attribute update failed' end
 end
 local before=clone(root_values)
 before.Stamina_Max_Adder=target
 local old_max=root_values.Stamina_Max
 root_values.Stamina_Max_Adder=target
 root_values.Stamina_Max=root_values.Stamina_Max_Base+target
 root_values.Stamina=math.floor(root_values.Stamina/old_max*root_values.Stamina_Max)
 physical_writes=physical_writes+1
 if SCENARIO=='uncertain_apply' and target==34 or SCENARIO=='uncertain_restore' and target==33 then
  return nil,'ECS write outcome is uncertain because the original bytes could not be verified after rollback'
 end
 successes=successes+1
 if SCENARIO=='unexpected_apply_scope' and target==34 then
  root_values.Stamina_Max_Base=111;root_values.Stamina_Max=145
 end
 if not(SCENARIO=='network_apply_timeout' and target==34) then pending_network={remaining=3,value=root_values.Stamina_Max} end
 local result=snapshot()
 local before_bits,after_bits={},{}
 for i,n in ipairs(ordered) do before_bits[i]=before[n];after_bits[i]=root_values[n] end
 result.trace={{name='Stamina_Max',before_bits=before_bits,after_bits=after_bits}}
 result.applied=true
 return result
end
runtime.functions={get=function() error('native function calls forbidden') end}
runtime.ecs.write=function() error('raw component writes forbidden') end
runtime.ecs.write_attribute_storage=runtime.ecs.write
runtime.ecs.write_fields=runtime.ecs.write
'''
checks=r'''
module.on_load()
assert(queries==0 and attempts==0,'startup invoked ECS')
for _=1,5 do module.on_update() end
assert(queries==0 and attempts==0,'unready startup queried or wrote')
ready=true
if SCENARIO=='initially_depleted' then root_values.Stamina=100 end
local injected=false
for tick=1,2800 do
 module.on_update()
 if SCENARIO=='initially_depleted' and tick==30 then
  assert(attempts==0,'probe wrote while stamina was depleted')
  root_values.Stamina=143
 end
 if successes==1 and not injected then
  if SCENARIO=='world_change' then session_id=18;injected=true
  elseif SCENARIO=='external_base' then root_values.Stamina_Max_Base=111;root_values.Stamina_Max=145;injected=true
  elseif SCENARIO=='depletion_after_apply' then root_values.Stamina=100;injected=true
  elseif SCENARIO=='unload_after_apply' then break end
 end
 if SCENARIO=='external_adder_before_restore' and not injected and has('NETWORK_APPLY_OBSERVED') then
  root_values.Stamina_Max_Adder=99;root_values.Stamina_Max=209;injected=true
 end
 if has('SUMMARY') then break end
end
module.on_unload()
local output=table.concat(logs,'\n')
assert(not has('UNEXPECTED_ERROR'),output)
assert(successes<=2,'successful write cap exceeded')
if SCENARIO=='normal' or SCENARIO=='initially_depleted' or SCENARIO=='depletion_after_apply' then
 assert(has('result=COMPLETE'),output)
 assert(successes==2 and root_values.Stamina_Max_Adder==33 and root_values.Stamina_Max==143,output)
 assert(has('APPLY_CALCULATION_TRACE') and has('RESTORE_CALCULATION_TRACE'),output)
 assert(has('APPLY_ROOT_OBSERVED') and has('RESTORE_ROOT_OBSERVED'),output)
 if SCENARIO=='normal' then assert(attempts==6,output) end
 if SCENARIO=='depletion_after_apply' then assert(root_values.Stamina==99,'restore overwrote progressed current with old baseline') end
elseif SCENARIO=='world_change' then
 assert(has('result=SESSION_CHANGED') and successes==1 and attempts==1 and root_values.Stamina_Max_Adder==34,output)
elseif SCENARIO=='external_base' then
 assert(has('result=READBACK_SCOPE_CHANGED') and successes==1 and attempts==1 and root_values.Stamina_Max_Base==111,output)
elseif SCENARIO=='external_adder_before_restore' then
 assert(has('result=CLEANUP_SCOPE_CHANGED') and successes==1 and attempts==1 and root_values.Stamina_Max_Adder==99,output)
elseif SCENARIO=='unexpected_apply_scope' then
 assert(has('result=APPLY_RESULT_UNEXPECTED') and successes==1 and attempts==1,output)
elseif SCENARIO=='network_apply_timeout' then
 assert(has('NETWORK_APPLY_TIMEOUT') and has('result=RESTORED_NETWORK_APPLY_UNCONFIRMED') and successes==2,output)
elseif SCENARIO=='multiple_players' or SCENARIO=='write_gate_unavailable' then
 assert(has('result=STARTUP_TIMEOUT') and attempts==0,output)
elseif SCENARIO=='uncertain_apply' then
 assert(has('result=OUTCOME_UNCERTAIN') and has('cleanup_required=true') and attempts==1 and physical_writes==1 and successes==0,output)
elseif SCENARIO=='uncertain_restore' then
 assert(has('result=OUTCOME_UNCERTAIN') and has('cleanup_required=true') and attempts==2 and successes==1,output)
elseif SCENARIO=='unload_after_apply' then
 assert(has('UNLOAD_CLEANUP') and successes==2 and root_values.Stamina_Max_Adder==33,output)
elseif SCENARIO=='cleanup_busy' then
 assert(has('result=CLEANUP_TIMEOUT') and has('cleanup_required=true') and successes==1 and root_values.Stamina_Max_Adder==34,output)
else error('unknown test scenario') end
'''
scenarios=('normal','initially_depleted','depletion_after_apply','world_change','external_base',
 'external_adder_before_restore','unexpected_apply_scope','network_apply_timeout','multiple_players',
 'write_gate_unavailable','uncertain_apply','uncertain_restore','unload_after_apply','cleanup_busy')
for scenario in scenarios:
 state=lua.luaL_newstate()
 assert state
 try:
  lua.luaL_openlibs(state)
  source=(f"local SCENARIO='{scenario}'\n"+bootstrap+'\nlocal module=(function()\n'+probe+'\nend)()\n'+checks).encode()
  status=lua.luaL_loadbufferx(state,source,len(source),b'@live-runtime-write-probe-fixture',None)
  if not status: status=lua.lua_pcallk(state,0,0,0,0,None)
  if status: raise AssertionError(scenario+': '+lua.lua_tolstring(state,-1,None).decode(errors='replace'))
  print('Lua 5.4 controlled-write lifecycle passed: '+scenario)
 finally: lua.lua_close(state)
print(f'{len(scenarios)} private lifecycle scenarios passed, no game process/native calls')
