# Diagnosejobs

Die eingebauten Diagnosejobs und die Messfähigkeiten liegen in diesem Ordner. `catalog.rs` beschreibt die eingebauten auswählbaren Jobs. `runtime_health.rs`, `queue_health.rs` und `mod_callbacks.rs` erfassen ihre jeweiligen Daten. `operation_timings.rs` enthält die geteilte Messung instrumentierter Lua- und Runtime-Operationen.

## Eigene Jobs

Eigene Jobdefinitionen liegen im Runtime-Datenordner unter `diagnostics/jobs/`. Standardmäßig ist das `shroudforge/runtime/diagnostics/jobs/`. Ein abweichender Runtime-Speicherpfad aus den Einstellungen wird berücksichtigt. Der Diagnosekatalog legt den Ordner bei seiner ersten Abfrage an und liest JSON-Dateien direkt aus diesem Ordner ein.

Eine Datei beschreibt eine Zusammenstellung bereits unterstützter Fähigkeiten:

```json
{
  "schemaVersion": 1,
  "id": "user.world-and-queue",
  "name": "World and queue",
  "description": "Captures runtime world and dispatcher health.",
  "capabilities": ["runtime", "queue"]
}
```

`id` muss mit `user.` beginnen. Unterstützte Fähigkeiten sind `runtime`, `queue`, `mods` und `operations`. Unbekannte Felder, doppelte Kennungen, ungültige Fähigkeiten und Dateien über 64 KiB werden abgewiesen. Der Modloader zeigt Ladefehler in den Diagnoseeinstellungen.

Jobdateien führen keinen Code aus. Für eine neue Messfähigkeit kommt die Messlogik als Rust-Modul in diesen Ordner und wird im Katalog registriert. Die Jobdatei kann sie danach in einer benannten Zusammenstellung verwenden.
