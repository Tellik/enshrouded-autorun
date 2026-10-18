# SF Crafting Essentials

`sf-crafting-essentials` combines recipe costs, item use and production timing in one mod. It uses named Lua types and services. It does not use function hashes, RVAs, profile-bound modifiers or executable hooks.

## Settings

| Setting | When it applies | Scope |
| --- | --- | --- |
| **No Recipe Resource Costs** | Before startup. Restart required. | Sets direct ingredient counts, category ingredient counts and water charges to zero for every recipe. Building, upgrades and other inventory costs are unchanged. |
| **Keep Used Items** | Live. | Sets the `consume` field to false on supported `PayUsageCost` and `ApplyBuffEvent` action events. Mana, stamina, fuel and special-ability costs are unchanged. Crafting ingredients and inventory transfers are unaffected. |
| **Recipe Time Mode** | Restart required. | **Live: Original recipe times** preserves each recipe's own base time and enables the live scale. **Restart required: One shared time for all recipes** sets every timed recipe to one base time. |
| **Production Time in Seconds** | Startup asset preparation. Restart required. | Enabled only in shared-time mode. Sets every positive recipe duration to the selected base time. Range: 1 to 900 seconds. Default: 3 seconds. Instant recipes and output quantities are unchanged. |
| **Production Duration Scale** | Live. | Enabled only with original recipe times. At 0%, the shortest timed recipe takes 1 second and other recipe times keep their ratios. At 100%, the existing world production setting is preserved. At 200%, production takes twice the normal time. |

## Asset timing and scope

In original-time mode, startup asset preparation leaves every recipe's base duration unchanged and the duration scale adjusts the world production factor live. In shared-time mode, startup asset preparation sets every timed recipe to the selected seconds value. Changing the mode or shared seconds requires a game restart. The controls are mutually exclusive. The seconds slider is disabled in original-time mode, and the live scale slider is disabled in shared-time mode.

The manifest's `patch` capability is used for that pregame KFC asset pass. Its `runtime` capability provides the typed Lua operations used during play. Neither capability installs an executable hook or writes game code.

The current settings apply to every matching entry. Shared Production Time covers all recipes with a positive duration. Original Times leaves each recipe's duration unchanged. No Recipe Resource Costs covers all recipe inputs and water charges. Keep Used Items covers all loaded matching action events. This version has no per-recipe or per-event picker.

Recipe costs and shared recipe durations are changed before startup because the game reads recipe data when loading production stations. Changing recipe costs during an order could also change its refund. The production scale and item-use flags use typed live write paths.

The original production calculation is `actualDuration = baseDuration / productionSpeedFactor`. In original-time mode, the mod reads the shortest positive recipe duration from loaded `keen::RecipeRegistryResource` values. At 0%, that shortest recipe takes exactly 1 second under the current positive world factor. Longer recipes keep their ratios. From 0% to 100%, the shortest duration moves linearly from 1 second to its normal duration under the current world setting. At 100%, the world factor is unchanged. At 200%, recipe durations take twice as long as normal. The setting uses the typed `ecs::GameSettings.factoryProductionSpeedFactor` service and never writes an infinite factor or a zero recipe duration. In shared-time mode, the live scale is disabled, and all timed recipes receive the chosen base duration during startup asset preparation.

The item-use fields live in `keen::actor::ActorSequenceResource.subSequences[i].events[j]`. The mod selects the original `PayUsageCost` and `ApplyBuffEvent` variant names and overrides only `value.consume`. These action sequences can be shared by multiple actors, so the effect may also apply to other actors using the same sequence.

## Inventory splitting

Keeping the source stack during an inventory split is not exposed as a setting. The current Runtime API can inspect transaction data, but it does not provide safe native Lua access to the live inventory transaction at the point where the game removes the source quantity. The feature needs a verified native transaction operation before it can be added without an executable hook or an unsafe substitute.
