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
costs skipped across 12 projectile-cost calls. Version 0.2.0 was then tested
on a Wine dedicated server: two connected players each observed one durability
lost per Cryptic Wand Multishot attack. The second player had no client mod.
The server's recognized wand counters rose to six allowed charges and six
extra costs skipped; the first player's client copy reported a remote world
and stayed inactive. Install the mod on the authoritative host or dedicated
server for multiplayer; its correction then applies to every player in that
world. A client-only install cannot alter a dedicated server's costs. Other
wands, the Multishot skill, player-hosted co-op, and lifecycle cases still
need gameplay testing.

Other projects are listed on the repository's
[main branch](https://github.com/Tellik/enshrouded-mods).
