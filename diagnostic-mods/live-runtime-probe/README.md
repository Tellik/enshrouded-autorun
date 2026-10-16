# Temporäre lesende Runtime-Probe

Dieses Verzeichnis ist ein installierbarer Entwickler-Mod mit der ID `sf-live-runtime-probe`. Es verwendet ausschließlich vorhandene Lese-APIs. Es enthält keine Patches, Schreibzugriffe, frei aufrufbaren Enginefunktionen oder zusätzliche Executor-Schnittstelle.

Den vollständigen Ordner mit `mod.json`, `extended.mod.json` und `src/mod.lua` als eigenen Mod installieren und den neuen Loader samt passender nativer DLL laden. Danach eine Welt öffnen. `on_load` verlangt keine bereits verfügbare Runtime. Im Menü und während des Weltladens protokolliert die Probe `WAIT` und versucht es erneut.

Alle Meldungen beginnen mit `[SF_LIVE_PROBE_V1]`:

- `START` bestätigt das tatsächliche Laden des Lua-Mods.
- `REGISTRY_OK` zeigt Originalregistrierungen und Storage-Typen.
- `ATTRIBUTE_CATALOG_OK` zeigt den ursprünglichen Attributkatalog.
- `ASSET_LIST_OK` bestätigt die geladene ItemInfo-Liste.
- `ASSET_DATA_OK` bestätigt den wirklichen Zugriff auf `.data` und verschachtelte Gleiter-/Platzierungsfelder. Eine Liste allein genügt nicht.
- `PLAYER_ROOT_OK` zeigt echte Health-/Stamina-Roots von Entities mit `PlayerInput`, `Stamina` und `Health`. Es wird keine feste Entity-ID verwendet und keine lokale Auswahl bei mehreren Spielern behauptet.
- `PLAYER_FIELDS_OK` zeigt dazu vorhandene `NetworkPlayerAttributes`-Felder durch `read_fields`.
- `CORE_READ_OK` verlangt mindestens ein erfolgreich dekodiertes Live-Item und erfolgreiche Spielerroots. Der vollständige Ressourcenabgleich folgt in `ASSET_SCAN_DONE`.
- `TYPE_RESULT` unterscheidet `observed`, `unobserved`, `presence-but-unreadable`, `typed_partial` und `query_failed`.
- `SUMMARY reason=complete` beendet die vollständige Liste. `budget-exhausted` oder `unloaded` weist verbleibende `untested` Typen aus.

Je Update werden höchstens vier Chunks desselben inkrementellen Queries ausgeführt. Erst nach dessen vollständigem Ergebnis folgt der nächste Typ. Anschließend wird maximal ein Entitysample pro Update geprüft. Eine vollständige Inventur der Hosttabelle mit einer Million Slots kann mehrere Minuten dauern. Das Budget beträgt 20.000 aktive Updates bei 25 Millisekunden geplantem Intervall, ungefähr acht Minuten und 20 Sekunden zuzüglich Aufrufzeit. Welt-unready-Phasen verbrauchen dieses aktive Budget nicht.

Bei einem beobachteten Typ prüft die Probe `read_bytes`, `read`, `runtime.values.decode` und bis zu drei reflektierte Inline-Feldpfade über `read_fields`. Fehlende Spalten werden nicht durch Defaultwerte ersetzt. Von mehreren passenden Entities werden bei fehlgeschlagenem Rohlesen höchstens drei versucht.

Nach dem Test den temporären Mod deaktivieren oder entfernen. Er ist kein Produktionsmod und wurde nicht in das Releasepaket aufgenommen.
