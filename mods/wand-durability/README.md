# Wand Durability v0.2.0

Apply the normal single-shot durability cost once per primary wand attack,
including attacks with built-in Multishot or skill-generated bonus projectiles.
This is normally one durability per attack. The game still controls projectile
count, damage, durability-saving effects, and the world's durability setting.

This experimental native mod requires the **unmodified ShroudForge 1.7.6 dev
build 40026840919**. It supports the exact Enshrouded **client build
26082781** and **dedicated-server build 26083076**. The mod stays inactive if
the executable or ShroudForge ABI does not match.

## Download

- [Wand Durability v0.2.0 prerelease](https://github.com/Tellik/enshrouded-mods/releases/tag/wand-durability-v0.2.0): download `wand-durability-0.2.0.zip`. Its `.sha256` asset is a checksum.
- [ShroudForge 1.7.6 dev release](https://github.com/bonsaibauer/shroudforge/releases/tag/v1.7.6-dev.40026840919): download `shroudforge-1.7.6-40026840919.zip` separately.

Use the ZIP assets, not GitHub's generated source archives. The Wand Durability
ZIP contains one `wand-durability` folder and no game files or ShroudForge
runtime. The packaged mod is **disabled by default**; enable it after copying.

## Which machine needs the mod?

| World | Install and enable on | Connecting players |
| --- | --- | --- |
| Single-player | Your local Enshrouded client | None |
| Player-hosted co-op | The host's Enshrouded client | Guests do not need it; hosted co-op gameplay is not yet verified |
| Dedicated server | The dedicated server | No client install needed; tested with and without the client mod |
| A remote world without the mod on its host/server | Install on that host/server to gain the correction | A client-only copy stays inactive and cannot change remote durability |

Use the **same ZIP** for client and server. If you play single-player and also
join a dedicated server, you may leave it installed and enabled on your client:
it works in your local world and automatically stays inactive in the remote
world. If that server has the mod, its installation applies to every player.
No per-player handshake or manual switching is required.

## Install on a local client

These steps apply to Windows and Linux/Proton. They are required for
single-player, or for the player **hosting** a co-op world. Guests joining that
world do not need to install the mod.

1. Close Enshrouded. Open its game folder, the one containing
   `enshrouded.exe`. In Steam, use **Manage → Browse local files**.
2. If ShroudForge is not installed, extract the **entire ShroudForge ZIP** into
   this folder. Its `dbghelp.dll`, `dinput8.dll`, `winmm.dll`, `shroudforge`,
   and `mods` entries must sit beside `enshrouded.exe`. ShroudForge's ZIP also
   includes other gameplay mods; disable any you do not intend to use.
3. Extract the Wand Durability ZIP **inside** `mods`. Confirm the resulting
   files include `mods/wand-durability/mod.json` and
   `mods/wand-durability/bin/wand-durability.dll`. Do not leave the ZIP in
   `mods` or add another folder level.
4. In `mods/wand-durability/extended.mod.json`, change `"enabled": false` to
   `"enabled": true`, then start the game. If replacing an earlier test
   package, save your enabled setting first; extracting this ZIP resets it to
   `false`.

On **Linux/Proton**, set these Steam launch options so Wine loads ShroudForge's
local DLLs:

```text
WINEDLLOVERRIDES="dbghelp=n,b;winmm=n,b" %command%
```

If the ShroudForge Modloader window needs WebView2 and does not open, you can
use the file setting above without the UI. While the game is closed, set
`modules.modloaderUi.enabled` and `modules.debugConsole.enabled` to `false`
in `shroudforge/config/modloader-config.json`. This leaves the native mod
loaded. On Windows, the ShroudForge UI can also enable the mod, but restart the
game afterward so its native DLL loads.

## Install on a dedicated server

Install on the **server machine**, in the folder containing
`enshrouded_server.exe`. Installing only on a connecting client cannot change
that server's durability costs. Players joining a modded server do not need
ShroudForge or Wand Durability on their own clients.

1. Stop the server using its normal control panel or service manager, and back
   up the world. Keep it stopped while copying files.
2. If ShroudForge is not installed, extract the **entire ShroudForge ZIP**
   beside `enshrouded_server.exe`. Disable the bundled gameplay mods you do
   not intend to run; some are enabled in the stock ZIP.
3. Extract the Wand Durability ZIP into the server's `mods` folder, yielding
   `mods/wand-durability/bin/wand-durability.dll`. Set `"enabled": true` in
   `mods/wand-durability/extended.mod.json`.
4. On a **Wine/Proton server**, set the DLL override for the game process.
   For a launch script that runs Wine directly, the game launch can be:

   ```text
   WINEDLLOVERRIDES="dbghelp=native,builtin;winmm=native,builtin" wine ./enshrouded_server.exe
   ```

   Add the setting to your existing Pterodactyl/Docker launch configuration
   if it starts Wine another way. A native Windows server does not need this
   override.
5. Start the server using its normal control panel. No client installation is
   needed for the server-wide correction.

## Check operation and update

In `mods/wand-durability/wand-durability.log` on the machine that owns the
world, look for `Exact-build client durability hook installed` or
`Exact-build dedicated-server durability hook installed`, followed by a
nonzero `session` and `world_kind=1`. A client connected to a remote world
instead reports `session=0 world_kind=2`; its copy makes no durability changes.
`wand_charges` counts allowed wand charge calls and `bonus_costs_skipped`
counts extra calls suppressed. The original game may still waive a charge for
a durability-saving effect.

Without a saving effect, a Cryptic Wand attack should cost one durability
whether or not Multishot adds a projectile. Bonus projectiles should still
hit and deal damage. Other weapons retain the game's normal behavior. If the
log says the image or ABI is unsupported, check the game and ShroudForge
versions; the mod deliberately leaves costs unchanged on unknown builds.

To disable the correction, set `"enabled": false` in the mod's
`extended.mod.json`; it stops applying within one second. To remove the mod,
stop the game or server first and remove `mods/wand-durability`. No permanent
game executable patch or save edit is made.

## Tested behavior and remaining limits

The 0.1.0 client passed a single-player Proton test with the Cryptic Wand's
built-in Multishot +1. Version 0.2.0 passed on a Pterodactyl-hosted Wine
dedicated server: two connected players each observed one durability lost per
Cryptic Wand Multishot attack, including a player without the client mod.
The server's recognized wand counters reached six allowed charges and six
extra costs skipped. The first player's installed client copy reported a
remote world and stayed inactive.

Native Windows gameplay, player-hosted co-op, other wands, skill-generated
Multishot, and travel/death or world-rejoin cases remain unverified. This is
why v0.2.0 is marked as a prerelease.

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
The 0.1.0 single-player Proton and two-player 0.2.0 dedicated-server tests
above are the live gameplay validation so far. Native Windows gameplay,
hosted co-op, other wand/skill cases, and lifecycle cases remain unverified.

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
