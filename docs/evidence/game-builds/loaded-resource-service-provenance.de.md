# Generische Auflösung bereits geladener Ressourcen

## Geltung und Belege

Die statischen Belege stammen aus der Original-EXE mit SHA256
`4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`.
Die vollständigen Instruktionen stehen in
`loaded-resource-service-provenance.json`.

Diese Untersuchung hat keine Spielzugriffe, Heapsuche, Spielaufrufe oder
Schreibzugriffe ausgeführt. Ein zusätzlicher, eng begrenzter Live-Abgleich des
Root-Agenten ist separat unter
`../creative-mode-146/loaded-resource-handle-proof.json` dokumentiert.

## 1. Kanonischer Ressourcenschlüssel

Die Engine sucht mit demselben `ResourceId`, den der vorhandene Parser bereits
implementiert. Es ist kein zweiter Hash- oder Identitätsvertrag erforderlich.

| Offset | Breite | Bedeutung |
| --- | --- | --- |
| `+0` | 16 Byte | GUID in der originalen Bytefolge |
| `+0x10` | 4 Byte | Qualified-Type-Hash |
| `+0x14` | 4 Byte | Part-Index |
| `+0x18` | 8 Byte | Reserviert, in den untersuchten Konstruktoren null |

Gesamtgröße und Array-Stride sind `0x20` Byte. Die Engine vergleicht die ersten
24 Byte inhaltlich. Die acht reservierten Byte gehen nicht in den Vergleich ein.

Die Parserimplementierung in
`src/parser/kfc-parser/crates/kfc-base/src/guid/resource.rs`,
`StaticHash for ResourceId` und `ResourceId::read/write`, ist deckungsgleich.

Der Hash ist exakt:

```python
h = little_endian_u32(guid_bytes[0:4])
for byte in type_hash.to_bytes(4, "little") + part_index.to_bytes(4, "little"):
    h = ((h ^ byte) * 0x01000193) & 0xffffffff
```

Belegte Konstruktionen:

- `0xe643c5` setzt den Typ `0x72ab39b6`, in der frischen Reflection
  `keen::ItemRegistryResource`. `0xe643ce` kopiert die GUID und setzt Part null.
- `0xe87845` setzt den Typ `0xb5ce8765`, in der frischen Reflection
  `keen::ItemInfo`. Die GUID stammt aus `ItemRegistryResource.itemRefs`.
  Bei `0xe87852` und `0xe8785f` werden Part und reservierte Felder null gesetzt.

## 2. Originaler Ressourcendienst und Indexbesitzer

`S` bezeichnet den geladenen Ressourcendienst aus `[ItemRegistry + 8]`.
Der Konstruktor `0x105fd30` installiert dessen originale Funktionsslots.

| Slot im Dienst | Originalfunktion | Aufgabe |
| --- | --- | --- |
| `S + 0x7d8` | `EXE + 0x1062eb0` | `ResourceId` zu Handle |
| `S + 0x7e8` | `EXE + 0x10629c0` | Handle zu geladenem Datenbereich |

`0x1062f30` und `0x1062a90` sind die zugehörigen aufrufenden Wrapper.
Die Funktionen müssen für einen lesenden Zugriff nicht im Spiel aufgerufen
werden. Ihre nachfolgend belegte Datenauflösung lässt sich lesend nachvollziehen.

Der originale Getter `0x123e440` wird mit `[S + 8]` aufgerufen. Er liest
`[Argument + 0x60]` und addiert `0x48`. Die vollständige Kette lautet somit:

```text
reader   = u64(S + 8)
metadata = u64(reader + 0x60)
map      = metadata + 0x48
```

Es sind **zwei Pointerladevorgänge**, gefolgt von einer Addition.
`map` wird nicht nochmals als Pointer dereferenziert.

## 3. StaticMap und GUID/Typ/Part zu Handle

Die drei Arrays besitzen jeweils einen vorzeichenbehafteten relativen
32-Bit-Offset und eine 32-Bit-Anzahl.

| Map-Offset | Feld |
| --- | --- |
| `+0` | Rel32 zum Bucketarray |
| `+4` | Bucketanzahl |
| `+8` | Rel32 zum Keyarray |
| `+0xc` | Keyanzahl |
| `+0x10` | Rel32 zum Valuearray |
| `+0x14` | Valueanzahl, damit Anzahl der Handle-Einträge |

Ein relativer Offset bezieht sich auf **die Adresse seines eigenen Feldes**.
Ein Nulloffset bedeutet null. Für einen nichtnull Offset gilt
`Ziel = Feldadresse + sign_extend_i32(Feldwert)`.

Der Originalalgorithmus `0x10607c0`:

1. Eine leere Bucketanzahl liefert keinen Treffer.
2. Bucketindex ist `ResourceId.static_hash() & (bucket_count - 1)`.
3. Ein Bucket besteht aus `u32 first_index` und `u32 count`, Stride 8.
4. Die Engine vergleicht die GUID, den Typ und den Part für die Keys dieses
   zusammenhängenden Bucketbereichs. Keystride ist 32.
5. Ein Treffer liefert die Adresse des entsprechenden Value-Slots, Stride 8.

`0x1062eb0` wandelt diese **Slotadresse**, nicht den Slotinhalt, in den Handle um:

```text
handle = (slot_address - values_base) / 8 + 1
```

Null ist kein gültiger Handle. Ein Nichttreffer ergibt null. Der Wrapper
`0x1062f30` liefert für eine vollständig leere GUID unmittelbar null.

Die Suchfunktion prüft selbst nicht alle Key-/Value-Grenzen, weil sie ihren
Originalcontainer voraussetzt. Ein unabhängiger Leser muss Bucketbereiche,
gleiche Key-/Valueanzahlen, relative Offsets und `1 <= handle <= value_count`
explizit begrenzen. Die aktuelle Live-Map hatte im Root-Abgleich **91.812 Keys
und 91.812 Values**. Das ist ein beobachteter Wert dieses Builds, kein festes
Limit für die API.

## 4. Handle zu geladenem Datenbereich

`0x10629c0` führt die folgende Übersetzung aus:

```text
entry       = values_base + (handle - 1) * 8
chunk_id    = u8(entry + 7)
offset      = u32(entry)
byte_size   = u32(entry + 4) & 0x00ffffff

chunk_table = u64(S + 0x30)
chunk       = chunk_table + chunk_id * 16
chunk_base  = u64(chunk)
chunk_size  = u64(chunk + 8)
data        = chunk_base + offset
```

Vor dem Laden prüft der Originalcode `chunk_id < u32(S + 0x58)`.
Die 16-Byte-Chunkdatensätze werden bei `0x1061a6b` und `0x1061a73` mit Basis
und Größe befüllt. Ein unabhängiger Leser sollte zusätzlich
`offset + byte_size <= chunk_size` und die Tabellengrenzen prüfen.

Die Originalrückgabe umfasst 24 Byte:

| Offset | Bei Erfolg |
| --- | --- |
| `+0` | Datenpointer, 64 Bit |
| `+8` | Byteanzahl, 64 Bit |
| `+0x10` | 32-Bit-Wort null |
| `+0x14` | 32-Bit-Wort eins |

Bei Handle null oder nicht verfügbarem Chunk liefert der Wrapper 24 Nullbytes.
Die beiden letzten Wörter sind bitgenau hergeleitet. Ihre vollständige
Retention-/Freigabesemantik wird hier nicht als gelöst behauptet.

### Eng begrenzter Live-Abgleich des Root-Agenten

Handle **80640** ergab über genau diese Datenübersetzung
`0x20008002640`, **247.432 Byte**. Dieser Pointer ist exakt der bereits im
ItemRegistry-Wrapper bei `+0x20` gespeicherte Wert. Sein reflektierter
`ItemRegistryResource`-Header enthält **4.207 itemRefs**.

Der Abgleich las 180 Byte. Callbackadresse, Indexbesitzer, Handle-Eintrag und
Chunkbasis wurden geprüft beziehungsweise erneut gelesen. Es erfolgten keine
Engineaufrufe oder Schreibzugriffe. Die größere Ressourcenlänge gegenüber der
reflektierten Headergröße 72 enthält die nachfolgenden Blobdaten.

Der Root-Agent hat außerdem den 32-Byte-Key dieses Handles aus dem Keyarray
gelesen. GUID-Bytes, Typ `0x72ab39b6`, Part null und acht reservierte Nullbytes
passen exakt zum kanonischen `ResourceId`. Beide Arrayanzahlen sind 91.812.
Dieser gesonderte Nachweis steht unter
`../creative-mode-146/loaded-resource-key-proof.json`.

## 5. Identität der ItemMap-Pointer

Die originale Lademethode `0xea8720` verbindet alle Ebenen:

- `0xea89b9` lädt `ItemRegistryResource` über Dienst und Wrapperhandle.
  `0xea89f3` speichert den zurückgegebenen Pointer in `[Registry + 0x20]`.
- `0xea8d5d` ruft `0xe877f0` auf. Diese Funktion geht die itemRefs-GUIDs
  durch und löst sie mit dem Typ `keen::ItemInfo`, Part null, in Handles auf.
- `0xea87ad` lädt jeden ItemInfo-Datenbereich über `0x1062a90`.
  `0xea87ec` erhält dessen Pointer in RDI. `0xea87fd` liest daraus die Item-ID.
- `0xea885d` schreibt **genau diesen Pointer** in die Item-ID-Map.
  Dieser Kontrollfluss erstellt keine ItemInfo-Kopie.

Damit sind die ItemMap-Pointer an dieselben bereits geladenen Ressourcenbytes
gebunden, die sich generisch über den kanonischen Parser-`ResourceId` auflösen
lassen. Der frühere dateibasierte `game.assets.Resource.data`-Pfad besitzt diese
Bindung bisher nicht.

## Offene Grenzen

Die belegte Datenbindung ist kein fertiger Schreibvertrag. Zulässiger
Änderungszeitpunkt, Lebensdauer bei einem Ressourcendienstwechsel und mögliche
abgeleitete Engine-Caches müssen gesondert verfolgt werden. Diese Untersuchung
fügt keinen Produktionsadapter, keine neue API-Schicht und keinen Patch hinzu.
