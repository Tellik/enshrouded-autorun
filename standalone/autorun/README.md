# Enshrouded Autorun (standalone test build)

This is a separate, client-only ShroudForge mod. It does not replace
`kfc-runtime.dll`, `shroudforge-runtime.dll`, or any other ShroudForge file.
ShroudForge 1.7.6 loads the mod's own `bin/autorun.dll` through
`native-plugin.ini`. The plugin is restricted to Enshrouded Steam build
`26082781`; it stays inactive if the executable or stock ShroudForge ABI does
not match. Single-player and dedicated-server Proton tests on stock
ShroudForge 1.7.6 passed with F7 as a custom hotkey.

## Install the files (Windows and Linux/Proton)

1. Close the game. Extract the unmodified ShroudForge 1.7.6 ZIP into the
   Enshrouded folder containing `enshrouded.exe`. Its three proxy DLLs go
   beside the executable; its `shroudforge` and `mods` folders go beside it.
   Back up an existing `mods/autorun` folder, if present.
2. Extract the Autorun ZIP **inside the game's `mods` folder**. The result
   must be `Enshrouded/mods/autorun/mod.json`, `extended.mod.json`,
   `native-plugin.ini`, and `bin/autorun.dll`. Do not put the ZIP itself in
   `mods`; the native DLL loader needs a directory.

The stock ShroudForge ZIP enables several bundled mods. World Editor uses
F8, Autorun's default key. Disable World Editor or change Autorun's key;
for an isolated first test, disable all bundled mods.

### Windows

1. Start Enshrouded and press F9 to open the ShroudForge Modloader.
2. Enable Autorun. Disable World Editor if you want to keep F8 for Autorun.
   Restart the game so its native DLL loads at startup.

Native Windows gameplay has not yet been tested. The Windows build and ZIP
layout passed automated checks.

### Linux / Proton

These steps reproduce the tested Proton setup without requiring WebView2.
Before the first launch, set Steam's Enshrouded **Launch Options** to:

```text
WINEDLLOVERRIDES="dbghelp=n,b;winmm=n,b" %command%
```

Then, while the game is closed:

1. Open `shroudforge/config/modloader-config.json`. Under `modules`, change
   the `enabled` values for `debugConsole` and `modloaderUi` to `false`.
   This prevents the two WebView2 windows from causing a startup error.
2. Open `mods/autorun/extended.mod.json` and change `"enabled": false` to
   `"enabled": true`.
3. For an isolated first test, set `"enabled": false` in each bundled mod's
   `extended.mod.json`, especially `mods/world-editor`. To hold ShroudForge
   at the tested version, also set `modules.updates.automatic.enabled` to
   `false` in its config file. The clean-install test also disabled the
   `worldEditor` module there.
4. Start the game. The Modloader window is intentionally disabled; configure
   Autorun by editing its `extended.mod.json` while the game is closed.

### Check the installation

Check `mods/autorun/autorun-plugin.log` for `Exact-build input hook installed`.
If it says the game image or ABI is unsupported, leave the mod off and check
for an updated build.

No server installation or save changes are needed. This version moves the
local client at normal forward speed. A dedicated-server test passed without
installing Autorun on the server.

## Change the hotkey

The default toggle is **F8**. Press it once to start and again to stop. **S**
also cancels movement. In the Modloader, open **Autorun → Keyboard shortcuts**
or **Compatibility → Hotkeys**, record another shortcut, then release all keys.
The plugin reads setting changes live and stops movement if a binding changes.

On Linux/Proton with the Modloader window disabled, close the game and edit
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
game or mod bindings. The package is disabled by default so each installation
can enable it after checking its game build and bindings.

## Use and limits

Wait for the world to load in a safe open area, then tap the toggle. Forward
movement uses the game's ordinary input speed and preserves left/right
steering. Backward movement or loss of game focus stops it. Turn it off before
opening menus. Menu, death, travel, disconnect, native Windows gameplay, and
other active multiplayer players still need dedicated checks. The plugin does
not force sprint or the game's slower Walk action.

For a first check, tap F8 and release all movement keys: the character should
continue forward. Tap F8 again to stop. Tap F8 once more and press S to
cancel. If you configured a different toggle key, use that key instead.

## Verified test

With unmodified ShroudForge 1.7.6 and Enshrouded Steam build `26082781`,
single-player and dedicated-server Proton tests used `"SDL:64"` (F7) instead
of the default F8. F7 started normal forward movement, a second F7 stopped
it, and S canceled a later run in both worlds. The game stayed stable. The
loader reported one plugin and no load issues; the plugin log recorded 55
local input writes before toggle-off and 57 before S cancellation on the
dedicated server. This confirms that editing the installed keybind takes
effect on the next launch. Live Modloader rebinding, multiplayer with other
active players, and native Windows gameplay remain untested.

A separate clean-install check removed the previous ShroudForge and mod files,
then extracted the stock ShroudForge 1.7.6 ZIP and this Autorun ZIP into the
paths above. After the Linux/Proton and bundled-mod settings above, the game
opened without a WebView2 error. The packaged default F8 binding moved
forward, F8 stopped it, and S canceled it in both single-player and a
dedicated-server world. The game remained stable. The plugin recorded 68
local writes before F8 stopped it and 58 before S canceled it in single-player;
the dedicated-server run recorded 53 and 54. The game executable, both KFC
data files, and both stock ShroudForge runtime DLLs retained their original
hashes.

Source and build workflow: https://github.com/Tellik/enshrouded-autorun
The bundled `LICENSE` retains ShroudForge's MIT notice for reused hook helpers.
`LICENSE.nlohmann` covers the JSON parser compiled into the plugin.
