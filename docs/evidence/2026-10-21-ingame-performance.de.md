# Untersuchung der Ingame-Performance vom 21. Oktober 2026

## Auftrag und Aussagegrenze

Untersucht wurden wiederkehrende Ruckler, ECS-Zugriffe und unnötiges Laufzeitlogging. Der Nutzer hat Einzelspieler als betroffenen Modus bestätigt und Zustandslogging statt wiederkehrender Debugausgaben verlangt. Dies ist eine gezielte Untersuchung, kein vollständiger Repository-Audit.

Die folgenden Kosten sind im ausgeführten Code belegt. Ein Frametime-Vergleich im betroffenen Einzelspieler wurde nicht durchgeführt. Deshalb ist weder Logging als alleinige Ursache noch das vollständige Verschwinden der Minutenruckler nachgewiesen.

## Befunde und Änderungen

| Ablauf und zuständiges Modul | Belegter vorheriger Aufwand | Änderung |
| --- | --- | --- |
| Manifestprüfung, `modloader` | `refresh_configuration` prüft jede Sekunde die Revision. `manifest_revision` öffnete beide Manifestdateien über den protokollierten Dateileser. Damit entstanden zwei Debugzeilen je Mod und Prüfung auch ohne Änderung. Jede Zeile durchläuft den synchronen Logwriter mit Windows-Mutex, Dateiöffnung und Ausgabe. | Die Revisionsprüfung verwendet dieselbe kanonische Paketöffnung ohne Zugriffslogging. Tatsächliche Paketleseaktionen bleiben protokolliert. Pfadprüfung, Dateiinhalte, Revision und Fehler bleiben gleich. |
| Laufzeitstatus, Windows-Bootstrap | Alle zwei Sekunden wurde der vollständige native Status auf DEBUG ausgegeben. Seine Zähler ändern sich auch bei unverändertem Betriebszustand. | Die Detailzeile folgt dem vorhandenen semantischen Statuswechsel. Scheduler und Heartbeat laufen unverändert weiter. |
| Netzwerkzustand, `transport` | Wiederholte Bestätigungen und Fehler derselben Ausfallphase erzeugten weitere Debugzeilen. | Erste Bestätigung, Peerwechsel, Ausfallbeginn und bisherige Erkennung der Erholung bleiben sichtbar. Wiederholungen aktualisieren die vorhandene Statistik ohne Logzeile. |
| Editorstatus, Mod `world-editor` | Ein identischer Exportfehler konnte alle 30 ms und ein identischer Empfangsfehler bei jedem Empfangsversuch erneut gemeldet werden. | Gleicher Fehlergrund wird während eines anhaltenden Fehlers einmal gemeldet. Ein geänderter Grund oder ein neuer Fehler nach erfolgreichem Aufruf wird wieder gemeldet. Wiederholungsversuche und Berechtigungen bleiben gleich. |
| Registry-Aktualisierung, `game-runtime` | Bei vorhandenem Weltmanager wurden jede Sekunde dieselben Registrierungsberichte samt Callback-Code neu aufgebaut. Die große JSON-Serialisierung hielt `operation_mutex` und `state_mutex`. ECS-Operationen verwenden dieselben Sperren. | Vollständige strukturelle Validierung und doppelte Callback-Codebeobachtung bleiben. Identische Daten überspringen den JSON-Neuaufbau. Bei Änderungen erfolgen Aufbau und Serialisierung vor den Publikationssperren. Alte Daten werden nach deren Freigabe zerstört. Die Größenabfrage kopiert den Bericht nicht mehr zwischen. |
| Registry-Suche, `game-runtime` | Das vorhandene Zeitbudget wurde nicht innerhalb der Headerprüfung eines kopierten Speicherblocks geprüft. | Prüfung nach jeweils 256 Headerpositionen. Fortsetzung exakt an der ersten ungeprüften Adresse. 8-ms-Standardbudget, Residenzschutz und erneute Suche nach fünf Sekunden bleiben erhalten. |
| Entitätsauflösung, `game-runtime` | IDs wurden trotz vorhandenem geprüftem Originalindex durch inkrementelle Weltsuche aufgelöst. Ein veralteter Handle-Slot konnte pro Komponentenlesung einen vollständigen Tabellenscan auslösen. | Der vorhandene stabile Original-ID-Lookup wird für Layouts mit Key- und Occupancy-Adressen wiederverwendet. Pointer, Generation, Epoche und Slot werden weiter geprüft. Legacy-Layouts behalten ihren bisherigen Ablauf. |
| Entitätssnapshot, `game-runtime` | Auch bei sparsamer Belegung wurde jeder Tabellenplatz einzeln geprüft. | Belegungswörter werden in aufsteigender Slotreihenfolge über ihre gesetzten Bits durchlaufen. Leere Slots, Nullpointer und überzählige Bits im letzten Wort bleiben ausgeschlossen. |
| Crafting-Ressourcen, Mod `sf-crafting-essentials` | Vor der Adapteranwendung wurde je Ziel ein Snapshot gelesen. Der Adapter las denselben Ressourcentyp anschließend erneut frisch. Zusätzlich entstand jedes Mal ein neuer Adapter samt Claim. | Sitzungsgebundene Adapter werden pro Ziel behalten. Jeder Apply liest weiterhin frisch und erwirbt seinen Claim. Bei zwei warmen Zielgruppen mit jeweils zwölf Zielen entfallen bis zu 24 vorbereitende Reads je 100-ms-Update. |
| Flugressourcen, Mod `sf-unlimited-flight` | Nach jedem aktiven Slider erfolgte eine zusätzliche Ressourcenabfrage, obwohl der nächste Adapter selbst frisch liest. | Bis zu vier zusätzliche Abfragen je Spieler und Update entfallen. Kontinuierliche Pitch-, Input- und Horizonwerte sowie Actionmasken erzeugen keine wechselnden Effekttexte mehr. Steuerberechnung, Recoveryhinweis, Schreibzahlen und Statussemantik bleiben erhalten. |
| Dokumentvalidierung, `loader-config` | Statische eingebettete Schemas wurden bei wiederholten Konfigurations- und Statuszugriffen erneut geparst und kompiliert. | Kompilierte Validatoren werden pro Prozess wiederverwendet. Jeder aktuelle Dokumentwert wird weiterhin validiert. Formate, Fehlermeldungen und Installationssperren bleiben gleich. |
| Effektberichte, `mod-runtime` | Auch identische Berichte erzeugten vor dem bestehenden Vergleich ein JSON-Objekt und einen Zeitstempel. | Gleichheit von Status und gekürztem Detail wird vor dieser Arbeit geprüft. Neue Fehlerdetails bleiben sichtbar. |

Die Einsparungen bei Ressourcenlesevorgängen sind aus den Schleifengrenzen abgeleitete Obergrenzen, keine gemessenen FPS- oder Frametime-Gewinne. Crafting-Scans und Mod-Updateintervalle wurden nicht pauschal verlangsamt. Die Override-Adapter behalten Identität und Claims, keine zur Wiederverwendung bestimmten Live-Snapshots.

## Geänderte Dateien und Aufrufer

Alle hier aufgeführten Quelldateien wurden vollständig gelesen und nach der Änderung erneut geprüft.

- `modules/game-runtime/native/ecs/read_entity_membership.cpp`, `resolve_entity_handle.cpp`, `registry/find_component_registry.h` und `registry/refresh_component_registry.cpp` besitzen native Entitätssuche, Speicherentdeckung und Registry-Publikation. Die vorhandene ABI wird von der Rust-Bridge und den Lua-ECS-Funktionen verwendet. `EcsRuntime::Tick` wird vom Bootstrap-Worker aufgerufen. Der externe Registry-Verifier verwendet denselben Discovery-Header.
- `modules/loader-config/validate_document.rs` und `read_loader_state.rs` besitzen die Schemaüberprüfung für Konfigurationsleser, Dokumentenschreiber und gemeinsame Statusupdates.
- `modules/modloader/mod_package_filesystem.rs` und `calculate_manifest_revision.rs` besitzen die kanonische Paketöffnung und die von Live-Konfigurationsprüfungen verwendete Revision.
- `modules/mod-runtime/session/record_runtime_state.rs` speichert die von Lua übermittelten Effektberichte für Runtime-Status und UI.
- `modules/entrypoints/windows-bootstrap/runtime_lifecycle.cpp` besitzt die bestehende Worker-Schleife und deren Statuslogging.
- `modules/transport/report_network_health.rs` wird vom periodischen Runtime-Update für Netzwerkzustände aufgerufen.
- `mods/sf-crafting-essentials/src/mod.lua` und `mods/sf-unlimited-flight/src/mod.lua` werden durch ihre vorhandenen Laufzeitcallbacks aufgerufen.
- `mods/world-editor/src/preview.lua` und `p2p.lua` werden durch den bestehenden Editor und dessen Updatecallback eingebunden.
- `docs/development/write-and-review-logs.en.md` beschreibt die gemeinsame Loggingregel. Diese datierte Untersuchung liegt unter `docs/evidence/` und ändert keine aktuellen Codezuständigkeiten oder Website-Routen.

Zusätzlich vollständig gelesen wurden die unmittelbar relevanten ECS-Sitzungs-, Speicher-, Komponentenlese- und Registry-Leseverträge, der Game-Thread-Dispatcher, die Ressourcen-Override-Ziele, deren Tracker und Claims, die relevanten Ressourcenleser, die betroffenen Rust-Dokumentein- und -ausgänge sowie die Build-Einstiege. Weitere Runtime-, Mod-, Diagnose- und Logaufrufer wurden gezielt durchsucht. Die vendorte JSON-Bibliothek, der Parser-Submodulcode und alle übrigen Repositorybereiche wurden nicht vollständig auditiert.

## Prüfung und verbleibende Grenzen

Erfolgreich ausgeführt:

- `cargo check -p shroudforge-loader-config -p shroudforge-api -p shroudforge-steam-networking`.
- `dev/build/build_kfc_runtime.ps1`, einschließlich des abschließenden Builds nach dem Discovery-Fix.
- `dev/build/build_windows_bootstrap.ps1`, ohne Warnungen oder Fehler.
- Lua-5.4-Syntaxprüfung der vier geänderten Moddateien mit der bereits vendorten Lua-Bibliothek. Die Skripte wurden dabei nur kompiliert, nicht ausgeführt.
- `rustfmt --check` für alle geänderten Rust-Dateien und `git diff --check`.

Cargo meldete vorhandene Warnungen im Parser-Submodul sowie mehrfach verwendete Example-Quellen. Testläufe, Ingame-Frametime-Messung, Veröffentlichung, Installation und Austausch laufender DLLs wurden nicht durchgeführt. An den parallel bearbeiteten Serverprofilen wurden im Rahmen dieses Auftrags keine Änderungen vorgenommen.

Eine vorhandene Serverdiagnose zeigte eine aktive Hintergrundspeichersuche bei noch inaktivem ECS-Gameplay-Zugriff. Diese andere Sitzung ist kein Beleg für die Ursache im betroffenen Einzelspieler. Offene Lastkandidaten sind vollständige Tabellenkopien vor manchen inkrementellen Scans, große allgemeine räumliche Queries, umfangreiche Diagnoseberichte unter Sperren und mögliche Seitenfehler bei der Kandidatenvalidierung. Diese Abläufe wurden nicht ohne passenden Laufzeitbeleg umgebaut.

## Nachprüfung der direkten ECS-Aufrufe

Die anschließende Untersuchung umfasst 239 eigene Quelldateien in den inventarisierten ECS-, Bridge-, Ressourcen-, Mod- und Sitzungsabläufen. Alle 239 Dateien dieses abgegrenzten Umfangs wurden vollständig gelesen, entsprechend 100 Prozent. Das ist kein vollständiger Repository-Audit. Die Pfade und Lesebelege stehen in `build/verification/ecs-standardization-inventory.csv` und `build/verification/ecs-review-final.json`. Zusätzlich gelesene Aufrufer und gezielte Testsuchen sind in den einzelnen Reviewnachweisen getrennt aufgeführt. Generierte Dateien, die vendorte JSON-Bibliothek, das Parser-Submodul und separate Testdateien gehören nicht zu diesem Quellenumfang.

### Einfache Begrenzung wiederkehrender Spielzugriffe

Auf ausdrücklichen Nutzerwunsch läuft `sf-unlimited-flight` nun mit 100 statt 16 ms. Das reduziert die geplante Häufigkeit von 62,5 auf 10 Updates pro Sekunde, rechnerisch um 84 Prozent. Tatsächliche Häufigkeiten hängen von Laufzeitkosten und Scheduler ab. Dies ist kein gemessener FPS-Gewinn. Steuereingaben und Landung werden dadurch gröber abgetastet. Pitchberechnung, erwartete Werte, Claims und Restore bleiben erhalten. Die README kennzeichnet die ausstehende Ingame-Prüfung dieses Intervalls. Survival und Crafting liefen bereits mit 100 ms.

Die Tastenabfrage des World Editors bleibt bei 30 ms. Seine vier wiederholten Bereitschaftsprüfungen laufen nun im bereits vorhandenen Ein-Sekunden-Zweig. Damit entfallen beim Warten auf Unterstützung ungefähr 129 dieser Prüfungen pro Sekunde. Ohne ausstehenden Editorauftrag wird außerdem keine zusätzliche Sitzung über `update_pending_queries` abgefragt. Es wurde kein zweiter Scheduler eingeführt.

### Tatsächliche Aufruf- und Sperrpfade

- Ein erfolgreich angewendetes unverändertes Attributtarget erzeugt pro Update zwei native `ReadMany` und einen Einzelkomponentenread. Konstruktion, Claim und Anwendung lesen nacheinander. Bei einem tatsächlichen Write kommen ein weiteres `ReadMany` und ein CAS hinzu. Survival kann bei sechs aktivierten Attributen zwölf `ReadMany`, sechs Einzelreads und eine Spielerquery je 100-ms-Update auslösen. Diese Zahlen beschreiben den Codepfad, keine Messung.
- Flight erzeugt mit aktiviertem Unlimited Flying bereits am Boden zwei `ReadMany`, zwei Einzelreads und eine Query pro Spieler und Update. Beim Gleiten kommen vier Steuerkomponentenreads, der Itemresolver und Ressourcenreads hinzu. Das neue 100-ms-Intervall reduziert die Wiederholung dieses gesamten Ablaufs.
- Normale Reads am Original-Owner laufen synchron unter der eigenen `operation_mutex` mit Ownerprüfung vor und nach dem Zugriff. Sie halten keine originale Scene-Sperre. ECS-Writes halten dagegen während Identitätsprüfung, Payloadvergleich, Tracking und bestätigtem Schreiben den exklusiven Original-Scene-Gate und den Shared-Gate der Threadregistrierung. Die Akquisition ist nicht wartend. Während des Haltens kann die Engine dennoch aufgehalten werden.
- Ressourcenclaims erreichen den originalen Lifetime-Gate auch ohne Änderung. Pitch und vier aktive Slider bedeuten beim Gleiten zusammen mit dem Itemresolver sechs Shared-Akquisitionen pro Update. Auch diese Häufigkeit sinkt durch das Flightintervall.
- Der Legacy-GameThread-Dispatcher prüft sein Zwei-Millisekunden-Budget zwischen vollständigen Jobs. Die Wartefrist des Aufrufers unterbricht einen laufenden Job nicht. Allgemeine räumliche ECS-Abfragen und grobe Prop-Abfragen können weiterhin eine vollständige Welt durchsuchen. Spawn- und Destroy-Verifikation können solche Arbeit wiederholen. Diese Abläufe bleiben als konkrete Lastkandidaten offen.

### Beibehaltene Vereinfachungen ohne zusätzliche Live-Datenkopien

| Modul und Dateien | Änderung und Aufrufer |
| --- | --- |
| `game-runtime/native/ecs/query_entities.cpp`, `ecs_session_state.h` | Die Querypublikation schreibt validierte Treffer direkt in den bereits besessenen Ausgabepuffer. Der zusätzliche Treffervector und seine Kopie entfallen. Vollständige Trefferzählung, Reihenfolge und erneute Identitätsprüfung bleiben erhalten. Die vorhandene Query-ABI wird von Bridge und Lua verwendet. |
| `game-runtime/native/ecs/registry/component_discovery_state.h`, `sample_component_layouts.cpp`, `describe_ecs_runtime.cpp` | Ein Set, das dieselben Layoutschlüssel wie die vorhandene Map enthielt, entfällt. Statuszähler verwenden die Map. Ein ungenutztes temporäres Namensset entfällt ebenfalls. Discovery und Diagnosen bleiben die Aufrufer. |
| `game-runtime/native/ecs/registry/read_component_registry.h`, `find_component_registry.h`, `refresh_component_registry.cpp` | Die Registry-Datentypen besitzen feldgleiche Standardvergleiche anstelle zweier manueller Vergleichssysteme. Discovery vergleicht weiterhin nur Einträge, Refresh den vollständigen Snapshot samt separat beobachtetem Callback-Code. |
| `game-runtime/native/ecs/bind_world_entities.cpp` | Shutdown gibt auch ausstehende exakte Prop-Abfragen und deren Ergebnisvektoren frei. Sperrreihenfolge bleibt Prop-Call vor Operation. Die alten Daten werden erst nach Freigabe der Sperren zerstört. |
| `game-runtime/native/voxels/read_registered_voxel_grid.cpp` | Vor dem Original-Gate werden Taskdeskriptoren nach einer konservativen Obergrenze der geschnittenen X/Z-Tiles reserviert. Zuvor entstand ein Deskriptor je angefragter Zelle. Beim Beispiel 256 × 1 × 256 sind das höchstens 1089 statt 65536 Deskriptoren. Zellpuffer, Ergebnis und Gate-Prüfungen bleiben erhalten. |
| `game-runtime/bridge/windows/provider/read_native_catalog.rs` | JSON-Abrufe beginnen mit einem kleinen vorhandenen Ausgabepuffer. Kleine Berichte benötigen dadurch keinen separaten Größenabruf mit erneutem nativem Berichtsaufbau. Größen-, Nullabschluss- und Versuchslimits bleiben begrenzt. |
| `game-runtime/bridge/windows/resources/find_resources_by_type.rs` | Kleine Ressourcenlisten werden im ersten Abruf übernommen, statt den vollständig validierten Ressourcenindex erst zu zählen und dann erneut zu lesen. Größere Ergebnisse vergrößern den Puffer innerhalb des bestehenden Sicherheitslimits. |
| `mod-runtime/lua/ingame/ecs/attributes/read_entity_attribute_storage.rs`, `resolve_attribute_storage.rs` | Die vorhandenen Batchsnapshots werden bei der Rootsuche ausgeliehen. Erst der eindeutig gefundene Root wird übernommen. Header-, Größen- und Mehrdeutigkeitsprüfung bleiben erhalten. |
| `mod-runtime/session/mod.rs` | Der vorhandene InlineField-Cache ist nach Typ gruppiert. Ein Treffer benötigt keinen neuen Pfadstring. Grenzen von 4096 Feldern und 16 MiB bleiben erhalten. Es werden keine Live-ECS-Werte gespeichert. |

Die beiden bewusst getrennten World-Editor-Strukturen bleiben erhalten. `prop_recipes` besitzt die registrierten Platzierungsdaten, `operation.recipes` deren eigene Kopie für eine laufende exakte Abfrage. Die versuchte gemeinsame Referenz wurde vollständig zurückgenommen, einschließlich der Fixtureanpassung. Ebenfalls zurückgenommen wurden kleinere Anfangspuffer bei allgemeinen räumlichen ECS-Abfragen und eine stückweise Bestätigungslesung großer Ressourcen. Beide hätten zugunsten von RAM zusätzliche vollständige Scans beziehungsweise zusätzliche `ReadProcessMemory`-Aufrufe erzeugen können. Es wurde keine SQL-Spiegelung oder zusätzliche Cacheverwaltung eingeführt.

Attributadapter werden in dieser Änderung nicht zusätzlich zwischengespeichert. Ein Claim ist an die ausgewählte Komponente gebunden, während ein frischer Rootread grundsätzlich eine andere Komponente finden kann. Eine pauschale Wiederverwendung würde diese Bindungsgrenze verschieben. Der einfachere Eingriff ist hier die reduzierte Updatehäufigkeit.

### Vorhandene Messdaten und abschließende Prüfung

Das Clientarchiv `shroudforge-client-1792542263.log` enthält 60 Diagnose-Samples. Im Job `user.world-editor-placement-input-trace`, Timing-Sitzung 2, wurden am 21. Oktober 2026, 00:09:21 bis 00:11:07 Uhr MESZ insgesamt 246 native Komponentenreads mit 36,850 Sekunden Gesamtdauer und 166,555 ms Maximum erfasst. Die 384 Queries derselben Sitzung benötigten zusammen 505,900 ms bei 2,465 ms Maximum. Die Samples sind kumulativ und wurden nicht miteinander aufsummiert.

Diese historische Sitzung meldet ausdrücklich `client-replica`, `original-engine-owner-chain` und `ecs-client-replica`. Die Laufzeiten belegen teure Reads in dieser damaligen Sitzung. Sie belegen weder eine gleich lange Blockierung des Spielthreads noch die aktuelle Einzelspielerursache. Der native Vergleichsschreibpfad zählt außerdem nicht in den bisherigen einfachen Writezählern mit. Ein Writezähler von null allein beweist deshalb keine Schreibfreiheit.

Nach den Rücknahmen wurden `dev/build/build_kfc_runtime.ps1` und `cargo check -p shroudforge-api -p shroudforge-runtime-bridge` erfolgreich abgeschlossen. `rustfmt --check` der geänderten Rust-Dateien, die Lua-5.4-Syntaxprüfung aller sieben betroffenen beziehungsweise zum Intervallvergleich gelesenen Modskripte und `git diff --check` waren erfolgreich. Die Cargo-Warnungen betreffen weiterhin vorhandene Parserwarnungen und mehrfach registrierte Example-Quellen. Sie sind keine Buildfehler und erklären für sich kein Ingame-Ruckeln.

Es wurden keine Tests ausgeführt, keine laufenden DLLs ersetzt und keine Installation veröffentlicht. Eine Einzelspieler-Frametime-Messung mit den neuen Änderungen steht aus. Die Änderungen sind daher als Codeoptimierungen und reduzierte Zugriffshäufigkeit belegt, nicht als bereits bestätigte Beseitigung der Minutenruckler.
