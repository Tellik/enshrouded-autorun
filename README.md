# Enshrouded Autorun

Autorun is a standalone, client-side mod for Enshrouded. Press **F8** to move
forward at the game's normal speed, press **F8** again to stop, or press your
backward key (**S** by default) to cancel. The toggle key is configurable.
Autorun uses [ShroudForge](https://github.com/bonsaibauer/shroudforge) to load
its own plugin; it does not replace or require a modified ShroudForge runtime.

This release supports **Enshrouded Steam build 26082781** with the unmodified
**ShroudForge 1.7.6 dev build 40026840919**. The mod checks the exact game
executable and stays inactive on a different build. Single-player and joining
a dedicated server were tested on Linux with Proton Experimental, with Autorun
installed only on the client. The Windows DLL builds successfully, but native
Windows gameplay has not been tested. Menus, travel, death, disconnects, and
multiplayer sessions with other active players also need further testing.

## Download

- [Autorun v0.2.0 release](https://github.com/Tellik/enshrouded-autorun/releases/tag/v0.2.0): download **`autorun-0.2.0.zip`**.
- [ShroudForge 1.7.6 dev release](https://github.com/bonsaibauer/shroudforge/releases/tag/v1.7.6-dev.40026840919): download **[the ShroudForge ZIP](https://github.com/bonsaibauer/shroudforge/releases/download/v1.7.6-dev.40026840919/shroudforge-1.7.6-40026840919.zip)**.

Use the ZIP assets, not GitHub's automatically generated source archives.
The Autorun ZIP contains only the mod; ShroudForge is a separate download.

## Install the files (Windows and Linux/Proton)

1. Close Enshrouded. Extract the ShroudForge ZIP into the game directory that
   contains `enshrouded.exe`. The three DLLs, `shroudforge` folder, and `mods`
   folder from the ZIP must sit beside the executable.
2. Extract the Autorun ZIP **inside** that `mods` folder. Check that the result
   is `Enshrouded/mods/autorun/mod.json` and
   `Enshrouded/mods/autorun/bin/autorun.dll`. Do not leave the ZIP itself in
   `mods` or add an extra `autorun` folder level.

ShroudForge's ZIP enables several bundled mods. World Editor also uses F8;
disable it or choose a different Autorun key. For an isolated first test,
disable all bundled mods. No server installation or save changes are needed.

### Windows

1. Start Enshrouded and press **F9** to open the ShroudForge Modloader.
2. Enable Autorun there, disable World Editor if you want to keep F8, and
   restart the game. Autorun's native DLL loads at startup.

If the Modloader window is unavailable, close the game and set
`"enabled": true` in `mods/autorun/extended.mod.json`, then restart. Windows
gameplay is not yet verified; please report the result if you test it.

### Linux / Proton

These steps reproduce the tested setup without WebView2. In Steam's
Enshrouded **Launch Options**, enter:

```text
WINEDLLOVERRIDES="dbghelp=n,b;winmm=n,b" %command%
```

Before the first launch, while the game is closed:

1. Open `shroudforge/config/modloader-config.json`. Under `modules`, set
   `debugConsole.enabled` and `modloaderUi.enabled` to `false`. This avoids
   the WebView2 startup error; the loader and mod still run.
2. Open `mods/autorun/extended.mod.json` and set `"enabled": true`.
3. Set `"enabled": false` in `mods/world-editor/extended.mod.json` to avoid
   the F8 conflict. For the clean baseline test, all ShroudForge bundled mods
   were disabled. Automatic ShroudForge updates were also disabled to hold the
   tested version (`modules.updates.automatic.enabled: false` in its config).
4. Start the game. The Modloader window is intentionally disabled on this
   path; edit Autorun's `extended.mod.json` while the game is closed.

## Use and configure

In a safe, open area, press F8 once and release the movement keys. The
character should keep moving forward. Press F8 again to stop, then press F8
and S to verify backward cancellation. Autorun preserves normal left/right
steering and does not force sprint. Turn it off before opening menus.

To change the toggle key, use the Modloader's Autorun keybind setting or edit
`settings.toggleAutorun.value` in `mods/autorun/extended.mod.json` while the
game is closed. The default is `"SDL:65"` (F8); `"SDL:64"` selects F7.
`settings.cancelAutorun.value` defaults to `"SDL:22"` (S). See the
[detailed mod guide](standalone/autorun/README.md) for more binding formats.

After launch, `mods/autorun/autorun-plugin.log` should contain
`Exact-build input hook installed`. If it reports an unsupported game image or
ABI, disable Autorun and wait for a compatible build.

## Development and attribution

The release ZIP is built from [`standalone/autorun/`](standalone/autorun/).
The older fork-integrated proof of concept and input probe remain in this
repository as [development evidence](docs/development/autorun.en.md); they
are not part of the Autorun release. The
[Windows package workflow](.github/workflows/autorun-windows.yml) builds and
checks the standalone ZIP and checksum.

ShroudForge is by bonsaibauer and its contributors. This fork preserves its
[MIT license](LICENSE) and [original README](README.shroudforge.md). The
Autorun package includes the notices for its compiled dependencies. Game
binaries, assets, saves, and local logs are not distributed here.
