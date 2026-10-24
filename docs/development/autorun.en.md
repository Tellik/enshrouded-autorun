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
   and debug windows open. Keep the test version fixed during investigation.
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

Repository setup and a manual Windows build workflow are present. No autorun
movement code, game-memory writes, gameplay tests, or multiplayer validation have
been completed. The upstream binary package can serve as the baseline without
waiting for a custom hosted build.
