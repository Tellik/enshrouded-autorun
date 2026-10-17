# SF Survival

SF Survival uses the public [typed runtime API](../../docs/runtime/runtime-registry-api.en.md).
Its Lua code contains no process addresses, gameplay patches or native modifier
bindings. All settings start disabled and can be changed while the mod is active.

| Setting | Original engine input |
| --- | --- |
| Infinite Stamina | `Stamina_Min = Stamina_Max` |
| Infinite Mana | `Mana_Min = Mana_Max` |
| Infinite Underwater Breath | `Oxygen_Min = Oxygen_Max` |
| Infinite Frost Timer | `BodyHeat_Min = BodyHeat_Max` |
| Fast Health Regen | Existing `Health_Recharge`, multiplied by 100 and capped at 65536 |
| Fast Mana Regen | Existing `Mana_Recharge`, multiplied by 100 and capped at 65536 |
| No Fall Damage | The actual player template's `FallDamage.fallDamageStartDistance`, at least its lethal distance |
| Infinite Weapon Durability | The original named `ecs::GameSettings.enableDurability` service field |

Attribute changes run the original root calculation through
`runtime.ecs.update_attribute`. The maximum and the previous minimum are checked
together before an infinite setting changes a bound. The engine then applies
its own clamp after consumption. The update interval is used to bind players,
observe changed maxima and react to settings. It does not periodically refill
the current attribute.

Fast regeneration requires existing natural regeneration. It retains the game's
original timeout and ramp. It does not create a separate regeneration event.

## Scope and restoration

Attribute inputs belong to each current authoritative player entity. Fall
configuration is shared by entities using the same template. The durability
switch belongs to the entire locally hosted world and also affects other
carried items. These are distinct original engine storage scopes.

The shared API remembers each changed value and restores it when its setting is
disabled or the loader unloads the mod in the same world session. The public
typed target factories provide the same behavior to other independent mods.
Restoration compares the last owned value before writing. An external change is preserved.
A busy engine gate retries on a later update. An uncertain write stops further
mutations.

If cleanup is temporarily busy when the complete mod is disabled, the runtime
keeps its scope alive and retries the disable. During this time the mod only
restores its inputs. Package refresh aborts when cleanup cannot finish.

Turn the settings off before leaving the world and wait for their restoration.
An already invalidated world session cannot authorize restoration of old shared
storage. If that occurs while shared settings are still owned, the mod stops
and reports that a game restart is needed.

## Verification

The Lua lifecycle fixture exercises all eight public API routes, shared-template
deduplication, changed maxima, boolean restoration, busy retries, external
changes, uncertain outcomes and session changes. These fixtures verify control
flow. The original attribute interpreters were independently emulated for the
bound calculations. Actual gameplay outcomes must also be checked in a loaded
offline world using the matching runtime build.
