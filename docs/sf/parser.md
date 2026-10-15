# ShroudForge Parser

The `src/parser/` crate converts Enshrouded data into snapshots consumed by the
loader API and compatibility modules. It also owns parser status and the KFC
asset transaction adapter.

The KFC parser is pinned as an upstream submodule at
`src/parser/kfc-parser/`. The Rust packages also pin an upstream Git revision.
After changing the submodule, publish that revision and update the Cargo pins
before the parser and modloader consume the change. ShroudForge's parser
adapter and snapshot logic remain in `src/parser/`.

Cargo workspace membership and the root build are defined in `Cargo.toml` and
`build.ps1`.
