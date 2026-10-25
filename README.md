# Wand Durability

Experimental ShroudForge mod that charges durability once per primary wand
attack, including built-in Multishot and skill-generated bonus projectiles.

See the [installation and testing guide](mods/wand-durability/README.md).
The source and build instructions are in this branch. Automated build artifacts
are test packages, not a validated gameplay release.

Currently targets Enshrouded client build 26082781 with stock ShroudForge 1.7.6.
The Cryptic Wand's built-in Multishot +1 was verified in single-player under
Proton on 2026-10-25; the plugin log recorded six allowed wand charges and six
extra costs skipped across 12 projectile-cost calls. Other wands, the Multishot
skill, player-hosted co-op, and lifecycle cases still need gameplay testing.
Dedicated-server support needs a separate verified executable profile. A
client-only install does not alter a dedicated server's durability accounting.

Other projects are listed on the repository's
[main branch](https://github.com/Tellik/enshrouded-mods).
