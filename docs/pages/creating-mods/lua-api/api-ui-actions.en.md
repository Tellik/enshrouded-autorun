---
{
  "id": "api-ui-actions",
  "locale": "en",
  "title": "Connect Modloader buttons to Lua",
  "summary": "An action ID in the mod file connects a visible button to a callback in your mod.",
  "navigation": {
    "path": [
      "Lua API guides",
      "Feature guides"
    ]
  },
  "apiSymbols": [
    "shroudforge.ui.on_action",
    "shroudforge.ui.toggle_modloader"
  ]
}
---

# Connect Modloader buttons to Lua

An action is declared under `groups[].actions[]`. Its `id` is passed to `shroudforge.ui.on_action`. The two IDs must match exactly.

```lua
shroudforge.ui.on_action("resetFeature", function()
  shroudforge.log.info("Reset action requested")
end)
```

The mod needs `runtime`. Register the callback while the mod runtime loads. `label` and `style` describe the visible button, while `confirm` adds an optional prompt.

The action only calls your callback. It does not guarantee a game change succeeded. Check return values and log the outcome. See the [API reference](#api) for function details.

An active client mod can open or close the Modloader UI through the same toggle used by the configured shortcut:

```lua
shroudforge.ui.toggle_modloader()
```

This signals the UI process directly and does not synthesize a key press. The Modloader UI must be running in the game session.

**Settings → General → Appearance → Show Mods in the game menu** controls the native **Mods** entry in Enshrouded's main menu. It uses the game's own buttons and the same Modloader toggle as this Lua function and the configured shortcut. Checkbox changes apply when the main menu is next drawn. The native binding is installed at game startup, so a ShroudForge update requires restarting the game.
