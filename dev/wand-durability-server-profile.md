# Dedicated-server hook evidence (experimental 0.2.0)

The official Windows Enshrouded Dedicated Server Steam app `2278520`, build
`26083076`, was downloaded on 2026-10-25 for offline inspection. Its
`enshrouded_server.exe` SHA-256 is
`d7baa768eb00aa895d9f5a1cd8747ccf66858c7b542f50983092b2e3261588ee`.
This matches the stock ShroudForge 1.7.6 server compatibility profile. PE
timestamp `1792507668` and image size `31264768` are additional gates.

The original per-projectile durability call site is RVA `0x430a0`; the next
instruction is RVA `0x430b1`. The exact 20-byte signature is unique in the
server image:

```text
41 B8 FF FF FF FF 48 8B CB 44 8B 08 E8 2F 85 11 00 48 8D 3D
```

The first six bytes are the displaced `mov r8d, -1`. The call immediately
following them resolves to RVA `0x15b5e0`. That helper's first 20 bytes are
identical to the working client durability helper at RVA `0x3af640`. The
surrounding server code has the same actor binding at `[rbp+0x390]`, durability
component in `rbx`, actor entity at `[rax]`, and call loop as client build
`26082781`. The server branch invokes the same generated hook code; only the
image identity, call-site signature, hook RVA, and skip RVA differ.

The stock runtime's `KfcRuntimeWorldContextKind` classifies a validated local
authority world as `1`; the mod also requires a nonzero world session. A remote
client reports a read-only world as `2` and does not suppress costs. The
`extended.mod.json` target list includes client and server, but both are
disabled by default in the distributable package.

The verified 0.2.0 Windows package was loaded into a local, localhost-bound
test server with stock ShroudForge 1.7.6 on 2026-10-25. Its mod log reported
`Exact-build dedicated-server durability hook installed` and then
`enabled=1 session=5 world_kind=1`. No player connected, so this validates
startup and the authority gate, not the durability effect. The hook must still
be tested with a connected player's Multishot wand before a release claim.
Other server builds fail closed.
