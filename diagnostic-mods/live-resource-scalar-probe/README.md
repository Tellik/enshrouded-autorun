# Kontrollierte ResourceService-Scalarprobe

**Vorbereitet, nicht installiert.** Separater temporärer Entwicklermod für den neuen `game.assets.write_loaded_field`-Pfad. Installation erst nach Review und mit dem passenden neuen Runtimebuild.

## Genaues Ziel

Die Probe verwendet ausschließlich die geladenen `keen::ItemInfo`-Ressourcen des originalen ResourceService. Sie inventarisiert den gesamten geladenen Katalog in Batches von 32 Ressourcen und erkennt echte Gleiter durch `Resource.data.equipment.slot == "Glider"`. Der Name wird gegen `keen::EquipmentSlotType` aus der aktuellen Reflection geprüft. Eine feste numerische Slot-ID ist nicht im Lua-Mod eingebaut. Gewöhnliche ItemInfo-Einträge mit ebenfalls vorhandenen Glider-Defaultfeldern zählen nicht als Gleiter.

Aus allen echten Gleitern wird genau eine Ressource gewählt, sortiert nach GUID und Partindex. Die Logs nennen deren vollständige Identität und ItemId. Das ist keine Aussage darüber, welchen Gleiter der Spieler ausgerüstet hat. Aus der bisherigen lesenden Capture wären vier Gleiter zu erwarten. Die Probe verlangt keine fest eingebaute Anzahl und keine fest eingebaute GUID.

Der belegte Pfad lautet `equipment.gliderConfig.yawAngleSpeed`. Die Probe prüft ausschließlich einen echten Scalar-Schreibpfad mit unabhängigem ResourceService-Readback. Sie behauptet keine Gameplaywirkung, keinen vollständigen Modtransfer und keine Änderung des festen Glider-Pitchlimits.

## Ablauf

1. Auf `runtime.assets.write`, die originale Enum-Reflection und eine aktive `runtime.world.session_id()` warten. Beim Laden erfolgen keine Ressourcenaufrufe oder Writes.
2. Den vollständigen geladenen ItemInfo-Katalog prüfen. Doppelte ResourceIds, falsche Quellen oder unlesbare Snapshots führen vor dem ersten Write zum Abbruch. Die Ressource muss `source="loaded-runtime"` und `loaded=true` haben.
3. Exakt einen echten Gleiter wählen. Session, GUID, Partindex und ItemId binden. Mit einem neuen `game.assets.get_loaded_resource(guid, "keen::ItemInfo", part)` die Baseline lesen.
4. Höchstens einen erfolgreichen Apply über die vorhandene Assets-API ausführen:

   ```lua
   game.assets.write_loaded_field(resource,
       "equipment.gliderConfig.yawAngleSpeed", baseline + 1, baseline)
   ```

   Baseline und Inkrement müssen endlich sein. Die Probe rundet durch Lua 5.4 `string.pack` und `string.unpack` nach Float32 und verlangt eine exakte Differenz von eins. Die echte Runtime stellt die standardmäßige Stringbibliothek bereit.

5. Einen weiteren neuen Resource-Service-Snapshot mit `get_loaded_resource` anfordern und daraus `.data.equipment.gliderConfig.yawAngleSpeed` lesen. Der vom Writer zurückgegebene Resource-Wert wird nicht als unabhängiger Readback verwendet.
6. Mit einem neuen Snapshot und erwartetem Wert `baseline+1` höchstens einen erfolgreichen Restore auf die Baseline ausführen.
7. Wieder mit einem neuen `get_loaded_resource`-Aufruf die ursprüngliche Baseline prüfen.

`Resource.data` wird nur gelesen. Der Mod verwendet keine Dateiresourcen als Fallback, keine ECS-Schreibzugriffe, keine Engine-Funktionsaufrufe und keine direkten Adressen. Die vorhandene API löst den Typ, Feldpfad und ResourceId auf und führt den einzelnen nativen 32-Bit-CAS aus.

## Grenzen und Fehlerverhalten

Die Probe fordert höchstens zwei erfolgreiche Writes an. Busy- und Snapshotkonflikte führen nur zu begrenzten Wiederholungen mit frischen Reads. Der vierte Parameter des Writers bindet jeden Versuch an die erwarteten aktuellen Scalarbits. Ein fremder Wert wird erhalten.

Bei anderer Session, anderer Glider-/ItemId-Identität oder fehlender Ressource wird kein unbedingter Restore ausgeführt. Wenn ein externer Teilnehmer bereits die Baseline hergestellt hat, reicht ein unabhängiger Readback und es erfolgt kein zweiter Write. Eine beliebige externe Scalaränderung wird nicht durch die alte Baseline überschrieben.

Nach einem als unsicher gemeldeten oder werfenden Schreibaufruf stoppt die Probe ohne automatischen Retry oder blinden Restore. `cleanup_required=true` bleibt ausdrücklich bestehen. Zusätzlich sperrt der neue native Fault-Latch nach einem unsicheren Ergebnis sämtliche weiteren Resourcewrites bis zur expliziten Neuinitialisierung.

Readback-Retries laufen höchstens etwa zehn Sekunden, Restore-Retries höchstens etwa 30 Sekunden. Ein ausbleibender Apply-Readback führt trotzdem in den begrenzten scoped Restore. Ein Unload kann genau einen erneut geprüften Restoreversuch ausführen. Die Zeiten gelten bei regelmäßigen 100-ms-Updates und verlängern sich durch die Aufrufdauer. Nach Prozessende oder Weltwechsel kann Lua keine weitere Wiederherstellung ausführen.

## Ergebnis

Alle Meldungen beginnen mit `[SF_RESOURCE_SCALAR_PROBE_V1]`. Ein vollständiger Durchlauf verlangt:

- `INVENTORY_START`, `RESOURCE_SELECTED`, `BASELINE`.
- `APPLIED`, anschließend `RESOURCE_APPLY_OBSERVED` aus neuem `get_loaded_resource`.
- `RESTORE_APPLIED`, anschließend `RESOURCE_RESTORE_OBSERVED` aus neuem `get_loaded_resource`.
- `SUMMARY result=COMPLETE`, `successful_writes=2`, `cleanup_required=false`.

Jede Zusammenfassung trägt `gameplay_verified=false`. Fehlender Readback, fremde Änderungen, Weltwechsel, Timeout und unsichere Ergebnisse haben eigene Resultatnamen. Ein erfolgreicher API-Return allein gilt nicht als unabhängiger Readback.

## Private Lifecycleprüfung

```powershell
& 'target/runtime-discovery-venv/Scripts/python.exe' 'diagnostic-mods/live-resource-scalar-probe-check.py'
```

27 private Fälle unter echter Lua 5.4 sind erfolgreich. Die Fixture liefert unveränderliche Resource-/Daten-Snapshots und verbietet den Zugriff auf `.data` des Writer-Returns, damit nur ein neuer Getter als unabhängiger Readback zählen kann. Sie prüft mehrere Inventarbatches, echte Gleiter gegenüber gewöhnlichen ItemInfo-Defaultfeldern, deterministische Einzelauswahl, Busy-/CAS-Retries, fehlende Gates und Enumdaten, falsche Quellen und Enumformen, doppelte ResourceIds, fehlende Gleiter, Weltwechsel, externe Werte und Item-/Slotänderungen, Unload, fehlende Ressource, Apply-/Restore-Rennen, unsichere und werfende Writes sowie Readback- und Restore-Timeouts.

Die private Fixture öffnet keinen Spielprozess und ersetzt weder einen Produktionscodec- noch einen Live-Schreibnachweis. Der Liveversuch ist noch offen.
