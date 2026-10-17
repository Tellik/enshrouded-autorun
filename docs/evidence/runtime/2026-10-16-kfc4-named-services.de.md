# KFC4: Typisierte originale Services und Haltbarkeit

Untersucht wurde der Client Steam-Build 25966634 mit SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Die statische Untersuchung las die EXE als Datei. Der ergänzende Liveabgleich las PID 5688 ausschließlich mit `ReadProcessMemory`. Dabei wurden keine Spielfunktionen aufgerufen und keine Spielwerte verändert. Eine Spielwirkung des neuen Service-Schreibpfads ist damit noch nicht nachgewiesen.

## Originaler Verbraucher und eigener Datenbesitz

Der Toolkit-Eingriff „Infinite Weapon Durability“ verändert den Signed-Adder bei RVA `0x3af24c` im Helfer `0x3af180`. Die Engine besitzt jedoch bereits den direkten Eingang `keen::ecs::GameSettings.enableDurability`. Dieser ist ein reflektiertes Bool bei Offset `0x14` in einem 144 Byte großen Struct. Der Typ ist kein Entity-Komponentensingleton. Die Engine registriert ihn als benannten DI-Service `ecs::GameSettings`.

Der Originalaccessor `0xe88bc0` lädt den Settingspointer aus `BalancingRegistry+0x90`, danach genau das Bool bei `Settings+0x14`. Das System `actor_spawn_entity` ruft ihn bei `0x224e78` auf und überspringt bei false den Haltbarkeitsabzug. Der zweite ereignisbezogene Verbrauchspfad bei `0x235330` prüft denselben Accessor bei `0x235355` und kehrt bei false zurück. Die Dauerverbrauchsschleife `durability_burndown` ist ein eigener Pfad für kontinuierlich verbrauchte Gegenstände. Die neue Einstellung behauptet keine Unterbindung sämtlicher Brennstoff- oder Zeitverbräuche.

`admin_player` beginnt bei `0x2cab10`. Sein Originaldeskriptor bei `0x1dfc640` enthält diese benannten Lese-Services: `BalancingRegistry`, `BasicPlayerInfos`, `ecs::ServerProgressSystem` und `impact::AttributeRegistry`. Im getrennten Mutable-Span bei Deskriptoroffset `0xb0` folgen `ecs::GameSettings` und `g38_daytime::Daytime`. Der Settingspointer besitzt damit eine originale Mutable-Bindung und erscheint an Serviceordinal 4.

Bei einer neuen `AdminChangeGameSettingsAction` kopiert die Originalfunktion bei `0x2cc0d2..0x2cc140` alle 144 Bytes unmittelbar in diesen Service. Sie verwendet dafür keinen ECS-Komponentenindex und keinen ECS-Änderungstracker. Anschließend quittiert sie die Aktionsversion und erzeugt `GameSettingsChangedEvent`. Der neue API-Zugriff übernimmt die kurzzeitig gegatete Änderung der echten Inline-Daten. Er erzeugt dieses Ereignis nicht. Die beschriebenen beiden Haltbarkeitsverbraucher lesen das Bool bei jedem Aufruf unmittelbar. Änderungen anderer Settingsfelder können weitere Benachrichtigungen benötigen. UI- und Persistenzfolgen sind durch einen reinen Servicespeicherzugriff nicht zugesagt.

## Namensbindung und lesender Liveabgleich

Der vorhandene originale Binder und seine Namensmap sind in `devdata/embervale-toolkit-3.1.0/glider-service-provenance.md` beschrieben. Relative Basis ist der bereits kanonisch aufgelöste `EcsView.registry_owner`.

| Feld | Offset |
| --- | --- |
| Services-Vectorpointer, Anzahl | `0x7459a8`, `0x7459b0` |
| Recordstride | `0xe8` |
| Recordname-Pointer, Länge, FNV-Hash | `0`, `8`, `0xa0` |
| Originaler und alternativer Servicepointer | `0xa8`, `0xb0` |
| Hashmap-Belegungsbits, Kapazität | `0x7459d8`, `0x7459e0` |
| Hashmap-UInt32-Schlüssel, UInt16-Recordindices | `0x7459f8`, `0x745a10` |
| Hashmap-Zähler | `0x745a24` |
| Systemrecords, Stride, Anzahl | `0x745a28`, `0x1f20`, `0xe14a28` |
| Normaler Bindingblock, Countbyte | `0x1710`, `0x1b10` |
| Alternativer Bindingblock, Countbyte | `0x1b18`, `0x1f18` |

Die Runtime berechnet den vorhandenen FNV-Namenshash zentral, validiert den vollständigen Originalnamen und lehnt doppelte Namen ab. Sie bestätigt den ursprünglichen Mutable-Deskriptor, die Function-RVA und beide Bindingblöcke. Ein Mod erhält weder diese Offsets noch einen Prozesspointer.

Live enthielt der Services-Vector 69 Records. Record 45 hieß vollständig `ecs::GameSettings`. Sein FNV-Hash war `0x943eebdd`. Normaler und alternativer Pointer zeigten beide auf `0x10027071e90`. Beide Bindingblöcke von `admin_player` enthielten an Ordinal 4 exakt diesen Pointer. `BalancingRegistry+0x90` zeigte auf dieselben 144 Bytes. Die fünf Playerfaktoren waren 1, `enableDurability` war true. Der Setup-Pfad `0x4943f0` registriert bei `0x4969af` den Namen vom String-RVA `0x150b690` mit Länge `0x11` und Pointer aus `[RDI+0x488]`. Der Livebesitzer enthielt dort wiederum exakt dieselbe Settingsadresse. Diese Adressen sind Belege der Capture und keine Modkonstanten.

Die Reflection des Structs besitzt Qualified-Hash `2298714128`, Metadaten-RVA `24402336`, Registryslot-RVA `24122752` und Größe 144. Die Runtime prüft den aktuellen Metadatenslot, die Größe und den Qualified-Hash bei jedem Zugriff.

## Gemeinsamer Lua-Vertrag für alle Mods

```lua
local settings, reason = runtime.services.get("ecs::GameSettings")
if not settings then return end
local fields = settings:read_fields({ "enableDurability" })
if fields then
    local ok, error = settings:write_field(
        "enableDurability", false, fields.enableDurability)
end
```

`RuntimeService` ist eine sitzungsgebundene View mit schreibgeschützten Eigenschaften `key`, `type`, `scope`, `session_id` und `write_supported`. Der Scope ist `host-world-shared`. Alle betroffenen Verbraucher dieser autoritativen Welt teilen den Wert. Der Mod muss seinen eigenen bestätigten Wert mit Pflicht-Expected restaurieren und darf eine alte View nach einem Weltwechsel nicht weiterverwenden.

Auflösung und Feldcodecs enthalten keinen Survival-Schalter. Weitere belegte Services können über denselben `serviceLayout.contracts`-Mechanismus freigegeben werden. Ein Name allein legitimiert keine Speichergröße oder Schreibphase. Nicht belegte, nicht reflektierte oder Speicher besitzende Services werden abgelehnt. Der erste freigegebene Vertrag ist der originale inline POD-Service `ecs::GameSettings`. DS, Blob- und Referenzdeskriptoren werden nicht schreibbar gemacht.

## Lebensdauer, Schreibphase und Fehler

`EcsRuntime::SessionGuard` hält den bestehenden Bindungsepoch fest. Danach hält `EngineContext::EcsWriteGuard` die bereits belegte originale globale Threadregistrierung und das exklusive Gate der zugehörigen autoritativen Scene. Die View, Rolle und Besitzkette werden vor und nach dem Zugriff ohne weitere Host-Mutexes geprüft. Während des Originalgates werden keine Speicherallokationen, Dispatcherwartepfade oder Engineaufrufe ausgeführt. Der Service gehört dem gehaltenen Originalregistrybesitzer derselben Scene. Der vollständige Namensrecord und beide systemseitigen Bindungen werden unter diesem Gate erneut gelesen.

Der native Zugriff verlangt vollständig beschreibbaren privaten Datenspeicher. Ausführbare Seiten, Images, Referenzspeicher und fremde oder gewechselte Bindungen bleiben gesperrt. Beim Schreiben wird der gesamte frische Snapshot mit den erwarteten Bytes verglichen. Der gemeinsame Reflection-Codec erzeugt ausschließlich eine Maske der ausdrücklich gewählten Inline-Wertblätter. Padding und alle übrigen Settingsbytes bleiben identisch. Anschließend liest der native Pfad das vollständige Ergebnis frisch unter demselben Gate zurück und bestätigt nochmals die Bindung und Besitzerkette.

Ein Konflikt verändert keine Daten. Ein nicht verifizierter Fehler stellt den vorherigen Snapshot wieder her, soweit dies überprüfbar möglich ist. Ein ungewisser Ausgang setzt die gemeinsame native Schreibsperre der aktuellen Sitzung. Ein Mod darf darauf keinen blinden Retry ausführen. Der neue Pfad hebt die bestehende Sperre für B-getracktes `Durability` nicht auf.

## Zusätzlicher originaler Reparaturbefehl

`inventory_actions` bei `0x3fd000` vergleicht `PlayerInput.fromAdminClient.adminRepairAllItemsAction.versionData.version` bei `+0x598` mit `ServerConsumedPlayerInput.consumedAdminRepairAllItemsAction.version` bei `+0xd4`. Bei einer neuen Version ruft es den originalen Reparaturhelfer `0x4138b0` auf und quittiert danach die Version. Der Helfer folgt `InventorySetup.linksEntities`, danach `Inventory.slots[n].data.pide`, löst die konkrete Item-Entity auf und schreibt ihr echtes `Durability.current = maximum` über den originalen Mutable-Accessor. `ItemState.containedInSlotId` beschreibt die Rückbeziehung zum Inventarslot.

Der ursprüngliche Produzentenkanal ist `ClientPlayerInput.adminCommands` bei `+0x3b8`, nicht ein beliebiger Eingriff in den vom Netzwerk importierten autoritativen `PlayerInput`. Die Versionsbreite ist UInt32. Die Consumer vergleichen nur Gleichheit, ohne Null-Sonderfall. Berechtigungen werden über `BasicPlayerInfos` und den originalen Spielerflagtest `0x3b3ed0` geprüft. Die neue Serviceschnittstelle manipuliert diese Aktionszähler nicht. Der Reparaturkanal ist zusätzlicher Beleg für vorhandene Engineoperationen, kein Ersatz für den noch gesondert zu belegenden Client-Produzenten- und Importvertrag.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Bestand | Toolkit-Registrierung, originale EXE-Anweisungen, vollständiger Change-Tracking-Bericht und Service-Binder-Bericht gelesen. |
| Architektur | Vorhandener Reflection-Codec, kanonischer Registrybesitzer, gemeinsame Sitzung und originale Scene-/Lifetime-Gates wiederverwendet. |
| Belege | Originale direkte Verbraucher und Mutable-Schreibstelle sowie lesende Liveidentität aller beteiligten Settingspointer bestätigt. |
| Prüfung | Neue Dateien und betroffene Ausschnitte statisch geprüft. Die private native Fixture `kfc-runtime-test-service` für Originalnamen, Mutable-Bindung, Sitzung, CAS und Zeigerfehler ist angelegt. Keine Tests und kein Gameplay-Schreibtest durch diese Untersuchung ausgeführt. |
| Offen | Livewirkung der neuen API und UI-/Persistenzbenachrichtigungen für andere Settingsänderungen. |
