# KFC-Parser

`kfc-parser/` ist direkt das Git-Submodul. Alle Rust-Verbraucher verwenden die dortigen Crates über lokale Cargo-Pfade. Eine zusätzliche Parserhülle gibt es nicht.

Formatzugriff und Ressourcenauflösung liegen in `crates/kfc-base`, die vorhandenen Codecs in `crates/kfc-content` und `crates/kfc-resource`. `crates/kfc/src/schema.rs` und `snapshot.rs` enthalten die bisherige Schemaprojektion und den Snapshotexport. Die lokale Integration erhält den vorher tatsächlich verwendeten Parserstand `a404b15c`, während der vorhandene `mod-loader-bridge` seinen bisherigen Stand behält.

Die ursprüngliche Submodulrevision bleibt `34a156f6`. Die vollständige lokale Integration wird als Patch unter `dev/kfc-parser-integration.patch` mitgeführt. Nach einem frischen Checkout `dev/prepare_parser.ps1` ausführen. `build.ps1` erledigt diesen Vorbereitungsschritt ebenfalls. Bereits angewendete Änderungen werden erkannt. Vor einer eigenständigen Veröffentlichung des Parser-Submoduls müssen dessen Änderungen dort versioniert und veröffentlicht werden.

Konkrete Spiel-Dateiauswahl liegt in `entrypoints/loader/game_files.rs`. Originale, Parserstatus und die persistente Dateitransaktion gehören zu `game-assets/`. Den Fingerprint für die gewählte Modkonfiguration berechnet `modules/modloader/calculate_preparation_fingerprint.rs`. Live-Spielzugriff und native ABI gehören zu `kfc-runtime/`.

Workspace und Releasezusammensetzung stehen in `Cargo.toml` und `build.ps1`. Die Strukturumstellung führt keine neuen Parserformate oder Modformate ein.
