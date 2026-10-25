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

Wand Durability uses the same stock ShroudForge 1.7.6 runtime and currently
targets client build `26082781` in single-player or player-hosted co-op worlds.
Local policy tests and native hook emulation pass; gameplay testing is pending.
Its test packages are disabled by default and are **not a verified release**.
A client-only install cannot change a dedicated server's durability accounting;
a verified dedicated-server executable profile is still needed. A release will
wait until gameplay testing is verified.

The `main` branch contains this landing page, the [MIT license](LICENSE), and
the entry point for Autorun's manually triggered Windows package workflow.
