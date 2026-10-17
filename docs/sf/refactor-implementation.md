# Umsetzung der Repositorystruktur

Dieser Bericht dokumentiert den damaligen Struktur- und Auditstand. Die anschließende Reduzierung mit aktuellen Dateinamen, Löschungen und Prüfungen steht im [Reduzierungsbericht](reduction-report.md). Prüfsummen und Pfadlisten dieser Strukturphase sind historische Momentaufnahmen.

Stand: 17. Oktober 2026. Ausgangscommit: `81f9c5fe2d70ea4ba10896fb22a75b2d082800a6`. Maßgeblich ist die anschließend ausdrücklich verlangte Verhaltenserhaltung.

Die beschlossene Struktur ist umgesetzt. Für alle 1.040 inventarisierten Ausgangspfade sind sämtliche 1.145 verschiedenen geplanten Zielpfade vorhanden. Die [Migrationsliste](refactor-migration.csv) enthält den Abgleich und die aktuellen Datei-Prüfsummen. Das ursprüngliche [Dateiinventar](refactor-file-inventory.csv) und die [Einzeldateianalyse](refactor-source-audit.md) bleiben als Ausgangsnachweis erhalten. Im Audit wurden alle 287 eigenen Codedateien vollständig gelesen, also 287 von 287.

## Tatsächliche Zuständigkeiten

| Ort | Implementierung |
| --- | --- |
| `entrypoints/` | Loaderkomposition, CLI, Prozessmodi, konkrete Spieldateiauswahl und Windows-Bootstrap. |
| `support/` | Loaderkonfiguration, atomisches JSON, Statusspeicher, Fensteraufträge, Pfade und Logging. |
| `packages/` | Paketdiscovery, Manifestlesen, EML-Normalisierung, Einstellungen, Speichern, Konflikte und Ausführungsplan. |
| `game-assets/` | Originaldateien, Fingerprints, Transaktion, Recovery und Restoreauftrag. |
| `execution/` | Lua-Host, Vorbereitung, Ausführungskontext, Lifecycle, Reload, Runtimeaufträge und Cleanup. |
| `modules/` | Modloader, Updater, Debugkonsole, Welteditor, Diagnostik, Nachrichten, Commands und Transport auf derselben Ebene. Jede vorhandene UI liegt beim jeweiligen Modul. |
| `kfc-parser/` | Direkt das Git-Submodul mit Formathelfern, Reflection, Schemaprojektion und Snapshotexport. |
| `kfc-runtime/` | Rust-Brücke, Compatibility, native ABI, ECS, Enginekontext, geladene Ressourcen, Dienste, Voxel, Bauoperationen, Eingaben und Profile. |
| `tools/`, `examples/` | Profilpflege, Inspektion, Parser-Buildvorbereitung und vorhandene Runtimebeispiele. |
| `website/` | Getrennte Browsermodule, gemeinsame Editorvorlage und sprachbezogene Inhalte. |

Es gibt keine zusätzliche `core/`- oder `vendor/`-Ebene und keine neu eingeschobenen `src/`-Quellwurzeln. Vorhandene `src/`-Verzeichnisse innerhalb von Mods und Submodulen bleiben erhalten.

`mods/` enthält weiterhin ausschließlich `sf-survival` und `sf-unlimited-flight`. Die vorhandenen EML-, Referenz- und Diagnosepakete bleiben in ihren bisherigen Verzeichnissen. Der Referenzwelteditor ist intern in Blueprint, Bibliothek, Vorschau, Operationen und Historie aufgeteilt.

Die großen Einstiegdateien wurden tatsächlich nach Aufgaben aufgeteilt. Es wurden keine vollständigen Monolithen lediglich unter einem neuen Dateinamen eingebunden. Bestehende öffentliche Rust-Einstiege werden teilweise durch Reexports erhalten. Ein fachliches Modul ist deshalb nicht automatisch eine eigene Cargo-Bibliothek. Diese Kompatibilitätsfassaden kompilieren jede Implementierung einmal und erhalten vorhandene statische Zustände.

## Erhaltenes Verhalten

Öffentliche Lua-Namen, native ABI, Prozessmodi, Tastenkürzel, Fensterprofile, IPC-Nachrichten, Konfigurationsschlüssel, Dateiaufträge, Statusformate und installierte Programmpfade bleiben erhalten. `mod.json` und `extended.mod.json` behalten ihre bisherigen Rollen. Die EML-Erweiterungserzeugung bleibt unabhängig vom Installer erhalten.

Systeminstallation und Modinstallation sind getrennte Aufgaben im Updatermodul. Die vorhandenen Desktop- und Headless-Prüfadapter behalten ihre bisherigen Eingaben und Entscheidungen. Eine funktionale Vereinheitlichung ihrer unterschiedlichen Regeln gehört nicht zum beauftragten Strukturumbau. Auch bestehende Archivzweige wurden unter der Vorgabe unveränderten Verhaltens nicht entfernt. Installierte Mods bleiben entpackte Verzeichnisse.

Cacheoptimierungen, neue Einstellungsformate, neue Benutzerabläufe und bestehende Fehlerkorrekturen wurden nicht mit dem Strukturumbau vermischt. Änderungen in Modmanifesten betreffen ausschließlich den Entwicklungspfad zu `$schema`. Lua-Arbeitsplatzdateien zeigen auf die verschobenen Definitionen. Die Lua-Dateien und `extended.mod.json` der beiden ausgelieferten Mods stimmen bytegenau mit dem gesicherten Auditbestand überein.

## Parserintegration reproduzieren

Der Hauptbestand verwendete tatsächlich Parsercode aus `a404b15c`, während das vorhandene Submodul auf `34a156f6` stand. Die Integration erhält den zuvor aktiven Parsercode und den bisherigen Bridgecode. Sie ergänzt die ehemaligen eigenen Format-, Schema- und Snapshotfunktionen direkt im Submodul.

Die vollständigen lokalen Submoduländerungen liegen zusätzlich in `tools/build/kfc-parser-integration.patch`. `tools/build/prepare-kfc-parser.ps1` erkennt eine bereits angewendete Integration und prüft vor einer erstmaligen Anwendung Revision und Patchkontext. `build.ps1` ruft diesen Schritt auf. Vor einem direkten Cargoaufruf in einem frischen Checkout ist der Vorbereitungsschritt einmal erforderlich. Die Rekonstruktion aus dem ursprünglichen Submodulstand wurde für alle 244 resultierenden Dateien geprüft.

Die Submodule wurden nicht veröffentlicht. Eine spätere eigenständige Parserveröffentlichung muss dessen Änderungen dort versionieren. Der mitgeführte Patch hält den Hauptrepository-Build bis dahin reproduzierbar.

## Ausgeführte Prüfungen

| Prüfung | Ergebnis |
| --- | --- |
| `cargo check --workspace --all-targets --offline` | Erfolgreich. Vorhandene Testziele und Beispiele werden nur kompiliert. |
| `cargo build --workspace --release --offline` | Erfolgreich. |
| Windows-Releasebuild über `build.ps1 -SkipTests -UseInstalledDependencies -SkipUiBuild -BuildNumber refactor-check` | Erfolgreich. Frontend wurde zuvor separat gebaut. ZIP und SHA-256-Datei wurden erzeugt. |
| Nativer MSVC-x64-Releasebuild | Erfolgreich. |
| Sechs bestehende native Fixtureziele und acht Entwicklungswerkzeuge | Erfolgreich kompiliert, nicht ausgeführt. |
| Windows-Bootstrap über MSBuild | Erfolgreich, Startgate und Proxy bleiben erhalten. |
| Modloader-Frontend `npm run build` | Erfolgreich, 582 Übersetzungsschlüssel in zwei Sprachen und beide Flaggen. |
| Websitebuild und vorhandener Sitevalidator | Erfolgreich, 1.015 Lua-Symbole. |
| Releasepaket | 26 Dateien und 25 verwaltete Pfade exakt abgeglichen. 18 Quellpayloads und die ZIP-Prüfsumme geprüft. Keine Entwicklungsprofile oder Laufzeit-Zustandsdateien enthalten. |
| Vollständige TypeScriptprüfung im Vergleich zum Ausgangsbestand | Dieselben neun vorhandenen Fehler. Keine zusätzlich entstandenen Fehler. |

Die erneuerten Website-API-Daten übernehmen bereits vorhandene Lua-Annotationen. Zwei zuvor nicht erzeugte Indexeinträge und präzisierte Indextyptexte stammen aus diesen unveränderten Quellen. Der Generator wurde nicht um neue Runtimefunktionen erweitert.

## Inhaltliche Nachweise

| Bereich | Abgleich |
| --- | --- |
| Updater | Alle 94 ursprünglichen Funktionskörper unverändert. |
| Support, Pakete, Spielassets und Updater | Unabhängiger Review von 301 ursprünglichen Funktionen aus 21 Dateien. 299 nach Leerraum- und Schemaauflösung identisch. Zwei Fälle enthalten ausschließlich notwendige Icon- beziehungsweise Testpfadanpassungen. Keine Verhaltensabweichung gefunden. |
| Native Aufteilung | Alle 251 ausgelagerten Funktionskörper erhalten. |
| Native ABI | Kanonischer Header bytegleich, 18.063 Bytes. SHA-256 `23bc42531227a0ac3dcd4f04ce016b215f5983e23257bef8679d7a539be5742e`. Alle 74 Exporte erhalten. |
| Bootstrap | Alle 30 ursprünglichen Funktionskörper erhalten. 180 WinMM-Weiterleitungen und `ShroudforgeStop` erhalten. |
| Lua-Verträge | 19 produktive Lua-, JSON- und Schemadateien bytegleich. Das erzeugte Definitionskonkatenat bleibt bytegleich mit 181.850 Bytes. |
| Modloader-Frontend | 94 von 96 Deklarations-ASTs identisch. Nur die Komposition von `App` und `ModulePreferencesEditor` wurde für modulgebundene Ansichten angepasst. Der ausgelagerte Nachrichtenerzeuger bleibt AST-identisch. |
| Referenzwelteditor | 103 Funktions-ASTs und 96 verbleibende Einstiegsanweisungen nach Rückabbildung der privaten Kontextfelder identisch. Pro Modinstanz eigener Zustand. Bestehende Fixture-Lader wurden auf die Factorymodule angepasst. |
| Website | Beide Localeobjekte einschließlich Editor-HTML unverändert. Der Seeder erzeugt nach der Textextraktion dieselben 366 Artefakte. |
| JSON-Verträge | 270 Dokumente abgeglichen. Änderungen beschränken sich auf Schema-/Entwicklungspfade und erforderliche Frontend-Buildkonfiguration. |

Detaillierte lokale Buildlogs und maschinenlesbare Abgleiche liegen unter `target/`, unter anderem `foundation-review.json`, `updater-function-preservation.json`, `world-editor-lua-preservation.json`, `modloader-ui-preservation.json`, `execution-implementation-report.json`, `parser-patch-verification.json` und `native-restructure/export-check.json`. Diese lokalen Arbeitsartefakte sind nicht Teil des Releasepakets.

## Grenzen und verbleibender lokaler Rest

Tests wurden gemäß Coding-Skill weder neu angelegt noch ausgeführt. Ein Spielstart und Live-Läufe wurden nicht durchgeführt. Die erfolgreichen Builds und die Inhaltsvergleiche ersetzen keinen vollständigen Laufzeitnachweis im Spiel.

Die neun bestehenden TypeScriptfehler, der vorhandene fehlende NichtWindows-Stub, der bestehende Welteditor-Scopefehler und der bereits falsche Paketpfad eines ignorierten Blueprinttests wurden unter der Vorgabe unveränderten Verhaltens bewusst nicht repariert. Windows ist die geprüfte Zielplattform.

Der damalige leere `src/`-Rest wurde in der anschließenden Reduzierungsphase vollständig entfernt.
