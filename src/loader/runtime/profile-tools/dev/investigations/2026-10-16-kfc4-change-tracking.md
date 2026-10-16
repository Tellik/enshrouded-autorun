# KFC4: Originaler generischer Vertrag zur Änderungsverfolgung

Stand: 16. Oktober 2026. Untersucht wurde ausschließlich die aktuelle Client-EXE mit SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Die folgenden RVAs sind Fakten dieses Images. Es wurden für diese Untersuchung keine Spielfunktionen im laufenden Prozess aufgerufen.

## Warum der Attributheader allein nicht ausreicht

Der originale Stamina-Kostenpfad bei `0x2c4350` ruft den Mutable-Accessor `0x2bc8b0` auf und danach bei `0x2c4364` den Signed-Adder `0x21bda0`. Der Accessor meldet den Zugriff über `0x9c5830`, bevor er den Komponentenpointer zurückgibt. Der Pfad bei `0x21fbd2` meldet ebenfalls vor dem Stamina-Adder bei `0x21fbea`.

Die aktuelle Reflection kennt im `keen::ecs::AttributeFlags` nur `Initialized` mit Bitindex 0 und `Break_OnWrite` mit Bitindex 1. Dirty-/Queuezustand liegt außerhalb des kompakten Attributheaders. Ein direktes Schreiben von Current oder Maximum ersetzt diesen Vertrag deshalb nicht.

## Vorhandener generischer Eintritt

Die Engine besitzt bei RVA `0x9c5960` bereits einen Eintritt, der die Meldung aus einem Entitytabelleneintrag und dem ursprünglichen Komponentenindex ableitet:

```cpp
void __fastcall track_component_access(
    std::uintptr_t entity_manager,       // RCX
    std::uintptr_t tracking_context,     // RDX
    std::uintptr_t entity_table_entry,   // R8
    std::uint16_t component_index,       // R9w
    std::uint8_t queue_index);           // fünftes Win64-Argument
```

Der fünfte Parameter wird nach dem Prolog als Byte bei `[rsp+0x60]` gelesen. Der Rückgabewert ist kein Erfolgsstatus. Die Engineaufrufer prüfen keinen solchen Status.

Der ursprüngliche Komponentenindex kommt aus der Engine-Registrierung. Der Reader hat für dieses Image 640 Einträge verifiziert. Es ist keine Konstante pro Mod und kein aus einem Typnamen neu berechneter Index. Für einen Zugriff muss der aktuell registrierte Eintrag ausgewählt werden.

Der Originalaufrufer `0x9dbb10` verwendet bei `0x9dbc56` ausdrücklich `tracking_context = manager + 0xebc030` und setzt den Queueindex auf 0. Der Aufrufer bei `0xa16526` verwendet denselben Kontext. Der Managerkonstruktor bestätigt den Rückpointer bei `0x9846c0`:

```text
[Manager + 0xebc2c0] = Manager
Context = Manager + 0xebc030
[Context + 0x290] = Manager
```

Der Kontext ist kein nachträglich durch Subtraktion vermuteter Besitzer. Konstruktor und Aufrufer belegen beide Richtungen.

Die ersten 33 Bytes des generischen Eintritts enden an einer vollständigen Instruktionsgrenze:

```text
48 89 5c 24 08 48 89 6c 24 10 48 89 74 24 18 48 89 7c 24 20 41 56 48 83 ec 30 4d 8b 50 20 49 8b f0
```

## Speicher- und Metadatenbeziehung

`0x9c5960` verwendet das gleiche ursprüngliche Entitylayout wie der lesende ECS-Pfad:

| Daten | Originaler Zugriff |
| --- | --- |
| Storagebasis | Pointer `[EntityEntry + 0x20]` |
| Entityzeile | `uint32 [EntityEntry + 0x30]` |
| Entity-ID | `uint32 [EntityEntry + 0x10]` |
| Entitymetadaten | `Storage + uint16[Storage + 0x14] + Row * 16` |
| Sequenzmarker A | `Storage + uint16[Storage + 0x1c] + Row * 4` |
| Sequenzmarker B | `Storage + uint16[Storage + 0x1e] + Row * 4` |
| Aktuelle Sequenz | `uint32 [Manager + 0x4dc]` |
| Registrierungsbesitzer | Pointer `[Manager + 0]` |
| Auswahl A | Bit des Komponentenindex ab `RegistryOwner + 0x190` |
| Auswahl B | Bit des Komponentenindex ab `RegistryOwner + 0x230` |

Kategorie A prüft zusätzlich Managermodus, einen aktiven Zustand bei `Manager+0xebc2d0` und Entitymetadaten. Kategorie B verwendet Entitymetadaten bei Offset `+0xc` und Flags bei `+0xd`. Diese Kategorien werden hier anhand ihrer belegten Aufzeichnung beschrieben. Eine vollständige Zuordnung beider Kategorien zu Netzwerk- oder Persistenzfunktionen ist nicht behauptet.

## Queueaufbau und Nebenwirkungen

Der Konstruktor ab `0x9846ce` richtet in einer Schleife acht Queues je Kategorie ein. Kategorie A beginnt bei Contextoffset 0, Kategorie B bei `0x140`. Der Stride ist `0x28`.

| Feld innerhalb einer Queue | Breite und Bedeutung |
| --- | --- |
| `+0` | Pointer auf `uint32`-Entity-IDs |
| `+8` | `uint64` Anzahl |
| `+0x10` | `uint64` Kapazität |
| `+0x18` | Vergrößerungscallback |
| `+0x20` | Allocatorpointer und dessen ursprüngliche Tagbits |

Die anfängliche Kapazität beträgt 16.384 IDs pro A-Queue und 4.096 IDs pro B-Queue. Die Engine kann diese Vektoren später vergrößern. Diese Werte sind deshalb keine belegten Maximalgrenzen.

Die Aufzeichner bei `0x9c5ab0` und `0x9c5b50` vergleichen die unteren 31 Bits des Sequenzmarkers mit der aktuellen Sequenz und verwenden `lock cmpxchg`. Ein erfolgreicher Übergang reiht die Entity-ID über `0xa0c100` ein. Wiederholte Meldungen innerhalb derselben Sequenz erzeugen dadurch keinen weiteren Eintrag.

Bei fehlendem Platz kann der originale Vektor einen Vergrößerungscallback über `0x4660a0` nach `0x46a910` erreichen. Scheitert das Einreihen, setzt der Aufzeichner Bit 31 des Sequenzmarkers und erhöht atomar den Pendingzähler bei Contextoffset `0x280` beziehungsweise `0x284`. Der Wiederholungsweg `0x9cd210` und `0x9cd180` erkennt solche Marker und versucht die IDs später in Queue 0 einzureihen. Queuevoll ist im Original deshalb kein boolescher Fehlerreturn des Trackers.

Ein Aufruf hängt höchstens eine ID an A und höchstens eine ID an B an. Der B-Weiterleitungszweig kann andere Entitys über einen ursprünglichen registrierten Beziehungstyp aufsuchen. Er aktualisiert ihre Sequenzmarker und läuft weiter, bis er die passende Zielmetadatenform erreicht. Das einzige Append dieses Zweigs liegt bei `0x9c5e38`, danach kehrt er zurück. Zwischenstationen werden nicht zusätzlich angehängt. Sequenzvergleiche beenden bereits besuchte Ketten.

## Threadvertrag und Grenzen einer Übernahme

Tracker, Aufzeichner und einfacher Vektorappend lesen keine TLS- oder Thread-ID-Daten. Der Queueindex ist ein gewöhnlicher Parameter. Der ursprüngliche serielle Aufrufer verwendet Queue 0. Das allein macht paralleles Schreiben in diese Queue nicht sicher. Pointerstore und Count-Inkrement des Vektors sind nicht atomar.

Eine Übernahme muss deshalb zuerst den ursprünglichen Entity-/Scene-Lifetimevertrag erfüllen und parallele Systemtasks ausschließen. Erst danach dürfen Manager, Kontext-Rückpointer, Entitygeneration und Zeile sowie die aktuellen Registrierungen erneut bestätigt werden. Ein No-op oder Compare-exchange-Konflikt sollte vor der Meldung abbrechen.

Vor einem ursprünglichen Trackeraufruf lassen sich beide Queue-0-Header prüfen und jeweils mindestens ein freier Eintrag verlangen. Unter ausgeschlossenen parallelen Schreibern reicht diese Reserve für den belegten maximalen Appendbedarf aus. Damit muss der Aufruf keinen Allocatorpfad erreichen. Bei voller Queue sollte die neue Runtime geschlossen abbrechen, solange der Allocator-Threadvertrag nicht vollständig geprüft ist.

Die Meldung vor einer Mutation ist durch die Mutable-Accessor-Pfade belegt. Es wäre falsch zu behaupten, dass alle generischen Aufrufer vor ihrer Mutation melden. Der Befehlspfad bei `0xa16350` meldet eine tatsächlich geänderte Komponente anschließend. Diese Untersuchung liefert daher einen vorhandenen generischen Vorher-Vertrag und keine pauschale Reihenfolgebehauptung für sämtliche Enginefunktionen.

Schreibfreigabe, konkrete Livewirkung und Netzwerk-/Persistenzresultate folgen aus diesen statischen Belegen nicht automatisch. Die Runtimeintegration und ihr tatsächlicher Spieltest werden separat dokumentiert.

## Isolierte Originalausführung und lesender Liveabgleich

Der unveränderte Originaleintritt `0x9c5960` wurde zusätzlich in Unicorn mit privaten synthetischen Manager-, Registry-, Entity-, Storage- und Queuedaten ausgeführt. Alle zehn Fälle sind erfolgreich. Sie bestätigen beide Aufzeichnungskategorien, die ursprünglichen IDs in den Queues, die Deduplizierung derselben Sequenz, getrennte Registrierungsbits, volle Queues ohne Allocator und die ausbleibende Doppelzählung bereits vorgemerkter Pendingmarker.

Auch der Beziehungszweig wurde dabei ausgeführt. Eine dreigliedrige Entitykette markiert die Sequenzen aller besuchten Entitys und hängt genau eine finale ID an B0 an. Ein Zyklus mit zwei Entitys endet am bereits besuchten Sequenzmarker. Die Reserve von höchstens einer Queue-ID je Kategorie ist damit sowohl im Kontrollfluss als auch an der Originalemulation geprüft. Diese Daten sind synthetisch. Die Emulation ist kein Gameplaytest.

Ein separater ausschließlich lesender Liveabgleich um `03:14:58 UTC` bestätigte für PID 24432 die originale aktive Scene-Kette. Der Entitymanager war `0x3006bed4b40`, sein Trackingkontext `0x3006cd90b70`. Der Rückpointer stimmt mit dem Manager überein. A0 hatte Anzahl 0 und Kapazität 16.384, B0 Anzahl 0 und Kapazität 4.096. Beide Vergrößerungscallbacks entsprechen RVA `0x4660a0`. Die Ownerkette blieb beim erneuten Lesen gleich. Dabei wurden weder Locks noch Callbacks aufgerufen oder Spielbytes geschrieben.

## Offene Epochphase

Die Teamsichtung hat eine weitere notwendige Grenze gefunden. Die Engine erhöht die Sequenz bei `0x9c80e0` am Beginn eines Frames. Der Abschlussweg `0xa1dd20` erreicht später die Queuekonsumenten und leert die Queues. Ein Zugriff zwischen zwei Frames kann deshalb einen noch aktuellen Sequenzmarker vorfinden, dessen Queueeintrag bereits verarbeitet wurde. Derselbe Tracker könnte dann korrekt deduplizieren und trotzdem keinen neuen Eintrag für diesen Zugriff erzeugen.

Der belegte generische Tracker und eine ruhige Speicherphase reichen allein deshalb noch nicht zur Schreibfreigabe aus. Die Einordnung in die ursprüngliche Epoch-/Commandphase wird separat verfolgt. Dieser Bericht empfiehlt keine künstliche Sequenzänderung und keine doppelte eigene Trackingimplementierung.

## Warum Pending kein zusätzlicher Schreibkanal ist

Die Originalfunktion `0x9cd180` wird nur für einen bereits negativen Sequenzmarker tätig. Sie liest weder die aktuelle Managersequenz noch den Inhalt der Komponente. Bei freiem Queueplatz hängt sie die Entity-ID an, löscht Bit 31 des bestehenden Markers und dekrementiert den zugehörigen Pendingzähler. Die unteren 31 Bits bleiben unverändert. Ein gewöhnlicher positiver Marker wird sofort übersprungen.

Der Scanner `0x9cd210` startet nur bei einem Pendingzähler ungleich null. Er geht über die belegten Entitytabellenplätze und versucht höchstens 4.096 Entitys pro Aufruf. Der fortgesetzte Tabellenindex liegt bei Contextoffset `0x288`. Der Aufrufer `0xa1dd20` erreicht Recovery bei `0xa1dfb1` und anschließend den A-Konsumenten bei `0xa1dfb9`. Das ist die Verarbeitung vorhandener Overflowmeldungen, keine zusätzliche Erkennung beliebiger Speicheränderungen.

Die acht ergänzenden Originalemulationsfälle bestätigen folgende Grenzen:

- Nach dem ursprünglichen Leeren der Queuelängen derselben Sequenz erzeugt ein erneuter Trackeraufruf keinen Eintrag. Das Leeren wurde in privaten Daten entsprechend den belegten Originalstores nachgebildet. Es wurde kein vollständiger Frame emuliert.
- Recovery erzeugt für einen positiven Marker weder einen Eintrag noch einen Pendingzustand.
- Recovery reiht negative Marker mit aktueller, älterer oder null Sequenz ein und behält jeweils deren unteren 31 Bits.
- Bei fehlendem Queueplatz bleiben negativer Marker und Pendingzähler bestehen.
- Der gewöhnliche Tracker ersetzt einen negativen Pendingmarker auch nach einem Sequenzwechsel nicht. Der ursprüngliche Compare-exchange erwartet den von Bit 31 bereinigten alten Wert und scheitert deshalb am weiter negativen Marker. Der Recoveryweg ist weiterhin erforderlich.

FrameSetup `0x9c80e0` erhöht `Manager+0x4dc` um die ermittelte Tickanzahl. Der anschließend bei `0x9c82e8` aufgerufene Cleanup `0x963f60` räumt den transienten Blockpool ab `Manager+0xeb9e58` und dessen Registrierungsdaten auf. Seine vollständigen drei Unwindfragmente enthalten keine Änderung an den Trackingqueues, Sequenzmarkern oder Pendingzählern. Bereits erzeugtes Pending kann dadurch bis zu einer späteren Recovery bestehen bleiben.

Damit ist Pending als allgemeiner Eintritt für Lua-Schreibzugriffe nicht belegt. Eine solche Übernahme würde hier ein künstliches Setzen des Overflowmarkers und des Zählers voraussetzen. Die Runtime verwendet diesen Weg nicht als Ersatz für die noch fehlende ursprüngliche Schreibphase.

## Generischer begrenzter Schreibvertrag ohne Trackingkategorie

Der ursprüngliche Tracker entscheidet für jeden registrierten Komponentenindex separat, ob überhaupt eine der beiden Kategorien beteiligt ist. Kategorie A liest ihr Bit bei `0x9c59bc` aus `RegistryOwner+0x190+(Index/64)*8`. Der Test bei `0x9c59c7` verzweigt bei null direkt zum B-Zweig. Kategorie B liest bei `0x9c5a41` aus `RegistryOwner+0x230+(Index/64)*8`. Der Test bei `0x9c5a4c` verzweigt bei null zum Funktionsende. Für einen Index ohne beide Bits erreicht die Originalfunktion keinen Aufzeichner und verändert keine Trackingdaten.

Das erlaubt einen engeren Vertrag unter dem ursprünglichen Lifetime-/Scene-Guard. Nach erneuter Bestätigung der aktuellen Entity, Registrierung und Besitzerkette darf die Runtime eine Komponente genau dann ohne zusätzliche Trackingphase bearbeiten, wenn beide aktuellen Registrierungsbits null sind. Die Freigabe wird aus dem laufenden Registrierungsbesitzer und dem ursprünglichen Komponentenindex abgeleitet. Sie benötigt keine Modliste, keinen fest eingetragenen Typnamen und keine Veränderung von Sequenzmarkern. Eine Komponente mit gesetztem A- oder B-Bit bleibt in diesem Vertrag gesperrt.

Die zusätzliche Originalemulation führt zwölf Kombinationen mit Managermodus 0, 1 und 2, aktivem oder inaktivem A-Zustand sowie direkter B-Metadatenform oder Aliasflag aus. Beide Registrierungsbits sind jeweils null. Die Testdaten enthalten bereits einen aktuellen A-Marker, einen negativen B-Marker und Queues ohne freien Platz. Nach jedem unveränderten Originalaufruf sind der gesamte private Storageblock und der Trackingkontext bytegenau unverändert. Es wurde kein Spielprozess geöffnet.

Der ausschließlich lesende Liveabgleich von PID 24432 bestätigt den praktischen Umfang. Alle Registrierungsnamen, Registrierungshashes und Runtime-Metadatapointer wurden mit der aktiven Registrierung abgeglichen. Die Tabellen und beide Bitmasken blieben beim Nachlesen unverändert.

| Live registrierte Runtime-Typen | Anzahl |
| --- | ---: |
| Ohne A und B | 421 |
| Nur A | 75 |
| Nur B | 24 |
| A und B | 21 |
| Gesamt | 541 |

Von den 59 reflektierten Attributlayouts, die gemeinsam die 307 Parser-Attributdefinitionen enthalten, haben 55 weder A noch B. Nur `Durability`, `Experience`, `Fuel` und `Level` sind B-getrackt. `Health`, `Stamina` und `Movement` haben keine Trackingkategorie. `NetworkPlayerAttributes` ist dagegen A-getrackt und bleibt mit dieser Richtlinie schreibgesperrt.

Bei 600 weiteren Player-Stamina-Beobachtungen wurden 286 Fälle mit Scene-Gatewert null vor und nach dem Lesen sowie gleicher Managersequenz erfasst. Der gemeinsame A-Marker der Playerentity war dabei jeweils bereits aktuell. Das Stamina-Registrierungsbit A ist aber null. Der Marker wurde durch einen anderen beteiligten Komponententyp derselben Entity gesetzt und darf deshalb keine pauschale Stamina-Schreibsperre begründen. Diese Beobachtung hielt selbst keinen Lock und ersetzt die Guardprüfung bei einer tatsächlichen Mutation nicht.

Die vorhandene Enginebeschreibung `network_player_attributes` bei `0x275c10` deklariert `Stamina` und `Health` als Runtime-Leseabhängigkeiten sowie `NetworkPlayerAttributes` als Runtime-Schreibabhängigkeit. Das beschreibt den vorhandenen Datenfluss. Eine tatsächliche Auswirkung eines Lua-Schreibzugriffs auf die daraus erzeugten Netzwerk-/UI-Werte muss weiterhin durch einen kontrollierten Apply-/Restore-Spieltest bestätigt werden. Die hier dokumentierte Freigabe betrifft Speicherbesitz und den originalen Trackingvertrag, nicht die Dauerhaftigkeit beliebiger Modänderungen.

## Reproduzierbare Belege

Die ignorierten Originalartefakte liegen unter `profile-tools/devdata/creative-mode-146/`:

- `current-attribute-vm-codeproof-20261016.json` enthält vollständige Bytes, Hashes, Instruktionen und die zusammengefasste Tracker-ABI.
- `current-tracking-entry-callers-20261016.json` enthält alle fünf direkten, instruktionsvalidierten Aufrufer des generischen Eintritts.
- `current-tracking-owner-references-20261016.json` enthält die Konstruktor- und Kontextbeziehungen.
- `current-change-tracker-oracle-20261016.json` enthält die zehn erfolgreichen Originalemulationsfälle.
- `current-tracking-recovery-oracle-20261016.json` enthält acht weitere erfolgreiche Fälle mit dem ursprünglichen Recoverycode und die vollständigen Unwindfragmente der untersuchten Frame-/Cleanupfunktionen.
- `current-untracked-components-oracle-20261016.json` enthält zwölf erfolgreiche Originalfälle für den generischen Zweig ohne A-/B-Registrierungsbit.
- `current-tracking-selection-inventory-20261016.json` enthält den lesenden Abgleich aller 541 Runtime-Typen und 59 Attributlayouts mit beiden ursprünglichen Auswahlmasken.
- `current-player-tracking-freshness-20261016.json` enthält 600 ausschließlich lesende Beobachtungen der tatsächlichen Player-Stamina-Beziehung.
- `current-live-tracking-context-20261016.json` enthält den lesenden Abgleich des aktuellen realen Managerkontexts und beider Queue-0-Header.
- `inspect-current-attribute-vm.py`, `inspect-current-tracking-entry.py` und `find-current-tracking-owner.py` lesen ausschließlich die EXE als Datei.
- `verify-current-change-tracking.py` führt ausschließlich private Unicorn-Emulation aus.
- `verify-current-tracking-recovery.py` ergänzt ausschließlich private Originalemulation und die dokumentierte synthetische Zusammensetzung des Queue-Countresets.
- `verify-current-untracked-components.py` führt ausschließlich private Originalemulation aus. `inventory-current-tracking-selection.py` liest ausschließlich den Spielprozess.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ziel | Einen vorhandenen generischen Vertrag je ursprünglichem Komponentenindex statt weiterer Adapter pro Mod ermittelt. |
| Belege | Aktuelles exaktes Image, Reflection, gültige Instruktionsgrenzen, Aufrufer und Konstruktor verfolgt. |
| Prüfung | Statische Daten- und Kontrollflussprüfung, 30 erfolgreiche Originalemulationsfälle sowie lesender Liveabgleich von Kontext, Queues und ursprünglichen Auswahlmasken. Keine Spielfunktion im Spiel ausgeführt und kein Prozessspeicher geschrieben. |
| Offen | Integration und tatsächliche Livewirkung. Für getrackte Komponenten bleibt die ursprüngliche Epoch-/Commandphase offen. Allocatorpfad bleibt ausgeschlossen. |
