# Arbeitsregeln für Runtime Diagnostics

Dieses Modul besitzt den gemeinsamen Diagnoseablauf. Diese Datei beschreibt Zuständigkeiten und startet keinen zusätzlichen Agenten.

## Zuständigkeiten

- `catalog/` besitzt genau ein Jobschema, die Validierung und die Dateisuche.
- `session/` besitzt die Konfiguration, Anfragen, Lebensdauer und Begrenzung eines Jobs.
- `execution/` besitzt den gemeinsamen Ausführungsvertrag, den Pythonadapter und `JobExports` in `write_job_exports.rs`. Die Lua-VM, gebundenen Modrechte und erlaubten kompilierten Rust-Aktionen bleiben im bestehenden Host `mod-runtime/ingame/run_diagnostic_job.rs`.
- `collectors/` besitzt die vorhandenen Messverfahren und ihre Aggregation.
- `status/` besitzt die bestehenden Statusdokumente einschließlich des von Runtime und UI verwendeten Modstatus.
- `diagnostics-jobs/` enthält fachlich gruppierte Jobdefinitionen und deren Skripte. `runtime/`, `world-editor/` und `game_inspection/` sind Aufgabenbereiche. Lege keine zusätzliche Liste auswählbarer Jobs an.
- `tests/` enthält Prüfungen und Fixtures. Sie gehören nicht in das ausgelieferte Jobpaket.
- `ui/` verwendet die vorhandene Dropdownliste und Steuerung.

## Änderungen

Lies vor einer Änderung die Definition, das zugehörige Skript, die Aufrufer und den Ergebnisverbraucher. Eine Jobkennung bleibt erhalten, wenn sich nur Dateiname oder Ordner ändert. Benenne neue Dateien nach ihrer konkreten Handlung. Verwende das vorhandene Schema und den passenden Adapter. Erzeuge weder eine zweite Konfiguration noch einen zusätzlichen Scheduler.

Python-Code darf seine Fachlogik bei einem Umzug behalten. Entferne den alten Dev-Einstieg erst, wenn seine Aufrufer den neuen Ort verwenden. Hilfsdateien und gemeinsam importierte Bibliotheken sind keine eigenen Jobs. Bewahre Offline-Nachweise und mache aus ihren Ergebnissen keine ungeprüfte Runtime-Freigabe.

Alle Adapter beachten denselben Start-/Stopp-Vertrag, das Zeitlimit und die Statusausgabe. Lua-Aufrufe behalten die Identität und Rechte des aktiven Mods. Pythonprozesse werden direkt ohne Shell gestartet und bei Abbruch beendet. Kompilierte Rust-Aktionen sind feste, im Schema erlaubte Funktionen, keine ausführbaren Rust-Quelldateien oder Cargo-Aufrufe zur Laufzeit. Rückleseprüfung und Wiederanlaufsperre des Voxeltests bleiben erhalten.

Ergebnisdateien gehören über `paths::export_dir` in `diagnostics/<job-id>/` des vorhandenen Exportordners. Sitzungsmeldungen und Collectormessungen verwenden das gemeinsame Debug Log mit Quelle `diagnostics`. Reguläre Collectormessungen bleiben auf DEBUG und beachten `logging.minimumLevel`. Lua-Skriptmeldungen behalten die Logidentität des aktiven Mods. Das vorhandene Statusdokument bleibt interner Zustand und UI-Grundlage. Gebündelte Skripte werden temporär unter `diagnostics/` im vorhandenen konfigurierten Cache bereitgestellt. Pythonargumente verwenden für Ergebnisse `{exports}` und für gemeinsame Diagnoseexporte `{diagnosticExports}`. Erzeuge für Ergebnisdateien keine weitere Ausgabekonfiguration oder Ablage unter Runtime.

Konfiguration, installierte Jobdateien, Logs, Profile, Exporte und Wiederherstellungsdaten sind Nutzerdaten. Verschiebe oder lösche sie nicht bei Quellbereinigungen. Baue keinen Interpreter oder Paketinstaller in die Jobausführung ein.
