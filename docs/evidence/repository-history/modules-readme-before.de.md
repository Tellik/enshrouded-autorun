# Eingebaute Module

Eine Handlung hat eine benannte Implementierung. Jedes Modul besitzt seinen Fachcode und seinen eigenen UI-Teil. Das Modloader-Menü setzt die Ansichten zusammen und übergibt Aufträge.

| Modul | Zuständigkeit und Beispiel |
| --- | --- |
| `modloader/` | Mods finden, Manifeste lesen, Einstellungen speichern, Konflikte prüfen und Ladefolge planen. Zum Speichern öffnest du `save_mod_settings.rs`. |
| `loader-config/` | Loader-Einstellungen und Status lesen, prüfen und zuverlässig speichern. Siehe `read_loader_settings.rs`, `save_loader_settings.rs` und `update_loader_state.rs`. |
| `logging/` | Logdateien öffnen, Einträge schreiben und Aktivitäten darstellen. Siehe `append_log.rs` und `ui/show-activity.tsx`. |
| `notifications/` | Nachrichten lesen, Ereignisse speichern, Lesestand verwalten und Toasts anzeigen. Siehe `read_news.rs`, `mark_news_read.rs` und `ui/show-toasts.tsx`. |
| `updater/` | Systemupdates prüfen und die vorhandenen Mod- und Systemaufträge abarbeiten. Siehe `check_system_update.rs`, `queue_mod_update.rs` und den eigenen `ui/`-Teil. |
| `runtime-diagnostics/` | Die vorhandenen Messungen ausdrücklich starten, erfassen und beenden. Siehe `start_callback_measurement.rs`, `capture_runtime_snapshot.rs` und `stop_callback_measurement.rs`. |
| `debug-console/` | Loganzeige und eigenes Fenster. |
| `world-editor/` | Editorfenster, Bibliothek, Vorschaubilder und eigene IPC-Bedienung. |
| `commands/` | Der vorhandene Befehlsmodus und seine Verfügbarkeitsmeldung. |
| `transport/` | Steam-Nachrichten, Lua-Transport und Verbindungszustand. Siehe `report_network_health.rs`. |
| `ui-resources/` | Gemeinsame Anzeigeelemente, Farben, Icons und Übersetzungen. |

Die Schaltfläche für Mod-Einstellungen liegt beim Modloader-Frontend. Der Fensterhost ruft die kanonische Speicheraktion des Modloaders auf. Er enthält keine zweite Implementierung dieser Regeln. Gemeinsame persistierte Dokumentverträge sind beim Modul definiert, dem die Daten gehören. Die generische Speicherung gehört `loader-config`.

Die übrigen großen Aufgaben bleiben eigenständig. `execution/` führt Lua-Mods aus. `game-assets/` behandelt Spielasset-Dateien und Transaktionen. `kfc-parser/` ist das direkte Parser-Submodul. `kfc-runtime/` besitzt die native Spielanbindung und ausgelieferte Profile.

Das Frontend baust du mit `./dev/build_modloader_ui.ps1`. Das erzeugte HTML liegt unter `build/ui/modloader/`. Die Quelldateien des Modloader-Fensters liegen direkt in `modloader/ui/`, gemeinsame Übersetzungen in `ui-resources/locales/`. Die Aktivitätsansicht gehört `logging/ui/`, Nachrichten und Toasts gehören `notifications/ui/`, Updateansichten gehören `updater/ui/`.

Die [Architekturübersicht](../docs/sf/Architecture.md) zeigt Handlung und Dateiname nebeneinander. Der [Umsetzungsbericht](../docs/sf/action-refactor-implementation.md) nennt die ausgeführten Änderungen und Prüfungen. Die älteren Refactor- und Reduzierungsberichte dokumentieren ihre jeweiligen früheren Arbeitsstände.
