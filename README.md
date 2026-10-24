# Enshrouded Autorun

An experimental client-side autorun mod, developed on a fork of
[ShroudForge](https://github.com/bonsaibauer/shroudforge).

**Status: input diagnostic loaded under Proton and F8 presses observed. Autorun
is not implemented yet.** There is no verified gameplay release. Build
artifacts are development packages.

The first version will toggle normal forward movement with F8, equivalent to
holding the game's forward control. It will preserve normal steering, collision,
jumping, and manually activated sprint. Backward movement and leaving active
gameplay will cancel autorun.

Single-player under Steam Proton is the first test target. Client-only
multiplayer compatibility is required afterward and must be verified while
joining another host. Native Windows gameplay testing is also pending.

## Development base

- ShroudForge **1.7.6**, tag `v1.7.6-dev.40026840919`.
- Upstream commit `20e27dce23afa0e8416d0de716bba3993a1fad30`.
- Initial Enshrouded Steam build: **26082781**.
- Pinned parser, JSON, and SDL submodules are included through Git submodules.

The build profile recognizes the target executable. That does not establish a
working movement hook. The first runtime investigation will observe input
without changing movement.

See the [implementation and validation plan](docs/development/autorun.en.md).

The [Autorun Input Probe](mods/autorun-input-probe/README.md) checks F8 toggling,
backward-key cancellation, keybinding changes, and mod lifecycle through logs.
It does not move the character. Its local Lua checks pass, and the first Proton
launch recorded ON, OFF, and ON F8 toggles. World, focus, and backward-key tests
are still pending. The optional loader windows were disabled locally after the
Debug Console failed to start through WebView2.

## Windows builds without a local Windows installation

Open [Actions → Autorun Windows Test Build](https://github.com/Tellik/enshrouded-autorun/actions/workflows/autorun-windows.yml)
and select **Run workflow** on `main`. GitHub builds the Windows binaries, runs
the upstream build tests, and stores a ZIP plus checksum as a downloadable
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
