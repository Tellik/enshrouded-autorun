# Reduzierung und klare Dateinamen

Stand: 17. Oktober 2026. Diese Phase folgt auf die abgeschlossene Strukturumsetzung. Die vorherigen `refactor-*`-Berichte und Prüfsummen bleiben historische Nachweise. Maßgeblich für aktuelle Umbenennungen ist die [Liste mit 52 Pfadänderungen](reduction-paths.csv). Die [maschinenlesbaren Belege](reduction-verification.json) enthalten Codevergleiche, Leselisten, Prüfungen und das Bereinigungsprotokoll.

## Verbindliches Namensschema

Rust-Dateien verwenden `snake_case`, Browsermodule `kebab-case` und native Dateien `snake_case`. Der Name bezeichnet die tatsächliche Aufgabe. Beispiele sind `asset_preparation.rs`, `command_line.rs`, `character_name.rs`, `blueprint_library.rs`, `function_inventory.h` und `resource_index.cpp`.

`lib.rs`, `mod.rs`, `main.rs` und `build.rs` bleiben technische Einstiege. Eine Implementierung wird nicht unter `main.rs` versteckt, wenn sie eine Bibliotheksaufgabe erfüllt. `misc`, `util`, `internal` und `exports` wurden dort ersetzt oder aufgelöst, wo der Inhalt eine genauere Aufgabe besitzt. Native Header unterscheiden konkrete APIs, Zustände und Verträge, beispielsweise `resource_api.h`, `resource_state.h` und `profile_contract.h`.

Die Modloader-IPC besteht jetzt aus `ipc_commands.rs`, `settings.rs`, `storage_paths.rs`, `config_actions.rs`, `command_results.rs`, `mod_actions.rs` und `external_links.rs`. Die bisherigen IPC-Kommandos bleiben erhalten. Die 22 ausgelagerten Funktionskörper wurden unverändert abgeglichen.

## Entfernte Altlasten und vermiedene Arbeit

- Neun nachweislich ungenutzte private Funktionen der Ausführung entfernt. Außerdem die ungelesene Cache-Schalterkopie, den unerreichbaren Buffer-Schreibzustand und fünf ungenutzte Logging-Makros entfernt.
- Die `util`-Sammelstellen der Ausführung aufgelöst. Lua-Registrierung, Inhalts-ID-Konvertierung, Tabellenansichten und Typnamen liegen bei ihrer jeweiligen Aufgabe.
- Verwaiste Testhüllen und Szenarien entfernt, die ausschließlich bereits vor dieser Phase gelöschte Modpakete laden. Aktive Flight-, Survival-, Override- und native Inventarprüfungen sowie eigenständige Lua-Kompilierungsfixtures bleiben erhalten.
- Überflüssige Weiterleitungshüllen und zwei identische Identifierprüfungen zusammengeführt. Die bestehende abweichende Prüfung für `.` und `..` bleibt erhalten.
- Den redundanten unveränderlichen `disabled_mods`-HashSet entfernt. Aktivierung wird aus dem vorhandenen Manifest gelesen. Damit entfallen die zweite Zustandskopie und Hashlookups beim Durchlaufen aktivierter Mods.
- Einstellungsdefinitionen bleiben bis zur Validierung typisiert. Unnötige JSON-Zwischenwerte und Klone entfallen. Der Labelaufbau schreibt direkt in einen String, statt für jedes Zeichen einen Vektor anzulegen.
- 63 ungenutzte TypeScript-Importbindungen, drei ungenutzte lokale Berechnungen, vier verwaiste UI-Komponenten und deren ungenutzte CSS-Regeln entfernt. Den identischen doppelten Herz-Iconeintrag entfernt.
- Fünf ungenutzte direkte Cargo-Abhängigkeiten entfernt. Eine private unaufgerufene Browserfunktion, zwei ungenutzte Generatorbindungen, eine doppelte native Deklaration und eine ungenutzte triviale lokale C++-Berechnung entfernt.

Allein der geprüfte Bereich Ausführung und Loader schrumpft von 36.726 auf 35.280 Zeilen, also um 1.446 Zeilen. Diese Angabe bezieht sich auf dessen konkreten Rust-/Lua-/TOML-Ausgangsbestand, nicht auf das gesamte Repository. Die Reduktion wird nicht mit einer ungemessenen Laufzeitbeschleunigung gleichgesetzt.

Der unabhängige Review hat eine vorgeschlagene JSON-Schreibvereinfachung zurückgewiesen. Der ursprüngliche Text-Roundtrip bleibt erhalten, weil er Parsergrenzen, numerische Normalisierung und Fehler vor Dateisystemänderungen bewahrt. Die übrigen vier geprüften Optimierungen erhielten das statisch nachvollzogene Verhalten.

## Quellen und Ausgaben

| Verzeichnis | Inhalt |
| --- | --- |
| `modules/modloader/ui/` | Handgepflegte Oberfläche, Übersetzungen und Buildkonfiguration. |
| `build/ui/modloader/` | Erzeugtes gebündeltes HTML für die Rust-Einbettung. |
| `build/ui/modloader-inputs/` | Erzeugte Flaggenmodule aus den vorhandenen SVG-Abhängigkeiten. |
| `website/` | Browserquellen, Artikel, Metadaten und ursprüngliche Spielcaptures. |
| `build/website/` | Vollständige veröffentlichungsfertige Website einschließlich API-Katalog, Schemakopien, Navigation und ECS-Referenz. |
| `target/`, `kfc-parser/target/` | Cargo-Ausgaben. |
| `build/native-runtime/`, `build/obj/`, `build/x64/` | Native Buildausgaben, Zwischenprodukte und Releaseeingaben. |
| `tools/build/` | Handgepflegte Buildwerkzeuge und kanonischer Parserpatch. Dieses Verzeichnis ist Quellcode. |

Die neun bisherigen generierten Dateien im Quellbaum wurden entfernt. Die Website erzeugt ihre sieben Ausgaben jetzt ausschließlich unter `build/website`. Sie sind bytegleich zur vorherigen Ausgabe. Die vier Capture-Eingaben bleiben bytegleich. Snapshotbereinigung arbeitet im Veröffentlichungsbaum. GitHub Pages publiziert diesen Baum. Das Modloader-HTML und die generierten Flaggen liegen ebenfalls ausschließlich unter `build/`.

Die bereits vor Beginn entfernten EML-, Referenz- und Diagnosepakete wurden nicht wiederhergestellt. Der Welteditor-Fensterhost, seine IPC-Schnittstelle und die öffentlich erreichbaren Lua-/Native-Funktionen bleiben erhalten. Unter `mods/` liegen weiterhin die beiden aktuellen Releasepakete.

## Tatsächliche Bereinigung

Nach den erfolgreichen Builds und Inhaltsprüfungen wurden 92.648 Build- und Cachedateien mit 51.099.296.455 logischen Bytes entfernt. Der gemessene freie Speicher auf Laufwerk A stieg um 47.607.164.928 Bytes, rund 47,6 GB. Dateivolumen und tatsächlich freigegebener Speicher unterscheiden sich unter anderem durch Hardlinks und Dateisystemallokation.

12.094 Artefaktverzeichnisse sowie 91 leere Arbeitsverzeichnisse wurden entfernt. Darin enthalten sind die 51 leeren Verzeichnisse des alten `src/`-Baums. Verzeichnislöschungen erfolgten einzeln und ausschließlich bei nachgewiesener Leere. Die Metadaten von Hauptrepository und Submodulen sowie Verzeichnisverknüpfungen wurden geschützt.

Die entfernten Ausgabewurzeln sind `build`, `target`, `x64`, `entrypoints/windows-bootstrap/build`, `kfc-parser/target`, der alte Modloader-`dist`-Ordner und die lokal installierten UI-Abhängigkeiten. Generierte Python-Bytecodes und ihre danach leeren Ordner wurden ebenfalls entfernt. Der Bereinigungslauf meldet keine Fehler.

474 echte Capture-, Original- und Belegdateien mit 848.478.804 Bytes wurden vor der Löschung nach `tools/runtime-profiles/devdata/cleanup-preserved/` verschoben. Ihre SHA-256-Prüfsummen wurden vor und nach dem Verschieben verglichen. Darunter befinden sich das Originalserverprogramm, originale KFC-Dateien, zugehörige Zustandsdaten und ein historischer Arbeitsbaum-Patch. Das lokale `preservation-manifest.json` hält Herkunft und Prüfsummen fest. Nachweislich identische Asset-Backupkopien und alte kompilierte Releasepakete wurden entfernt. Nutzerkonfiguration und Laufzeitzustand unter `shroudforge/` wurden nicht als Buildartefakte gelöscht.

## Prüfungen vor dem Entfernen der Ausgaben

| Prüfung | Ergebnis |
| --- | --- |
| Rust-Workspace mit allen Targets | Erfolgreich kompiliert. |
| Rust-Workspace als Release | Erfolgreich gebaut. |
| Modloader-Frontend | Erfolgreich gebaut. 582 Schlüssel, zwei Sprachen, beide Flaggen. |
| TypeScript mit `noUnusedLocals` und `noUnusedParameters` | Vollständig fehlerfrei. |
| Websitebuild und Validator | Erfolgreich. 1.015 Lua-Symbole, 155 Seiten und 1.442 ECS-Typen. |
| Native Runtime und Windows-Bootstrap | Erfolgreich als MSVC-x64-Release gebaut. |
| Sechs vorhandene native Fixtureziele und Registry-Inspektionswerkzeug | Erfolgreich kompiliert, nicht ausgeführt. |
| Vollständiger Releasepaketbuild | Erfolgreich, mit denselben 26 Dateipfaden und 25 verwalteten Pfaden. |
| Native und Bootstrap-Exporte | Alle 74 beziehungsweise 181 Exporte unverändert. Kanonischer ABI-Header unverändert. |
| Erhaltungsprüfung | 1.052 Ausführungsfunktionskörper nach privaten Pfadänderungen identisch. 67 native Dateien abgeglichen. 16 Browserquellen nach den deklarierten Änderungen AST-/literalgleich. |
| Unabhängiger Optimierungsreview | Vier Optimierungen ohne statischen Verhaltensbefund. Abweichenden JSON-Schreibkandidaten zurückgesetzt und erneut geprüft. |

Tests wurden weder neu angelegt noch ausgeführt. Ein Spielstart wurde nicht durchgeführt. Die verbliebene Warnung eines isolierten Inspektionsbeispiels begründet keine Löschung der gemeinsam verwendeten Methode. Deren tatsächlicher Laufzeitaufrufer ist erhalten. Warnungen des unveränderten Parser-Submoduls wurden nicht durch fremde Änderungen verdeckt.

Die Lesebelege unterscheiden Volllektüre und gezielte Prüfung. Im Modul-/Paket-/Supportbereich wurden 91 Codedateien vollständig gelesen und 16 weitere Rust-Dateien über Compiler und Aufrufersuche geprüft. Im Website-/Native-Bereich wurden 27 Codedateien vollständig gelesen und weitere Dateien gezielt beziehungsweise über Funktionskörpervergleich geprüft. Der unabhängige Review las 19 aktuelle und drei ursprüngliche Quelldateien vollständig. Für die Ausführung wurde kein erneuter vollständiger Inhaltsaudit aller 138 Ausgangsdateien behauptet. Dort sind die entfernten Kandidaten inhaltlich geprüft und die verbleibenden Implementierungen über Compiler, Referenzen und Körpervergleich abgeglichen.

## Nach der Bereinigung neu bauen

Der abschließende Arbeitsbaum enthält keine Buildausgaben. Ein Build erzeugt sie bei Bedarf erneut.

Für das vollständige Windows-Paket im Repository-Root `./build.ps1 -SkipTests` ausführen. Der Einstieg bereitet den Parser vor, installiert die gepinnten UI-Abhängigkeiten und erzeugt das HTML vor der Rust-Kompilierung. `-UseInstalledDependencies` und `-SkipUiBuild` sind unmittelbar nach der Bereinigung nicht anwendbar.

Für eine reine Rust-Prüfung zunächst in `modules/modloader/ui/` `npm ci` und `npm run build` ausführen. Bei einem frischen Checkout zusätzlich `tools/build/prepare-kfc-parser.ps1` ausführen. Danach im Repository-Root `cargo check --workspace --all-targets --offline` verwenden.

Die Website wird mit `npm run site:check` erzeugt und geprüft. Für eine Vorschau `build/website/` ausliefern. Die Quellen unter `website/` benötigen keine eingecheckten generierten Kataloge mehr.
