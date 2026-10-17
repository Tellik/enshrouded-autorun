# No Fall Damage über die tatsächlichen Spieldaten

## Ergebnis

Der Toolkit-Eingriff bei RVA `0x27402d` führt in die originale Funktion `fall_damage_infliction`. Ihr tatsächlicher Eingang ist `keen::ecs::DynamicFallDamage.detectedFallDamagePercentage`. Dieser Wert entsteht aus den beiden Konfigurationsfeldern `keen::ecs::FallDamage.fallDamageStartDistance` und `fallDamageLethalDistance`. Es gibt einen eindeutigen originalen Ausschaltzweig. Bei endlichen Werten mit `StartDistance >= LethalDistance` setzt das Spiel den Schadensanteil auf null. Damit wird die positive Schadensverarbeitung übersprungen. Dieser Weg füllt keine Lebenspunkte nach.

Die Konfiguration ist auch in den aktuellen KFC4-Dateien konkret nachgewiesen. Ein lesender Export aller 12.267 `keen::ecs::TemplateResource`-Ressourcen fand 511 Vorlagen mit einer `FallDamage`-Variante und keinen Parserfehler. Ein zusätzlicher VM_READ-Abgleich bestätigte die tatsächliche Vorlage der geladenen Spielerentität und ihren direkten Konfigurationszeiger in die geladene Ressource.

| Ressource | GUID | Start | Tödliche Distanz |
| --- | --- | --- | --- |
| `1_Player` | `c3db66d6-89a5-4a6c-a8e3-d9a1f6009812` | 8 | 16 |
| `1_Player_AG2` | `3fcbbfd9-56da-4acf-9592-0a1fc41ecef1` | 8 | 16 |
| `AutomatedPlayer` | `454b15fd-5839-44d4-9dec-bd2e9156d9e8` | 8 | 16 |

Alle drei enthalten außerdem `fallDamageSequence = 2db6b053-4e90-4782-ab37-f7ffe8835063`. Die Ressourcenliste enthält weitere Gegner und Begleiter. Eine Änderung aller 511 Vorlagen wäre deshalb eine andere Wirkung als eine ausdrücklich auf den Spieler begrenzte Auswahl.

## Originaler Wirkungsweg

1. `fall_damage_init` bei `0x2742b0` setzt die anfängliche Fallhöhe aus der Y-Koordinate des `CurrentTransform`. `actor_teleport_fall_damage` bei `0x270660` setzt sie bei der entsprechenden Teleportmarkierung erneut.
2. `fall_damage_detection` bei `0x2737f0` verwendet `Actor`, `CurrentTransform`, `DynamicLocomotion`, `MovementCollider` und `Velocity`. Es verwaltet `wasFalling`, die Start-/Scheitelhöhe und `detectedFallDistance`. Das bereits vorhandene Kollisionsfeld `MovementCollider.results.m_value.preventFallDamage` liegt bei `+0x48`. Die Originalprüfung bei `0x2738e7` setzt den Fallstatus zurück und führt später über den entsprechenden Pfad zur Löschung des Schadensanteils. Dieses Kollisionsresultat ist ein vom Spiel erzeugter Zwischenwert. Ein beliebiger Schreibzugriff zwischen Frames garantiert keine dauerhafte Wirkung.
3. `fall_damage_calculation` bei `0x273650` liest die echte `FallDamage`-Vorlagenkonfiguration. Die Prüfung `0x2736c9` und der Sprung `0x2736cd` unterscheiden den Ausschaltzweig vom normalen Distanzverhältnis. Der Ausschaltzweig schreibt bei `0x27371a` null nach `DynamicFallDamage + 0x0c`.
4. Im normalen Zweig gilt ab der Startdistanz `percentage += clamp((detectedDistance - start) / (lethal - start), 0, 1)`. Der vorhandene Anteil wird bei `0x273755` addiert. Es handelt sich nicht um ein bedingungsloses Überschreiben. Die Endspeicherung erfolgt bei `0x27379f`.
5. `fall_damage_infliction` bei `0x273d60` verarbeitet nur einen positiven Anteil. Die Schranke liegt bei `0x273e44`. Es löst die Attribut-IDs `Health = 0x8eb84995` und `Health_Max = 0xdc8358be` über den originalen Attributresolver auf. Die Berechnung lautet `newHealth = max(0, floorf(float(Health) - float(Health_Max) * percentage))`. Der vollständige Helfer `0x133e300..0x133e37b` rundet nach unten.
6. Das ganzzahlige Ergebnis wird bei `0x274025` erzeugt und bei `0x27402d` in das aufgelöste Health-Feld geschrieben. Anschließend berechnet `0x2315c0`, aufgerufen bei `0x274034`, das originale Attributprogramm erneut. Danach laufen gegebenenfalls die konfigurierte Sequenz und die ursprüngliche Folgebehandlung. Der verarbeitete Anteil erhält bei `0x2740fd` das umgekehrte Vorzeichen.

Die Engine-Ausführungsdeskriptoren bestätigen die Verbindung unabhängig von ähnlichen Typnamen. `fall_damage_calculation` und `fall_damage_infliction` nennen `FallDamage` als Konfiguration. Sie nennen außerdem `DynamicFallDamage` als Schreibabhängigkeit, die Infliction zusätzlich `Health` und `ActorInput`.

## Was das Toolkit tatsächlich macht

Die eindeutige Signatur `45 89 0C 88 48 8B CB` trifft `0x27402d`. Der Toolkit-Code vergleicht den vorgeschlagenen Health-Wert mit dem vorhandenen Wert und unterdrückt eine Verringerung an genau dieser Stelle. Anschließend setzt er die originale Verarbeitung fort.

Der upstream Ausschaltzweig und die Toolkit-Höhle sind deshalb in ihren möglichen Folgeeffekten nicht identisch. Die Höhle verhindert lediglich die Health-Speicherung. Der Ausschaltzweig verhindert bereits den positiven Infliction-Block einschließlich der dortigen Sequenzverarbeitung. Eine Aussage über das tatsächliche Liveverhalten dieser Folgeeffekte erfordert einen Spieltest.

## API-Folge

### Tatsächliche Bindung der geladenen Spielerentität

Der lesende Prozessabgleich in PID 24432 bestätigte `EntityId = 1`, Definition `0x3006b140dd0`, Name `1_Player` und GUID `c3db66d6-89a5-4a6c-a8e3-d9a1f6009812`. Die GUID liegt bei Definitionsoffset null, der Namenszeiger bei `+0x10` und die Namenslänge bei `+0x18`. Entität und Definitionsheader waren beim erneuten Lesen identisch.

Der ResourceService lieferte dieselbe GUID mit Part null, Handle 23726 und Datenbereich `0x20006737a30` mit 48.896 Byte. Die `FallDamage`-Variante war Arrayelement 77. Ihr 24-Byte-Payload lag bei Ressourcenoffset 27.320, also `0x2000673e4e8`, und enthielt die Werte 8 und 16. Host und Client verwendeten in dieser Offlinewelt denselben ResourceService.

Die Definition hält bei `+0x88` einen Zeiger auf 23 Konfigurationsrecords mit jeweils 16 Byte. Record 7 enthält Komponentenindex 210 und den Payloadzeiger `0x2000673e4e8`. Dieser Zeiger ist exakt derselbe geladene Ressourcenbereich. Die Konfigurationsliste und ihre Originalbytes waren beim erneuten Lesen identisch. Die Verbindung zwischen aktiver Spielervorlage und Ressourcenpayload ist damit belegt, es handelt sich nicht um einen Vergleich gleicher Zahlen in getrennten Objekten.

Der originale Systemvorbereiter `0x9a1390` löst bei `0x9a16e0` den Konfigurationsindex aus dem registrierten Systemrecord auf. Bei `0x9a16ff` liest er den Zeigerspaltenoffset aus `layout + 0x584 + index*2`, addiert bei `0x9a1708` die Storagebasis und bei `0x9a170b` die Entityrow mal acht. Die Zeigerspalte landet bei `0x9a170f` im SystemState ab `+0x470`. Der originale Iterator `0x9b5490` dereferenziert bei `0x9b5520` den entsprechenden Rowzeiger und liefert ihn bei `0x9b5529` im Konfigurationsslot. Genau dieser Slot wird von `fall_damage_calculation` und `fall_damage_infliction` als `FallDamage` gelesen. Alle Unwindfragmente der drei zusätzlichen Funktionen sind im Beleg enthalten.

Die direkte VM_READ-Gegenprüfung dieser letzten Storage-Zeigerspalte wurde vorbereitet. Vor ihrer Ausführung war die Sitzung beendet. Nach dem nächsten Weltstart muss sie noch einmal gegen den bereits exakt zugeordneten Ressourcenpayload verglichen werden. Die vorhandene Definitionlistenbindung und die statische Originalkette sind dadurch nicht als Liveaufruf der neuen API zu verstehen.

Die kanonische Datenadresse ist `TemplateResource.components[index].value.fallDamageStartDistance`. Der Komponentenindex muss über den tatsächlichen Variantentyp `keen::ecs::FallDamage` gefunden werden. Ein individueller Mod benötigt dafür weder Code-RVAs noch eine zweite Feldliste.

### Konkreter Fehler im Lua-Aufrufer

Der spätere Abgleich des produktiven Lua-Datenflusses fand die Ursache für die Meldung „Player template has no FallDamage configuration“. Die gemeinsame API liefert bei `BlobVariant.type` den qualifizierten Typnamen als Zeichenkette. Sowohl `MappedVariantValue` als auch `VariantValue` implementieren diesen Vertrag, und `definitions/global.lua` dokumentiert `Variant<T> { type: string, value: T }`. `Resource.type` ist dagegen ein reflektiertes `Type`-Objekt.

Der No-Fall-Mod verwechselte beide Formen und prüfte `component.type.qualified_name`. Dadurch wurde die vorhandene Variante nie ausgewählt. Die Auswahl lautet jetzt `component.type == "keen::ecs::FallDamage"`. Der gemeinsame Mappervertrag bleibt erhalten. Der bestehende Regressionstest baute bisher dieselbe falsche Objektform nach. Seine Variante bezieht ihren Typnamen nun aus einem echten `MappedValue` und der produktiven Lua-Konvertierung. Damit wird der tatsächliche Zeichenkettenvertrag statt einer erfundenen Fixtureform geprüft. Diese Änderung behebt die nachgewiesene Variantenauswahl, ein Live-Falltest bleibt erforderlich.

`FallDamage` hat 24 Byte Konfigurationslayout. Der registrierte Laufzeittyp ist dagegen `DynamicFallDamage` mit 16 Byte. Die Registry ordnet beide unter Komponentenindex 210 ein, aber ihre Speicher dürfen nicht gleichgesetzt werden. Die Konfigurationsfelder dürfen deshalb niemals über den Laufzeitkomponentenoffset beschrieben werden. Das erklärt einen konkreten Fehler früherer Versuche, ausschließlich ähnliche Komponentenbezeichnungen zu verwenden.

Der KFC-Pfad ermöglicht eine echte Änderung vor dem Laden. Für einen Livezugriff müssen zusätzlich zum nachgewiesenen Originalpayload dessen Lebensdauer und ein einzelner atomarer Feldvergleich unter dem passenden Eigentümerschutz gesichert sein. Diese Untersuchung belegt die tatsächliche Datennutzung, führt den Live-Schreibvertrag jedoch nicht selbst aus. Das einmalige Schreiben des flüchtigen `detectedFallDamagePercentage` zwischen beliebigen Frames ist kein gleichwertiger dauerhafter Ersatz.

## Belege und Prüfung

Der vollständige Beleg liegt in `devdata/embervale-toolkit-3.1.0/no-fall-damage-dataflow-20261016.json`. Er enthält die exakte EXE-Identität, sämtliche Unwindfragmente der fünf Engine-Funktionen, den vollständigen manuell abgegrenzten `floorf`-Blattcode, Reflectionfelder, Engine-Deskriptoren, Registry-Zuordnungen und alle 511 konkret aus KFC4 exportierten Vorlagen. EXE-SHA256 ist `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Der Ressourcenbeleg enthält außerdem den Hash der gelesenen `enshrouded.kfc`.

Die Bindungsaufnahme liegt in `devdata/embervale-toolkit-3.1.0/current-player-fall-template-link-20261016.json`. Der produktive Rust-Feldwalker wurde außerdem mit genau diesem vollständigen Originalsnapshot und der unveränderten Parserregistry ausgeführt. Alle vier ausgewählten Start-/Tödlichkeitsfelder aus Host-/Clientaufnahme wurden an den exakten Originaloffsets mit den richtigen Werten aufgelöst. Die übrigen Walkerprüfungen vergleichen 57 verschieden gepolsterte Bloblayouts mit dem kanonischen KFC-Mapper und prüfen Fehlerpfade.

Zusätzlich wurden die unveränderten Originalbytes `0x2736c0..0x2737a4` in privatem Unicorn-Speicher ausgeführt. Alle 55 Fälle bestanden. 48 Fälle bestätigen den Ausschaltzweig für verschiedene endliche Start-/Tödlichkeitsdistanzen, Fallstrecken und positive oder negative Altanteile. Sieben Fälle bestätigen die normale Schwelle, Begrenzung und Addition. Der Beleg `no-fall-damage-original-calculation-oracle-20261016.json` enthält Originalbytes und Hash sowie sämtliche Eingaben und Ausgaben. Dieser Test verwendet keinen Prozesszugriff und keinen nativen Spielaufruf.

Diese Untersuchung verwendet EXE-/KFC-Dateien und lesende Prozessaufnahmen. Sie führt keine Prozessschreibzugriffe oder nativen Spielaufrufe aus. Ein tatsächlicher Falltest, die letzte direkte Storage-Spaltengegenprüfung und die Liveausführung der neu gebauten API bleiben ausstehend.
