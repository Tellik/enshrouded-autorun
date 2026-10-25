# Wand Durability — experimental 0.1.0

Apply the normal single-shot durability cost once per primary wand attack,
regardless of built-in Multishot or skill-generated bonus projectiles. Normally
this is one durability. Projectile count, timing, chance, damage, existing
durability-saving effects, and the world's durability setting remain controlled
by the game.

This is an experimental native plugin for stock ShroudForge 1.7.6, ABI 12,
and Enshrouded **client build 26082781 only**. It is disabled by default.
Gameplay validation is pending. It supports an authoritative world running in
the client executable: single-player and player-hosted co-op. In a hosted world
the effect applies to all players' primary wand attacks.

**A client-only install cannot fix a dedicated server's durability accounting.**
This first package does not support the dedicated-server executable. That
requires inspecting its own build and adding a separate verified hook profile.

## Install and test

1. Close Enshrouded. Extract the `wand-durability` folder into the game's `mods`
   folder alongside Autorun. Keep the stock ShroudForge runtime installed.
2. Enable Wand Durability in ShroudForge and restart the game to load the DLL.
3. In single-player, record the Cryptic Wand's durability, fire one complete
   attack, then check durability again. Without a durability-saving effect,
   each attack should cost exactly one whether it emits one, two, or more
   projectiles.
4. Repeat with the Multishot skill, several consecutive attacks, another wand,
   and a melee weapon or bow. Test a nearly broken wand, fast travel, death,
   and world leave/rejoin. Verify bonus projectiles still hit and deal damage.
5. Inspect `wand-durability.log` in this mod folder. `wand_charges` counts original
   charge calls allowed through, and `bonus_costs_skipped` should increase on
   extra events. The original helper can still waive a cost for a saving effect.

Existing Proton launch settings for ShroudForge also load this mod. No additional
key binding or external program is required. Disabling the mod restores normal
costs within one second. Restart after removing the folder to unload its hook.
No permanent game executable patch or save editing is performed.

## Implementation and limits

The current game calls its durability subtraction once for each successful
`SpawnEntityEvent`, outside that event's own projectile-count loop. Wands can
contain multiple such events in one attack. This mod intercepts just that call
site, recognizes nine primary wand sequence IDs from the original `ItemInfo`
resources, and suppresses repeated charges with the same engine attack identity.

The identity includes world session, actor, sequence, trigger time, engine call
index, weapon entity, and resolved durability component. There is no wall-clock
debounce. The first charge uses the original tracked engine operation, and later
charges skip that operation. Unknown sequences (including other weapon types),
unsupported images, unreadable state, and cache contention retain original costs.
Special skill attacks outside these nine primary sequences retain original costs.

The plugin checks the complete executable SHA-256, PE identity, unique instruction
signature, and original bytes before installing its hook. It reads settings and
world identity on a worker thread, not inside the game's callback. The callback
uses a bounded 128-entry journal and never dereferences stored pointers later.
The DLL and tiny hook allocation remain loaded until process exit so disabling
does not leave an executable jump into freed memory.

Source-derived inspection and unit tests cannot establish live gameplay behavior.
Native Windows gameplay, Proton gameplay, hosted co-op, and lifecycle cases must
be tested before treating this as a release. Dedicated-server support is pending.

Completed locally: original resource audit (58 wand items, nine primary attack
sequences, no other weapon types sharing them), exact-image hook signature and
call-target checks, per-attack policy tests, and instruction emulation of both
hook branches. Emulation verifies Windows x64 callback stack alignment, shadow
space, flags, general-purpose registers, and all sixteen XMM registers.

## Build

Use the repository's Wand Durability Windows Package workflow, or Visual Studio
with CMake on Windows x64. Initialize the pinned JSON submodule first.

```text
cmake -S mods/wand-durability/native -B build/wand-durability -A x64
cmake --build build/wand-durability --config Release
ctest --test-dir build/wand-durability -C Release --output-on-failure
python dev/build/package_wand_durability.py --dll build/wand-durability/Release/wand-durability.dll --out build/wand-durability-0.1.0.zip
```

On Linux the same CMake project builds the portable policy tests and the hook
byte exporter. Hook helpers are unchanged from the Autorun branch at `b174ac9`.
They originate in ShroudForge under the MIT license. The JSON dependency retains
its separate MIT license.
