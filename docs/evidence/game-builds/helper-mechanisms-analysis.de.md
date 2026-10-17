# Datenbasierte Toolkit-Funktionen

Die vier Einträge verwenden unterschiedliche Mechanismen. Enemy Level findet ein serialisiertes `TemplateResource`-Blob über seinen Namen und eine typisierte Komponentenliste. Terraform Radius findet 39 fest eingebaute Objekt-GUIDs und überschreibt benachbarte Werte. Prop Swapper (Live) sucht und ersetzt GUID-Referenzen im Prozessspeicher. Cinematic Camera ersetzt den Import von `GetRawInputData` und glättet relative Mauseingaben. Keiner der hier nachgewiesenen Pfade liefert einen allgemeinen Entity-Manager oder ein aufgelöstes Archetypenlayout.

## Quellen und Methode

Ausgewertet wurden `complete-registration-catalog.json`, `emulated-registration-table.bin`, `dll-instruction-ranges.json`, `dll-pe-metadata.json`, `strings.json`, die lokale Datei `resource-10-300-1033.dll` und der aktuelle KFC4-Dump `../enshrouded-client-25966634-kfc4/live-discovery/reflection.json`. Der Parsercode für Blobvarianten wurde in `src/parser/kfc-parser/crates/kfc-resource/src/mapped/mod.rs` nachgelesen. Der aktuelle Parserdump `reflection_data.json` besitzt für die relevanten Typen dieselben normalisierten Layoutdaten, siehe den separaten Gleiterbericht.

Die Importtabelle der DLL wurde aus ihren PE-Daten gelesen. Damit sind `GetSystemInfo`, `VirtualQuery`, `GetCurrentProcess`, `GetTickCount`, `VirtualProtect`, `QueryPerformanceCounter` und `QueryPerformanceFrequency` an den genannten indirekten Aufrufstellen durch Original-Importnamen belegt. Unwind-Fragmente wurden über `primaryBeginRva` ihren gemeinsamen Funktionen zugeordnet. Es wurde keine DLL geladen, kein Code im Spiel ausgeführt, kein Prozess gestartet und kein Spielspeicher verändert. Die vorhandene Registrierungsemulation wurde als Artefakt gelesen und nicht erneut ausgeführt.

Alle Toolkit-RVAs gelten ausschließlich für die DLL mit SHA256 `f3521708762136cf9af76642b55c26a604b94ab7f31efd94468d4d2e12e3fc0a`. Die virtuelle Tabellenbasis ist DLL-RVA `0x212370`, der Eintragsabstand `0xa38`.

| Index | Name | `kind20` | Eintrags-RVA | Konstruktorwerte bei `+0x7c0`, `+0x7c4`, `+0x7c8` |
| --- | --- | --- | --- | --- |
| 4 | Prop Swapper (Live) | 2 | `0x214c50` | `0`, `0`, `0` |
| 5 | Enemy Level | 4 | `0x215688` | `1`, `45`, `15` |
| 6 | Terraform Radius | 3 | `0x2160c0` | `2`, `200`, `80` |
| 34 | Cinematic Camera | 5 | `0x227ee0` | `1`, `100`, `55` |

Diese Einträge besitzen keine AOB-Signatur. `kind20` wird im Aktivierungsdispatcher `0x5770` gesondert behandelt. Die Registrierungsemulation allein beweist noch keine ausgeführte Spielwirkung.

## Enemy Level, `kind20 = 4`

### Aktivierung und Konfiguration

Dispatcher `0x5770` behandelt diesen Typ ab `0x5857`. Er liest den gewählten Wert aus dem aktuellen oder ursprünglichen Einstellungsbuffer und stellt Eintragsindex, Aktivierung und Wert in die Globals `0x20f014`, `0x23bbd8` und `0x23bbdc`. Der laufende Toolkit-Worker `0x193c0` entnimmt diesen Auftrag bei `0x19b2e`, prüft erneut `kind20 == 4` und ruft bei `0x19b79` den eigentlichen Worker `0x3e990` auf.

Der Konfigurationshelper `0x13290` speichert zusätzlich einen ausgewählten Namen als Byteanker bei Eintrag `+0x974`, dessen Länge bei `+0x9d4`, einen Offset vom Namen zum Zielwert bei `+0x9d8` und zwei Plausibilitätsgrenzen bei `+0x9dc` und `+0x9e0`. Die Ankerlänge wird auf maximal 96 Bytes begrenzt. Der Konstruktor füllt diesen Namen zunächst nicht aus. Der eigentliche Worker verlangt einen nichtleeren Anker und einen von null verschiedenen Zieloffset. Der Tabellenname „Enemy Level“ bezeichnet somit keinen fertigen universellen Suchanker für alle Gegner.

### Suchbereich und Eingangsparameter

`0x3ee80` erhält den konfigurierten Toolkit-Eintrag und einen Passschalter. Es enumeriert Speicherregionen über `GetSystemInfo` und `VirtualQuery`, akzeptiert committed `MEM_PRIVATE`-Bereiche, verwirft Guard-/No-Access-Schutz und trennt zunächst nicht schreibbare von schreibbaren Kandidaten. `0x3e990` versucht den zweiten Pass erst nach erfolglosem ersten Pass. Große Regionen werden begrenzt, Seiten werden über einen zusätzlichen Residenzhelper gruppiert.

Der Bereichsmatcher `0x3eba0` erhält den Eintrag, Bereichsanfang und Bereichsende. Er sucht byteweise die vollständige konfigurierte Namensfolge. Bei Namensfund `S` berechnet er `L = S + Eintrag[0x9d8]` und prüft diesen Kandidaten mit `0x3f0f0`. Dieser Helper ruft zunächst den Strukturvalidator `0x3ecd0` auf.

### Exakte Beziehung zum KFC4-Templateblob

Der Validator `0x3ecd0` interpretiert keine Entity-Manager-Tabelle. Seine drei Eingänge sind Toolkit-Eintrag, Namensadresse `S` und Kandidatenadresse `L`. Die nachfolgenden Offsetbeziehungen lassen sich vollständig auf die aktuelle Reflexion und den vorhandenen KFC-Blobcodec abbilden.

Definiere `B = S - 0x18` als Anfang eines möglichen `keen::ecs::TemplateResource`-Blobs.

| Gelesene Adresse | Prüfung oder Berechnung | Aktuelle reflektierte Bedeutung |
| --- | --- | --- |
| `S - 0x18 = B + 0` | `uint32 == 0x18` | `TemplateResource.name`, BlobString-Relativoffset zum Namen |
| `S - 0x14 = B + 4` | `uint32 == Ankerlänge` | BlobString-Länge |
| `S - 8 = B + 0x10` | Relativoffset `R`, ungleich null | `TemplateResource.components`, BlobArray-Relativoffset |
| `S - 4 = B + 0x14` | Elementanzahl `N`, `1 <= N <= 1024` | Anzahl der `BlobVariant<keen::ecs::Component>`-Einträge |
| `C = (S - 8) + R` | Anfang der Komponentenliste | BlobArray beginnt relativ zum Offsetfeld |
| `E = C + i * 12` | Eintragsabstand 12 Bytes | BlobVariant-Descriptor |
| `E + 0` | Typ-Hash `0xe550da37` | Exakt `keen::ecs::Level.qualifiedHash` |
| `E + 4` | Bloboffset `D` | Typisierter Payload-Offset |
| `E + 8` | Payloadgröße `0x10` | Exakt `keen::ecs::Level.size == 16` |
| `E + D + 12` | Muss exakt gleich `L` sein | Erster Speichereintrag des eingebetteten Level-Attributs |

`keen::ecs::TemplateResource` ist im aktuellen Dump Typindex 1084, Größe `0x18`. Sein Feld `name` liegt bei null und `components` bei `0x10`. Der Parsercodec dekodiert den Variantenpayload als **`P = E + D + 4`**, siehe `MappedValue::from_blob_variant`. Das Toolkit verlangt **`L = E + D + 12 = P + 8`**. Dieser Unterschied von acht Bytes ist entscheidend.

`keen::ecs::Level` ist Typindex 2633, Größe `0x10`, qualifizierter Hash `0xe550da37`. Es erbt `UInt32Attribute` und `AttributeComponent`. Der reflektierte Header enthält `rootId` bei null, `storageOffset` bei `+4`, `flags` bei `+6` und `storageCount` bei `+7`. `Level.dataStorage` ist ein statisches Array aus zwei `uint32`-Werten bei **`+8`**. Somit zeigt `L` exakt auf **`Level.dataStorage[1]`** und `L+4` auf das zweite Element. Der Toolkit-Worker überschreibt hier weder `rootId` noch einen EntityManager-Pointer. Der semantische Attributname des zweiten Elements wurde in dieser Analyse nicht aus einem KFC-Attributprogramm hergeleitet.

Nach der Strukturprüfung akzeptiert `0x3f0f0` den ersten Wert nur innerhalb der konfigurierten Grenzen `+0x9dc` und `+0x9e0`. Den zweiten Wert akzeptiert es im Bereich `1..100000`. Diese Prüfungen bestätigen das gefundene Attributlayout und Plausibilität. Sie ersetzen keine Entity-ID, Generation oder Live-Archetypenprüfung.

### Änderung und Wiederherstellung

`0x3eba0` speichert die Adresse `L` bei Eintrag `+0x9e8`, den ursprünglichen `uint32` bei `+0x9f0` und ein Gültigkeitsflag bei `+0x9f4`. Der eigentliche Worker `0x3e990` prüft gecachte Treffer erneut über Namen, Komponentenhash, Descriptorgröße und Payloadbeziehung. Veraltete Treffer werden verworfen und erneut gesucht.

Bei aktivem Auftrag ändert `0x3ead9` genau die vier Bytes bei `L`. Ein Eingabewert null wird auf eins gesetzt. Vorher wird `VirtualProtect` aufgerufen und anschließend der alte Seitenschutz wiederhergestellt. Beim Abschalten werden nach erneuter Strukturprüfung die gespeicherten vier Originalbytes restauriert. Ein unabhängiger Schreib-Readback ist in diesem Pfad nicht belegt.

**Ergebnis:** Der nachgewiesene Mechanismus ändert einen Level-Wert innerhalb eines im Prozess gefundenen serialisierten `TemplateResource`-Blobs. Er löst weder lebende Gegnerinstanzen noch EntityManager, Komponentenindizes oder Archetypen auf. Ob die Engine den gefundenen Blob später für Spawns benutzt und wann die Änderung sichtbar wird, bleibt ohne unabhängigen Laufzeitbeleg offen.

## Terraform Radius, `kind20 = 3`

Dispatcher `0x5770` stellt ab `0x57c5` einen Auftrag in `0x20f00c`, `0x23bbd0` und `0x23bbd4`. Der Worker `0x193c0` bearbeitet ihn ab `0x1996e`. Bei Aktivierung begrenzt er den Parameter auf `1..200`, wandelt ihn nach Float um und teilt durch die belegte Konstante **10.0**. Anschließend ruft er `0x40490` mit diesem Float und dem zweiten Argument **5000** auf. Die Konstruktorvoreinstellung 80 entspricht dabei Radiusparameter 8.0.

`0x40490` verwendet eine fest eingebaute Tabelle mit **39 Ankern**, Basispunkt `0x16e4f0`, Abstand `0x38`. Ein Datensatz enthält 16 GUID-Bytes, einen Layoutschalter bei `+0x10` und einen Klartextnamen bei `+0x30`. Insgesamt 22 Einträge sind Floatfälle und 17 Einträge Fälle mit fünf Integerwerten.

| Gruppe | Erfasste Ankernamen |
| --- | --- |
| Advanced_Rake und Rake | Je `Remove` und `Flatten` |
| Bronze_Pickaxe, Celestium_Pickaxe, Copper_Pickaxe, Gilded_Pickaxe, Iron_Pickaxe, Pickaxe, Scrappy_Pickaxe, Steel_Pickaxe | Je `Remove`, `Add`, `Damage1`, `Damage2` |
| Mining_Sieve | `Remove`, `Add`, `Damage1` |

Beispielsweise lautet der erste gespeicherte 16-Byte-Anker für `Advanced_Rake/Remove` roh `2f1f2f94ee339d4b92deee30673234d5`. Der Anker für `Bronze_Pickaxe/Remove` lautet roh `a00eb21b8df69a4c8cb4eb7db3e03e14`. Hier werden bewusst die exakten Bytes angegeben, ohne eine nicht geprüfte GUID-Textformatierung vorauszusetzen.

Der Suchhelper `0x40a10` enumeriert ebenso committed private Speicherregionen in zwei Passgruppen. `0x408e0` sucht die 16-Byte-GUIDs, beschleunigt durch eine Tabelle nach dem ersten GUID-Byte. Es vergleicht beide Qwords vollständig und ruft danach `0x40780` zur Plausibilitätsprüfung auf. Der Ankeranfang wird bei `0x23bd00 + Index*8` gecacht. Es wird hierbei **kein Komponenten- oder Ressourcentyp-Hash validiert**.

Für Layoutschalter null akzeptiert `0x40780` den Float bei Anker `A + 0x10` nur bei `0.05 < Wert < 256`. Er speichert den Originalfloat. Der Worker schreibt den berechneten Float-Radius später genau bei `A + 0x10`. Für Layoutschalter ungleich null prüft und speichert er fünf aufeinanderfolgende 32-Bit-Werte bei `A + 0x10`, die jeweils höchstens 100000 sein dürfen. Der Worker setzt anschließend **alle fünf Werte auf 5000**. Die Funktion „Terraform Radius“ verändert damit im vorhandenen Toolkit zusätzlich diese fünf Integerwerte.

Die reflektierten Layouts `keen::impact_nodes::FloatValueNode` und `keen::impact_nodes::DamageTerraformingSetupValueNode` passen zu diesem Muster. Sie erben `ImpactValueNode.objectId` bei `+0x2c` und besitzen `value` bei `+0x3c`, also genau 16 Bytes nach der GUID. Der zweite Werttyp ist `keen::impact::DamageTerraformingSetup` mit den fünf `sint32`-Feldern `softDamage`, `slightlyHardDamage`, `moderatelyHardDamage`, `hardDamage` und `veryHardDamage`. **Diese Layoutübereinstimmung beweist den konkreten Typ des gefundenen GUID-Ankers noch nicht.** Das Toolkit prüft an dieser Stelle weder den Variantentyp noch den Besitzercontainer. Ein gleich gebautes ID-/Wertobjekt bleibt prinzipiell möglich.

Beim Abschalten werden zunächst die gecachten GUIDs erneut verglichen. Danach werden die jeweils gespeicherten vier oder 20 Originalbytes zurückgeschrieben. Bei nicht mehr gültigen Ankern verwirft der Code die Caches, statt sie blind zu restaurieren.

**Ergebnis:** Ein spezialisierter Such- und Schreibpfad über 39 kompilierte GUIDs und feste Payloadrelationen ist belegt. Eine allgemeine Suche aller Terraforming-Typen, ein EntityManager oder eine generische KFC-Ressourcenauflösung entsteht daraus nicht. Ein Parserpfad könnte diese Knoten über ihre tatsächlichen Ressourcen, Varianten und Feldnamen auflösen, müsste die konkrete GUID-/Besitzerrelation aber zuerst aus den KFC-Daten bestätigen.

## Prop Swapper (Live), `kind20 = 2`

Der Worker `0x193c0` ruft für den passenden Auftrag bei `0x197d6` den Handler `0x5540` auf. Dieser behandelt drei konfigurierbare 16-Byte-Blöcke im Toolkit-Eintrag als Anker (`+0x828`), gesuchte Referenz (`+0x838`) und Ersatzreferenz (`+0x848`). Jeder Block muss ungleich null sein. Such- und Ersatzreferenz dürfen nicht gleich sein. Die Konstruktorfelder `+0x858` und `+0x85c` begrenzen das Suchfenster relativ zum Anker auf **−64 bis +2048 Bytes**. Das Flag bei `+0x860` aktiviert diesen Suchweg.

Der Suchworker `0x159b0` sucht zunächst Vorkommen des Ankerblocks über den 16-Byte-Vergleich `0x3ffe0`. Anschließend sucht er im begrenzten Fenster nach der konfigurierten Suchreferenz. Bei genau einem Treffer wird der Abstand zum Anker gecacht. Bei weiteren Ankern kann dieser Abstand erneut geprüft werden. Bei mehreren Referenztreffern besitzt der Code einen gesonderten Mehrtrefferpfad. Der Writer `0x15900` kopiert den vollständigen Ersatzblock an die gefundene Referenzadresse und speichert bis zu 16 geänderte Adressen. Dafür ändert er vorübergehend Seitenschutz mit `VirtualProtect` und ruft anschließend `FlushInstructionCache` auf.

Der Handler prüft zuerst gecachte Adressen. Ohne brauchbaren Cache beginnt er in nicht schreibbaren Bereichen, während der periodische Worker auch den schreibbaren Heap untersucht. Die gefundenen Referenzen werden in diesem Heap-Pfad regelmäßig auf den Ersatzwert zurückgeführt, wenn sie wieder den Suchwert enthalten. Beim Abschalten restauriert `0x15610` nur Adressen, die noch exakt den eigenen Ersatzblock enthalten. Anderweitig veränderte Stellen werden ausgelassen.

**Ergebnis:** „Live“ bezeichnet hier die Änderung bereits geladener GUID-Referenzen und ihre periodische Nachführung. Der geprüfte Handler besitzt keine Entity-ID, Generationsprüfung, Archetypenabfrage oder typisierte Komponentenauflösung. Die Bedeutung und der konkrete Besitzer der drei GUID-Blöcke werden durch ihre konfigurierte Auswahl und räumliche Nähe getragen. Der hier betrachtete Writer beweist keinen bestimmten KFC-Feldnamen für diese Referenzen.

## Cinematic Camera, `kind20 = 5`

Der Aktivierungszweig in `0x5770` ab `0x5923` liest den gewählten Wert `u`, begrenzt ihn auf maximal 100 und berechnet durch Integerarithmetik **`floor(min(u,100)^2 / 10)`**. Das Ergebnis wird bei DLL-RVA `0x20f024` gespeichert. Mit der Voreinstellung 55 ergibt sich 302. Bei Aktivierung ruft der Handler `0x12e80` auf.

`0x12e80` liest die PE-Importtabelle der laufenden EXE, sucht darin zuerst `USER32.dll` und anschließend **`GetRawInputData`**. Es speichert den Originalfunktionspointer und ersetzt die acht Bytes des passenden IAT-Slots durch den eigenen Wrapper **`0x12cc0`**. Dies erfolgt über vorübergehend geänderten Seitenschutz. Das ist eine Änderung der Importadressentabelle und keine aus KFC gewonnene Kamera-Funktionssignatur.

Der Wrapper ruft zunächst das echte `GetRawInputData` auf. Für `RID_INPUT`, einen hinreichend großen relativen Mouse-Datensatz und aktivierten Zustand bearbeitet er die beiden 32-Bit-Mausdeltas bei RAWINPUT `+0x24` und `+0x28`. Absolute Mausdatensätze werden ausgelassen. Die Zeitbasis stammt aus `QueryPerformanceCounter` und `QueryPerformanceFrequency`. Der erste Zeitschritt verwendet den hinterlegten Wert 0.008 Sekunden. Bei einer Lücke von mehr als 0.25 Sekunden werden die Filterzustände zurückgesetzt.

Pro Achse ruft der Wrapper den Filter **`0x13030`** auf. Dieser hält vier Doublewerte pro Achse, berechnet zeitabhängige Blendfaktoren über den mathematischen Helper `0x56810`, verwendet eine Vorstufe mit Konstante 0.02, zwei nachfolgende Glättungsschritte und einen Bruchrest für die Rückwandlung nach Integerdeltas. Der Stärkewert wird vor dem Filter durch 2000.0 geteilt, bei der Voreinstellung also 0.151. Bei erkanntem Richtungswechsel wird dieser Parameter zusätzlich mit 0.15 multipliziert. Die vollständige mathematische Implementierung von `0x56810` und den Rundungshelpern wurde hier nicht eigenständig bewiesen.

Beim Abschalten wird im geprüften Aktivierungszweig das Filterflag gelöscht. Der bereits installierte IAT-Wrapper bleibt dort erhalten und lässt den ungefilterten Pfad wieder passieren. Der Wrapper besitzt zusätzlich einen abgetrennten Keyboard-Callbackpfad. Dessen vollständige Rolle wurde hier nicht ausgewertet.

**Ergebnis:** Der nachgewiesene Cinematic-Camera-Pfad verändert relative Mauseingaben vor der Verarbeitung im Spiel. In diesem Pfad werden weder Kameraressourcen noch ECS-Kamerakomponenten aufgelöst oder geschrieben. Eine bestimmte Kamera-Entity, FOV-Änderung oder freie Kamera lässt sich daraus nicht ableiten.

## Bedeutung für eine generische Runtime

Der stärkste übertragbare Strukturbeweis ist der Enemy-Level-Validator. Er bestätigt das bestehende KFC-Modell `TemplateResource.components[]` mit typisierten Blobvarianten und dem Reflektionshash von `Level`. Das lässt sich als Parser-/Ressourcenpfad ausbauen. Es beantwortet jedoch keine offene Frage nach dem lebenden EntityManager, Archetypen, Komponentenstorage oder sicheren Engine-Thread-Updates.

Terraform und Prop Swapper liefern konkrete Beispiele für GUID-bezogene Ressourcen- und Feldrelationen. Ihre eingebauten Anker ersetzen keine generische Registry. Bei einer Übernahme sollte zuerst der tatsächliche Ressourcen-/Variantentyp und sein Besitzer aus den aktuellen KFC-Daten ermittelt werden. Die festen Tabellen und Umfeldfenster sollten nicht als allgemeiner Runtimevertrag ausgegeben werden.

Cinematic Camera ist ein Eingabefilter. Sein Mechanismus trägt zur Mausverarbeitung bei und löst keine zusätzliche KFC- oder ECS-Typinformation.

## Grenze des vorhandenen Attribut-VM-Nachweises für KFC4

KFC-Attributprogramme lassen sich über `src/loader/api/examples/inspect_attribute_resources.rs` profilfrei auslesen. Das Beispiel extrahiert die Typregistry frisch aus der EXE und liest die Ressourcen über `KFCFile`, `KFCReader` und `Value::from_bytes`. Es verwendet weder das deaktivierte Runtimeprofil noch einen laufenden Spielprozess. Der aktuelle KFC4-Reflexionsdump enthält weiterhin `AttributeContainerResource.attributes`, `BaseAttributeResource` mit `type`, `ids`, `structure` und `debugNames` sowie `AttributeStructure.calculation`. Die Größen betragen 8, 28 und 16 Bytes. Damit ist das erwartete Datenformat im aktuellen Dump belegt, die aktuellen Programminhalte wurden hier noch nicht neu exportiert.

`tools/enshrouded/verify-attribute-vm.py` unter dem Verzeichnis `src/loader/runtime/profile-tools/dev` ist ebenfalls profilfrei. Der bestehende Offline-Oracle emuliert die drei Originalinterpreter für `sint32`, `uint32` und `float32` in Unicorn mit eigenen Eingabedaten. Er öffnet keinen Spielprozess. Seine SHA256-Tabelle enthält jedoch nur den alten Client `af2f5a1227911d8aa06b3908d6bd0211838211cae14ea91099cb57d0df990781` und den Server `001c1b40ed091d8c1aee583adde3800d7c858ae2c7f4dff54fca2938b2be1637`. Für die aktuelle EXE `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a` existieren dort keine nachgewiesenen Interpreter-RVAs. Ein unveränderter Aufruf scheitert deshalb bereits an der SHA256-Zuordnung.

In Devdata liegen nur die Attribut- und Ressourcenexporte vom 09.10.2026 unter `attribute-origin-20261009` und `origin-pipeline-20261009`. Frische KFC4-Attributressourcen oder KFC4-VM-Vektoren wurden bei der gezielten Dateisuche nicht gefunden. `resources.json` im Toolkit-Verzeichnis ist das PE-Ressourcenverzeichnis der Toolkit-DLL und kein KFC-Attributexport. Die vorhandenen Dateien `target/client-vm-vectors.json` und `target/server-vm-vectors.json` gehören zu den beiden alten SHA256-Werten und enthalten jeweils 65 Programme mit 2.080 Vektoren. Die drei alten Client-RVAs `0x1e1b00`, `0x214090` und `0x1f3040` ließen sich in den bereits vorliegenden KFC4-Cross-Build-Artefakten nicht unmittelbar als exakt zugeordnete Interpreter wiederfinden. Es wurde keine neue Matchingrunde durchgeführt.

### Gezielt untersuchter Interpreterkandidat `0x21c510`

Die separat untersuchten Attributaddierer bei `0x21bda0` und `0x3e7d90` rufen denselben Helfer `0x21c510` mit Ergebniszeiger in RCX, Storage-Span in RDX, Programm-Span in R8 und null in R9 auf. Die direkte statische Disassemblierung der aktuellen EXE bestätigt für diesen Helfer den PE-Funktionsbereich **`0x21c510..0x21d0d8`**, einschließlich darin liegender Dispatchtabellen. Der gespeicherte Funktionsdump `code-discovery/functions.json` nennt `codeSha256 = 9653e94159b37caa4d9472b28f8810c73c2545301feceaf31a1239477aafecd8` und 2.893 dekodierte Bytes. Die hier nachstehenden Befunde stammen aus dem tatsächlichen Code, nicht aus dem Namen des aufrufenden Mods.

| Codebeleg | Nachgewiesene Bedeutung |
| --- | --- |
| `0x21c531 mov r11,[r8+8]`, `0x21c582 mov r12,[r8]` | Programmspan mit Zeiger bei 0 und Wortanzahl bei 8. |
| `0x21c589 mov r9d,[r12+rax*4]`, `0x21c59a shr edx,0x10` | 32-Bit-Programmwörter und Opcode im oberen Halbwort. |
| `0x21c5cf movzx r10d,r9w` | Operand im unteren Halbwort. |
| `0x21c54d mov esi,0x10` | Initialer Stack mit Kapazität 16. |
| `0x21c632 mov eax,[r13+r12*4]` | Laden eines 32-Bit-Storagewerts. |
| `0x21c907 cdq`, `0x21c908 idiv ecx` | Division mit Vorzeichen. |
| Clamp ab `0x21ca33` mit `jge` und `cmovg` | Vergleich der Clampgrenzen und Werte mit Vorzeichen. |
| Min `0x21ccd3 cmovl`, Max `0x21cd28 cmovg` | Min-/Max-Vergleiche mit Vorzeichen. |
| Skalierung ab `0x21cb4d`, Referenzwrite `0x21cc45 mov [r9+r8*4],ecx` | Neuberechnung mit einer beschreibbaren referenzierten Storagestelle. |
| `0x21cddf mov [r14+8],eax`, `0x21cde3 mov byte ptr [r14],0` | Erfolgsstatus bei Ergebnisoffset 0, skalares Ergebnis bei Offset 8. |

Das ist ein starker **signed/sint32-Interpreterkandidat**. Die Annahme eines UInt32-Interpreters wird durch die Division und Vergleichsoperationen nicht getragen. Die endgültige Verknüpfung mit einem bestimmten KFC-Skalarroot bleibt gesondert nachzuweisen. Die Registerbelegung passt grundsätzlich zum vorhandenen Oracle, das Ergebnislayout ist jedoch verschieden. Der alte Python-Oracle liest mit `<B3xI` den Wert bei **Offset 4**. Der aktuelle Helfer schreibt bei `0x21cddb` dort einen zusätzlichen Wert aus ECX und erst bei Offset 8 das skalare Ergebnis. Ein unverändert übernommener Oracle würde damit den falschen Wert vergleichen.

Zusätzlich wurden ausschließlich die bereits gespeicherten Codehashes der drei alten Interpreter in `origin-pipeline-20261009/client/functions.json` mit den vorhandenen Hashfeldern des aktuellen Dumps verglichen. Kein alter `codeSha256` besitzt dort einen exakten Treffer. Alle drei alten `relocatedCodeSha256` sind null und liefern deshalb keine belastbare Zuordnung.

| Alter Root | Skalar laut altem Oracle | Gespeicherter `codeSha256` |
| --- | --- | --- |
| `0x1e1b00` | sint32 | `5bf8e73708f7ec04e83824b7dbee1583d63deb4cb7ff2dbcaa4b9b3c49890ac7` |
| `0x1f3040` | float32 | `7d69082b8472d4a354da5ec67268053662e90551642a20b806cbff4a0b9dc48a` |
| `0x214090` | uint32 | `3e040af6eefa19fa142d1a2b615b15ac7980fc569cf6c5a2b8a59f2eb7261e79` |

Damit sind aus diesem begrenzten Abgleich keine neuen uint32-/float32-RVAs ableitbar. Es wurde kein neuer Matcher gestartet und keine Enginefunktion ausgeführt oder freigegeben.

Der nächste begrenzte Nachweis wäre ein frischer KFC4-Ressourcenexport, gefolgt von der unabhängigen Lokalisierung der weiteren Interpreter sowie einer vollständigen Eingabe- und Ergebnislayoutprüfung. Danach kann ein passend angepasster Oracle neue Vektoren erzeugen und der vorhandene Rust-Differenztest das Rust-Rechenmodell vergleichen. Neue Programmopcodes könnten zusätzliche Modellarbeit erfordern. Das Rust-Modell lehnt derzeit nicht nachgewiesene Opcodes ausdrücklich ab. Die gleich gebliebenen Enum-Namen und Datenlayouts beweisen allein keine unveränderte native Rechensemantik.

Das beantwortet die Nutzerfrage nach konkreten, aus KFC gespeisten Attributrechenwegen weiter. Es beweist keine allgemeine Enginefunktion, kein lebendes Entitylayout und keine vollständige Spielphysik. `runtime.ecs.evaluate_attributes` arbeitet bereits auf eigenen Eingabewerten. Für tatsächliche `update_attribute`-Schreiboperationen verlangt die bestehende API zusätzlich den exakt freigegebenen `attributeCalculationModel` sowie den gültigen ECS-Zugriff. Das native Rechenmodell für KFC4 ist bislang **nicht durch einen neuen Oracle-Lauf validiert**. In dieser Untersuchung wurden weder der Exporter noch Emulator, Tests oder Spielcode ausgeführt.

## Quellenhashes

| Datei | SHA256 |
| --- | --- |
| `resource-10-300-1033.dll` | `f3521708762136cf9af76642b55c26a604b94ab7f31efd94468d4d2e12e3fc0a` |
| `complete-registration-catalog.json` | `295a2b94dc0d1fc3ae3658ad3ce48139966705a5c83facb3ada7d150729130cc` |
| `emulated-registration-table.bin` | `15a9ea0910b3d913e3663328b75c9869fdbb06382fe14a0369faffa15d17a95d` |
| `dll-instruction-ranges.json` | `aeb3988c92d083f6bdab250a2c51459f7db5bb1b392b8c373dbac10c988fe1ff` |

## KI-Codex-Check

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ziel und Umfang | Alle vier angeforderten Kind-Handler untersucht, Enemy Level und Terraform besonders vertieft. |
| Repository und Belege | Tabellenbytes, PE-Imports, zusammengehörige Codefragmente, KFC4-Metadaten und Blobcodec abgeglichen. |
| Architektur und Struktur | Ausschließlich diesen Bericht im ignorierten Devdata-Verzeichnis ergänzt. |
| Konsistenz und Umfang | Ressourcen-/Templateblobs, Referenzersetzung, Eingabefilter und lebende ECS-Entitäten klar unterschieden. |
| Dateicheck | Strukturierte Artefaktauswertung und gezielte Binäranalyse, kein vollständiger DLL-Audit behauptet. |
| Prüfung | Statische Analyse, Quellenhashes und mathematische Offsetbeziehungen geprüft. Keine Tests und keine Spieländerungen ausgeführt. |
| Offen / nicht geprüft | Spieleffekte, tatsächliche Ressourcenbesitzer bei Terraform/Prop Swapper, native Engine-ABIs und lebendes Entity-/Archetypenlayout. |
