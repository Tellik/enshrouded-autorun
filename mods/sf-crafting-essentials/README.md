# SF Crafting Essentials

`sf-crafting-essentials` combines recipe costs, item use and production timing in one mod. It uses named Lua types and services. It does not use function hashes, RVAs, profile-bound modifiers or executable hooks.

## Settings

| Setting | When it applies | Scope |
| --- | --- | --- |
| **No Recipe Resource Costs** | Before startup. Restart required. | Sets direct ingredient counts, category ingredient counts and water charges to zero for every recipe. Building, upgrades and other inventory costs are unchanged. |
| **No Item Consumption** | Live. | Prevents supported use actions from consuming food, potions, bandages and other covered items. The status reports how many loaded action sequences were scanned and how many supported fields were found. The scan is incremental. Recipe ingredients, trades and inventory transfers are unchanged. Some item actions are not supported. |
| **Recipe Time Mode** | Restart required. | **Live: Original recipe times** preserves each recipe's own time as the baseline and enables the live scale. **Restart required: One shared time for all recipes** sets every timed recipe to one base time. |
| **Production Time in Seconds** | Startup asset preparation. Restart required. | Enabled only in shared-time mode. Sets every positive recipe duration to the selected base time. Range: 1 to 900 seconds. Default: 3 seconds. Instant recipes and output quantities are unchanged. |
| **Production Duration Scale** | Live. | Enabled only with original recipe times. It proportionally adjusts the loaded duration of every timed recipe. At 0%, the shortest recipe takes 1 second. Other recipes keep their relative times. At 100%, original recipe times are restored. The world speed setting is unchanged. Changes also affect running factory production. |

## Asset timing and scope

In original-time mode, startup asset preparation leaves every recipe's base duration unchanged. The live scale then adjusts each loaded timed recipe duration by the same ratio, preserving the relative times between recipes. In shared-time mode, startup asset preparation sets every timed recipe to the selected seconds value. Changing the mode or shared seconds requires a game restart. The controls are mutually exclusive. The seconds slider is disabled in original-time mode, and the live scale slider is disabled in shared-time mode.

The manifest's `patch` capability is used for that pregame KFC asset pass. Its `runtime` capability provides the typed Lua operations used during play. Neither capability installs an executable hook or writes game code.

The current settings apply to every matching entry. Shared Production Time covers all recipes with a positive duration. Original Times leaves each recipe's duration unchanged. No Recipe Resource Costs covers all recipe inputs and water charges. No Item Consumption covers matching consume fields in the six supported action-event types above. It does not cover every item, recipe, trade or inventory transaction. This version has no per-recipe or per-event picker.

Recipe costs and shared recipe durations are changed before startup because the game reads recipe data when loading production stations. Changing recipe costs during an order could also change its refund. The production scale and item-use flags use typed live write paths.

The original factory calculation is `actualDuration = baseDuration / ecs::GameSettings.factoryProductionSpeedFactor`, subject to the game's float rounding. In original-time mode, the mod reads loaded `keen::RecipeRegistryResource` values and the typed world factor. It computes one duration ratio from the shortest positive recipe's actual duration and applies that ratio to every timed `craftingDuration.value` through the guarded loaded-resource writer. This is the same recipe field changed by startup asset preparation. At 0%, the shortest recipe takes 1 second. Longer recipes keep their relative times. From 0% to 100%, the shortest duration moves linearly from 1 second to its original duration under the current world factor. At 100%, every original recipe duration is restored. The mod leaves the world's production speed factor unchanged and never writes a zero recipe duration. Recipe data is updated incrementally. Running factory production rereads the recipe duration and recalculates its deadline from the existing start time, so shortening a recipe below its elapsed time can complete that cycle on the next update. In shared-time mode, the live scale is disabled, and all timed recipes receive the chosen base duration during startup asset preparation.

The item-use fields live in `keen::actor::ActorSequenceResource.subSequences[i].events[j]`. The mod selects the six reflected event variant names listed above and overrides only `value.consume` when it is a boolean that was originally true. The scan is bounded per update to avoid a long blocking pass. These action sequences can be shared by multiple actors, so the effect may also apply to other actors using the same sequence. The current game capture contains 2,157 action-sequence resources and 4,262 `ItemInfo` records. Those are different counts, and sequence-field coverage must not be presented as coverage of every item.

## Inventory splitting

Keeping the source stack during an inventory split is not exposed as a setting. The current Runtime API can inspect transaction data, but it does not provide safe native Lua access to the live inventory transaction at the point where the game removes the source quantity. The feature needs a verified native transaction operation before it can be added without an executable hook or an unsafe substitute.
