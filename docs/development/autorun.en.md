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
`dev/build/release_build.ps1`. It runs the upstream Windows bootstrap and Rust
workspace tests before packaging. Native C++ test executables are separate
targets and will need explicit execution when the movement integration is added.
Successful compilation is not an in-game compatibility test.

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
