# Eine Handlung, eine Datei, ein Eigentümer

Stand: 17. Oktober 2026.

Die Struktur und die tatsächlichen Implementierungen wurden gemeinsam umgebaut. Eine Speicheraktion, ein Updateauftrag oder eine Profilanalyse liegt jetzt beim zuständigen Modul. Das Fenster übergibt Eingaben und zeigt Ergebnisse. Es gibt keinen zusätzlichen allgemeinen Auftragsmotor und keine zweite Entwicklungsinstallation.

## Wo man sucht

```text
modules/
  modloader/             Mods finden, lesen, prüfen und konfigurieren
    ui/                  Fenster, IPC und Modloaderansichten
  loader-config/         Loaderkonfiguration, Zustand und atomare Speicherung
    ui/                  Loader-, Modul- und Speichereinstellungen
  updater/               Systemupdates, Modinstallation und bestehende Auftragswarteschlange
    ui/                  Updateansichten
  notifications/         Nachrichten, Ereignisse, Lesestand und Toastanzeige
    ui/
  logging/               Logdateien und Aktivitätsanzeige
  runtime-diagnostics/   Messungen, Aufnahme und Diagnosestatus
  debug-console/        Debugkonsole mit eigenem UI-Teil
  world-editor/         Welteditor mit eigenem UI-Teil
  commands/             Vorhandene Konsolenbefehle
  transport/            Steamnachrichten und Netzwerkzustand
  ui-resources/         Gemeinsame Anzeigeelemente, Symbole und Übersetzungen
execution/              Lua-APIs, Modlebenszyklus und Vorbereitung
game-assets/            Originalsicherung und Assettransaktionen
  ui/                    Sicherungs- und Wiederherstellungsansicht
kfc-parser/             Direkt eingebundenes Parser-Submodul
kfc-runtime/
  native/               C++-Implementierung und C-ABI
  bridge/               Eigene Rust-Bibliothek für den Runtimezugriff
  profiles/             Ausgelieferte Spielprofile
entrypoints/            Programme und DLLs starten
mods/                   sf-survival und sf-unlimited-flight
dev/
  profiles/             Wiederverwendbare Untersuchungs- und Profilaktionen
docs/
  modding/starter-mod/   Vorhandene Modvorlage
  modding/examples/      Dokumentierte Lua-Beispiele
  runtime-investigations/ Abgeschlossene Untersuchungen und Quellenbelege
website/                Websitequellen und deren Erzeugung
assets/                 Eingebundene Produktmedien
```

Jedes Modul besitzt seine zugehörige UI. Die Geschäftsregeln des Modloaders sind eine eigenständige Bibliothek. Die Lua-Ausführung kann diese verwenden, ohne eine Fensterbibliothek einzubinden. Die bestehenden Cargo-Paketnamen bleiben für vorhandene Aufrufer erhalten.

## Konkrete Änderungen

| Gesuchte Handlung | Tatsächliche Implementierung |
| --- | --- |
| Installierte Mods finden | `modules/modloader/find_installed_mods.rs` |
| Modmanifest lesen | `modules/modloader/read_mod_manifest.rs` |
| Mod-Einstellungen speichern | `modules/modloader/save_mod_settings.rs` |
| Mod aktivieren oder deaktivieren | `modules/modloader/set_mod_enabled.rs` |
| Fehlende EML-Kompatibilitätserweiterung erzeugen | `modules/modloader/ensure_eml_extension.rs` |
| Erweiterung mit bestehender Revisionsprüfung aktualisieren | `modules/modloader/update_mod_extension.rs` |
| Modkonflikte prüfen | `modules/modloader/check_mod_conflicts.rs` |
| Ladereihenfolge planen | `modules/modloader/plan_mod_load_order.rs` |
| Loader-Einstellungen speichern | `modules/loader-config/save_loader_settings.rs` |
| JSON atomar schreiben | `modules/loader-config/write_json_atomically.rs` |
| Logzeile schreiben | `modules/logging/append_log.rs` |
| Modereignisse abgleichen | `modules/notifications/synchronize_mod_events.rs` |
| Toasts anzeigen | `modules/notifications/ui/show-toasts.tsx` |
| Systemupdate anfordern | `modules/updater/check_system_update.rs` |
| Vorhandene Updateaufträge bearbeiten | `modules/updater/control/run_headless_update_controller.rs` |
| Runtime starten | `execution/runtime/start_ingame_runtime.rs` |
| Laufende Mods aktualisieren | `execution/runtime/update_runtime_mods.rs` |
| Werte dekodieren | `execution/lua/runtime/values/decode_owned_value.rs` |
| Attribute berechnen | `execution/lua/runtime/ecs/attributes/evaluate_attributes.rs` |
| Spielassets vor dem Start vorbereiten | `entrypoints/loader/asset_preparation/prepare_startup_assets.rs` |
| Profilentwurf erstellen | `dev/profiles/create_runtime_profile.cpp` |

Die alten Sammelbereiche **packages/** und **support/** sind nach ihren tatsächlichen Eigentümern aufgelöst. Der ehemalige Werkzeugbereich liegt unter **dev/**. Die ursprüngliche Profilkonsole enthält elf Befehlszuordnungen, deren Implementierungen jetzt in konkreten Dateien liegen. Die wiederholte Funktions- und Signatursuche verwendet eine kanonische Implementierung. Die beiden KFC-Ressourcenexporter verwenden eine gemeinsame Datei mit ihren bisherigen unterschiedlichen Auswahlen.

Der Modloaderhost trennt 26 vorhandene IPC-Aktionen vom Fensterablauf. Snapshotaufbau, Modsuche, Icons, Logauswertung und Profilinformationen liegen in benannten Dateien. Benachrichtigungen werden weiterhin an derselben Stelle vor der Ausgabe abgeglichen. Das reine Zusammenstellen des Snapshots schreibt keine Benachrichtigungen mehr nebenbei.

Auch die fünf zuvor noch im Modloader abgelegten fremden Ansichten liegen bei ihren Eigentümern. Loader-, Modul- und Speichereinstellungen liegen unter **modules/loader-config/ui/**. Der Modkatalog liegt unter **modules/updater/ui/**. Die Sicherungsansicht liegt unter **game-assets/ui/**. Alle fünf Funktionskörper sind bytegleich und AST-gleich. Die finale Importauflösung wurde mit einem weiteren Frontendbuild und strenger TypeScript-Prüfung bestätigt. Der Modloader setzt die Ansichten zusammen.

Updater-Fachtypen gehören zum Updater. Installationscode importiert sie nicht aus dem UI. Systemupdate, Modinstallation, Warteschlange und Controller haben eigene konkrete Aktionen. Bestehende Unterschiede zwischen Einzel- und Gesamtspeicherung bleiben erhalten, insbesondere die atomare Gesamtspeicherung.

Die Runtime-Bridge ist ein eigenes Cargo-Paket. Ihre Windows-Aufrufe sind nach der jeweiligen Operation aufgeteilt. Native Symbolnamen und Messung verwenden jeweils eine kanonische Implementierung. Werteverarbeitung und ECS-Attributzugriff wurden nach tatsächlichen Aufgaben zerlegt. Vorhandene große Prüfmodule bleiben bei dem Code, den sie prüfen.

## Was bewusst gleich bleibt

Mod-Einstellungen bleiben in **extended.mod.json**. Loader-Einstellungen bleiben in der bisherigen Loaderkonfiguration. Die Schemas und die Zuordnung der 17 Dokumentverträge behalten ihre bisherigen Namen, Speicherorte in der Installation und Zustandsabschnitte. Der gemeinsame Dateischreiber kennt die fremden Fachmodule nicht mehr. Der jeweilige Eigentümer übergibt seinen Vertrag.

**ensure_eml_extension.rs** ergänzt bei Bedarf die vorhandene Kompatibilitätserweiterung eines EML-Pakets. **update_mod_extension.rs** führt den bestehenden geprüften Änderungsablauf aus. Diese Aufgaben wurden nicht mit dem Herunterladen oder Entpacken eines Mods gleichgesetzt. Vorhandene Archivkompatibilitätszweige wurden in dieser strukturellen Änderung nicht entfernt oder neu in den Laufzeitablauf eingebaut.

Die 34 IPC-Kommandos und vier Browsercallbacks des Modloaderhosts sind erhalten. Die Aufrufreihenfolge, Revisionsprüfungen, Fehlerwerte, Zeitgeber, Locks und Transaktionsgrenzen wurden beim Aufteilen abgeglichen. Die bisherigen Hotkeys bleiben erhalten. Insbesondere bleiben Modloader F9, Debugkonsole F10 und der vorhandene konfigurierbare Welteditor-Hotkey mit Standard F2 bestehen.

Die beiden vorhandenen Mods bleiben unter **mods/**. Bei ihren Manifesten wurde lediglich der lokale Schema-Verweis an den neuen Quellpfad angepasst. Es wurde keine neue Modkategorie eingeführt.

Die vorhandenen Diagnoseaktionen bleiben bestehen. Das Registry-Lua-Beispiel liegt bei der Dokumentation. Seine neue Einbindung als auswählbarer Liveauftrag wäre zusätzliches Produktverhalten und gehört nicht zu dieser verhaltensgleichen Strukturänderung.

## Entwicklung ausführen

Die PowerShell-Dateien enthalten die tatsächlichen Schritte. **build.ps1** ruft diese in der bestehenden Reihenfolge auf. Die Version wird weiterhin vor den Builds gelesen und geprüft und anschließend unverändert an die Paketaktion übergeben.

```powershell
# Modloader und dessen UI bauen
./dev/build_modloader.ps1

# Native Runtime bauen
./dev/build_runtime.ps1

# Website erzeugen und validieren
./dev/build_website.ps1

# Vorhandene Buildausgaben paketieren
./dev/package_release.ps1

# Vollständigen Release ohne Testausführung bauen
./build.ps1 -SkipTests
```

Die Profilerfassung startet keine versteckten Compilerläufe oder Korpustests mehr. Vorhandene Prüfungen besitzen ausdrücklich benannte Dateien und Schalter. Die vollständige Releasefolge behält ihren bisherigen **SkipTests**-Schalter. In dieser Umsetzung wurden keine Tests ausgeführt.

## Bereinigung und erhaltene Daten

Der lokale Ordner **shroudforge/** war kein Quellcode und keine notwendige Buildeingabe. Seine 180 Dateien wurden vor dem Entfernen aus dem Repository mit SHA-256 abgeglichen und außerhalb des Repositorys erhalten. Die Sicherung liegt unter **C:/Users/mail/AppData/Local/Temp/shroudforge-action-implementation/preserved-local-runtime-data/**. Das zugehörige Manifest heißt dort **runtime-data-preservation.json**. Dies verändert keinen Datenpfad einer bestehenden Spielinstallation.

693 vorhandene Originaldateien und Mitschnitte mit insgesamt **2.257.470.391 Byte** liegen unter **dev/profiles/captures/**. Ihre Prüfsummen wurden beim Verschieben abgeglichen. Dieser ignorierte Bereich enthält Eingabedaten und Belege. Das [Erhaltungsmanifest](../../dev/profiles/captures/preservation-manifest.json) dokumentiert die Zuordnung lokal. Es wird wie die erfassten Originaldaten nicht in Git übernommen.

22 abgeschlossene Python-Quellbelege bleiben bytegleich unter **docs/runtime-investigations/source-evidence/** erhalten. Historische Websiteartikel verlinken entfernte Referenzmods über die nachgewiesene feste Git-Revision. Es werden keine entfernten Mods wieder angelegt.

Die nach den Builds erneut erzeugten Compilerprodukte und installierten Node-Abhängigkeiten sind entfernt. In diesem Schritt wurden **13.677 Dateien mit 4.283.969.975 Byte**, 2.410 zugehörige Ausgabeverzeichnisse und sechs weitere leere Ordner entfernt. **build/** und **target/** entstehen beim nächsten Build wieder. Die alten Wurzelordner **packages/**, **support/**, **tools/**, **templates/**, **examples/** und **shroudforge/** sind nicht mehr vorhanden.

## Prüfung und Belege

Der [Pfadnachweis](action-refactor-paths.csv) ordnet die ursprünglichen Dateien den aktuellen Implementierungen zu und enthält SHA-256-Werte der Ziele. Die [Prüfergebnisse](action-refactor-verification.json) enthalten die maschinenlesbaren Vergleichsergebnisse und Buildnachweise.

Die Implementierung wurde anhand gesicherter Originaldateien abgeglichen. Beispiele sind 61 Funktionskörper im Modloader-, Konfigurations- und Loggingbereich, 57 Bridge-Funktionskörper, 52 Python-Implementierungen und die vorhandenen C++-Profilaktionen. Bei bewusst herausgelösten Teilschritten wurde der neue Ablauf durch Einsetzen der Helfer mit dem Original verglichen. Das ist ein Inhaltsnachweis für die angegebenen Bereiche und kein Ersatz für einen Spielbetrieb.

Der ursprüngliche Entwicklungsbereich wurde vorab mit **60 von 60 Quelldateien vollständig gelesen**. Weitere Implementierungsdateien und Aufrufer wurden in den jeweiligen Arbeitspaketen geprüft. Die enthaltenen Leselisten unterscheiden vollständig gelesene ursprüngliche Dateien von gezielt geprüften Aufrufern. Diese Umsetzung wird nicht als neuer vollständiger Inhaltsaudit aller Repositorydateien ausgegeben. Der [frühere Vorschlag](simplification-proposal.md) und sein damaliges Leseraster bleiben historische Belege.

| Prüfung | Ergebnis |
| --- | --- |
| Vollständiger Releasebuild über `build.ps1 -SkipTests` | Erfolgreich, einschließlich Runtime, Proxys, Rust-Programmen, Bootstrap und ZIP. |
| Workspace einschließlich aller Targets | Erfolgreich kompiliert. Ein ungenutzter Import wurde abschließend entfernt. |
| Modloaderfrontend | Erfolgreich gebaut, 94 Module und 582 Übersetzungsschlüssel in zwei Sprachen. |
| TypeScript mit strenger Prüfung ungenutzter Werte | Erfolgreich. |
| Websitebuild und Validator | Erfolgreich, 1.015 Lua-Einträge, 155 Seiten und 1.442 ECS-Typen. |
| PowerShell-Syntax | Alle 15 eigenen Skripte erfolgreich geparst. |
| Native ABI und DLL-Exporte | Header unverändert. 74 Runtime- und 181 Bootstrap-Exporte einschließlich Ordinalen und Weiterleitungen unverändert. |
| Releasepaket | 26 Einträge, 25 verwaltete Pfade. Alle 25 Nutzdateien bytegleich mit ihren Build- oder Quelldateien. CRC und ZIP-Prüfsumme geprüft. |
| Modulgrenzen | 17 Workspacepakete ohne Abhängigkeitszyklus. Kein fremder Crate-Quellcode mehr über `#[path]` eingebunden. |
| Erhaltene lokale Daten | Alle 693 Mitschnitte und 180 extern gesicherten Laufzeitdateien nach der Bereinigung erneut mit SHA-256 geprüft. |

Zusätzlich wurden die acht vorhandenen nativen Entwicklungsziele erfolgreich gebaut. Sie wurden nicht gestartet. Im Welteditor sind die bisherigen 643 Fensterzeilen auf 371, in der Debugkonsole die bisherigen 427 auf 268 reduziert. 32 bestehende Funktionskörper, sechs Parser- und Auftragsblöcke, sieben herausgelöste Helfer und beide vollständigen Fensterabläufe stimmen mit den Originalen überein. Die beiden großen Lua-Dateien mit zusammen 2.856 Zeilen sind in 38 Produktionsdateien und zwei unverändert verschobene Prüfmodule zerlegt. Die größte dieser Produktionsdateien hat 210 Zeilen.

Die maschinenlesbaren Belege enthalten die genauen Befehle, Vergleichsmethoden, Dateihashes und Leselisten. Die ursprünglichen Logdateien liegen unter **C:/Users/mail/AppData/Local/Temp/shroudforge-action-implementation/**. Parserwarnungen aus dem bestehenden Submodul und die Kompatibilitätsnamen mehrerer Cargo-Beispiele bleiben bestehen.

Keine Tests, Diagnoseprogramme oder Spiele wurden gestartet. Das Verhalten wurde durch Inhalts-, Vertrags-, Compiler- und Paketprüfungen abgesichert. Eine erneute Ingame-Prüfung wurde nicht durchgeführt.
