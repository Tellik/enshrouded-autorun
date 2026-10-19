# Integrate the KFC parser

`kfc-parser/` is the Git submodule itself. Rust consumers use its crates through local Cargo paths. There is no additional parser wrapper.

Format access and resource resolution live in `crates/kfc-base`. Existing codecs live in `crates/kfc-content` and `crates/kfc-resource`. `crates/kfc/src/schema.rs` and `snapshot.rs` contain schema projection and snapshot export. The submodule is pinned to upstream `main` at `2cf45e36cd4575fa3867761864a36e1883c82add`. This revision adds the ShroudForge parser and loader bridge, including the `mod-loader-bridge` crate.

After a fresh checkout, run `dev/parser/prepare_parser.ps1`. Both `dev/build/dev_build.ps1` and `dev/build/release_build.ps1` perform this preparation step too. The script accepts the current `main` pin and the previous integrated revision `742b1cb7` without applying a patch. For the legacy revision `34a156f6`, it retains support for `dev/parser/parser-integration.patch` and detects changes that have already been applied.

Game-file selection belongs to `modules/entrypoints/loader/game_files.rs`. Original files, parser state, and the persistent file transaction belong to `modules/game-files/`. The selected mod configuration fingerprint is calculated in `modules/modloader/calculate_preparation_fingerprint.rs`. Live game access and the native ABI belong to `modules/game-runtime/`.

The workspace is defined in `Cargo.toml` and the release composition is in `dev/build/release_build.ps1`. This integration does not introduce new parser or mod formats.
