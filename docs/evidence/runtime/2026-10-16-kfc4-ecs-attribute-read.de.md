# KFC4-Entityspeicher und Attributauflösung am 16.10.2026

## Belegter Stand

Der laufende Client PID 16724 wurde am 16.10.2026 um 04:18:43 MESZ für die Clientwelt und um 04:27:54 MESZ für die autoritative Hostwelt extern und ausschließlich lesend erfasst. Die EXE hat SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Die Capture verwendet den vorhandenen `discovery.live.Process` mit `PROCESS_VM_READ` und `PROCESS_QUERY_INFORMATION`. Sie lädt keine neue DLL, führt keine Enginefunktion aus, installiert keine Hooks und schreibt keinen Spielspeicher.

Das ist ein Nachweis für tatsächlich vorhandene Entitydaten. Er bestätigt noch nicht die Ausführung des neu gebauten Lua-/DLL-Pfads oder einen Gameplayeffekt durch Schreibzugriffe.

In beiden Welten zusammen sind Speicherzugriffe für 384 verschiedene Originaltypen belegt. In der Hostwelt wurden 1.488 echte Attributroots aus 57 verschiedenen Rootdefinitionen zusätzlich durch den produktiven Entity-Rootjoin und den Lua-Ergebniscodec geprüft. Die Serverkomponenten des beobachteten Spielers enthalten tatsächlich Health 450/450 und Stamina 137/143.

## Besitzer und Typen

Die validierte Singleton-/Client-/Controller-Kette liefert MainOwner. Dessen Pointer bei `+0x1d0` liefert den Ausführungsbesitzer R. Der Komponentenregistrierungsbesitzer liegt inline bei `R+0x928`, der EntityManager inline bei `R+0xe15dc0`. Der Manager-Backpointer stimmt mit dem Registrierungsbesitzer überein. Singleton-, Client- und Controller-Vtable sowie der Controller-Backpointer wurden geprüft.

Alle 640 Originalregistrierungen wurden mit der aktuellen EXE-Reflection verglichen. Die 541 Storage-Typen besitzen jeweils den erwarteten Metadatenpointer, qualifizierten Hash und die erwartete Größe. Originalregistrierungsname und tatsächlicher Storage-Typ bleiben getrennt, beispielsweise `Animal` und `DynamicAnimal`. Die parallelen Typ- und Größenarrays stimmen mit den Registrierungsrecords überein.

Der Manager meldet 131.072 Tabellenplätze. Das ist die Größe der gelesenen Slottabelle und keine Behauptung über 131.072 aktive Entities. Entity-ID, Generation, Layoutpointer, Storagepointer und Row bestimmen den Zugriff. Komponentenindices stammen aus der Originalregistrierung. Die Spaltenadresse folgt dem bereits vorhandenen nativen Vertrag `storage + columnOffset + row * stride`.

## Vollständiger Abgleich des beobachteten Tabellenstands

| Kategorie | Ergebnis |
| --- | ---: |
| Originalregistrierungen geprüft | 640 |
| Registrierte Storage-Typen | 541 |
| Typen mit gesetzten Layoutbits im Capture | 194 |
| Typen mit mindestens einem validierten Storage-Snapshot | 161 |
| Typen mit Layoutbits, aber ohne validierte Spalte | 33 |
| Registrierte Storage-Typen ohne beobachtetes Layoutbit | 347 |
| Wegen geänderter Entity-/Spaltenidentität verworfene Leseversuche | 0 |
| Fehlgeschlagene Speicherleseversuche für die Samples | 0 |
| Bei doppelter Lesung bytegleiche Samples | 161 von 161 |

Die 161 Samples wurden jeweils zweimal gelesen. Dazwischen wurden Entity-ID und Generation, Tabellenplatz, Layout, Storage, Row und die konkrete Spaltenbeschreibung erneut geprüft. Nach dem Gesamtdurchlauf waren Besitzerkette, Entitytabellenpointer, Tabellenplatzanzahl, Registrierungspräfixe sowie parallele Typ- und Größenarrays unverändert. Das ist eine Folge überprüfter Lesevorgänge und kein global atomarer Welt-Snapshot. Insgesamt wurden 2.072.146 Bytes in 6.822 Leseaufrufen gelesen.

Die 33 fehlenden Spalten sind nicht pauschal Tags. Vier Typen haben tatsächlich keine reflektierten Felder und Größe eins: `AffectedByIncreasedPickupSkill`, `DestroyOnLootActivated`, `DisableColliderModeChanges` und `FishingRodRender`. Die übrigen 29 besitzen echte Felder. Dazu gehören `ArmorPoints`, `Mana` und `DynamicAnimal`. Alle 33 haben originale `storage_flags=0`. Ihre `registration_flags` sind null oder eins. Fast alle betroffenen Spalten haben Stride null. Bei `DynamicAnimal` wurde Stride eins statt reflektierter Größe 240 gelesen. Dieser Zugriff wird abgelehnt. Die Bedeutung dieser Layoutbits und fehlenden Spalten bleibt für diese Fälle offen.

## Autoritative Hostwelt und echte Attributroots

Die ursprüngliche aktive Szenenliste ist ein zweiter, eigenständiger Besitzerpfad. Vom geprüften Client führt `Client+0x630` zu SceneOwner. Die Liste bei `SceneOwner+0x4e0` enthält Datenpointer, Anzahl bei `+0x4e8` und Kapazität bei `+0x4f0`. Der Eintragsabstand beträgt `0x90`. Im Capture gab es genau einen aktiven Eintrag mit Status fünf bei Eintrag `+0x18`. Sein C-Pointer bei Eintrag `+0` liefert über `C+0x1d0` den Host-Ausführungsbesitzer R2. Dessen Registry liegt wiederum bei `R2+0x928` und der Host-EntityManager bei `R2+0xe15dc0`. Liste, Anzahl, Kapazität, UUID, Status, Besitzerpointer und Manager-Backpointer wurden vor und nach dem Capture verglichen.

| Hostkategorie | Ergebnis |
| --- | ---: |
| Registrierte Storage-Typen | 541 |
| Typen mit gesetzten Layoutbits | 340 |
| Typen mit validierten Storage-Samples | 319 |
| Typen mit Layoutbits ohne validiertes Sample | 21 |
| Typen ohne beobachtete Layoutbits | 201 |
| Validierte einzelne Samples | 1.750 |
| Davon Health-Roots | 1.360 |
| Davon Stamina-Roots | 73 |
| Unterschiedliche validierte Attributroots | 57 von 59 |
| Racing- oder Speicherlesefehler der Samples | 0 |
| Bytegleiche Doppelreads | 1.750 von 1.750 |

Die Hostslottabelle enthält 1.048.576 Plätze. Auch dies ist keine aktive Entityanzahl. Es wurden 11.676.276 Bytes in 39.235 gezielten Leseaufrufen gelesen. Die zuvor beschriebenen Identitäts- und Spaltenprüfungen wurden für jedes Sample angewendet. Die beiden nicht beobachteten Attributroots heißen `ThreatBonus` und `ManaRechargeMod`. Ihre Definitionen sind im Katalog vorhanden. Aus fehlender Instanziierung wird kein fehlender Parsertyp abgeleitet.

Client und Host zusammen liefern validierte Samples für 384 verschiedene registrierte Storage-Typen. Die übrigen 157 sind damit nicht als erfolgreich gelesen ausgewiesen.

Die Hostentity mit Entity-ID 1, Generation null und Tabellenplatz 334.759 besitzt die originalen Komponenten `PlayerInput`, `PlayerLocomotion`, `PlayerState`, `CharacterSetup` und die autoritativen Attributroots. `PlayerInput.fromClientIsReadyForGameplay` ist wahr. Die ID wird nur als Capturebeleg benutzt und nicht als Suchkonstante in Mods eingebaut.

| Health-Root `0x8eb84995`, Storageoffset 8, Anzahl 8, Flags 1 | Wert |
| --- | ---: |
| `Health` | 450 |
| `Health_Min` | 0 |
| `Health_Max_Base` | 150 |
| `Health_Max_Adder` | 300 |
| `Health_RechargeTimeout` | 500 |
| `Health_RechargeRampup` | 800 |
| `Health_Recharge` | 8 |
| `Health_Max` | 450 |

| Stamina-Root `0x04b6aa8b`, Storageoffset 8, Anzahl 8, Flags 1 | Wert |
| --- | ---: |
| `Stamina` | 137 |
| `Stamina_Min` | 0 |
| `Stamina_Max` | 143 |
| `Stamina_RechargeRampup` | 2.000 |
| `Stamina_RechargeTimeout` | 250 |
| `Stamina_Recharge` | 95 |
| `Stamina_Max_Base` | 110 |
| `Stamina_Max_Adder` | 33 |

Die aktuellen Werte stimmen mit dem unabhängig erreichten Client-`NetworkPlayerAttributes`-Snapshot überein. Das beschreibt zwei verschiedene Originalspeicher und ihre beobachteten Werte. Der lesende Lua-Vertrag für den Host lässt sich ohne feste Entity-ID ausdrücken:

```lua
local reported = false
return {
    update_interval_ms = 250,
    on_update = function()
        if reported then return end
        local players, reason = runtime.ecs.query("keen::ecs::PlayerInput",
            "keen::ecs::Stamina", "keen::ecs::Health")
        if not players or #players == 0 then return end
        local complete = true
        for _, entity in ipairs(players) do
            local stamina = runtime.ecs.read_attributes(entity, "Stamina")
            local health = runtime.ecs.read_attributes(entity, "Health")
            if stamina and health then
                shroudforge.log.info(tostring(entity) .. ": stamina=" ..
                    tostring(stamina.values.Stamina) .. "/" ..
                    tostring(stamina.values.Stamina_Max) .. ", health=" ..
                    tostring(health.values.Health) .. "/" .. tostring(health.values.Health_Max))
            else complete = false end
        end
        reported = complete
    end,
}
```

Diese Abfrage zeigt alle Spieler mit den genannten Komponenten. Sie behauptet bei mehreren Spielern keine lokale Zuordnung. Eine Änderung der Werte oder der VM-Ausführung wurde nicht durchgeführt.

## Clientseitige Health- und Stamina-Snapshots

Die anfänglich erreichte Clienttabelle enthält im Capture keine instanziierten `keen::ecs::Health`- oder `keen::ecs::Stamina`-Rootkomponenten. Beide Typen tragen in der Reflection `server_only`. Die Clientwelt enthält stattdessen unter anderem `NetworkHealth`, `NetworkStamina` und `NetworkPlayerAttributes`. Diese sind eigenständige originale Typen und werden nicht zu Attributroots umgedeutet.

Die einzige in diesem Tabellenstand beobachtete Entity mit `ClientPlayerInput` hat Entity-ID 1, Generation null und Tabellenplatz 72.615. Ihr zugehöriger typgeprüfter `NetworkPlayerAttributes`-Snapshot enthält:

| Reflektiertes Feld | Gelesener Wert |
| --- | ---: |
| `health` | 450 |
| `healthMax` | 450 |
| `stamina` | 137 |
| `staminaMax` | 143 |
| `staminaReg` | 95 |
| `staminaRegDelay` | 250 |
| `mana` | 138 |
| `manaMax` | 138 |
| `movementNormal` | 5 |
| `movementSprint` | 9 |
| `movementSneak` | 2 |
| `movementSwim` | 2 |

Der Snapshot enthält die weiteren ursprünglichen skalaren Feldnamen und Werte. Die Maßeinheiten und Wirkung dieser Werte werden nicht allein aus den Zahlen abgeleitet. Beispielsweise ist das Lesen von `movementSprint=9` kein Nachweis, dass dessen direkte Änderung autoritative Bewegung verändern würde.

Das erste separate `NetworkHealth`-/`NetworkStamina`-Sample gehört Entity 6888 und enthält Health 19/19 sowie Stamina 20/20. Es wird deshalb nicht als Spielerwert ausgegeben.

Mit dem vorhandenen API-Vertrag lässt sich die entsprechende Abfrage nach Laden der neuen Runtime so ausdrücken:

```lua
local reported = false
return {
    update_interval_ms = 250,
    on_update = function()
        if reported then return end
        local entities, reason = runtime.ecs.query(
            "keen::ecs::ClientPlayerInput", "keen::ecs::NetworkPlayerAttributes")
        if not entities or #entities == 0 then return end
        local complete = true
        for _, entity in ipairs(entities) do
            local value = runtime.ecs.read_fields(entity,
                "keen::ecs::NetworkPlayerAttributes",
                {"health", "healthMax", "stamina", "staminaMax", "movementSprint"})
            if value then
                shroudforge.log.info(tostring(entity) .. ": " ..
                    tostring(value.stamina) .. "/" .. tostring(value.staminaMax))
            else complete = false end
        end
        reported = complete
    end,
}
```

Dieses Lua-Beispiel ist ein vorhandener Schnittstellenvertrag. Die externe Capture wird nicht als Ausführung dieses Beispiels ausgegeben.
Die Beispiele setzen den Query in späteren Updates fort. `nil, reason` bei einem noch laufenden Scan ist kein Modladefehler und wird nicht durch `assert` abgebrochen.

## KFC4-Attributkatalog und korrigierter Join

Die aktuelle EXE-Reflection und 17 KFC-Attributressourcen wurden frisch eingelesen. Der Katalog enthält 307 ursprüngliche Attribut-IDs in 59 Roots und 59 reflektierte Speicherlayouts. Alle 307 Programme einschließlich leerer Programme wurden anhand des aktuellen `AttributeOps`-Enums dekodiert. 66 Programme sind nicht leer.

KFC4 hat den Attributheader geändert. `rootId` ist 32 Bit breit, `storageOffset` 16 Bit und `flags` sowie `storageCount` jeweils acht Bit. Das reflektierte `definition`-GUID-Feld aus KFC3 fehlt. Die bisherige API verlangte dieses Feld trotzdem und interpretierte Headerfelder mit den alten Breiten. Damit konnte sie die aktuellen Komponenten nicht korrekt zuordnen.

Die vorhandene API löst die Feldbreiten jetzt aus der aktiven Reflection auf. Ein Entityzugriff vergleicht den tatsächlich initialisierten `rootId` mit der ursprünglichen ID aus dem KFC-Katalog und verlangt genau einen passenden, registrierten Storage. Offset, Elementanzahl und Initialized-Bit werden bei jedem Lesen erneut geprüft. Der Initialized-Maske aus dem aktuellen Enum entspricht der originale Consumer bei RVA `0x21bdf5`, der `test byte ptr [rsi+6],1` ausführt. KFC3 behält seine eigene reflektierte Headerstruktur und seine GUID-Prüfung.

Für Stamina und Health sind die aktuellen Programme und ihre Abhängigkeiten ausgelesen. Stamina hat Current bei Slot 0, Maximum bei Slot 2 und Recharge bei Slot 5. Health hat Current bei Slot 0, Maximum bei Slot 7 und Recharge bei Slot 6. Die Maximumprogramme enthalten jeweils `LoadRef(Current)`, Minimum, vorheriges Maximum, `Base+Adder` und `ScaleToNewMax`. Eine Maximumänderung hat damit eine Beziehung zum aktuellen Wert.

## Originaler KFC4-Interpretervergleich

Die drei aktuellen Interpreter wurden anhand ihres gemeinsamen Dispatchmusters, ihrer originalen PE-Funktionsgrenzen und ihrer skalarspezifischen Instruktionen identifiziert. Signed liegt bei RVA `0x21c510`, Float bei `0x2316c0` und Unsigned bei `0x24d6d0`. Signed verwendet `idiv`, Unsigned `div` und Float die entsprechenden SSE-Skalaroperationen. Alle drei schreiben den Erfolgstag bei Ergebnisoffset 0 und den eigentlichen Scalarwert bei Offset 8. Offset 4 enthält ein separates Hilfsfeld. Der vorhandene Entwickler-Oracle verwendete für ältere KFC3-Builds Offset 4. Er wurde für das exakte aktuelle SHA-256 um das belegte KFC4-Layout erweitert, während die alten Builds ihren bisherigen Offset behalten.

Alle 66 nichtleeren Originalprogramme wurden mit 32 normalen Eingabesätzen ausgeführt. Zusätzlich wurden 2.080 Grenzwertvektoren und 528 Berechnungsschritte aus acht rückwärts durchlaufenen Berechnungsfolgen je Root erfasst. Der Katalog umfasst alle 307 Definitionen in 59 Roots. 241 Definitionen haben ein leeres Programm und behalten ihren gespeicherten Wert, wie der originale Consumer beim Überspringen leerer Programme belegt. Ein leerer Eintrag wird hier nicht als emulierte VM-Ausführung gezählt.

Insgesamt wurden **4.720 aktuelle Originalausführungen** mit dem produktiven Rustmodell verglichen. Ergebnisbits und sämtliche durch `LoadRef` veränderten Speicherbits stimmen überein. Die Emulation läuft ausschließlich in Unicorn auf privaten Scratch- und Stackseiten. Das PE ist dabei schreibgeschützte Eingabe. Es werden keine Spielprozesse geöffnet und keine Windows-Nativfunktionen des Spiels aufgerufen.

Die Grenzwertprüfung hat einen tatsächlichen Fehler im bisherigen Modell gefunden. Bei Float-`Clamp` mit gleichen Grenzen `+0` und `-0` wählte Rusts `min`/`max` das falsche Nullzeichen. Das Original sortiert mit einem strikten `COMISS`-Vergleich und wählt über `MINSS` bei Gleichheit den ursprünglichen Wert. Der Rechencode bildet diese Auswahl jetzt ausdrücklich ab. Ein gezielter Regressionstest und sämtliche aktuellen Differentialvektoren sind danach erfolgreich. Die vorhandenen 2.080 KFC3-Client- und 2.080 KFC3-Servervektoren bleiben ebenfalls erfolgreich.

Die vier `ScaleToNewMax`-Programme für Stamina, Health, Mana und BodyHeat sind mit jeweils 312 Vektoren enthalten. Geprüft wurden unter anderem unverändertes Maximum, vorherige Spannweite null, Float32-Rundung an großen Integerwerten, überlaufende 32-Bit-Zwischensummen und das originale `CVTTSS2SI`-Ergebnis außerhalb des darstellbaren Bereichs. Das ist ein differential geprüfter Eingabekorpus, keine erschöpfende Prüfung aller 32-Bit-Kombinationen. Das Modell lehnt nicht endliche Floatwerte weiterhin ausdrücklich ab.

Der originale Signed-Adder bei `0x21bda0` addiert zunächst auf den ausgewählten gespeicherten Slot und berechnet danach die Rootprogramme in absteigender Reihenfolge neu. Die Float- und Unsigned-Consumer bei `0x232360` und `0x2bb0f0` bestätigen dieselbe Reihenfolge und das Überspringen leerer Programme. Erfolgreiche Interpreterergebnisse werden aus Rückgabeoffset 8 in den jeweiligen Slot geschrieben. Dieser Rechennachweis ersetzt den noch erforderlichen Entity-Lifetime- und Änderungsverfolgungsvertrag für Prozessschreibzugriffe nicht.

## Originale Änderungsverfolgung vor einer Mutation

Die Stamina-Kostenpfade liefern einen zusätzlichen konkreten Befund. Vor dem originalen Adder wird ein Mutable-Accessor verwendet, beispielsweise `0x2bc8b0` vor dem Aufruf bei `0x2c4364`. Der Accessor meldet den Zugriff über `0x9c5830`, bevor er den Komponentenpointer zurückgibt. Der kleine Attributheader enthält nur `Initialized` und `Break_OnWrite`. Die Änderungsregistrierung liegt außerhalb dieses Headers.

Der vorhandene generische Eintritt bei `0x9c5960` nimmt Entitymanager, Trackingkontext, Entitytabelleneintrag, ursprünglichen Komponentenindex und einen Queueindex entgegen. Er leitet die beiden Sequenzmarker aus den originalen Storageheadern und der Entityzeile ab. Aufrufer bei `0x9dbc56` und `0xa16526` verwenden den Kontext bei `Manager + 0xebc030`. Der Managerkonstruktor bestätigt dessen Rückpointer bei `Manager + 0xebc2c0`.

Die Aufzeichner `0x9c5ab0` und `0x9c5b50` verwenden atomare Sequenzvergleiche und zwei Gruppen mit jeweils acht Entity-ID-Queues. Die Queueverwaltung besitzt eigene Kapazitäten und einen Überlaufpfad. Ein bloßes Schreiben von Current oder Maximum bildet diese Meldung nicht nach. Die vollständige Übernahme dieses generischen Vertrags und seine Synchronisierung werden gesondert geprüft. Aus diesem Befund folgt keine Freigabe für Raw-Writes oder das direkte Schreiben eines Netzwerksnapshots.

## Dateien und Prüfung

Die ignorierten Originalartefakte liegen unter `profile-tools/devdata/creative-mode-146/`:

- `capture-current-attributes.py` ist das ausschließlich lesende externe Capturewerkzeug.
- `current-attributes-live-20261016.json` enthält alle registrierten Typidentitäten, Statuslisten, 161 rohe Samples und dekodierte Inline-Skalare.
- `current-attributes-host-live-20261016.json` enthält den eigenständig hergeleiteten Hostpfad und 1.750 validierte Samples für 319 verschiedene Storage-Typen.
- `current-attributes-column-classification-20261016.json` enthält die 33 nicht validierbaren Spalten mit ursprünglichen Flags, Feldlisten und konkreten Strides.
- `current-attributes-live-20261016.json.lua-codec.json` bestätigt die vollständige Dekodierung aller 161 Client-Snapshots durch den produktiven Lua-Wertcodec. Zusätzlich stimmen 297 gespeicherte Skalare über 726 mit dem produktiven Feldcodec dekodierte Felder überein.
- `current-attributes-host-live-20261016.json.lua-codec.json` bestätigt die vollständige Dekodierung aller 1.750 Host-Snapshots durch `runtime.values.decode`. 8.792 Felder wurden durch denselben Feldcodec wie `runtime.ecs.read_fields` dekodiert. Alle 527 separat gespeicherten Inline-Skalare stimmen im deklarierten Speichertyp überein.
- `current-attributes-host-live-20261016.json.attribute-codec.json` bestätigt den produktiven Entity-Rootjoin und das Lua-Ergebnisformat für 1.488 tatsächliche Rootsnapshots, 57 Roottypen und 11.751 Scalarwerte.
- `current-attribute-vm-20261016.json` enthält 2.112 aktuelle Original-VM-Vektoren.
- `current-attribute-vm-boundaries-20261016.json` enthält 2.608 zusätzliche Grenzwert- und Rootfolgevektoren.
- `current-attribute-vm-summary-20261016.json` enthält die Coverage und die geprüften `ScaleToNewMax`-Fälle.
- `current-attribute-vm-codeproof-20261016.json`, `current-tracking-entry-callers-20261016.json` und `current-tracking-owner-references-20261016.json` enthalten originale Instruktions- und Aufruferbelege.

`target/kfc4-attribute-resources.json` enthält die frisch geparsten Definitionen. `target/enshrouded-attribute-resolution.json` enthält den vollständigen aufbereiteten Katalog mit Namen, IDs, Roots, Programmen und Speicherlayouts.

Die Codecprüfungen erweitern vorhandene ignorierte Integrationseinstiege. Sie verwenden `SHROUDFORGE_TEST_REGISTRY`, `SHROUDFORGE_TEST_EXE` und optional `SHROUDFORGE_TEST_ECS_SNAPSHOTS`. Float32-Vergleiche erfolgen anhand ihrer exakten Float32-Bits. Dadurch wird eine mögliche Rundung der JSON-Dezimaldarstellung beim Einlesen als Float64 nicht mit einer Abweichung der ursprünglichen Float32-Speicherung verwechselt. Es wird keine Epsilon-Toleranz benutzt.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ziel und Umfang | Aktive Entitytabelle systematisch über alle registrierten Storage-Typen geprüft. Header- und Rootauflösung in der vorhandenen API korrigiert. |
| Belege | Frische aktuelle EXE-Reflection, Originalregistrierungen, Parserressourcen und lesende Prozessdaten verwendet. |
| Architektur | Bestehender Attributkatalog und Lua-Einstiegspunkte erweitert. Keine pro Mod eingebauten Adressen oder neue Gameplaypatches. |
| Prüfung | API-Unitlauf mit 29 erfolgreichen und fünf regulär ignorierten Tests. Zusätzlich aktuelle KFC4-Katalogprüfung, vollständiger Lua-Wertcodec für Client und Host sowie produktiver Entity-Rootjoin gegen 1.488 Hostsnapshots explizit erfolgreich ausgeführt. Alle 4.720 aktuellen und 4.160 bisherigen KFC3-VM-Vektoren stimmen bitgenau mit dem Modell überein. Externe Livecaptures erfolgreich. |
| Offen | Neue DLL-/Lua-Ausführung, fehlende Spalten und nicht beobachtete Typen, synchronisierte Änderungsverfolgung und autoritative Schreibwirkung. |

## Tatsächlicher Lua-Test mit installierter DLL

Der neue Build `1.6.8-kfc4-typed-runtime-20261016` wurde anschließend transaktional installiert. Alle 59 verwalteten Dateien wurden gegen das entpackte Paket gehasht. Die native DLL hat SHA-256 `d3040698c732cb460081776aabd2a13390adcf6f49702127300f2735451cf9a9`, die Rust-Runtime `523ccf4c34aab24a39c03be8955707027971afb32a16d4ec9da840a9a0b50e25`. Das Paket hat SHA-256 `2d27ed56b883e7db7190886c241cdb979555676ed7580a432239800d30698cd2`.

Die temporäre, rein lesende Entwicklerprobe `diagnostic-mods/live-runtime-probe` verwendet die vorhandene Lua-API. Beim damaligen Test lag sie noch unter `build/live-runtime-probe`. Sie wurde zusätzlich unter `mods/sf-live-runtime-probe` in der Spielinstallation installiert. Ihr `on_load` scheitert nicht an einer noch ungeladenen Welt. In PID 28384 und nach einem vom Nutzer bestätigten eigenen Neustart erneut in PID 24432 bestätigte die tatsächliche Lua-Ausführung:

| Live-Marker | Ergebnis |
| --- | --- |
| `REGISTRY_OK` | 640 Originalregistrierungen, 541 Speichertypen |
| `ATTRIBUTE_CATALOG_OK` | 307 Attribute, 59 reflektierte Speicherlayouts |
| `FUNCTION_CATALOG_OK` | 46.137 inventarisierte Codeeinträge, 722 Systeme, 2.904 zugeordnete Referenzen und 586 beteiligte Typen. Lese-/Schreibrollen aufgelöst, native Signaturen weiterhin nicht vollständig belegt |
| `ASSET_LIST_OK` | 4.262 geladene ItemInfo-Ressourcen |
| `ASSET_DATA_OK` | Tatsächliches `.data` einschließlich verschachtelter Gleiter- und Platzierungsfelder, Quelle `loaded-runtime` |
| `PLAYER_ROOT_OK` | `keen::ecs::Health` 450/450 mit Base 150 und Adder 300. `keen::ecs::Stamina` 113/113 mit Base 110 und Adder 3 |
| `PLAYER_FIELDS_OK` | `NetworkPlayerAttributes` derselben Entity bestätigt 450/450 und 113/113, Sprintwert 9 |
| `CORE_READ_OK` | Gemeinsamer typbasierter Zugriff auf echte Ressourcen und autoritative ECS-Daten über Lua erfolgreich |

Die geänderte Stamina von 113/113 gegenüber den früheren 137/143 stammt aus einem späteren Livezustand. Der Test vergleicht die jeweiligen Originalspeicher derselben Sitzung und setzt die frühere Capture nicht als Sollwert voraus. Die Runtime meldet `entityReadRole="local-authority"`, `entityAccessReady=true` und `writable=false`. Es wurden keine Spielerwerte verändert.

Die vollständige Inventur wurde in PID 24432 abgeschlossen. Die tatsächliche Lua-Probe prüfte alle 541 registrierten Speichertypen und alle 4.262 geladenen `ItemInfo`-Ressourcen. 320 Speichertypen wurden gelesen, 218 nicht beobachtet und drei zunächst als vorhanden, aber nicht lesbar gemeldet. Keine Query scheiterte, kein getesteter Codec blieb teilweise dekodiert und keine Ressource schlug beim Lesen fehl. Der Test liest pro beobachtetem Typ einen passenden Snapshot und bis zu drei Inlinefelder. Er beweist keine vollständige Traversierung sämtlicher verschachtelter externer Datenstrukturen.

### Freigegebene Hashslots als Ursache der drei scheinbar unlesbaren Typen

Die anschließende Untersuchung des originalen ID-Lookups bei RVA `0x9a7ab0` und seines Komponentenverbrauchers bei `0x9a5900` hat einen Fehler im bisherigen Tabellenlauf aufgedeckt. `Manager+0x158` enthält die Hashkapazität und `Manager+0x188` die Pointertabelle. Entfernte Einträge können einen nichtleeren Pointer und plausible alte Entityfelder behalten. Das Original prüft zusätzlich die Belegungsbits über `Manager+0x150`, den Schlüssel über `Manager+0x170` und löst Hashkollisionen durch lineares Weitersuchen auf.

Eine ausschließlich lesende Vollaufnahme derselben Sitzung enthielt 1.048.576 Slots, davon 24.168 belegte Slots und 24.983 nichtleere Pointer. Bei allen 24.168 belegten Slots stimmte der Schlüssel mit der Entity-ID überein. 814 unbelegte Slots enthielten weiterhin plausible alte Entityfelder. Die dabei gesehenen scheinbaren Spalten für `ActiveNpcStateSaved` und `AddDestructibleOnLoot` gehörten solchen unbelegten Slots. Der Befund erklärt die vorherigen Fehler als falsche Entity-Mitgliedschaft. Aus Schrittweite null wird keine neue Speicherklasse abgeleitet.

Der gemeinsame Resolver validiert jetzt dieselbe ursprüngliche ID-Hashtabelle, einschließlich Belegungsbits, Schlüssel, aktuellem Entitypointer und Kollisionsfolge. Er bestätigt Schlüssel, Entityidentität und Tabellenheader anschließend erneut. Zusätzlich entfernt bereits die Pointeraufnahme unbelegte Quellbuckets. Das ist erforderlich, weil eine weitere Liveaufnahme 287 unbelegte Buckets mit einem inzwischen an anderer Stelle aktiven Pointer nachwies. Ohne diesen Filter wären doppelte Queryergebnisse möglich. Mehrere Treffer derselben Identität werden auch im inkrementellen Query zusammengeführt.

Diese Prüfung gilt für Query, Komponentenlesen, Handleauflösung, Entity-ID-Snapshots und den weiterhin deaktivierten Schreibpfad. Die strukturellen Offsets stehen im vorhandenen KFC4-Profil. Ältere Profile ohne diesen optionalen Nachweis behalten ihren bisherigen Dispatcher-Vertrag. Private Regressionen prüfen aktive Einträge, unbelegte alte Pointer, falsche Schlüssel, ersetzte Pointer, Kollisionen, ungültige Kapazitäten und die Filterung unbelegter Quellbuckets. Zusätzlich stimmen 553 private Ausführungen des originalen Hashlookups und Komponentenhelpers mit dem nachgebauten Vertrag überein.

Die Belege liegen in `build/ecs-table-membership-20261016.json`, `build/native-procedure-slices-20261016.json` und `build/kfc4-typed-live-read-20261016.json`. Der Tabellenfix war zum Zeitpunkt dieser Aufnahme noch nicht in der laufenden DLL geladen. Eine weitere tatsächliche Lua-Inventur mit dem Folge-Build ist erforderlich. Das erfolgreiche Lesen ist keine Freigabe für Schreibzugriffe. Deren ursprüngliche Ausführungsphase und bestätigte Spielwirkung bleiben gesondert zu beweisen.

## Installierter Folgebuild R3

Nach dem vom Nutzer bestätigten regulären Spielende wurde `1.6.8-kfc4-typed-runtime-20261016-r3` installiert. Alle 59 verwalteten Dateien stimmen mit der geprüften Stage überein. Ein vorheriger R2-Installationsversuch wurde nach einer Windows-Dateisperre abgebrochen. Der vollständige Vergleich bestätigte anschließend für alle 59 Pfade den unveränderten Backupstand, bevor R3 erfolgreich installiert wurde.

Der neue Prozess PID 10696 lädt tatsächlich die R3-DLLs. Die native DLL hat SHA-256 `f101ccb54e611e2cc560e13e7dc773f35142db43e18a77b37c9f0dc2a282ba15`, die Rust-DLL `88a531499d4a9617baa8ea0c05a409032d2f4dcc2478008ce9d3acf1d3804808`. Der Beleg liegt in `build/kfc4-runtime-r3-loaded-modules-20261016.json`. Das ZIP hat SHA-256 `ef92b1ff53a94d58de6bd815943b84104145a9dfb4497291ade49b2509cfc473`.

R3 enthält den korrigierten Hashmitgliedschaftsresolver, den begrenzten ursprünglichen Scene-Schreibguard für Komponenten ohne beide Trackingbits, `write_fields` mit optionalen erwarteten Feldwerten sowie die ursprüngliche Entity-Template-UUID-Bindung. `runtime.ecs.get_template(entity)` löst diese GUID über dieselbe aktive Parserreflection und denselben geladenen ResourceService auf. Es wird keine feste Spielervorlage im Mod benötigt.

Der zusätzliche `game.assets.write_loaded_field(resource, path, value, expected)` verwendet den kanonischen Parser für Typ, Feld und ResourceId. Er erlaubt genau einen ausgerichteten 32-Bit-Skalar in der belegten privaten Ressourcen-Bulkallokation. Der Original-Threadregistry-Guard sichert die Lebensdauer von Contentworker, Service und Reader. Der native CAS stellt den ursprünglichen Datenseitenschutz wieder her. Unsicherheit sperrt alle weiteren Ressourcenwrites bis zur Ressourcen-Neuinitialisierung. Ausführbare Seiten und Containerdeskriptoren werden nicht beschrieben. Rückgabeobjekte lesen ihren neuen Snapshot erst beim ersten `data`-Zugriff. Die Proben verlangen deshalb zusätzlich einen unabhängigen Getter.

Auto Stamina Refill verwendet benannte ECS-Attribute. No Fall Damage verwendet die tatsächliche Spielervorlage und den im Originalcode belegten `FallDamage`-Ausschaltzweig. Die [Transfermatrix](2026-10-16-mod-resource-ecs-migration.de.md) nennt die vier verbleibenden Modifierpfade und ihre konkreten offenen Bindungen. Sie werden nicht als migriert gezählt.

Die private Prüfung bestätigt 324 Ressourcenchecks, 99 ECS-Checks, 35 Propquerychecks, 53 Weltkontextchecks und 30 Buildinginputchecks. Im Workspace bestehen 65 Tests bei neun regulär ignorierten Einstiegen. Die drei kontrollierten Lua-Proben bestehen 14 Attribut-, 24 Flugzustands- und 27 Ressourcenszenarien. Der produktive Byteenumcodec wurde mit benannten Werten, ausgewählten Erwartungswerten und unveränderten Nachbarbytes geprüft. Alle drei Profile erfüllen das JSON-Schema. Die tatsächliche originale Fallschadenberechnung besteht zusätzlich 55 private Unicorn-Fälle.

Die drei Probe-Mods sind installiert und im neuen Prozess geladen. Sie warten aktuell auf eine einzelne spielbereite Offlinewelt. Bis dahin haben sie keine Werte geschrieben. Noch ausstehend sind ihre tatsächlichen Apply-/Restore-Ergebnisse, der originale Gravity-Consumer nach dem Flugzustandswechsel sowie ein wirklicher Fall- und Gleiterwirkungstest. Ein erfolgreicher Build und private Prüfungen werden hier nicht als fertiger Gameplaynachweis ausgegeben.

## R3-Liveergebnisse und R4-Abfragekorrektur

In der laufenden Offlinewelt mit Prozess PID 13668 wurden drei begrenzte Proben tatsächlich ausgeführt. Der ECS-Schreibtest änderte `Stamina_Max_Adder` von 33 auf 34. Das originale Attributprogramm berechnete daraufhin Maximum und aktuellen Wert von 143 auf 144. `NetworkPlayerAttributes.staminaMax` meldete ebenfalls 144. Beim Restore kehrten die drei Werte nachweislich auf 33 beziehungsweise 143 zurück. Das bestätigt den typisierten ECS-Schreibpfad für diesen konkreten Rootwert einschließlich der originalen Berechnung und Replikation.

Der ResourceService-Schreibtest änderte den geladenen Gleiterwert `equipment.gliderConfig.yawAngleSpeed` von 65 auf 66 und stellte ihn auf 65 zurück. Ein unabhängiger Getter bestätigte beide Zustände. Damit ist der beschränkte Ressourcenschreiber live belegt. Eine wahrnehmbare Änderung der Gleiterdrehung wurde nicht geprüft.

Der Flugstest schrieb `DynamicLocomotion.state` von `Walking` auf `BuildingFlying`. Während des unabhängigen Beobachtungsfensters blieb `Gravity.isActive` wahr. Der Zustand wurde auf `Walking` zurückgesetzt. Diese Probe bestätigt gerade keine Flugwirkung.

Die R3-Abfrage läuft weiterhin über die volle Entitytabellenkapazität von 1.048.576 Einträgen. Ein Laufzeitdiagnosesnapshot hielt sie bei Cursor 393.216 und 3.005 Treffern fest. Über 72.702 Abfragen waren 72.077 noch unvollständig und 625 erfolgreich. Deshalb warten Auto Stamina Refill, No Fall Damage und die vollständige Typinventur trotz bereits geladener Welt. Das ist der konkrete Laufzeitfehler, den R4 beheben soll.

R4 filtert unbelegte Zeilen aus der internen Abfrageliste und bewahrt für jeden Treffer den echten ursprünglichen Tabellenindex, den die generationsgeprüfte Handleauflösung benötigt. Der private ECS-Regressionstest besteht mit 107 Prüfungen, darunter Fälle mit leeren Slots und erhaltenen Originalindizes. `build/shroudforge-1.6.8-kfc4-typed-runtime-20261016-r4.zip` wurde erfolgreich gebaut und hat SHA-256 `36d16c22af77b9cc37797e41dfbc13e91c41eeebe0fac830d7524ddf7fbe3bdd`. Nach dem regulären Spielende wurde R4 mit dem unabhängigen Updater installiert. Alle 59 verwalteten Dateien stimmen mit dem vorbereiteten Stage überein. Die installierte native DLL hat SHA-256 `1ab2355e40120911d40ea5dc627db44eb42d26e9ad3613fb98c74ad3e5344b84`, die Runtime-DLL `d970ab5700b49a6f657ecb93328f23b9e26f42da0738e0a910e43bf4eeaa6f7e`.

### R4-Liveergebnisse

Der neue Prozess PID 5688 lädt die installierten R4-DLLs mit den erwarteten Prüfsummen. Die Runtime ist in der Offlinewelt bereit, meldet 541 registrierte ECS-Speichertypen und stellt den KFC4-ResourceService mit 91.812 Einträgen bereit. Der Weltkontext-Dispatcher und das alte Profil für Engineaufrufe bleiben nicht verfügbar. Das hindert die hier getesteten ECS- und ResourceService-Wege nicht daran, ausgeführt zu werden.

Die tatsächliche Lua-Inventur wurde mit R4 beendet. Sie prüfte 541 von 541 registrierten Speichertypen und las 4.262 von 4.262 ItemInfo-Ressourcen. 321 Typen wurden in der aktuellen Welt beobachtet, 220 waren dort nicht vorhanden. Es gab null unlesbare Treffer, teilweise Dekodierungen oder fehlgeschlagene Abfragen. Das ist eine vollständige Inventur der registrierten Typen in dieser Welt, keine Behauptung, alle Enginefunktionen oder Signaturen aufgelöst zu haben. Der Funktionskatalog nennt 46.137 Codeeinträge, 722 Systeme, 2.904 zugeordnete Referenzen und 586 Typen. Native Funktionssignaturen sind weiterhin nicht vollständig belegt.

Der typisierte ECS-Schreibtest wurde mit R4 live wiederholt. `Stamina_Max_Adder` änderte sich von 33 auf 34. Das originale Attributprogramm setzte Maximum, aktuellen Wert und `NetworkPlayerAttributes.staminaMax` von 143 auf 144. Der kontrollierte Restore stellte 33 und 143 wieder her. Der ResourceService-Test änderte `ItemInfo.equipment.gliderConfig.yawAngleSpeed` von 65 auf 66 und stellte 65 wieder her. Ein unabhängiger Getter las beide Änderungen. Eine tatsächliche Gleiterdrehwirkung wurde damit noch nicht geprüft.

Der Nutzer bestätigte beim Spielen zunächst, dass beim Laufen etwas Ausdauer abgezogen wird und kurz danach wieder hinzukommt. Das entspricht dem Code von `sf-auto-stamina-refill`, der alle 250 ms den Spieler über `PlayerInput` und `Stamina` sucht und den reflektierten Attributroot auf Maximum setzt. Die spätere Prüfung zeigte `enabled: false` am Modmanifest und keinen Eintrag des Mods in der aktiven Runtime-Liste. `settings.autoRefill: true` blieb gespeichert, war bei deaktiviertem Mod aber wirkungslos. Bei ganz ausgeschaltetem Mod sank die Spielausdauer beim Laufen nach Nutzerbeobachtung. Zusammen mit dem vorherigen Wiederauffüllen bei aktivem Mod ist das ein bestätigter Live-A/B-Nachweis des periodischen Refillverhaltens beim Laufen. Ob Waffenangriffe selbst Ausdauer verbrauchen, bleibt offen. Der Nutzer konnte das aktuell nicht isoliert belegen. Die frühere Beobachtung mit aktivem Refill kann einen kurzen Verbrauch verdeckt haben. Die Runtime kann beim aktiven Mod zwischendurch „Live ECS query is still scanning“ melden. Der Refill hängt daher an einer erfolgreich abgeschlossenen Spielerabfrage und erfolgt nicht synchron im ursprünglichen Angriffspfad.

Nach dem Test wurde der Mod wieder aktiviert. Anschließend schaltete der Nutzer in dessen Einstellungen `autoRefill` aus und speicherte. Das aktive Manifest zeigt `enabled: true` und `settings.autoRefill: false`. Der weiterhin laufende Prozess PID 5688 listet den Mod als aktiv. Das Clientlog meldet `Runtime settings applied live`. Sein Runtime-Effektstatus lautet `no-change: Automatic stamina refill is disabled`, und es gibt keinen Ladefehler. Bei aktivem Mod und ausgeschaltetem `autoRefill` bestätigte der Nutzer, dass die Ausdauer beim Sprinten sinkt. Zusammen mit dem zuvor beobachteten Wiederauffüllen bei aktivem Refill bestätigt das den Live-A/B-Test sowohl der Refillerwirkung als auch der internen Liveeinstellung.

Der Flugzustandstest blieb ohne nachgewiesene Flugwirkung. Der Mod schrieb `DynamicLocomotion.state`, während die unabhängig gelesene `Gravity.isActive` wahr blieb. Der Zustand wurde zurückgesetzt. `sf-no-fall-damage` ist geladen, hat im letzten Status jedoch weiterhin eine laufende ECS-Abfrage gemeldet. In früheren R4-Updates erreichte er eine Spielervorlage, fand darin aber keine `FallDamage`-Konfiguration und schrieb nichts. Das widerspricht der separat gelesenen Vorlagenressource und bleibt ein offener Auflösungsfehler. Die vier Mods `sf-infinite-item-split`, `sf-infinite-item-use`, `sf-no-resource-cost` und `sf-unlimited-flight` melden weiterhin beim Laden das fehlende `runtime.gameplay.patch`.

Damit sind unter R4 der vollständige ECS-Typinventurlauf, echte ECS-Attributwrites samt ursprünglicher Berechnung und Rückmeldung sowie der sichtbare periodische Auto-Stamina-Effekt belegt. Nicht belegt sind vollständige Funktionssignaturen, die Flugwirkung, die Fallschadenwirkung und die vier Legacy-Modifierpfade. Die nächsten Liveprüfungen isolieren den Waffenausdauerverbrauch mit ausgeschaltetem `autoRefill` und verfolgen separat, warum die Laufzeit-Spielervorlage ihre reflektierte `FallDamage`-Variante nicht zurückliefert.
