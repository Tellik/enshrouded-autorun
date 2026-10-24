# Enshrouded Autorun (standalone test build)

This is a separate, client-only ShroudForge mod. It does not replace
`kfc-runtime.dll`, `shroudforge-runtime.dll`, or any other ShroudForge file.
ShroudForge 1.7.6 loads the mod's own `bin/autorun.dll` through
`native-plugin.ini`. The plugin is restricted to Enshrouded Steam build
`26082781`; it stays inactive if the executable or stock ShroudForge ABI does
not match. This standalone build still needs live gameplay verification.

## Install

1. Install unmodified ShroudForge 1.7.6 beside `enshrouded.exe` and close the
   game. Back up an existing `mods/autorun` folder, if present.
2. Extract this ZIP **inside the game's `mods` folder**. The result must be
   `Enshrouded/mods/autorun/mod.json`, `extended.mod.json`,
   `native-plugin.ini`, and `bin/autorun.dll`. Do not put the ZIP itself in
   `mods`; the native DLL loader needs a directory.
3. Enable Autorun in the ShroudForge Modloader and restart the game. The DLL
   loads at game startup. Under Proton, if the Modloader window cannot run
   without WebView2, close the game and change `"enabled": false` to
   `"enabled": true` in `mods/autorun/extended.mod.json` instead.
4. Check `mods/autorun/autorun-plugin.log` for `Exact-build input hook
   installed`. If it says the game image or ABI is unsupported, leave the mod
   off and check for an updated build.

No server installation or save changes are needed. This version is designed
for normal forward movement on the local client; multiplayer still needs a
live standalone-package test.

## Change the hotkey

The default toggle is **F8**. Press it once to start and again to stop. **S**
also cancels movement. In the Modloader, open **Autorun → Keyboard shortcuts**
or **Compatibility → Hotkeys**, record another shortcut, then release all keys.
The plugin reads setting changes live and stops movement if a binding changes.

If the Modloader window is unavailable under Proton, close the game and edit
`mods/autorun/extended.mod.json`. Change
`settings.toggleAutorun.value` from `"SDL:65"` (F8) to `"SDL:64"` (F7),
`"SDL:63"` (F6), or another supported key. `settings.cancelAutorun.value`
defaults to `"SDL:22"` (S); match it to your backward movement key. Keyboard
modifiers use strings such as `"Ctrl+SDL:64"`. The plugin also supports mouse
buttons 1–5 through `"Mouse:4"` and one keyboard-or-mouse alternative such as
`"SDL:64|Mouse:4"`. An empty string disables a shortcut. Unsupported keys or
malformed settings fail closed and are reported in `autorun-plugin.log`.

The current native plugin recognizes A–Z, 0–9, F1–F12, Enter, Escape,
Backspace, Tab, Space, Insert, Delete, Home, End, Page Up/Down, arrow keys,
and mouse buttons 1–5. It does not consume those keys, so avoid conflicting
game or mod bindings. The package is disabled by default until standalone
gameplay testing is complete.

## Use and limits

Wait for the world to load in a safe open area, then tap the toggle. Forward
movement uses the game's ordinary input speed and preserves left/right
steering. Backward movement or loss of game focus stops it. Turn it off before
opening menus. Menu, death, travel, disconnect, native Windows gameplay, and
other active multiplayer players still need dedicated checks. The plugin does
not force sprint or the game's slower Walk action.

Source and build workflow: https://github.com/Tellik/enshrouded-autorun
The bundled license retains ShroudForge's MIT notice for reused hook helpers.
