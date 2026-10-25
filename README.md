# Enshrouded Mods

This is the landing page for Enshrouded mod projects. Choose a branch for that
project's source and instructions. [Releases](https://github.com/Tellik/enshrouded-mods/releases)
are shared across the repository; download the ZIP asset named for the mod,
not GitHub's automatically generated source archive.

| Branch | Project | Where to start |
| --- | --- | --- |
| [`autorun`](https://github.com/Tellik/enshrouded-mods/tree/autorun) | Client-side Autorun mod with configurable hotkey | [Installation guide](https://github.com/Tellik/enshrouded-mods/blob/autorun/README.md) · [v0.3.0 prerelease](https://github.com/Tellik/enshrouded-mods/releases/tag/v0.3.0) (`autorun-0.3.0.zip`) |
| [`wand-durability`](https://github.com/Tellik/enshrouded-mods/tree/wand-durability) | Experimental: one normal durability cost per primary wand attack, including Multishot bonus projectiles | [Testing guide](https://github.com/Tellik/enshrouded-mods/blob/wand-durability/mods/wand-durability/README.md) · [Test builds](https://github.com/Tellik/enshrouded-mods/actions?query=branch%3Awand-durability) · No release yet |
| [`1.7.6`](https://github.com/Tellik/enshrouded-mods/tree/1.7.6) | Unmodified ShroudForge 1.7.6 source snapshot | [ShroudForge README](https://github.com/Tellik/enshrouded-mods/blob/1.7.6/README.md) |

Autorun requires the **unmodified** [ShroudForge 1.7.6 dev release](https://github.com/bonsaibauer/shroudforge/releases/tag/v1.7.6-dev.40026840919),
downloaded from the upstream release. The `1.7.6` branch is a source reference;
it is not a separate runtime package. Autorun's current package supports
Enshrouded Steam build `26082781`. The Autorun branch has separate Windows and
Linux/Proton installation steps; gameplay has been tested on Linux/Proton in
single-player and as a dedicated-server client. Native Windows gameplay
remains untested.

Wand Durability uses the same stock ShroudForge 1.7.6 runtime and has separate
exact-build profiles for client build `26082781` and dedicated-server build
`26083076`. The same [0.2.0 test package](https://github.com/Tellik/enshrouded-mods/actions/runs/40233421427)
can be installed on a client or server. It automatically runs where the world
is authoritative: in single-player, on a player host, or on a dedicated server.
A server installation applies the correction to every player; a client copy
stays inactive while connected to a remote server. The Windows build, policy
tests, and native hook emulation pass. The Cryptic Wand's
built-in Multishot +1 was verified in single-player under Proton on 2026-10-25;
the plugin log recorded six allowed wand charges and six extra costs skipped
across 12 projectile-cost calls with 0.1.0. The 0.2.0 server hook loaded in a
local test server and reached an authoritative world, but no player connected.
Version 0.2.0 gameplay, other wand/skill cases, hosted co-op, dedicated-server
attacks, and lifecycle behavior remain unverified. Test packages are disabled
by default and are **not a verified release**. A release will wait for gameplay
testing.

The `main` branch contains this landing page, the [MIT license](LICENSE), and
the entry point for Autorun's manually triggered Windows package workflow.
