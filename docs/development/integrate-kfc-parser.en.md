# Integrate the KFC parser

`kfc-parser/` is the Git submodule itself. Rust consumers use its crates through local Cargo paths. There is no additional parser wrapper.

Format access and resource resolution live in `crates/kfc-base`. Existing codecs live in `crates/kfc-content` and `crates/kfc-resource`. `crates/kfc/src/schema.rs` and `snapshot.rs` contain the existing schema projection and snapshot export. The pinned submodule revision is `742b1cb796238ebdec143842545a5a8f5fc79459` on `kfc4_compatibility`. It includes the complete ShroudForge integration, while `mod-loader-bridge` remains unchanged.

After a fresh checkout, run `dev/parser/prepare_parser.ps1`. Both `dev/build/dev_build.ps1` and `dev/build/release_build.ps1` perform this preparation step too. The script accepts the pinned revision without applying a patch. For the legacy revision `34a156f6`, it retains support for `dev/parser/parser-integration.patch` and detects changes that have already been applied.

Game-file selection belongs to `modules/entrypoints/loader/game_files.rs`. Original files, parser state, and the persistent file transaction belong to `modules/game-files/`. The selected mod configuration fingerprint is calculated in `modules/modloader/calculate_preparation_fingerprint.rs`. Live game access and the native ABI belong to `modules/game-runtime/`.

The workspace is defined in `Cargo.toml` and the release composition is in `dev/build/release_build.ps1`. This integration does not introduce new parser or mod formats.
