# Enshrouded Mods

This is the landing page for Enshrouded mod projects. Choose a branch for that
project's source and instructions. [Releases](https://github.com/Tellik/enshrouded-mods/releases)
are shared across the repository; download the ZIP asset named for the mod,
not GitHub's automatically generated source archive.

| Branch | Project | Where to start |
| --- | --- | --- |
| [`autorun`](https://github.com/Tellik/enshrouded-mods/tree/autorun) | Client-side Autorun mod with configurable hotkey | [Installation guide](https://github.com/Tellik/enshrouded-mods/blob/autorun/README.md) · [v0.3.0 prerelease](https://github.com/Tellik/enshrouded-mods/releases/tag/v0.3.0) (`autorun-0.3.0.zip`) |
| [`wand-durability`](https://github.com/Tellik/enshrouded-mods/tree/wand-durability) | Experimental: one normal durability cost per primary wand attack, including Multishot bonus projectiles | [Client and server installation guide](https://github.com/Tellik/enshrouded-mods/blob/wand-durability/mods/wand-durability/README.md) · [v0.2.0 prerelease](https://github.com/Tellik/enshrouded-mods/releases/tag/wand-durability-v0.2.0) (`wand-durability-0.2.0.zip`) |
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
`26083076`. Install the same ZIP on your local client for single-player, on
the host's client for player-hosted co-op, or on the dedicated server for all
players there. Connecting clients do not need it; a client copy automatically
stays inactive in a remote world. The v0.2.0 prerelease is disabled by default.
Two players verified one durability lost per Cryptic Wand built-in Multishot
attack on a Wine dedicated server, including one without the client mod. An
earlier 0.1.0 single-player Proton test also passed. Other wands, the
Multishot skill, hosted co-op, and lifecycle behavior remain unverified.

The `main` branch contains this landing page, the [MIT license](LICENSE), and
the entry point for Autorun's manually triggered Windows package workflow.
