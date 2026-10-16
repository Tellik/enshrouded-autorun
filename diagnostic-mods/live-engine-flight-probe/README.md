# Kontrollierte Probe des originalen Engine-Flugzustands

**Diagnoseprobe, kein Release-Mod.** Der Quellcode liegt im Repository unter `diagnostic-mods/`. Diese Probe braucht den optionalen vierten Parameter `expected_fields` von `runtime.ecs.write_fields`. Installation und Aktivierung sind nur für einen gezielten Live-Test mit passendem Runtimebuild vorgesehen.

## Ziel und belegte Beziehung

Die Probe untersucht genau diese Beziehung im laufenden Spiel:

```text
DynamicLocomotion.state = LocomotionState.BuildingFlying
    → originale Locomotion-State-Auswahl behält BuildingFlying
    → originale Gravity-Berechnung setzt Gravity.isActive auf false
    → Restore des gespeicherten State-Namens
    → normaler State und erneutes Gravity.isActive = true
```

Der Dateneinstieg stammt aus dem originalen Enginepfad des Builder-Flugzustands. Konstruktor `0x2bd590` setzt das reflektierte Feld. StateChooser `0x41e030` bewahrt den Zustand. `locomotion_execution` wählt damit den originalen Bewegungspfad. `gravity_calculate_actor` setzt für diesen Zustand selbst `Gravity.isActive=false`. Die statischen Aufrufketten und die privaten Originalcodefälle stehen in `src/loader/runtime/profile-tools/devdata/embervale-toolkit-3.1.0/building-flight-state-20261016.json` und `flight-dataflow-20261016.md`.

Die Probe schreibt ausschließlich das reflektierte `DynamicLocomotion.state` über die vorhandene Lua-API. Sie schreibt weder Gravity noch Velocity oder Transform und verwendet keine Funktions-RVAs oder Patches. Der Enumwert wird über den Namen `BuildingFlying` aufgelöst. Eine feste numerische 11 steht nicht im Lua-Mod.

Das Ergebnis betrifft diesen originalen Enginezustand und die unabhängige Gravity-Wirkung. Dreidimensionale Steuerung, Kollision, der komplette Builder-UI-Modus und die Wirkung bestehender Glider-Mods sind damit nicht automatisch nachgewiesen.

## Ablauf und Wiederherstellung

1. Auf aktuelle Lese- und Schreibgates, eine Session und die originale Enum-Reflection warten. Beim Laden erfolgen keine ECS-Aufrufe oder Schreibzugriffe.
2. Den inkrementellen Query nach `PlayerInput`, `DynamicLocomotion` und `Gravity` fortsetzen, bis genau eine Entity gefunden wird. Deren opaken, generationgeprüften Handle und diese Session binden.
3. Auf einen bekannten State-Namen ungleich `BuildingFlying` und `Gravity.isActive=true` warten. Eine bereits ausgeschaltete Gravity ist kein Nachweis der Probe.
4. Höchstens einen erfolgreichen Apply ausführen:

   ```lua
   runtime.ecs.write_fields(entity, "keen::ecs::DynamicLocomotion",
       {state="BuildingFlying"}, {state=savedBaselineState})
   ```

5. An mindestens drei aufeinanderfolgenden 100-ms-Updates den State und das unabhängig gelesene `Gravity.isActive=false` beobachten. Jede Gravity-Lesung wird von zwei State-Lesungen und einer Session-Lesung eingerahmt.
6. Höchstens einen erfolgreichen Restore ausführen:

   ```lua
   runtime.ecs.write_fields(entity, "keen::ecs::DynamicLocomotion",
       {state=savedBaselineState}, {state="BuildingFlying"})
   ```

7. An mindestens drei Updates einen bekannten State ungleich `BuildingFlying` und `Gravity.isActive=true` beobachten. Die Engine darf den Baseline-State nach dem Restore regulär weiterentwickeln.

Die Expected-Tabelle muss im selben internen Snapshot geprüft werden, der den bestehenden nativen CAS speist. Damit überschreibt ein zwischen Lua-Read und API-Aufruf extern geänderter State keine fremde Änderung. Die bestehenden Generation-, Layout- und Guardprüfungen bleiben erforderlich.

Busy- und Snapshotkonflikte führen zu frischen Reads und begrenzten Wiederholungen. Bei Weltwechsel wird nicht in die neue Welt geschrieben. Ein anderer State wird erhalten und als externer Übergang gemeldet. Nach einem unsicheren oder werfenden Schreibaufruf erfolgen weder automatische Wiederholung noch blinder Restore.

Ein Apply ohne nachgewiesene Gravity-Wirkung führt nach etwa zehn Sekunden trotzdem in den begrenzten Restore. Restore-Retries laufen höchstens etwa 30 Sekunden. Ein Unload versucht nur einmal einen erneut geprüften Restore, sofern die Probe noch Eigentümer des States ist und kein endgültiger Abbruch vorliegt. Keine Ausführung fordert mehr als zwei erfolgreiche Schreibzugriffe an. Die Zeitangaben gelten bei regelmäßigen 100-ms-Updates und verlängern sich um die Aufrufdauer.

## Ergebnis und Live-Nutzung

Vor dem Liveversuch eine regulär gespeicherte Offlinewelt laden und am sicheren Boden stehen. Während der kurzen Probe nicht selbst den Flug-/Baumodus wechseln. Die Probe wartet selbst auf aktive Gravity und bindet genau einen passenden Spieler. Sie wird separat installiert und gestartet, nachdem der passende neue Runtimebuild geprüft ist.

Alle Meldungen beginnen mit `[SF_ENGINE_FLIGHT_PROBE_V1]`. Vollständiger Nachweis:

- `ENUM_RESOLVED`, `TARGET_BOUND`, `BASELINE`.
- `APPLIED` und drei `ENGINE_SAMPLE` mit `state=BuildingFlying gravity_active=false`.
- `ENGINE_GRAVITY_APPLY_OBSERVED`.
- `RESTORE_APPLIED` und drei `RESTORE_SAMPLE` mit normalem State und `gravity_active=true`.
- `ENGINE_GRAVITY_RESTORE_OBSERVED`.
- `SUMMARY result=COMPLETE successful_writes=2` und `cleanup_required=false`.

Ein API-Return allein gilt nicht als Engine-Wirkungsnachweis. Für fehlende Gravity-Wirkung, externe Zustände, Timeouts und unsichere Ergebnisse gibt es getrennte Ergebnisnamen. `cleanup_required=true` bedeutet ausdrücklich, dass kein abgeschlossener Restore behauptet wird. Nach Prozessende oder Weltwechsel kann diese Lua-Probe keine weitere Wiederherstellung ausführen.

## Lokale Prüfungen

```powershell
& 'target/runtime-discovery-venv/Scripts/python.exe' 'diagnostic-mods/live-engine-flight-probe-check.py'
```

24 private Lifecyclefälle unter echter Lua 5.4 sind erfolgreich. Sie decken verzögerte Bereitschaft und Querys, Busy-/CAS-Retries, anfänglich inaktive Gravity, schon aktiven Flugzustand, fehlende Enumdaten, falsche Enumformen, mehrere Spieler, externe Stateänderungen vor und nach Apply, Expected-State-Rennen bei Apply und Restore, Welt-/Generationwechsel, unsichere und werfende Writes, Unload, Cleanup-Timeout, verspätete oder ausbleibende Gravity, flackernde Gravity und externe Reaktivierung ab.

Der Fixture prüft zusätzlich die Quellcodeform des bestehenden Enum-Validators und -Converters. Bekannte Werte werden als Enum-Name-String dekodiert und akzeptiert. Das ist eine Quellcodeprüfung und ein strikter Mockvertrag, keine ausgeführte Produktionscodecprüfung. Die privaten Prüfungen öffnen keinen Spielprozess. Sie ersetzen den noch offenen Liveversuch nicht.
