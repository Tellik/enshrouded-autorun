# Enshrouded Autorun

An experimental client-side autorun mod for
[ShroudForge](https://github.com/bonsaibauer/shroudforge). The independently
installable package works with **unmodified ShroudForge 1.7.6**; the fork also
retains the earlier movement proof of concept and its diagnostics.

**Status: the standalone package has passed single-player and dedicated-server
Proton gameplay tests on stock ShroudForge 1.7.6.** A changed F7 binding
started movement, a second F7 stopped it, and S canceled it; the game stayed
stable in both tests. The dedicated server had no Autorun installation. The
standalone package is disabled by default. Menu, focus-loss, death, travel,
native Windows, and multiplayer sessions with other active players still need
testing.

A second test began by removing the previous loader and mods, then installing
stock ShroudForge and Autorun directly from their ZIPs. With the guide's Proton
settings and bundled mods disabled, the packaged default F8 toggle and S stop
worked in both single-player and a dedicated-server world. No WebView2 error
appeared; the game and loader files retained their expected hashes.

The default hotkey is **F8**, but it can be changed in the Modloader or in
`mods/autorun/extended.mod.json`. The first press starts normal forward
movement; another press or backward input stops it. The native request also
expires if the game loses focus, the world session changes, or the local input
disappears. Menu, death, and travel behavior still need live verification.

Download the separate `autorun-0.2.0.zip` build artifact from the
[verified standalone Windows build](https://github.com/Tellik/enshrouded-autorun/actions/runs/40124704227),
and follow the [standalone guide](standalone/autorun/README.md), which has
separate Windows and Linux/Proton installation steps and hotkey instructions.
It installs into the game's `mods` directory; no ShroudForge runtime files or
server files need replacing.

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
and select **Run workflow** on `autorun-poc`. Select `standalone_only` for the
independent mod ZIP and checksum without rebuilding the ShroudForge fork. The
full workflow additionally runs the native ECS and upstream build tests and
packages the fork. Artifacts are available for seven days; this workflow does
not publish GitHub releases.

The standalone ZIP contains only Autorun and its own plugin. The full fork
package includes upstream bundled mods and their defaults; disable those mods
for baseline testing. In particular, World Editor already uses F8.

The inherited upstream release and documentation workflows are retained for
reference and are disabled in this fork.

## Attribution

ShroudForge is created by bonsaibauer and its contributors. Its source history,
[MIT license](LICENSE), and [original README](README.shroudforge.md) are preserved.
The original README describes upstream ShroudForge, including upstream features
and releases; those are not autorun compatibility claims.

Enshrouded game binaries, assets, saves, local logs, and Steam account data are
not part of this project.
