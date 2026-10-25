# Wand Durability — experimental 0.2.0

Apply the normal single-shot durability cost once per primary wand attack,
regardless of built-in Multishot or skill-generated bonus projectiles. Normally
this is one durability. Projectile count, timing, chance, damage, existing
durability-saving effects, and the world's durability setting remain controlled
by the game.

This is an experimental native plugin for stock ShroudForge 1.7.6, ABI 12,
Enshrouded **client build 26082781**, and **dedicated-server build 26083076**.
It is disabled by default. Exact executable hashes are required; unknown builds
stay inactive.

The earlier 0.1.0 client's Cryptic Wand Multishot +1 test passed in
single-player under Proton on 2026-10-25. Its log recorded six allowed wand
charges and six extra costs skipped across 12 projectile-cost calls. Version
0.2.0, other wands, the Multishot skill, hosted co-op, dedicated-server play,
and lifecycle cases remain untested in live gameplay.
The mod acts in whichever process owns the world. In a player-hosted world, that
is the host's client; on a dedicated server, install it on the server. One
server installation applies to every player in that world, whether or not
their client has the mod. A client copy stays inactive in a remote world.

**A client-only install cannot fix a dedicated server's durability accounting.**
Version 0.2.0 adds a separate, exact-build server hook profile. The official
server executable was inspected offline, and a local test server loaded the
hook and reached an authoritative world. A connected player's durability
effect still needs testing.

## Install and test

1. Stop the game or server. Extract the `wand-durability` folder into `mods`
   beside the relevant executable. Keep the stock ShroudForge 1.7.6 runtime
   installed there. For a dedicated server, back up the world before testing.
2. Set `enabled` to `true` in this mod's `extended.mod.json` and restart the
   process. The same ZIP targets client and server; each process selects its own
   exact executable profile.
3. In single-player, record the Cryptic Wand's durability, fire one complete
   attack, then check durability again. Without a durability-saving effect,
   each attack should cost exactly one whether it emits one, two, or more
   projectiles.
4. Repeat with the Multishot skill, several consecutive attacks, another wand,
   and a melee weapon or bow. Test a nearly broken wand, fast travel, death,
   and world leave/rejoin. Verify bonus projectiles still hit and deal damage.
5. For player-hosted co-op, install on the host and compare the host's and a
   guest's wand costs. For a dedicated server, install on the server, restart it,
   and test from a connected client. A client-side copy alone cannot change the
   server's costs.
6. Inspect `wand-durability.log` in the mod folder on the authoritative machine.
   Confirm the exact-build hook for the client or dedicated-server role, plus a
   nonzero `session` with `world_kind=1`. `wand_charges` counts original charge
   calls allowed through, and `bonus_costs_skipped` should increase on extra
   events. The original helper can still waive a cost for a saving effect.

Existing client Proton launch settings for ShroudForge also load this mod. A
separate Proton server must load ShroudForge in its own process; ShroudForge's
server guide documents the required DLL overrides for Proton Docker. No extra
key binding is required. Disabling the mod restores normal costs within one
second. Restart after removing the folder to unload its hook. No permanent game
executable patch or save editing is performed.

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
The 0.1.0 single-player Proton test above is the only live gameplay validation so far.
Native Windows gameplay, hosted co-op, dedicated-server play, other wand/skill
cases, and lifecycle cases remain unverified.

Completed locally: original resource audit (58 wand items, nine primary attack
sequences, no other weapon types sharing them), exact client and server image
hashes, unique hook signatures, original-byte and durability-helper checks,
per-attack policy tests, and instruction emulation of both hook branches.
Emulation verifies Windows x64 callback stack alignment, shadow space, flags,
general-purpose registers, and all sixteen XMM registers.

## Build

Use the repository's Wand Durability Windows Package workflow, or Visual Studio
with CMake on Windows x64. Initialize the pinned JSON submodule first.

```text
cmake -S mods/wand-durability/native -B build/wand-durability -A x64
cmake --build build/wand-durability --config Release
ctest --test-dir build/wand-durability -C Release --output-on-failure
python dev/build/package_wand_durability.py --dll build/wand-durability/Release/wand-durability.dll --out build/wand-durability-0.2.0.zip
```

On Linux the same CMake project builds the portable policy tests and the hook
byte exporter. Hook helpers are unchanged from the Autorun branch at `b174ac9`.
They originate in ShroudForge under the MIT license. The JSON dependency retains
its separate MIT license.
