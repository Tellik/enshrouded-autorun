# KFC4-Gleiter: Von den funktionierenden Patchstellen zu echten Eingaben

## R7-Liveprüfung und Korrekturen

Im Prozess 18332 waren beide R7-DLLs anhand ihrer SHA-256-Prüfsummen bestätigt.
Der Nutzer beobachtete weiterhin normales Fliegen ohne erkennbare Verbesserung.
Die Statusaufzeichnung zeigte wiederkehrende unvollständige ECS-Abfragen.
Der gemeinsame Query-Pfad verwarf nach 50 ms seinen letzten Fund, während der
nächste Gesamtscan noch lief. Die Korrektur hält bekannte Ergebnisse während
der Aktualisierung verfügbar und prüft für jede Rückgabe erneut ID, Generation,
aktive Tabellenzugehörigkeit und angeforderte Komponenten. Neue Entities werden
mit dem nächsten vollständigen Snapshot sichtbar. Der erste Scan bleibt bis zu
seinem Abschluss ausstehend. Die private native Prüfung besteht 126 Checks.

Zusätzlich hatte der Mod den bereits belegten effektiven Actor-State nicht
vollständig nachgebildet. Bei aktiver Aktion gilt
`(currentState | setStateMask) & ~unsetStateMask`. Die HangGliding-Auswahl liest
nun alle drei reflektierten Masken und berücksichtigt die ursprüngliche
Aktivitätsbedingung. Dieser Fehler konnte sämtliche Gleiterregler vom echten
Gleiter fernhalten. Ein Regressionstest deckt die Aktionsmasken ab.

Diese Korrekturen benötigen eine erneute Liveprüfung. R7 zählt nicht als
erfolgreicher Nachweis für Aufwärtsflug.

## Fortsetzung: belegter Datenpfad zur Aufwärtsneigung

Die frühere Schlussfolgerung zu `pitchAngleSpeed` war zu absolut. Die feste
Klemmung begrenzt ein Zwischenziel. Anschließend verwendet das unveränderte
Original `ItemInfo.equipment.gliderConfig.pitchAngleSpeed` für den Winkelschritt.
Eine negative endliche Rate erzeugt dort eine negative fertige Pitchrotation.
Null hält die Neigung und eine positive Rate führt zurück nach unten.

Das ist durch 108 Parameterfälle, 120 vollständige Originalrotationsschritte,
12 Interpolationsfälle und 177 zusätzliche Frames mit schwankendem Zeitschritt
belegt. Für die letzte Folge wurden die tatsächlichen Spielerwerte
`ActorRotation.gliderRotationSpeed.rotationSpeed = 200` und
`interpolationMode = ExponentialLerp` verwendet. Sie stammen aus einer rein
lesenden, zweimal identischen Capture der ursprünglichen TemplateResource.
Die Folge erreicht ungefähr −84,709 Grad, hält und senkt den Gleiter wieder.

SF Unlimited Flight verwendet diesen Datenpfad jetzt als aktiv abgetastete
Steuerung für `YawSpeedPitchSpeed`. Die Lua-Eingaben stammen aus
`ActorInput.desiredLocalMoveInput.z`. Die tatsächlich beobachtete Neigung folgt
aus `CurrentTransform.transform.orientation` und `Gravity.direction`.
Die korrekte Reflection heißt `orientation`, frühere Texte verwendeten dafür
teilweise fälschlich `rotation`. Beim Landen, unpassendem Eingabemodus oder
mehreren Spielern wird die Rate über die gemeinsame API wiederhergestellt.

Eine negative Rate hat keinen nativen Ablaufzeitpunkt. Eine lange Lua-Pause
kann sie über den angenommenen Abtasthorizont hinaus aktiv lassen. Die
begrenzte Regelung ist daher kein harter nativer Pitchgrenzwert und noch kein
Live-Nachweis der gesamten Flugphysik. `YawSpeedPitchTargetAngle` ist weiterhin
nicht gleichwertig unterstützt. Der feste Zwischenziel-Clamp bleibt vorhanden.

Die reproduzierbaren Belege liegen in
`devdata/embervale-toolkit-3.1.0/glider-post-clamp-original-20261016.json`,
`trace-glider-post-clamp-20261016.py` und
`player-glider-rotation-template-live-20261016.json`.

Untersuchtes Image: `enshrouded.exe`, Steam-Build 25966634, SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Ausgangspunkt sind die bekannten Toolkit-Eingriffe. Die Untersuchung hat weder ausführbaren Spielcode noch Ressourcen oder ECS-Werte verändert.

## Ergebnis und konkrete Wirkungskette

Die vier bekannten Regler lesen tatsächlich dieselben geladenen `keen::ItemInfo`-Bytes, die die bestehende Ressourcen-API erreicht. Der fehlende Bezug zum ECS ist nun bis zum ursprünglichen Komponentenindex belegt:

```text
PlayerInput + registrierter Actor
  → Actor.currentAction.triggerContext.usedItem.itemId
  → ursprüngliche g38_items::ItemRegistry
  → geladenes ItemInfo aus demselben ResourceService wie game.assets
  → ItemInfo.equipment.gliderConfig
  → ursprüngliche Winkel-/Geschwindigkeitsrechnung
  → Actor.rotationInterpolation bzw. Velocity.velocity
```

`Actor` ist im aktuellen Live-Registryeintrag Index 3, Größe 3.832, mit Runtime-Metadaten-RVA `0x19f5f10`. Die Namen und Typen stammen aus der Reflection und der aktuellen Engine-Registrierung. Der Index wird hier als Beleg angegeben und gehört nicht als Konstante in einen Mod.

## Start an den bekannten Originalanweisungen

| Toolkit-Effekt | Originalstelle | Tatsächlich geladener Eingang |
| --- | --- | --- |
| Glider Move Speed | `0x42b6d0` | `ItemInfo.equipment.gliderConfig.accelerationForward` |
| Glider Lift Speed | `0x42b8fb` | `ItemInfo.equipment.gliderConfig.airResistanceLongitudinal` |
| Glider Descend Speed | `0x42b886` | `ItemInfo.equipment.gliderConfig.airResistanceVertical` |
| Glider Turn Speed | `0x26ae36` | `ItemInfo.equipment.gliderConfig.yawAngleSpeed` |

Die Toolkit-Caves multiplizieren den jeweils geladenen Float32-Wert mit `max(float(Reglerwert) * 0.01f, 0.25f)`. Eine identische Änderung des konsumierten Koeffizienten würde deshalb denselben lokalen Recheneingang liefern. Die Bezeichnungen Lift und Descend sind Toolkit-Bezeichnungen. Die Originalfelder sind Widerstandskoeffizienten.

Diese Aussage ist eine Verbindung des belegten Datenflusses mit der belegten Reglerrechnung. Sie ist noch kein Live-Wirkungsnachweis einer neuen schreibenden API.

## Der tatsächliche Besitzer der Item-ID

### actor_rotation

Das System beginnt bei `0x269d30`. Der lokale Frame `F = rbp - 0x40` wird durch `0x9ba5a0` aufgebaut und durch `0x9b5490` mit den aktuellen Komponenten versehen. Die aktuelle originale Systeminstanz hat 15 lesende Runtime-Pointer und drei schreibende Accessoren. Ihr Runtime-Deskriptor an Position 15 enthält den Hash von `keen::ecs::Actor` und den ursprünglichen Komponentenindex 3.

Nach zwei Konfigurationspointern liegt der erste Schreibaccessor bei `F + 0x90`, somit `rbp + 0x50`. `0x269ec6` lädt diesen Actorpointer nach RBX. Er bleibt über den lokalen Slot `rbp - 0x60` mit dem Gleiterzweig verbunden. Bei `0x26a7eb` wird er erneut geladen. `0x26a831` lädt `[Actor + 0x38]` nach ECX und `0x26a834` ruft den originalen Item-Getter `0xe7a250` auf. Der Rückgabepointer geht nach RDI und bleibt bis zum Yaw-Zugriff `0x26ae36` erhalten.

### locomotion_execution

Das System beginnt bei `0x435d30` und erreicht den Gleiterhelfer `0x42b0e0`. Die aktuelle Systeminstanz besitzt neun lesende Pointer und sechs schreibende Accessoren. Der erste Schreibdeskriptor, Position 9, ist wiederum derselbe registrierte Actor.

Im lokalen Frame liegt dieser Actoraccessor bei `F + 0x68`. `0x42b1de` lädt seinen Komponentenpointer. `0x42b1e5` liest wiederum dessen `+0x38` und `0x42b1e8` ruft denselben Item-Getter auf. Der Rückgabepointer wird bei `0x42b1ff` nach R12 übernommen. Von dort lesen alle drei Geschwindigkeitsregler ihre Koeffizienten.

### Typgeprüfter Feldpfad statt Offsetvermutung

Die Reflection ergibt ohne zusätzliche Ersatzstruktur:

| Ursprünglicher Typ und Feld | Relativer Offset |
| --- | ---: |
| `Actor` erbt `BaseActor.currentAction` | 0 |
| `ActionSequence.triggerContext` | 0 |
| `ActionSequenceTriggerContext.usedItem` | `0x30` |
| `ItemInstanceInfo.itemId` | `0x08` |
| Gesamter Pfad `currentAction.triggerContext.usedItem.itemId` | `0x38` |

Der ursprüngliche Gleiterzweig prüft über `0x21a610` den effektiven Actor-State. Das Enum `keen::actor::StateFlag` benennt Bit 37 als `HangGliding`. Die zusätzlich geprüfte Aktivitätsmaske bei `Actor + 0x1b1` ist exakt `currentAction.playbackContext.state`, dessen reflektiertes `ActionSequenceFlags.IsActive` Bit 0 ist. Dies sind Eingaben und Zustandsbedingungen des Originals. Ein bloßes Setzen eines Statebits wird dadurch nicht zu einer vollständig gültigen Gleiteraktion.

## Service- und Ressourcenidentität im laufenden Spiel

Die ausschließlich lesende Capture von PID 24432 bestätigt:

- Die normale und alternative Servicebindung beider Originalsysteme zeigen auf dieselbe autoritative `g38_items::ItemRegistry` bei `0x300bc040138`.
- Die bisherige Assets-API erreicht über den Client eine andere ItemRegistry bei `0x3004f7d73e8`.
- Beide verwenden denselben ursprünglichen ResourceService bei `0x1001d5df660`.
- Alle **4.207** belegten Host-Item-ID-Einträge wurden gegen GUID, Typ und Part des clientseitigen Ressourcenindex verglichen. Für alle 4.207 ist der Datenpointer identisch und `ItemInfo.itemId` stimmt mit dem Mapkey überein.
- Genau vier dieser Einträge haben das reflektierte `equipment.slot = EquipmentSlotType.Glider`. Aus vorhandenen Gleiter-Defaultfeldern wird keine Gleiteridentität abgeleitet. Diese Felder existieren auch in gewöhnlichen ItemInfo-Ressourcen.

| Gleiter-GUID | Item-ID | Vorwärtskoeffizient | Längswiderstand | Vertikalwiderstand | Yaw in Grad/s |
| --- | ---: | ---: | ---: | ---: | ---: |
| `f5520401-f5aa-41cd-99e1-609b884cc492` | 284247111 | 1 | 0,215 | 1,6 | 60 |
| `80c9eaeb-7fa0-480d-a55f-91c2c51b383e` | 3357517884 | 1 | 0,155 | 3 | 55 |
| `1da8d177-3a28-43b1-b430-44b38022d925` | 1397467393 | 0,9 | 0,26 | 0,95 | 65 |
| `33834394-99c5-47b2-93b2-36aa6a2eaa73` | 1028290781 | 0,8 | 0,39 | 0,7125 | 70 |

Die Werte sind gerundete Darstellungen der tatsächlich gelesenen Float32-Werte. Der Spieler stand bei dieser Capture. Seine aktuelle Aktion war inaktiv, `usedItem.itemId` war null und `HangGliding` war falsch. Der konkrete Join einer aktiven Gleiteraktion bleibt deshalb für den nächsten gezielten Flugtest offen. Es wurde keine beliebige Gleiter-GUID als aktuell ausgerüsteter Spieler-Gleiter ausgegeben.

## Warum ein einzelnes vermeintliches Geschwindigkeitsfeld nicht genügt

Der Yaw-Koeffizient begrenzt den Zielwinkel-Schritt mit `yawAngleSpeed * GradNachRadiant * dt`. Pitch und Roll verwenden ihre eigenen Nachbarfelder. Daraus bildet die Engine eine neue Zielorientierung.

Danach liest `0x26b172` zusätzlich `ActorRotation.gliderRotationSpeed` aus der Konfiguration. Die Engine schreibt dessen Geschwindigkeit und Interpolationsmodus nach `Actor.rotationInterpolation.speed` beziehungsweise `.interpolationMode`. Die berechnete Zielorientierung geht nach `.targetOrientation`. Bei `speed == FLT_MAX` übernimmt der Abschlussweg diese direkt nach `CurrentTransform.transform.rotation`. Andernfalls arbeitet die ursprüngliche Interpolation zusätzlich mit `DynamicLocomotion.rotationSpeedFactor`, einem Movement-Attribut und weiteren Zuständen. Der Toolkit-Yaw-Regler ist damit eine konkrete Änderung der Zielwinkelrate innerhalb dieser Kette.

Die Vorwärtsrechnung addiert einen Koeffizientenanteil zur aktuellen `Velocity.velocity`. Sie hängt außerdem von Orientierung, Teilschritt, Aktivierungsfaktor und `DynamicLocomotion.gliderUpstreamUpwardsFactor` ab. Anschließend zerlegt die Engine die Geschwindigkeit, dämpft die drei Anteile mit den drei Widerstandskoeffizienten und addiert sie wieder. Die resultierende Geschwindigkeit wird bei `0x42b9bd`, `0x42b9c2` und `0x42b9c8` in dieselbe registrierte `keen::ecs::Velocity` geschrieben. Eine Verdoppelung eines Widerstands ist daher keine Verdoppelung einer Fluggeschwindigkeit.

## Originaler Ressourcen-Lifetimevertrag

Der ResourceService besitzt einen eigenen ursprünglichen Thread. Der Konstruktor `0x10617e0` registriert bei `0x1061bed` über `0xf22c00` einen Thread inline bei `Service + 0x128`. Sein Name lautet `KfcResourceContent`, die Länge ist 18 und der Callback liegt bei `0x105d7f0`. `0x106226a` setzt über `0xf3cf20(Thread, Thread)` den Ausführungskontext auf den Thread selbst.

Die Live-Capture bestätigt dieselben Beziehungen. Der Name liegt inline bei `Thread + 0x20`, die Kapazität ist 64, der Handle bei `Thread + 0x60`, der Kontext bei `Thread + 0x78`, Stop bei `Thread + 0x80` und der Callback bei `Thread + 0x88`. Die originale globale Threadliste enthält einen exakt passenden Eintrag. Initflag `Service + 0x7a4` ist eins. Threadheader und globale Registrierungszeilen waren beim Nachlesen unverändert.

Der Destruktor `0x1065090` stoppt zunächst die Arbeit und ruft bei `0x10650d6` den Unregister-/Joinpfad `0xf280a0` auf. Dieser benötigt den bereits separat belegten exklusiven Zugriff auf die globale Threadregistrierung. Erst danach wird die Payloadprotektion zurückgestellt und bei `0x106514a` der Reader über `0x1239170` freigegeben. Der Readerdestruktor gibt unter anderem seine Allokationen bei `Reader + 0x60` und `Reader + 0x78` frei.

Ein korrekt gehaltener ursprünglicher Shared-Guard dieser globalen Registrierung kann das Entfernen des exakt validierten Service-Threads vor diesem Abbau verhindern. Die Implementierung muss zuerst den Image-globalen Guard halten und erst dann den registrierten inline Namen zum Service zurückrechnen. Ein zuvor ungeschützt geliehener Servicepointer und eine nachträgliche Namensprüfung ersetzen das nicht. Die Serviceidentität muss zusätzlich wieder mit dem vorhandenen Ressourcenbesitzer übereinstimmen.

### Tatsächlicher Schreibschutz und begrenzter Scalarvertrag

Der Konstruktor ruft bei `0x10623aa` den Helfer `0xf3cb60` mit dem gesamten Payloadbereich und Modus eins auf. Dessen Importcall bei `0xf3cb9b` geht nach `KERNEL32.dll!VirtualProtect`, IAT-RVA `0x13af7a8`. Modus eins wählt `PAGE_READONLY`. Der Destruktor verwendet bei `0x106512b` Modus `0x0f`, der `PAGE_READWRITE` wählt. Die beiden direkten Aufrufer dieses Helfers sind Konstruktor und Destruktor. Der Helfer ist kein Unmap-Aufruf.

Live liegen alle vier geprüften Gleiterfelder in `MEM_COMMIT`, `MEM_PRIVATE` und `PAGE_READONLY`. Die Allocationprotektion ist `PAGE_READWRITE`. Der ursprüngliche Payload beginnt bei `0x20000000000` und umfasst 513.802.240 Byte. Alle vier Scalaradressen sind durch vier teilbar.

Die Leseschutzprotektion macht einen generischen Datenzugriff nicht grundsätzlich unmöglich. Ein enger neuer Vertrag kann unter dem korrekt validierten Lifetime-Guard einen einzelnen reflektierten, ausgerichteten 32-Bit-Scalar mit erwarteten Bits vergleichen und atomar ersetzen. Dafür muss die betroffene Datenseite kurz beschreibbar gemacht und ihre exakt vorherige Protektion in jedem Ausgang wiederhergestellt werden. Ein Fehler beim Wiederherstellen der Protektion oder ein nicht überprüfbarer Wert ist als unsicherer Ausgang zu melden. Dieser Vertrag darf keine ausführbaren Seiten erlauben und verspricht keine atomare Transaktion über mehrere Gleiterfelder.

Die gleichzeitigen ursprünglichen `MOVSS`-Leser müssen für einen solchen einzelnen ausgerichteten 32-Bit-Wechsel keine komplette Resource-Kopie erhalten. Die tatsächlich konsumierte Payloadidentität bleibt erhalten. Der ursprüngliche GameServer-Scene-Guard allein wäre dagegen kein Ressourcen-Lifetimebeleg, weil Client und Host dieselben Ressourcenbytes teilen.

Der Vertrag ist im nativen Quellcode inzwischen umgesetzt und gegen private Speicherobjekte geprüft. Die Lua-Anbindung, der neue vollständige Build und die kontrollierte Livewirkung sind getrennte Schritte. Dieser Bericht behauptet noch keinen erfolgreichen Schreibzugriff im Spiel.

### Native Umsetzung und privater Nachweis

`EngineContext::ResourceLifeGuard` hält den vorhandenen ursprünglichen globalen Threadregistry-Shared-Guard. Globale Registrierungsadressen, Zeilenformat und NativeThread-Felder werden aus dem schon vorhandenen `ecsWriteLayout` wiederverwendet. Es gibt keine zweite Liste dieser Offsets. Das zusätzliche `resourceWriteLayout` enthält nur die Service-Worker-Bindung, dessen Initialisierungsflag und den ursprünglichen Bulk-Payloadbereich. Der Readeroffset stammt aus dem vorhandenen `resourceLayout`.

Der Guard rekonstruiert den Service erst unter dem ursprünglichen Registry-Guard aus dem registrierten Inline-Namen und verlangt dieselbe Serviceidentität wie die bestehende Assets-Auflösung. Er validiert Self-Kontext, Originalcallback, Handle, Stopflag, Initflag, Reader und Bulkbereich. Er prüft die ursprüngliche Ownerkette vor und nach dem Scalarwechsel erneut. Sein Layoutsnapshot wird vor dem ursprünglichen Lock angelegt. Unter diesem Guard werden keine eigenen Mutexes genommen, Speicher allokiert oder Enginefunktionen ausgeführt.

`KfcRuntimeResourceCompareExchangeScalar` übernimmt den vorhandenen `ResourceId` und dessen vom Parser gelieferten Static-Hash. Der Pfad löst denselben Ressourcenindex wie `KfcRuntimeResourceRead` auf. Er verlangt dieselbe vollständige Ressourcengröße und einen innerhalb des gesamten ResourceSpan liegenden, ausgerichteten Vierbyte-Scalar. Der ganze ResourceSpan muss zum ursprünglichen Bulk gehören. Die Zielseite muss `MEM_PRIVATE`, `MEM_COMMIT`, genau `PAGE_READONLY` oder `PAGE_READWRITE` sein und dieselbe AllocationBase wie der Bulk besitzen. Unabhängig gestreamte Chunk-Allokationen, Images und ausführbare Seiten werden damit nicht freigegeben.

Die API serialisiert eigene Seitenschutzwechsel über ihren schon vorhandenen ResourceRuntime-Mutex. Für eine zuvor lesegeschützte Datenseite setzt sie vorübergehend `PAGE_READWRITE`, führt genau einen `InterlockedCompareExchange` aus und stellt die ursprüngliche Protektion wieder her. Danach werden Schutz, Allokationsidentität, Owner, Ressourcenindex und Payloadspan geprüft. Eine bereits beschreibbare private Seite behält ihre ursprüngliche Protektion. Der Ablauf ruft keinen ursprünglichen Gamecode auf.

| Nativer Rückgabewert | Bedeutung |
| --- | --- |
| `0` | Abgelehnt, kein bestätigter Scalarwechsel. |
| `1` | Erwartete Bits stimmten überein, der einzelne CAS wurde angewandt, Identität und Schutz sind bestätigt. |
| `2` | Erwartete Bits stimmten nicht überein, der vorhandene Scalar wurde nicht überschrieben. |
| `3` | Ausführung oder Nachprüfung ist ungewiss, kein automatischer Retry und kein Erfolg behaupten. |

Bei einem fehlgeschlagenen Restore versucht RAII noch eine letzte Bereinigung. Der Ausgang bleibt trotzdem `3`, auch wenn diese spätere Bereinigung gelingt. Jeder ungewisse Ausgang sperrt anschließend sämtliche Ressourcen-Schreibzugriffe dieses Runtime-Laufs. `CanWrite` liefert dann falsch und der Status nennt den Grund. Weitere Mods können diese Sperre nicht durch einen neuen Schreibaufruf umgehen. Lesende Zugriffe bleiben verfügbar. Erst explizite ResourceRuntime-Neuinitialisierung oder Shutdown löschen den Fault. Eine isolierte Owner-Profilkonfiguration löscht ihn nicht. Der Vertrag enthält keine Mehrfeldtransaktion. Ein erfolgreicher einzelner CAS ersetzt außerdem keinen anschließend frisch zu lesenden Resource-Snapshot und dessen Feldvergleich.

Die private `kfc-runtime-test-resource`-Fixture hat am 16.10.2026 **324 Checks** bestanden. Sie führt den produktiven Pfad gegen eigens angelegte Speicherobjekte im Testprozess aus. Geprüft sind echte private Readonly-Datenänderung, CAS-Konflikt ohne Überschreiben, bereits beschreibbare Daten, ungültige Größe, Offset, Breite und Ausrichtung, vollständige Bulkgrenzen, falscher Service oder Reader, Readerwechsel, Stop und Init, fehlende oder doppelte Registrierung, Callback und Self-Kontext, belegter Registry-Lock, fehlende Provenienz, ausführbare Seiten, Image-Daten sowie injizierte Prepare- und Restorefehler. Der Restorefehler-Test bestätigt Ergebnis `3`, die abschließende Rückkehr der privaten Testseite zu `PAGE_READONLY`, die dauerhafte globale Schreibsperre mit weiter verfügbaren Reads und den Faultreset bei Shutdown. Der Test öffnet keinen Spielprozess.

## Lesendes Lua-Beispiel ohne Adressen oder fest eingebauten Hash

Die bestehende API kann den tatsächlich benutzten Item-ID-Pfad bereits ausdrücken:

```lua
local item_path = "currentAction.triggerContext.usedItem.itemId"
local entities, reason = runtime.ecs.query(
    "keen::ecs::PlayerInput", "keen::ecs::Actor")
if not entities then return end -- inkrementelle Query im nächsten Update fortsetzen
for _, entity in ipairs(entities) do
    local fields = runtime.ecs.read_fields(entity, "keen::ecs::Actor", {item_path})
    if fields then
        local item_id = fields[item_path]
        if type(item_id) == "table" or type(item_id) == "userdata" then
            item_id = item_id.value
        end
        -- Den geladenen ItemInfo-Katalog einmal inkrementell nach data.itemId
        -- indizieren. Null und mehrdeutige IDs nicht willkürlich auswählen.
        -- Ein Treffer liefert die vorhandene Resource samt GUID und
        -- Resource.data.equipment.gliderConfig. Die Resource ist derzeit readonly.
        shroudforge.log.info("Aktueller Aktionsgegenstand: " .. tostring(item_id))
    end
end
```

Das Beispiel ist ein bestehender lesender, typbasierter API-Vertrag. Es behauptet keinen hier ausgeführten Lua-Flugtest. Die oben beschriebene native Scalarerweiterung verändert die Schreibschutzkennzeichnung geliehener Lua-Resource-Snapshots nicht.

## Reproduzierbare Belege und Prüfung

- `devdata/embervale-toolkit-3.1.0/current-player-glider-consumer-link-20261016.json` enthält Originalregistrierungen, beide System-Accessorlisten, die tatsächlichen ItemRegistry-Servicebindungen, alle 4.207 GUID-/Pointer-Joins und die vier reflektiert erkannten Gleiter.
- `devdata/embervale-toolkit-3.1.0/current-resource-lifetime-live-20261016.json` enthält Workerregistrierung, Originalcallback, Payloadbereich und `VirtualQueryEx`-Ergebnisse der vier Gleiter.
- Die Reproduktion liegt derzeit in `build/capture-glider-consumer-link.py` und `build/capture-resource-lifetime.py`. Beide verwenden ausschließlich `PROCESS_VM_READ`, `PROCESS_QUERY_INFORMATION`, `ReadProcessMemory` und `VirtualQueryEx`.
- Die vorherigen Toolkit-Regler- und Gleichungsbelege bleiben unter `devdata/embervale-toolkit-3.1.0/game-glider-instructions.json`, `glider-service-provenance.md` und `analyse.de.md` erhalten.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ausgangspunkt | Bekannte funktionierende Patchstellen, anschließend originale Register-, Accessor-, Typ- und Berechnungskette. |
| Architektur | Vorhandene Actor-Reflection, Registrierung, ResourceId und Assets-API miteinander verbunden. Keine Struktur pro Mod hinzugefügt. |
| Prüfung | 4.207 vollständige Pointer-Joins, vier echte Gleiter, ausschließlich lesende Lifetime-/Page-Capture und 324 erfolgreiche private native Checks. |
| Offen | Lua-Anbindung und vollständiger neuer Build, aktiver Spieler-Gleiterjoin während des Fluges und kontrollierte Gameplaywirkung des Scalarwriters. |
