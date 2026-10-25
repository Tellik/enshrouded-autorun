# Wand Durability

Experimental ShroudForge mod that charges durability once per primary wand
attack, including built-in Multishot and skill-generated bonus projectiles.

See the [installation and testing guide](mods/wand-durability/README.md).
The source and build instructions are in this branch. Automated build artifacts
are test packages, not a validated gameplay release.

Currently targets Enshrouded client build 26082781 and dedicated-server build
26083076 with stock ShroudForge 1.7.6.
The earlier 0.1.0 client's Cryptic Wand test passed in single-player under
Proton on 2026-10-25; its log recorded six allowed wand charges and six extra
costs skipped across 12 projectile-cost calls. Version 0.2.0 and other wands,
the Multishot skill, player-hosted co-op, dedicated-server play, and lifecycle
cases still need gameplay testing. Install the mod on the authoritative host or
dedicated server for multiplayer; a client-only install cannot alter a dedicated
server's durability accounting. The server profile is binary-inspected but not yet tested
in a live server session.

Other projects are listed on the repository's
[main branch](https://github.com/Tellik/enshrouded-mods).
