# Kontrollierter Lua-Schreibprobe

Temporärer Entwicklermod `sf-live-runtime-write-probe`. Die Spielinstallation wird durch dieses Artefakt nicht verändert. Installation und Freigabe erfolgen nach dem separaten Review des nativen Schreibvertrags.

## Ablauf

Der Probe wartet auf die bestehenden Lese- und Schreibgates sowie eine von null verschiedene `runtime.world.session_id()`. Er setzt den inkrementellen Query nach `PlayerInput`, `Stamina` und `NetworkPlayerAttributes` fort, bis genau eine Entity gefunden wird. Er bindet deren opaken Handle und diese konkrete Session. Es wird keine feste Entity-ID verwendet und kein lokaler Spieler vorgetäuscht.

Bei voller Stamina müssen die echten Base-/Adder-/Maximumwerte und der unabhängig gelesene Netzwerk-Maximalwert übereinstimmen. Dann erfolgen höchstens zwei erfolgreiche Änderungen:

1. `runtime.ecs.update_attribute(entity, "Stamina_Max_Adder", original + 1)`.
2. Nach unabhängiger Root- und Netzwerkbeobachtung dieselbe API mit dem ursprünglichen Adder.

Beide Aufrufe verwenden die originale, in der bestehenden API dekodierte Rootberechnung. Die Logs enthalten ihren tatsächlichen `Stamina_Max`-Berechnungstrace mit Current-/Maximumbits. Die Stamina-Currentwerte werden bei einer Wiederherstellung neu berechnet. Ein zwischenzeitlich verbrauchter Current wird nicht durch den alten Baseline-Current ersetzt.

Der Probe verwendet weder Raw-Writes noch Modifier, Patches oder `runtime.functions`.

## Grenzen und Wiederherstellung

Bei einer anderen Session oder Änderungen an Base, Minimum, Rechargeparametern, Adder oder Maximum überschreibt der Probe keine fremden Werte mit alten Baselines. Er meldet den Abbruch und gegebenenfalls `cleanup_required=true`.

Busy- und Snapshotkonflikte werden mit frischen Reads erneut versucht. Ein als unsicher gemeldetes Ergebnis oder eine Ausnahme aus dem Schreibaufruf wird ausdrücklich nicht als unveränderter Zustand behandelt. Danach erfolgt kein blinder weiterer Schreibversuch.

Nach dem Apply wird der Netzwerk-Maximalwert höchstens etwa zehn Sekunden beobachtet. Auch wenn diese Beobachtung fehlschlägt, versucht der Probe anschließend die begrenzte Wiederherstellung. Restore-Retries laufen höchstens etwa 30 Sekunden. Ein Unload führt nur einen zusätzlichen, erneut auf Session und Scope geprüften Restoreversuch aus, sofern noch kein endgültiger Scope-/Unsicherheitsabbruch vorliegt. Die Zeitangaben basieren auf 100-ms-Updates und können durch Aufrufdauer länger ausfallen.

Wird die Runtime beendet oder die Welt gewechselt, bevor ein Restore möglich ist, kann ein Lua-Mod nicht weiterlaufen. Ein solcher Fall wird nicht als erfolgreicher vollständiger Test gemeldet.

## Verwendung und Marker

Nur den Ordner `diagnostic-mods/live-runtime-write-probe` als separaten Entwicklermod in den von der Runtime verwendeten Modordner übernehmen. `diagnostic-mods/live-runtime-write-probe-check.py` gehört nicht in das Modpaket. Erst nach Prüfung und Freigabe des nativen Schreibvertrags mit geladener Welt und genau einem passenden Spieler starten. Der Probe wartet selbst auf volle Stamina.

Alle Meldungen beginnen mit `[SF_TYPED_WRITE_PROBE_V1]`. Für den vollständig erfolgreichen Durchlauf müssen diese Beobachtungen vorhanden sein:

- `TARGET_BOUND`, `BASELINE`.
- `APPLIED`, `APPLY_CALCULATION_TRACE`, `APPLY_ROOT_OBSERVED`.
- `NETWORK_APPLY_OBSERVED`.
- `RESTORE_APPLIED`, `RESTORE_CALCULATION_TRACE`, `RESTORE_ROOT_OBSERVED`.
- `NETWORK_RESTORE_OBSERVED`.
- `SUMMARY result=COMPLETE`, `successful_writes=2` und `cleanup_required=false`.

Eine fehlgeschlagene Netzwerkbeobachtung hat einen eigenen Ergebnisnamen. Sie wird nicht durch einen erfolgreichen API-Return ersetzt.

## Private Lifecycleprüfung

Ausgeführt mit der vorhandenen lokalen Lua-5.4-DLL:

```powershell
& 'target\runtime-discovery-venv\Scripts\python.exe' 'diagnostic-mods/live-runtime-write-probe-check.py'
```

14 Szenarien sind erfolgreich. Sie prüfen unready Startup, inkrementelle Querys, Busy-/CAS-Retries, verspätete und ausbleibende Netzwerkbeobachtung, anfänglich verbrauchte Stamina, fortschreitenden Current nach dem Apply, Weltwechsel, externe Base-/Adderänderungen, unerwarteten Apply-Scope, mehrere Spieler, fehlendes Schreibgate, unsichere Apply-/Restoreergebnisse, Unload-Cleanup und Restore-Timeout. Der Test verwendet private Mockdaten und öffnet keinen Spielprozess. Ein erfolgreicher Mocklauf ist kein erfolgreicher Live-Schreibtest.
