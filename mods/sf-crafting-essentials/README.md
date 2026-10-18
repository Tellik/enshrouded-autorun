# SF Crafting Essentials

The mod ID is `sf-crafting-essentials`. It combines four previously separate features in one package.

## Settings

- **No Resource Costs** controls the verified runtime operation for resource consumption.
- **Infinite Item Use** controls the verified runtime operation for item consumption.
- **Keep Source Stack When Splitting** controls the verified runtime operation for splitting item stacks.
- **Apply Configured Production Time** enables or disables the duration change. When disabled, original durations are kept.
- **Production Time in Seconds** sets the shared base duration for recipes with a positive `keen::Time` duration. The default is 3 seconds. The range is 1 to 900 seconds, matching the longest original recipe. Instant recipes and recipe quantities are unchanged.

The KFC parser writes the production duration before the game starts by updating `keen::RecipeRegistryResource`, `recipes`, and `craftingDuration.value`. Changes require the pregame pass.

The three inventory and resource features run during gameplay. They are enabled only when the current game build provides the corresponding verified runtime operation. If an operation is unavailable, the runtime status reports the reason and the mod does not apply an assumed substitute effect.
