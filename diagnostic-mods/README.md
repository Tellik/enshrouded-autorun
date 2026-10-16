# Diagnosemods

Diese fünf Pakete sind Entwicklerproben für die gemeinsame Lua-Runtime-API. Die aktuellen Spielmods benötigen sie nicht. Kein Diagnosepaket hat eine Abhängigkeit zu SF Survival oder SF Unlimited Flight. Alle fünf Manifeste enthalten `"dependencies": []`.

`build.ps1` nimmt ausschließlich direkte Unterordner von `mods/` in ein Release auf. `diagnostic-mods/` bleibt außerhalb des Paketbaus.

| Ordner | Mod-ID | Zugriff | Zweck |
| --- | --- | --- | --- |
| `live-runtime-probe` | `sf-live-runtime-probe` | Nur lesend | Prüft Runtime-Registrierungen, Spielerattribute und geladene Ressourcen. |
| `live-survival-probe` | `sf-live-survival-probe` | Nur lesend | Beobachtet Spielerattribute, Fallkonfiguration, Welteinstellungen und Gleiterressourcen. |
| `live-runtime-write-probe` | `sf-live-runtime-write-probe` | Schreibend, danach Restore | Erhöht `Stamina_Max_Adder` vorübergehend um eins und stellt den Ausgangswert wieder her. Höchstens zwei erfolgreiche Änderungen. |
| `live-resource-scalar-probe` | `sf-live-resource-scalar-probe` | Schreibend, danach Restore | Erhöht `ItemInfo.equipment.gliderConfig.yawAngleSpeed` einer geladenen Gleiterressource vorübergehend um eins und stellt den Ausgangswert wieder her. Höchstens zwei erfolgreiche Änderungen. |
| `live-engine-flight-probe` | `sf-live-engine-flight-probe` | Schreibend, danach Restore | Setzt `DynamicLocomotion.state` kontrolliert auf `BuildingFlying` und stellt den vorherigen Zustand wieder her. Höchstens zwei erfolgreiche Änderungen. |

Die Proben verwenden direkt die öffentliche Lua-API. Sie importieren keine anderen Mods und lesen keine Einstellungen von Survival oder Unlimited Flight. Die drei Schreibproben sind Diagnosewerkzeuge. Sie gehören nicht in einen normalen Spielmodsatz.

Die vier `*-check.py`-Dateien prüfen private Lua-Fixtures. Sie gehören ebenfalls nicht in ein Modpaket. Die Probe-README-Dateien dokumentieren Ablauf, Abbruchbedingungen und Wiederherstellung.
