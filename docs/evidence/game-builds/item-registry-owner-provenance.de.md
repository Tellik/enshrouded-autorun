# Originale Besitzketten für die geladene ItemRegistry

## Geltung

Rein statisch aus der installierten Originaldatei `enshrouded.exe` abgeleitet.
Keine Spielzugriffe, Heapsuche, Spielaufrufe oder Änderungen an ausführbarem Code.
Die RVAs gelten für SHA256
`4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`,
PE-Zeitstempel `1792083981`, Imagegröße `44142592`.
Die vollständigen Instruktionsbelege mit Bytes stehen in
`item-registry-owner-provenance.json`.

`ClientOwner`, `ClientController`, `SceneOwner`, `MainOwner`, `C` und `B` sind
beschreibende Bezeichnungen. Sie behaupten keine wiederhergestellten C++-RTTI-Namen.

## Gemeinsamer globaler Einstieg

`EXE + 0x1fe1cb0` ist ein **direkt eingebettetes Objekt**, kein Pointer-Slot.
Die Funktion `0x836630` initialisiert dort `0x290` Bytes. Bei `0x83665b`
setzt sie die Vtable auf `EXE + 0x1d45960`.

Der originale Getter `0x83e8d0` besteht aus
`lea rax, [rip + 0x17a33d9]` und `ret`. Sein Ergebnis ist genau dieses Objekt.
Bei `0x8fdeef` wird die Initialisierung aufgerufen und bei `0x8fdef4` der Getter.
Anschließend nutzt der Originalcode dessen virtuelle Methoden.

Die Vtable bei `0x1d45960` enthält bei `+0x20` die Funktion `0x84db10`.
Diese hält ihr ursprüngliches RCX in RDI, allokiert `0x35660` Bytes und
konstruiert den `ClientOwner` über `0x82a530`. Bei `0x84dfa1` speichert
sie die Rückgabe in `[Root + 0x258]`.

Der Konstruktor setzt bei `0x82a559` die Vtable des `ClientOwner` auf
`EXE + 0x1d45920`. Deren `+0x18` zeigt auf `0x846940`.
Diese beiden Vtablewerte sind konkrete Typprüfungen für begrenzte Leser.

## Alternative über den aktiven ClientController

```text
EXE + 0x1fe1cb0                         eingebettetes Root-Objekt
  [ + 0x258 ]                          ClientOwner, Vtable 0x1d45920
  [ + 0x35280 ]                        ClientController, Vtable 0x1d45c30
  [ + 0x1b8 ]                          MainOwner
  [ + 0x47d4e0 ]                       Objekt mit Allokationsgröße 0x10010
  [ + 0x3e10 ]                         g38_items::ItemRegistry
```

Jede Zeile mit eckigen Klammern ist ein eigener 64-Bit-Pointerladevorgang.

- `0x839640`, erreichbar über `ClientOwner`-Vtable `+8`, allokiert den
  `0x210`-Byte-Controller. Bei `0x8396bd` setzt es dessen Vtable auf
  `0x1d45c30` und speichert ihn bei `0x8396f2` in `[ClientOwner + 0x35280]`.
- Bei `0x8396f9` wird `[ClientController + 0x28] = ClientOwner` gesetzt.
  Dieser Rückzeiger ist eine zusätzliche Besitzprüfung.
- Controller-Vtable `+0x18` ist `0x848ad0`. Diese Funktion behält ihr
  RCX in R15. Bei `0x849f61` liest sie `[R15 + 0x1b8]` als `MainOwner`.
- `0x841f61` ruft `0x39f890` auf. Die Rückgabe wird bei `0x841f6d`
  in `[MainOwner + 0x47d4e0]` gespeichert.
- In `0x39f890` landet die neue Allokation mit Größe `0x10010` in R15.
  Bei `0x39f9ff` wird `0xe64360` aufgerufen. `0x39fa04` speichert dessen
  Rückgabe in `[R15 + 0x3e10]`.
- `0x3a1407` lädt genau dieses Feld. Der anschließende Aufruf `0x3a143d`
  registriert es unter dem Originalnamen `g38_items::ItemRegistry`.

**Korrektur einer möglichen Verwechslung:** `[MainOwner + 0x1d0]` ist hier
ein anderer Kontext. Dieser wird erst bei `0x841f92` für `0x9f0ff0` gelesen.
Er enthält nicht die bei `0x841f61` erzeugte Rückgabe.

## Alternative über SceneOwner-Einträge

```text
EXE + 0x1fe1cb0
  [ + 0x258 ]                          ClientOwner
  [ + 0x630 ]                          SceneOwner
       [ + 0x440 ]                     Anfang des Entry-Arrays
       u64( + 0x448 )                  Anzahl der Entries
       Array + index * 0x90            Inline-Entry
         [ + 0 ]                       C
         [ + 0x1d8 ]                   B, Allokationsgröße 0xea0
         [ + 0x1d8 ]                   g38_items::ItemRegistry
```

- `0x84826c` ruft den `0x5e0`-Byte-Konstruktor `0xe33a40` auf.
  Bei `0x848271` speichert der Aufrufer die Rückgabe in `[ClientOwner + 0x630]`.
- `0xe470f0` erhält `SceneOwner` in RCX und behält ihn in RSI.
  `0xe471b2` liest den 64-Bit-Count bei `+0x448`. `0xe471f0` liest den
  Arraypointer bei `+0x440`. Der Schleifenfortschritt bei `0xe47458` ist `0x90`.
- Bei `0xe472c3` wird C in `[Entry + 0]` gespeichert. `0xe472e0` lädt
  denselben Pointer als RDX für `0xe39d30`, das ihn in RDI behält.
- `0xe3a7b6` ruft den `0xea0`-Byte-Konstruktor `0x4943f0` auf.
  `0xe3a7bb` speichert seine Rückgabe B in `[C + 0x1d8]`.
- Innerhalb dieses Konstruktors ruft `0x4947da` ebenfalls `0xe64360` auf.
  `0x4947df` speichert die Rückgabe in `[B + 0x1d8]`.
- `0x495d2b` lädt `[B + 0x1d8]`. `0x495d47` registriert genau diesen
  Pointer als `g38_items::ItemRegistry`.

Die beiden `+0x1d8` sind somit zwei **verschiedene Besitzerobjekte** und
zwei verschiedene Dereferenzierungen. Es handelt sich nicht um einen kopierten
Feldnamen oder eine aus gleicher Größe vermutete Zuordnung.

### Entry-Zustand und Lebensdauer

`Entry + 0x18` ist ein Byte-Zustand. Die Sprungtabelle bei `0xe475d8`
besitzt neun Fälle.

| Zustand | Ziel | Beobachteter Kontrollfluss |
| --- | --- | --- |
| 0 | `0xe47549` | Entry entfernen, Array um `0x90` kompaktieren, Count verringern |
| 1 | `0xe47253` | C erstellen und in Entry speichern |
| 2 | `0xe472e0` | `0xe39d30` auf C ausführen, dabei B erstellen |
| 3 | `0xe4732d` | Nächsten Initialisierungsschritt ausführen |
| 4 | `0xe4736f` | Auf Bereitschaft warten und Übergang ausführen |
| 5 | `0xe474b6` | Bleibt bestehen, bis Byte `Entry + 0x2b` gesetzt wird |
| 6 | `0xe474cc` | Abbauschritt ausführen |
| 7 | `0xe4750d` | Abbau abschließen und `[Entry + 0] = 0` setzen |
| 8 | `0xe47533` | In Abbauzustand wechseln |

Zustand 5 ist aus dem Kontrollfluss ein Kandidat für eine geladene Szene.
Ein eindeutiger Bezug zum lokalen Spieler ist damit nicht bewiesen. Die Tabelle
kann leer sein oder mehrere Einträge enthalten. Der erste Eintrag ist kein
allgemeingültiger aktiver Weltnachweis. Eine maximale Engine-Anzahl oder eine
Kapazität wurde hier nicht hergeleitet.

## Verbindung zur Parser-Ressource

`0xe64360` allokiert einen eigenen `0x328`-Byte-Wrapper.
Dieser enthält den Allocatorpointer bei `+0`, den übergebenen Ressourcendienst
bei `+8` und einen **32-Bit-Ladehandle** bei `+0x10`.

Der Handle stammt aus `0x1062f30`. Dabei wird der Schlüssel `0x72ab39b6`
und ein 16-Byte-GUID-Blob übergeben. In der frischen KFC4-Reflection ist dieser
Schlüssel exakt `keen::ItemRegistryResource`, Größe 72.

Für den `0x10010`-Besitzer stammt der Blob aus
`[[ConstructorArgs + 0x118] + 0x40]`.
Für B stammt er aus `[[ConstructorArgs + 0xe8] + 0x40]`.

Das beweist die Verbindung des originalen Service-Wrappers zur benannten
Parser-Ressource. Der Handle ist kein direkter `ItemInfo*`.
Die Wrapper-internen geladenen Itemtabellen, der genaue Gleiter-Consumer und
ein zulässiger Änderungszeitpunkt sind durch diese Besitzketten allein noch
nicht nachgewiesen. Keine der Ketten wird als Produktionsprofil aktiviert.
