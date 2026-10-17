# Eine Handlung, eine Datei, ein Eigentümer

> Historischer Vorschlag vor der Umsetzung. Die aktuelle Struktur steht in [Architecture.md](Architecture.md), die ausgeführten Änderungen und Prüfungen im [Umsetzungsbericht](action-refactor-implementation.md).

Stand: 17. Oktober 2026. Überarbeiteter Vorschlag nach der erneuten Inhaltsprüfung. Noch keine Umsetzung des Produktcodes.

## Die Regel

Wer eine Handlung ändern möchte, öffnet genau die Datei, die diese Handlung implementiert.

Beispiel: Mod-Einstellungen speichern gehört vollständig in **save_mod_settings.rs**. Dort stehen der Ablauf, die Revisionsprüfung, die Änderung und die Fehlerbehandlung. Das Fenster übergibt den Auftrag. Es enthält keine zweite Speicherimplementierung.

Ein gemeinsamer Dateischreiber, Parser oder ABI-Vertrag bleibt ein klar benannter Baustein. Fachliche Abläufe werden nicht hinter Sammeldateien, fremden Dateipfadimporten und nebenbei gestarteten Untersuchungen versteckt.

Neue PowerShell-Einstiege allein lösen dieses Problem nicht. Zuerst wird die Implementierung entwirrt. Danach können vorhandene Befehle und eine Diagnoseauswahl dieselbe Implementierung aufrufen.

## Der Ordner shroudforge im Repository

**A:/Github/shroudforge/shroudforge/** enthält lokale Laufzeitdaten: Konfiguration, Status, Logs und WebView-Daten. Keine Datei daraus ist in Git erfasst. Der Build verwendet diesen Ordner nicht als Eingabe.

Der Code in **support/paths.rs** bildet den Datenpfad aus dem angegebenen Spielordner und dem Unterordner **shroudforge**. Welcher frühere Aufruf die hier vorhandenen Daten erzeugte, wurde nicht nachgewiesen.

Für das Quellrepository ist dieser konkrete Ordner nicht erforderlich. Er ist als lokaler Laufzeitrest zu behandeln. Vor einer tatsächlichen Entfernung werden noch benötigte persönliche Einstellungen zugeordnet. Er wird nicht zum Bestandteil einer neuen Entwicklungsumgebung.

Die vorher vorgeschlagene Testinstallation ist gestrichen. Auch eine zusätzliche Ablage für eine solche Installation ist gestrichen. Im installierten Spiel bleibt dessen vorhandener ShroudForge-Datenpfad bestehen.

## Das Repository in einfacher Sprache

| Ort | Hier sucht man |
| --- | --- |
| **modules/** | Modloader, Updater, Nachrichten, Debugkonsole und andere eingebaute Funktionen. Jedes Modul besitzt seine fachlichen Aktionen und seinen eigenen UI-Teil. |
| **execution/** | Lua-Mods starten, deren Lebenszyklus ausführen und Lua-APIs bereitstellen. |
| **game-assets/** | Originale Spielassets sichern, Änderungen vorbereiten, anwenden und wiederherstellen. |
| **kfc-parser/** | KFC-Dateiformate lesen und schreiben. Direkt eingebundenes Submodul. |
| **kfc-runtime/** | Auf das laufende Spiel zugreifen. Native API, Bridge und freigegebene Spielprofile. |
| **entrypoints/** | Vorhandene Programme und DLLs starten. Fachliche Aktionen liegen bei ihrem Eigentümer. |
| **mods/** | Die vorhandenen Mods **sf-survival** und **sf-unlimited-flight**. |
| **dev/** | Tatsächlich wiederverwendbare Build-, Analyse- und Profilaktionen. Eine Handlung pro Implementierungsdatei. |
| **docs/** | Anleitungen, Modvorlage und abgeschlossene Untersuchungsberichte. |
| **website/** | Website und deren vorhandene Erzeugungsschritte. |
| **assets/** | Gemeinsame Produktbilder und andere eingebundene Medien. |
| **build/** und **target/** | Erzeugte Programme und Compilerprodukte. |

**packages/** und **support/** werden nach Eigentümer aufgelöst. **tools/** geht nach tatsächlicher Zusammenführung seiner Aktionen in **dev/** auf. **templates/** geht in **docs/** auf. Das einzelne Beispiel unter **examples/** wird beim passenden Diagnoseauftrag verwendet. Es bleibt keine zweite Implementierung neben diesem Auftrag liegen.

## So sehen die eigentlichen Codebesitzer aus

Der folgende Baum nennt konkrete Handlungen. Bestehende notwendige Typen, Schemas, Übersetzungen und Moduldefinitionen kommen jeweils beim selben Eigentümer hinzu.

~~~text
modules/
  modloader/
    find_installed_mods.rs
    read_mod_manifest.rs
    ensure_eml_extension.rs
    read_mod_settings.rs
    validate_mod_settings.rs
    save_mod_settings.rs
    check_mod_conflicts.rs
    plan_mod_load_order.rs
    create_mod.rs
    ui/
      create_window.rs
      run_window.rs
      read_ui_snapshot.rs
      show-mod-list.tsx
      show-mod-settings.tsx

  loader-config/
    read_loader_settings.rs
    validate_loader_settings.rs
    save_loader_settings.rs
    resolve_installation_paths.rs
    read_loader_state.rs
    update_loader_state.rs
    write_json_atomically.rs

  notifications/
    read_news.rs
    mark_news_read.rs
    mark_news_unread.rs
    synchronize_mod_events.rs
    write_notification_event.rs
    ui/
      show-news.tsx
      show-notice-popup.tsx
      show-toasts.tsx

  updater/
    check_system_update.rs
    download_system_update.rs
    install_system_update.rs
    install_mod_update.rs
    read_update_status.rs
    provider_config.rs
    catalog_item.rs
    ui/

  runtime-diagnostics/
    capture_runtime_snapshot.rs
    start_callback_measurement.rs
    stop_callback_measurement.rs
    read_diagnostic_status.rs
    read_ecs_registry.lua
    ui/
      show-diagnostic-jobs.tsx

  logging/
    initialize_logging.rs
    write_activity.rs

  debug-console/
  world-editor/
  commands/
  transport/
  ui-resources/
~~~

Dies ist das Namens- und Eigentümerschema für die angesprochenen Aktionen, kein Auftrag zum blinden Umbenennen aller bestehenden Dateien. Die tatsächlichen Implementierungen werden beim Herauslösen abgeglichen. Weitere echte Aktionen erhalten entsprechend konkrete Namen. Bestehende andere Module behalten ihren eigenen vorhandenen UI-Teil.

Die Modregeln unter **modules/modloader/** bleiben ohne UI kompilierbar. Der UI-Teil bleibt technisch getrennt. Die Lua-Ausführung kann so dieselben Modregeln verwenden, ohne Fensterbibliotheken einzubinden.

Mod-Einstellungen bleiben in **extended.mod.json**. Loader-Einstellungen bleiben in der vorhandenen Loader-Konfiguration. Die EML-Erweiterung behält ihre bestehende Aufgabe. Es entsteht kein neues Einstellungsformat.

## Was im Hintergrund tatsächlich entwirrt wird

| Heutige Stelle | Geprüftes Problem | Tatsächliche Änderung |
| --- | --- | --- |
| **modloader/ui/host/window.rs**, 1.492 Zeilen | Fensterereignisse und zahlreiche Mod-, Konfigurations-, Update-, Nachrichten- und Diagnosehandlungen stehen zusammen. | Die Implementierung jeder Handlung geht zum zuständigen Modul. Im Fenster bleiben Fensterereignisse und ausdrücklich benannte Aufrufe. |
| **modloader/ui/host/snapshot.rs**, 729 Zeilen | Mischt Modsuche, Icons, Logauswertung, Profilinformationen, Updatestatus und Statuslogging. | Diese Handlungen werden in ihre konkreten Dateien beim Eigentümer verschoben. Die Snapshotdatei stellt ausschließlich das Ergebnis für die UI zusammen. |
| **snapshot()** | Ruft beim Lesen **sync_mod_events()** auf und schreibt damit zusätzlich Nachrichten. | **synchronize_mod_events.rs** wird ein ausdrücklicher Schritt. Die bisherige Reihenfolge und Wirkung bleiben beim Herauslösen erhalten. |
| **modloader/ui/host/settings.rs** | Einzel- und Gesamtspeicherung wiederholen Regeln fremder Module. Auch Netzwerkregeln werden hier geprüft. | Regeln gehen zum jeweils zuständigen Modul. Die Speicherung verwendet einen kanonischen Schreibweg. Die heutige Gesamtspeicherung bleibt eine atomare Transaktion. |
| **packages/lib.rs**, **updater/lib.rs**, **modloader/ui/host/mod.rs** | Binden fremde Implementierungen über **#[path]** ein oder exportieren sie über fachfremde Bibliotheken. | Aufrufer verwenden den tatsächlichen Eigentümer. Fremde Implementierungen werden nicht als eigene private Dateien eingehängt. |
| **updater/mod-installation/catalog.rs** und **install.rs** | Der Fachcode importiert **ProviderConfig** und **CatalogItem** aus einem UI-Ordner. | Diese Fachtypen gehören zum Updater. Dessen UI verwendet die Fachtypen. |
| **support/storage/state.rs** | Ein zentraler Schema-Verteiler kennt zahlreiche fremde Fachbereiche. | Jeder Eigentümer prüft seinen Vertrag. Der gemeinsame atomare Dateischreiber kennt die fachlichen Module nicht. |
| **modloader/ui/app.tsx** und **news/ui/notices.tsx** | Appzustand, Toastzeitgeber, Nachrichtenaufbau und mehrere Seiten sind vermischt. | Nachrichtenerzeugung, Lesestand, Popup und Toastanzeige gehören zum Benachrichtigungsmodul. Die App setzt Ansichten zusammen. |

Eine kurze sichtbare Aufrufkette ersetzt die heutigen Querimporte. Beispiel: Die UI fordert **save_mod_settings** an. Die eine Speicheraktion prüft und speichert die Änderung mit den bestehenden Regeln. Das Ergebnis geht an die Toastanzeige. Speichercode liegt weder im Fenster noch in der Toastdatei.

## Entwicklung: vorhandene Arbeit zusammenführen

~~~text
dev/
  build_modloader.ps1
  build_runtime.ps1
  build_website.ps1
  package_release.ps1
  prepare_parser.ps1

  profiles/
    inspect_executable.cpp
    discover_function_candidates.cpp
    scan_function_signatures.cpp
    capture_process_modules.cpp
    capture_ecs_registry.py
    export_kfc_resources.rs
    inspect_system_dependencies.rs
    create_runtime_profile.cpp
    validate_runtime_profile.cpp
    approve_runtime_profile.cpp
    compare_recipe_resources.py
    export_observed_registry.py

    checks/
      generate_attribute_reference_vectors.py
      verify_component_copy_adapters.py
      verify_building_input_layout.py
      verify_cursor_voxel_context.py
      verify_inventory_decrement.py
      verify_building_event_arguments.py
~~~

Diese Dateien enthalten beziehungsweise übernehmen die wirkliche Handlung. Es entsteht kein zweiter Satz Skripte, der die alten Sammelprogramme nur verdeckt.

Der vorhandene vollständige Paketbuild wird in klar benannte Buildhandlungen und die ausdrückliche Paketaktion aufgeteilt. Eine Paketaktion darf diese vorhandenen Buildhandlungen in sichtbarer Reihenfolge aufrufen. Sie enthält keine zweite Compilerlogik.

Die bereits gemeinsam verwendete PE-, Reflection- und KFC-Verarbeitung wird unter konkreten Namen weiterverwendet. Die dafür notwendigen Typen, Kataloge und Schemas liegen im selben Profilbereich. Kein zusätzlicher allgemeiner **helpers/**- oder **common/**-Sammelordner wird eingeführt.

Konkrete Reduktionen nach Inhaltsprüfung:

1. **console/main.cpp** enthält elf Befehle sowie PE-Lesen, Hashen, Scans, JSON-I/O und Profilprüfung. Die tatsächlichen Befehlsimplementierungen werden in die genannten Aktionsdateien zerlegt. Die Programmhauptdatei ordnet nur die bestehenden Befehle zu.
2. **select_build_command** wiederholt Teile der Funktionssuche und des Signaturscans. Diese Implementierungen werden zusammengeführt. Die Auswahl eines Builds verwendet anschließend dieselben Aktionen.
3. **inspect_attribute_resources.rs** und **inspect_building_inputs.rs** werden zu **export_kfc_resources.rs** zusammengeführt. Die unterschiedlichen Typauswahlen und die echte ItemInfo-Feldprojektion bleiben erhalten.
4. **inspect_runtime_functions.rs** heißt nach seiner tatsächlichen Aufgabe **inspect_system_dependencies.rs** und verwendet weiterhin den vorhandenen Produktionsresolver.
5. **audit-mod-origins.py** wird nicht als allgemeiner Diagnosejob übernommen. Es vermischt unterschiedliche Belege und verlangt neun inzwischen nicht mehr vorhandene Einzelmodnamen. Wiederverwendbare Modifier- und Hookprüfungen werden getrennt. Historische Paketbelege gehen mit ihrem Bericht in die Dokumentation.
6. Die sechs regulären **verify-*.py** sind überwiegend bereits jeweils ein fachlicher Nachweis. Sie erhalten klare Namen und bleiben externe Profilprüfungen. Eine weitere künstliche Zerlegung spart keine Arbeit.
7. Die 22 zusätzlichen Python-Dateien im ignorierten Capturebereich werden ebenfalls inhaltlich zugeordnet. Wiederverwendbare Erfassung und Analyse geht in die kanonischen Aktionen auf. Sitzungsfragmente, feste Prozess-IDs und buildspezifische Versuche werden nicht pauschal zu 22 neuen Jobs.
8. Importierte Prüfdateien dürfen nicht beim Import vollständige andere Untersuchungen starten. Text aus gelöschten Buildskripten wird nicht per **exec** nachgeladen. Solche Abhängigkeiten werden vor einer Übernahme aufgelöst.
9. Alte relative Pfade wie **src/loader/**, **build/Release/**, **devdata/** und vom Arbeitsverzeichnis abhängige Katalogsuche werden durch ausdrücklich übergebene bestehende Eingaben und eindeutige Ausgaben ersetzt.
10. Die globale Ignore-Regel **build/** erfasst heute auch **tools/build/prepare-kfc-parser.ps1** und den notwendigen Parserpatch. Handgepflegter Buildcode darf nicht als Buildausgabe verschwinden. Die Quellen werden ausdrücklich erfasst und von erzeugten Ausgaben getrennt.

## Welche Diagnosejobs sinnvoll sind

Die Diagnoseansicht soll echte verfügbare Handlungen auswählen. Zuerst wird die Implementierung bereinigt, danach wird dieselbe Handlung in der Auswahl angeboten.

| Handlung | Zuständigkeit |
| --- | --- |
| Runtime-Zustand aufnehmen | Vorhandenes Diagnosemodul und Provider-Snapshot. |
| Callbackzeiten messen | Vorhandene Diagnose-Sitzung. Start und Stopp bleiben ausdrückliche Aktionen. |
| ECS-Registry lesen | Inhalt des heutigen Lua-Beispiels an den bestehenden Lua-Lebenszyklus anbinden. Diese Anbindung muss implementiert werden. |
| Fabriken und Inventare erfassen | Sinnvoller Kandidat für einen Live-Diagnosejob. Heute existiert nur ein externer PID-Leser. Eine interne Anbindung wäre neue Arbeit. |
| EXE untersuchen, Ressourcen exportieren, Profil erstellen und prüfen | Die benannten externen Profilaktionen. Eine gemeinsame Auswahl kann diese Aktionen anbieten, die Analyse bleibt im Entwicklerprogramm. |

Ein Dropdown besitzt keine zweite Kopie der Handlung. Es verwendet dieselbe kanonische Aktion wie der direkte Aufruf. Ein neues **job.json** pro Datei und ein universeller Auftragsmotor sind für diesen Plan nicht vorgesehen.

Im Zielablauf startet die normale Profilerstellung keine versteckten Tests. Im heutigen Discoverywrapper löst beispielsweise **-VerifyBufferAdapters** zusätzlich **cargo test** aus. Diese zusätzliche Handlung muss ausdrücklich benannt und ausgewählt werden.

## Was entfällt und was erhalten bleibt

- **shroudforge/** im Repository ist kein benötigter Quellcodebereich. Benötigte lokale Werte werden vor einer Entfernung zugeordnet.
- **templates/mod/** wird **docs/modding/starter-mod/**.
- **examples/runtime/registry-overview.lua** geht nach implementierter Anbindung in den einen Registry-Diagnoseauftrag auf.
- **kfc-runtime/profiles/dev/** wird aufgelöst. Aktiver Entwicklungscode und Kataloge gehören zur Profilarbeit in **dev/profiles/**. Abgeschlossene Untersuchungen gehören nach **docs/**.
- **tools/** wird nach Zusammenführung der Implementierungen entfernt.
- **packages/** und **support/** werden nach fachlicher Zuordnung entfernt.
- Produktprofile bleiben beim Runtimeprodukt. Parsercode bleibt direkt im Parser-Submodul.
- Originale EXE-/KFC-Dateien und einzigartige Mitschnitte sind Eingabedaten oder Belege. Ihre vorhandenen Prüfsummen werden beim Zuordnen beachtet. Sie werden nicht als Code und nicht als ersetzbare Compilerprodukte behandelt.
- Modulbezogene Tests bleiben beim geprüften Modul. Bereits generierte Programme gehören nach **build/** oder **target/**.

## Reihenfolge

1. Jede vorhandene Handlung ihrem tatsächlichen Eigentümer und einer kanonischen Datei zuordnen. Bei mehreren Implementierungen die Unterschiede festhalten.
2. Dopplungen zusammenführen und Sammeldateien aufteilen. Fachtypen und Schema-Verträge beim Eigentümer verankern.
3. Fremde Dateipfadimporte, Seiteneffekte beim Import und versteckte Zusatzaktionen entfernen. Vorhandene Wirkung, Reihenfolge, Formate und Transaktionsgrenzen erhalten.
4. Die nun überflüssigen alten Dateien, Ordner und Aufrufwege entfernen. Build- und Dokumentationspfade im selben Schritt anpassen.
5. Erst danach die bereinigten Aktionen an die Diagnoseauswahl anbinden.

Ein neuer Name oder ein neuer Startbefehl gilt nicht als Erledigung, solange die alte vermischte Implementierung dahinter weiterbesteht.

## Was in dieser Runde geprüft wurde

Inventarisiert wurden **403 eigene Quellcodedateien** einschließlich ignorierter Entwicklungsquellen. Git-Metadaten, die beiden Submodule, rohe Website-Captures und lokale Laufzeitdaten sind davon getrennt. In dieser Vertiefungsrunde wurden **81 von 403 Dateien vollständig gelesen, also 20,10 Prozent**. Für weitere Dateien wurden Aufrufer und Importe gezielt geprüft. Die frühere Repository-Analyse wird nicht als neue Volllektüre ausgegeben.

Der Entwicklungswerkzeugbereich wurde dagegen vollständig gelesen: **60 von 60 Quelldateien, 100 Prozent, 8.302 Zeilen**. Darin enthalten sind die 22 Python-Dateien im ignorierten Capturebereich und das ebenfalls ignorierte Parservorbereitungsskript. Das [Dateiraster](action-inventory.csv) hält für jede Datei Aufgabe, Entscheidung, Verantwortung, Inhaltsbeleg und SHA-256 fest.

Die zusätzlichen 21 vollständig gelesenen Produkt- und Builddateien sind:

~~~text
build.ps1
entrypoints/loader/main.rs
entrypoints/loader/command_line.rs
entrypoints/loader/process_modes.rs
modules/modloader/ui/app.tsx
modules/modloader/ui/host/mod.rs
modules/modloader/ui/host/window.rs
modules/modloader/ui/host/snapshot.rs
modules/modloader/ui/host/settings.rs
modules/modloader/ui/host/config_actions.rs
modules/modloader/ui/host/mod_actions.rs
modules/news/service.rs
modules/runtime-diagnostics/lib.rs
modules/runtime-diagnostics/cli.rs
modules/runtime-diagnostics/ui/controls.tsx
modules/updater/lib.rs
modules/updater/ui/host/window.rs
packages/lib.rs
packages/settings/update.rs
support/lib.rs
support/paths.rs
~~~

Ergänzende Aufrufer und Importe wurden gezielt verfolgt, darunter die Typimporte im Updater-Fachcode, die Verbindung von UI-Snapshot und Nachrichten, weitere Konfigurations- und Statusaufrufe und die Profil-Buildziele. Dies ist kein erneuter vollständiger Inhaltsaudit aller Produktdateien.

Quellbelege:

- [Datenpfade](../../support/paths.rs) und [Paketbuild](../../build.ps1).
- [Fensterhost](../../modules/modloader/ui/host/window.rs), [Snapshotaufbau](../../modules/modloader/ui/host/snapshot.rs) und [Hostverknüpfung](../../modules/modloader/ui/host/mod.rs).
- [Loader-Einstellungsspeicherung](../../modules/modloader/ui/host/settings.rs), [Mod-Einstellungsspeicherung](../../packages/settings/update.rs) und [Schema-Verteiler](../../support/storage/state.rs).
- [Fremdimporte der Paketbibliothek](../../packages/lib.rs), [Fremdimporte des Updaters](../../modules/updater/lib.rs), [Katalog-Fachcode](../../modules/updater/mod-installation/catalog.rs) und [UI-Fachtypen](../../modules/updater/ui/host/types.rs).
- [Profilkonsole](../../tools/runtime-profiles/console/main.cpp), [Discoverywrapper](../../tools/runtime-profiles/run-runtime-discovery.ps1), [Attributexporter](../../tools/inspection/inspect_attribute_resources.rs) und [Baumaterialexporter](../../tools/inspection/inspect_building_inputs.rs).
- [Vermischter Modaudit](../../tools/runtime-profiles/audit-mod-origins.py), [Diagnoseimplementierung](../../modules/runtime-diagnostics/lib.rs) und [Registry-Beispiel](../../examples/runtime/registry-overview.lua).

In dieser Runde wurde der Vorschlag überarbeitet. Produktcode, lokale Konfiguration und Laufzeitdaten wurden nicht verändert. Keine Builds, Tests, EXE-Ausführungen oder Spielstarts wurden durchgeführt.
