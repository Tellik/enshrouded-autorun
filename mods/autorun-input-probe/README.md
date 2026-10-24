# Autorun Input Probe

This temporary diagnostic checks the F8/S bindings and observes the game's
local-client movement input. The exact-build native hook is armed by F8, but
**this build does not write movement**. It reports hook hits, uniquely matched
local-player hits, and the last observed movement axes. The hook is disabled on
other Enshrouded builds and on a dedicated server.

The package is disabled by default. During this test, disable the bundled
gameplay mods that use F8, including World Editor. The local test installation
enables only this probe.

## Test

1. Enter a single-player world, wait a few seconds, then tap F8 once.
2. Stand still for two seconds. Hold the normal forward key for two seconds and
   release it. Hold S for two seconds and release it. S cancels the diagnostic.
3. Check `shroudforge/logs/shroudforge-client.log` in the game installation for
   `[autorun-probe] Native hook` entries. The `local` count should rise and
   the axes should change from `(0,0)` to roughly `(0,1)` under forward input.
   S should cancel the hook. `writes` must remain zero.
4. Tap F8 again, then switch to another application and return. The hook should
   stop when the game loses focus. A second F8 tap toggles it off.

The read-only component snapshot may need several seconds to discover the
unique local player when a world first loads. The probe keeps refreshing that
identity after it stops logging ordinary input changes.

This stage validates whether the hook runs after the game's movement input is
produced. The actual forward-input write remains disabled until those live
observations match the earlier read-only W/S test.
