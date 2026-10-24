# Enshrouded Autorun

An experimental client-side autorun mod, developed on a fork of
[ShroudForge](https://github.com/bonsaibauer/shroudforge).

**Status: the F8 movement proof of concept works in a single-player Proton
world and when joining a dedicated server with only the client mod installed.**
Toggle-off and S cancellation worked without a crash in both tests. This is
still a development build, and the autorun mod is disabled by default in the
package. Menu, focus-loss, death, travel, native Windows, and multiplayer
sessions with other active players still need testing.

The first version toggles normal forward input with F8, equivalent to holding
the game's forward control. Pressing F8 again or moving backward cancels it.
The native request also expires if the mod stops renewing it, the game loses
focus, the world session changes, or the local input disappears. Menu, death,
and travel behavior still need live verification.

The first dedicated-server test confirmed client-only movement delivery without
a server mod. Broader multiplayer compatibility, including other active players
and lifecycle cases, and native Windows gameplay testing are still pending.

## Development base

- ShroudForge **1.7.6**, tag `v1.7.6-dev.40026840919`.
- Upstream commit `20e27dce23afa0e8416d0de716bba3993a1fad30`.
- Initial Enshrouded Steam build: **26082781**.
- Pinned parser, JSON, and SDL submodules are included through Git submodules.

The input hook is limited to this exact executable build. A game update requires
fresh validation before enabling it again.

See the [implementation and validation plan](docs/development/autorun.en.md).

The [Autorun Input Probe](mods/autorun-input-probe/README.md) does not move the
character. In a single-player Proton test, 276 of 281 hook calls matched the
verified local input. It observed W as `(0, +1)` and S as `(0, -1)`, with zero
movement writes. The subsequent [Autorun mod](mods/autorun/) test visibly
moved the character at ordinary forward speed with F8 and stopped on F8/S.
Its native log counted 74 local writes before toggle-off and 181 before S.
The dedicated-server build then recorded 114 local writes before S cancellation
and 337 before F8 toggle-off; the character moved forward and the game stayed
stable. This build passed the native ECS selection test and upstream Windows
tests in [run 40119687161](https://github.com/Tellik/enshrouded-autorun/actions/runs/40119687161).

## Windows builds without a local Windows installation

Open [Actions → Autorun Windows Test Build](https://github.com/Tellik/enshrouded-autorun/actions/workflows/autorun-windows.yml)
and select **Run workflow** on `autorun-poc`. GitHub builds the Windows binaries, runs
the native ECS check and upstream build tests, and stores a ZIP plus checksum as a downloadable
artifact for seven days. This workflow does not publish GitHub releases.

The workflow uses ShroudForge's existing build and packaging scripts. Its package
currently includes the upstream bundled mods and their defaults. For baseline
testing, disable those mods before entering a world. In particular, World Editor
already uses F8. The supplied upstream binary release can be used for the first
Proton loader test without compiling this fork.

The inherited upstream release and documentation workflows are retained for
reference and are disabled in this fork.

## Attribution

ShroudForge is created by bonsaibauer and its contributors. Its source history,
[MIT license](LICENSE), and [original README](README.shroudforge.md) are preserved.
The original README describes upstream ShroudForge, including upstream features
and releases; those are not autorun compatibility claims.

Enshrouded game binaries, assets, saves, local logs, and Steam account data are
not part of this project.
