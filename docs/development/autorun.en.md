# Autorun implementation and validation

## Target behavior

F8 toggles normal forward movement on and off. Movement should follow the game's
normal forward-control direction, with steering and other player actions still
available. The mod must not force sprint or the game's slower Walk action.
Backward movement cancels autorun. Menus, chat, focus loss, death, teleport,
disconnect, and mod unload must cancel it. Unknown gameplay state fails closed.

Single-player on Proton is the proof of concept. A client-only installation when
joining another host is a required later milestone. A successful host or local
world test alone is insufficient to claim multiplayer compatibility. Native
Windows gameplay remains a separate verification target.

## Verified starting point

- Source: upstream `v1.7.6-dev.40026840919`, commit
  `20e27dce23afa0e8416d0de716bba3993a1fad30`.
- The supplied source archive was compared with the Git checkout: all 1,700
  regular source files matched before this fork's setup changes.
- The compiled upstream ZIP's SHA-256 is
  `4496e2f21254da844d970b6f9056e617cf5b9ab23e77d62a411cdebde766d0dc`,
  matching the digest published for the release asset.
- Target executable: Steam build `26082781`, SHA-256
  `d64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882`.
  PE timestamp `1792507660` and image size `44163072` also match the shipped
  client profile.
- This profile does not declare the older building-input runtime operations.
  Historical movement offsets from June 2026 are not verified for this target.
- ShroudForge 1.7.6 uses declared `control: "keybind"` settings and
  `shroudforge.input.is_keybind_down(settingKey)`. The old `is_key_down` API
  was removed. SDL keyboard observation runs in a helper process, so Proton
  focus and input behavior need live verification.
- World Editor binds F8 by default. Disable bundled gameplay mods for the
  baseline and resolve the conflict before testing an autorun hotkey.

## Work sequence

1. Load the supplied compiled ShroudForge release under Proton. Keep bundled
   gameplay mods disabled, record startup diagnostics, and verify the loader
   and runtime initialize. The optional windows may be disabled if WebView2
   cannot start. Keep the test version fixed during investigation.
2. Add an input-only diagnostic mod. Verify one toggle per key press, key release,
   configurable bindings, focus loss, and reload behavior. This diagnostic must
   not move the character.
3. Trace the native input producer and consumer for the exact game build. Use
   read-only observations of idle, forward, backward, strafe, jump, and sprint
   input to establish axes, timing, normalization, and gameplay gating.
4. Add a bounded forward-input request at a verified point after physical input
   collection and before movement consumption/recording/networking. The request
   needs an owner, timeout, explicit release, and cleanup on lifecycle changes.
5. Expose the request through the existing native bridge and Lua runtime. Add the
   autorun mod and test the single-player behavior and cancellation cases.
6. Test joining another host and a dedicated server, including lag, death, fast
   travel, disconnect, and reload. Confirm no server mod or save edits are needed
   before describing the mod as client-only multiplayer compatible.

## Ownership

Native input integration belongs in `modules/game-runtime/native/player-input/`
and its validated hook/profile integration. Rust adapters belong in the existing
`modules/game-runtime/bridge/` structure. Lua bindings belong in
`modules/mod-runtime/lua/`. The eventual mod belongs directly under `mods/`.
Choose concrete interfaces after tracing their real callers and lifecycle.

Build orchestration uses `.github/workflows/autorun-windows.yml` and the existing
`dev/build/release_build.ps1`. It runs the native ECS test, upstream Windows
bootstrap, and Rust workspace tests before packaging. Successful compilation
is not an in-game compatibility test.

## Current status

Repository setup and a manual Windows build workflow are present. An input-only
diagnostic is available under `mods/autorun-input-probe/`. Lua syntax, manifest
structure, and a mocked lifecycle check passed. The behavior check covered
holding a key, repeat presses, backward-key priority, activation while a key is
held, live rebinding, and unload reset. It does not verify the native input API
or real focus behavior.

A local baseline uses the supplied compiled release, omits bundled gameplay
mods, enables only the probe, and disables automatic updates and the loader's
network module. Installation records and logs stay outside this public source
checkout. The game executable and KFC files were hashed before and after
installation and were unchanged.

The first Proton launch reached `Runtime initialized` and loaded the probe.
The SDL input helper reported ready. The first F8 press produced one ON toggle;
later presses produced OFF and ON. The optional Debug Console failed with
WebView2 `HRESULT(0x80070002)` (file not found). Local test settings now disable
the Debug Console and Modloader UI, which are separate from the game runtime.
On the next launch, the windows were skipped and runtime startup completed
without that WebView2 error. In a single-player world, F8 switched the probe
ON and S canceled it. Background-focus behavior has not been confirmed.
After world entry, the native snapshot reported a ready ECS registry and
guarded local-authority entity access, while `game_thread=unavailable` and the
general operation profile remained unavailable. This does not establish a
usable native input hook.

The installed game produced a type registry for this exact build. Its reflected
`ClientPlayerInputData` has a 952-byte size, with `movementStick` at offset 696
and `digitalInput` at 936. These differ from older layouts and are evidence for
read-only sampling only. The probe now attempts bounded reads of the movement
stick through the existing ECS API. A single-player test observed idle `(0, 0)`,
forward `(0, +1)`, backward `(0, -1)`, and a normalized diagonal sample near
`(-0.71, +0.71)` in the authority world's `PlayerInput` component. Its regular
ECS query found no `ClientPlayerInput` in that owner. These are observations,
not evidence that the authority component is a safe place to write or that a
client-only installation will work in multiplayer.

The existing native reader can resolve the separate main-client ECS owner and
copy its unique `ClientPlayerInput` without invoking engine functions. This
fork adds a read-only Lua `runtime.world.player_input.get()` wrapper around that
reader, returning the reflected movement vector, sequence, and world session.
The Windows build and tests passed, and a single-player Proton test then read
the client input live. Client and local-authority samples agreed at idle
`(0, 0)`, forward `(0, +1)`, and backward `(0, -1)`. F8 toggled the diagnostic
flag and S canceled it. The native reader briefly reported unavailable while
the world initialized, then found the unique local input. These samples confirm
the local client field and normal direction; they do not prove a safe write
phase or multiplayer delivery. A later movement request must use a verified
producer-to-consumer hook rather than write this snapshot.

An offline inspection of the exact `26082781` executable identified a likely
client input producer at RVA `0x2437c0`. Its two direct callers resolve the
`keen::ecs::ClientPlayerInput` component by the qualified type hash recorded
in reflection. The producer writes the reflected `movementStick` fields at
offsets `0x2b8` and `0x2bc` after processing ordinary directional input.
This identifies a candidate for a future native hook; live ownership, update
timing, and the subsequent network path still require validation before it
can safely change movement.

The read-only exact-build hook subsequently ran in a single-player Proton world.
After keeping the fully validated local input through bounded periodic rescans,
the second test recorded 281 hook calls, 276 local-player matches, and zero
writes. Its raw input changed from idle `(0, 0)` to W `(0, +1)` and S `(0, -1)`;
S stopped the diagnostic. This supports the hook's timing and local identity,
but does not yet prove that a modified vector reaches gameplay or networking.

The next source stage added a 500 ms renewed native request, a one-second hook
and local-input activity limit, exact-build and world/focus guards, native
backward cancellation, and the `mods/autorun/` F8 toggle. The mod is disabled
by default. The Windows build and repository tests passed in
[run 40114984834](https://github.com/Tellik/enshrouded-autorun/actions/runs/40114984834).
The first single-player Proton test visibly moved the character at about the
normal W speed after F8. A second F8 stopped it, and S canceled a later run;
the game stayed stable. The native log counted 74 of 74 local hook writes
before toggle-off and 181 writes before the S stop. The Lua keybind API still
has no independent gameplay menu/death query. Focus loss, menus, death, travel,
native Windows, and joining a multiplayer host remain unverified and cannot
yet be claimed as supported.

The first dedicated-server test stayed stable but F8 did not move the character.
The main-client ECS manager contained multiple `ClientPlayerInput` entities,
so the single-player uniqueness rule rejected the request. A subsequent
read-only multiplayer trace observed 1,649 movement-producer calls on exactly
one input address. That input followed W as `(0, +1)`, S as `(0, -1)`, and
release as `(0, 0)`. The current implementation matches that observed input
address against the component addresses of all ECS candidates and fails closed
if the producer changes input addresses, the session changes, or the ECS match
is missing or ambiguous. The Windows native ECS check and upstream tests passed
in [run 40119687161](https://github.com/Tellik/enshrouded-autorun/actions/runs/40119687161).
With only the client mod installed, the next dedicated-server Proton test moved
the character forward after F8 and stayed stable. The first activation recorded
115 local hook matches and 114 writes before S canceled it. The second recorded
337 local matches and writes before F8 toggle-off. This verifies basic
client-only movement delivery and both stops on one dedicated-server session.
Menus, focus loss, death, travel, disconnect, native Windows, and sessions with
other active players remain unverified.

## Standalone stock-ShroudForge package

The fork-integrated native request above is a proof of concept, not a runtime
dependency for the separately installable mod. `standalone/autorun/` now builds
a package-local `autorun.dll` loaded through stock ShroudForge's
`native-plugin.ini` mechanism. It checks the stock ABI-12 world exports and the
exact Enshrouded image before installing its own input hook. The mod reads the
ShroudForge keybind settings from its own `extended.mod.json`, with F8 and S as
defaults. It also accepts supported alternate keys and fails closed on invalid
settings or an unsupported game image. The ZIP contains no replacement
ShroudForge runtime DLLs.

[Windows build 40122879851](https://github.com/Tellik/enshrouded-autorun/actions/runs/40122879851)
compiled and packaged the independent plugin. The ZIP layout and checksum
passed verification. Its DLL imports only Windows system libraries and no
dynamic MSVC C++ runtime. In the local install test, both installed ShroudForge
runtime DLL hashes matched the supplied stock 1.7.6 release. The stock loader
reported one native plugin and no load issues. In a single-player Proton world,
the configured F7 toggle started normal forward movement, a second F7 stopped
it, and S canceled it; the game stayed stable. The plugin log recorded local
input writes and both stop reasons. A subsequent dedicated-server Proton test
with the same stock runtime and package again moved the character forward,
stopped on F7, and canceled on S without a crash. The standalone plugin log
recorded 55 local writes before F7 toggle-off and 57 before S cancellation.
No Autorun installation was needed on the dedicated server. This verifies
basic client-only delivery in one server session; other active players and
lifecycle cases remain untested.

## Clean ZIP installation verification

The next check moved the existing ShroudForge proxy DLLs, loader directory,
and mods directory to a verified backup outside the game folder. The game
executable and both KFC data files matched the originals captured before
ShroudForge's first managed start. The supplied stock ShroudForge
`1.7.6-40026840919` ZIP was extracted beside `enshrouded.exe`, and the
standalone `autorun-0.2.0.zip` was extracted inside `mods`. Every unmodified
installed file matched its archive byte-for-byte. The old input probe and
fork-integrated Autorun were absent. The package's default F8 binding was
retained. All five bundled ShroudForge mods were left disabled to isolate
Autorun and avoid World Editor's F8 shortcut. The Proton setup disabled the
Modloader and Debug Console windows before first launch because WebView2 was
unavailable, and disabled automatic ShroudForge updates to hold the tested
runtime version.

Both a single-player world and a dedicated-server world started without a
WebView2 error and remained stable. F8 started forward movement, another F8
stopped it, and S canceled the next run in each world. The plugin log recorded
68 local writes before toggle-off and 58 before S cancellation in single-player;
the dedicated-server test recorded 53 and 54. The stock loader reported the
Autorun DLL loaded, and its runtime initialized. The game executable, both
KFC data files, and both stock ShroudForge runtime DLLs retained their original
hashes. The standalone installation guide now includes the Proton pre-launch
settings and the F8 conflict with World Editor found during this check.
The corrected standalone ZIP and checksum passed packaging in
[Windows build 40124296476](https://github.com/Tellik/enshrouded-autorun/actions/runs/40124296476).
Its compiled plugin differed from the gameplay-tested DLL only in PE
timestamps; the code bytes matched.
