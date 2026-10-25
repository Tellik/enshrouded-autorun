# Wand Durability

Wand Durability makes a primary wand attack take its normal durability cost
once, even when Multishot adds projectiles. It is an experimental native mod
for the unmodified ShroudForge 1.7.6 dev build 40026840919 and exact Enshrouded
client build 26082781 or dedicated-server build 26083076.

Download the [v0.2.0 prerelease](https://github.com/Tellik/enshrouded-mods/releases/tag/wand-durability-v0.2.0)
and follow the [installation guide](mods/wand-durability/README.md). The
release ZIP contains only Wand Durability; download ShroudForge separately.

Install on your **local client** for single-player, on the **host's client**
for player-hosted co-op, or on the **dedicated server** for all players there.
The same ZIP works in each role. A client copy automatically stays inactive
when connected to a remote world, so it can remain installed for single-player
use. Installing only on a client cannot fix an unmodded dedicated server.

Version 0.2.0 passed a two-player dedicated-server test with the Cryptic Wand's
built-in Multishot +1, including a player who did not install the client mod.
Both saw one durability lost per attack. Earlier 0.1.0 single-player Proton
testing also passed. Other wands, skill-generated Multishot, player-hosted
co-op, and lifecycle cases still need gameplay testing; this release is
marked as a prerelease.

Other projects are listed on the repository's
[main branch](https://github.com/Tellik/enshrouded-mods).
