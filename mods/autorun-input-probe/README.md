# Autorun Input Probe

This diagnostic checks ShroudForge 1.7.6's keybind API and Lua lifecycle on the
target system. It changes a local test flag and writes logs. It does not apply
movement, edit game assets, change player statistics, or send network requests.

The source package is disabled by default. Enable it after disabling the other
bundled gameplay mods. World Editor uses F8 by default and conflicts with this
test. The prepared local baseline enables only this probe.

## Test

1. Start Enshrouded with the compiled ShroudForge 1.7.6 loader and this mod.
2. At the main menu, press F8, hold it for two seconds, then release it. Press
   and release F8 again. Expect exactly two `Toggle #` messages: ON, then OFF.
3. Press F8 once more, release it, then press S. Expect an OFF cancellation.
4. Switch to another application and tap F8 there. Return to the game. The
   background taps should not add toggle messages.
5. Repeat in a single-player world and while a game menu or chat is open. Record
   whether inputs are still reported in those contexts; this probe does not gate
   gameplay menus or chat.
6. Optionally change the toggle binding in the loader. A binding change resets
   the diagnostic flag. Holding the new binding during the change must not
   count as a press until it has been released and pressed again.
7. Disable the probe in the loader or exit the game. Check for the unload entry
   when the runtime provides a normal unload callback.

Use F10 for the loader's debug window, or inspect
`shroudforge/logs/shroudforge-client.log` under the game installation. Search for
`[autorun-probe]`. The loader's **Write test status to log** action is an
additional way to verify the callback is running.

## Limits

The existing keybind API reports false both when a key is released and when the
game loses focus. It does not expose an independent focus signal to this mod.
Returning focus while a key is held may therefore look like a new press. This
test flag is not a safe movement controller and does not auto-cancel on focus
loss, menus, chat, or world changes. Those cases must be handled by the eventual
movement integration before it can drive the character.

Keyboard chords require exact modifiers in ShroudForge. The plain S test binding
does not cover Shift+S; movement cancellation must eventually use the game's
actual backward-input action and respect remapped controls.
