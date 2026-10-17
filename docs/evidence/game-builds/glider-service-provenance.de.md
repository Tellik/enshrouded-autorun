# Item-ID zu geladenem ItemInfo, statischer Servicepfad

Untersuchte EXE SHA256: `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Ausschließlich bestehende Artefakte und Dateibytes gelesen. Kein Spielzugriff und keine Codeänderung.

## Zwei Aufrufketten

| Pfad | Nachgewiesene Herkunft von RDX für `0xe7a250` |
| --- | --- |
| `actor_rotation`, Root `0x269d30` | Der Aufruf bei `0x269d9b` initialisiert einen Frame `F = rbp-0x40` über `0x9ba5a0`. `0x26a82a` lädt RDX aus `[rbp+0xe8]`, also `[F+0x128]`. Der Call erfolgt bei `0x26a834`. |
| `locomotion_execution`, Root `0x435d30` | `0x435d69` initialisiert `F = rsp+0x60` über `0x9ba5a0`. `0x436517` setzt RCX auf F, `0x43651c` ruft `0x42b0e0` auf. Dieser übernimmt RCX nach R14 bei `0x42b0ff` und lädt den Service aus `[R14+0x1b8]` nach RBX bei `0x42b195`. Bei `0x42b1e2` wird RDX gleich RBX gesetzt. Der Call erfolgt bei `0x42b1e8`. |

Die Item-ID ist im ersten Pfad `u32[rbx+0x38]`, im zweiten `u32[*(F+0x68)+0x38]`. Der jeweilige Getterrückgabewert geht nach RDI beziehungsweise R12. Die vollständige Komponentenidentität des ID-Besitzers wird durch diese Untersuchung nicht zusätzlich behauptet.

## Was der Frameinitialisierer tatsächlich kopiert

Für Eingabe `I = ursprüngliches RCX`, `F = ursprüngliches RDX` verwendet `0x9ba5a0` folgende Struktur:

```text
N = *(I + 0)
T = *(N + 0xc0)
n = u8[T + 0x400]
B = 8
    + 8 * u8 [N + 0x4b0]
    + 8 * u16[N + 0x4b8]
    + 48 * u16[N + 0x4ba]
memcpy(F + B, T, n * 8)
```

Belege sind `0x9ba5b4`, `0x9ba5bd..0x9ba5fc` für die Größen und `0x9ba5f5..0x9ba600` für T und n. Die Kopie selbst steht bei `0x9ba6eb..0x9ba6fd`. Weitere Pointerblöcke aus N folgen danach. Der Pointer auf das Serviceziel wird somit in den lokalen Frame kopiert. Das Zielobjekt wird hier nicht im Stackframe konstruiert.

Der nachfolgende Iterator `0x9b5490` aktualisiert die vorgelagerten Entity-/Komponentenbereiche. Ein ausdrücklicher Store auf die beiden oben verwendeten Serviceslots wurde in den untersuchten Systemkörpern nicht gefunden. Das allein beweist weder eine uneingeschränkte Lebensdauer noch Thread-Sicherheit.

## Benannte Listen in den Systemdeskriptoren

Die folgenden Deskriptoren besitzen bei `+0x98` einen Pointer und bei `+0xa0` eine Anzahl. Ihre Einträge sind **16 Byte große StringViews aus Namenspointer und Länge**. Das zweite Qword ist kein DI-Index. Beispielsweise ist `0x17` die Länge von `g38_items::ItemRegistry`.

| System | Deskriptor-RVA | Listen-RVA | Anzahl | Position von ItemRegistry |
| --- | --- | --- | --- | --- |
| actor_rotation | `0x1dc2080` | `0x1dc2040` | 4 | 1, nullbasiert |
| locomotion_execution | `0x1deada0` | `0x1deacc0` | 8 | 6, nullbasiert |

Die erste Liste lautet `ActionSequenceRegistry`, `g38_items::ItemRegistry`, `g38_skilltree::SkillTree`, `impact::AttributeRegistry`. Die zweite Liste lautet `ActionSequenceRegistry`, `BalancingRegistry`, `BasicPlayerInfos`, `TerraformingEfficiencyRegistry`, `ecs::ColliderSystem`, `ecs::VoxelGrid`, `g38_items::ItemRegistry`, `impact::AttributeRegistry`. Der ItemRegistry-Namensstring liegt bei RVA `0x150b7c8`.

Die Namensreihenfolge passt zu den beiden Serviceslots. Für actor_rotation ist der dazugehörige Blockanfang `0x120`, für locomotion_execution `0x188`. Dann ergibt sich `0x120+1*8=0x128` und `0x188+6*8=0x1b8`. Die vollständige Bindereihenfolge bis zum Pointerblock T ist im folgenden Abschnitt statisch belegt. Die konkreten N-Zähler und Instanzwerte wurden nicht live gelesen.

## Nachgewiesener Binder von Namen zu Pointerblock T

Gezielte Suchen nach den bekannten Registrar-Displacements in ausführbaren Sektionen ergaben neben dem Registrar `0x9d4b10` und einem Abbaupfad `0x976600` den Initialisierungspfad **`0x980280`**. Jeder Treffer wurde durch Disassemblierung ab der PE-Funktionsgrenze validiert.

`0x980280` iteriert Systemrecords mit Abstand `0x1f20`. Bei Ownerbasis W liegen sie ab `W+0x745a28`. Der jeweilige Systemrecord S enthält bei `S+0` den Deskriptorpointer D. Der Call bei **`0x980729`** übergibt an **`0x9a3300`**:

```text
RCX = S
RDX = W
R8  = &D[0x98]         // Span der 16-Byte-StringViews
R9  = u64[D+0xa8]      // pro Eintrag ausgewertete Bits
```

Ein weiterer Call bei `0x980757` bindet einen zweiten Namensspan ab `D+0xb0` mit Bits aus `D+0xc0` und hängt dessen Einträge an dieselben Pointerblöcke an. Die erste Liste enthält in beiden hier untersuchten Deskriptoren bereits die ItemRegistry.

Der Binder `0x9a3300` berechnet aus jedem vollständigen Namensbytebereich FNV-1a-32 mit Startwert `0x811c9dc5` und Multiplikator `0x1000193`, belegt bei `0x9a3353..0x9a3383`. Er sucht den Hash in einer Map an W über Belegungsbits, UInt32-Schlüssel und lineares Weitersuchen. Die Map liefert einen UInt16-Recordindex.

| Feld relativ zu W | Bedeutung im Binder |
| --- | --- |
| `+0x7459a8` | Pointer auf den Records-Vector |
| `+0x7459d8` | Belegungsbitsetpointer |
| `+0x7459e0` | Kapazität |
| `+0x7459f8` | UInt32-Schlüsselarraypointer |
| `+0x745a10` | UInt16-Wertearraypointer für Recordindices |
| `+0x745a24` | getesteter 32-Bit-Zähler |

Die entscheidenden Anweisungen sind `0x9a3497 movzx eax,word ptr [rcx]`, `0x9a349a imul r8,rax,0xe8` und `0x9a34a1 add r8,[W+0x7459a8]`. Anschließend kopiert `0x9a34a8..0x9a34af` den Originalpointer aus `Record+0xa8` nach **`S+0x1710+8*Ordinal`**. Der Append-Zähler ist ein Byte bei **`S+0x1b10`**, exakt `0x400` Bytes hinter dem Pointerblock.

Ein zweiter Block beginnt bei **`S+0x1b18`**, sein Countbyte liegt bei **`S+0x1f18`**. Dort landet normalerweise `Record+0xb0`. Bei gesetzter Flagmaske `0x04` von `Record+0xb8` wird stattdessen null eingetragen. Fehlende verpflichtende Einträge und der gesperrte alternative Fall setzen Statusbits bei `S+0x20`. Fehlende Namen werden als null angehängt. Die konkreten Status- und Optionalitätsbits wurden hier nicht zusätzlich fachlich benannt.

Die Pointer werden in Reihenfolge der Namenslisten angehängt. Der Binder führt im untersuchten Ablauf nach dem Hashvergleich keinen zusätzlichen Vergleich der Namensstrings aus. Eine unabhängige Auflösung sollte die gespeicherten vollständigen Recordnamen trotzdem gegenprüfen und Hashkollisionen nicht voraussetzungslos ausschließen.

### Anschluss an die Systemausführung

Zwei unmittelbar relevante Work-Kontextbauer **`0x9a1390`** und **`0x9a1a60`** lesen den Systemrecord S aus dem Eingabepaket bei `+0x10`. Sie erzeugen einen lokalen Kontext N bei `rsp+0x40`, gleich `rbp-0xc0`. Dessen Iterator I liegt bei `rsp+0x30` und enthält den Pointer auf N. Bei `0x9a198a` beziehungsweise `0x9a2066` wird mit RCX gleich I die Funktion aus `S+0x28` aufgerufen.

Damit entspricht `N = *(I+0)` exakt dem Eingang von `0x9ba5a0`. Die Serviceblockauswahl ist direkt belegt:

```text
T = S + 0x1b18, wenn byte[PaketElement+0xa1] == 1
T = S + 0x1710, sonst
N[0xc0] = T
```

Belegstellen sind **`0x9a1509..0x9a151d`** und **`0x9a1be2..0x9a1bf5`**. Der Store erfolgt nach `[rbp]`, somit nach `N+0xc0`. Das ist genau der Pointer T, dessen `+0x400`-Count `0x9ba5a0` liest und dessen Pointerarray der Frameinitialisierer kopiert.

Die Kette ist damit statisch verbunden: **Systemdeskriptor-Namen → FNV-Lookup → Recordpointer → geordnete System-Serviceblöcke → Work-Kontext `N+0xc0` → lokaler Frame → ItemInfo-Getter**. Hierfür benötigt die Originalengine weder eine Heap-Suche noch einen vom Toolkit eingebauten Hook.

### Offene Basisrelation zum separaten Registrar

Der separat untersuchte Registrar `0x9d4b10` verwendet bei seiner Eingabebasis R einen Records-Vector bei **`R+0x7462d0`** und die Map bei **`R+0x7462f8`**. Der Binder verwendet die strukturell entsprechenden Felder dagegen bei **`W+0x7459a8`** und **`W+0x7459d0`**. Die Displacements unterscheiden sich jeweils um **`0x928`**. Der Registrar übernimmt RCX bei `0x9d4b29` unverändert nach RDI.

Eine Gleichheit der tatsächlich adressierten Container würde `W = R+0x928` erfordern. Diese Owner-/Unterobjektrelation wurde in dieser Teiluntersuchung nicht bewiesen. Gleiche Recordlayouts, gleicher Hashalgorithmus und gleiche Namen reichen dafür allein nicht aus. Die Binderkette ist unabhängig davon innerhalb der Basis W direkt belegt. Der globale oder anderweitig unabhängig erreichbare Besitzer bleibt zusätzlich aufzulösen.

## Der Lookup selbst

`0xe7a250` erhält die UInt32-ID in ECX und den Servicepointer in RDX. ID null liefert null. Sonst ruft er `0xe6c570` mit `RCX = Service+0x38` und einem Pointer auf die ID auf. Bei einem Treffer dereferenziert er den zurückgegebenen Wertslot einmal und liefert dessen Pointer.

Der Maphelper verwendet relativ zum Mapanfang folgende Felder: Belegungsbitsetpointer bei `+8`, Kapazität bei `+0x10`, UInt32-Schlüsselarraypointer bei `+0x28`, Pointerwertearray bei `+0x40` und einen getesteten 32-Bit-Zähler bei `+0x54`. Der Startslot ist `ID & (Kapazität-1)`. Bei belegten ungleichen Schlüsseln sucht er linear mit Wraparound weiter. Der Rückgabeslot ist `Wertearray + Slot*8`.

Relativ zum Service liegen diese Mapfelder somit bei `+0x40`, `+0x48`, `+0x60`, `+0x78` und `+0x8c`. Dies sind für die untersuchte EXE belegte Layoutrelationen. Der Helper führt keine Heap- oder Signatursuche aus und verändert in diesem Ablauf weder Service noch ItemInfo.

## Grenze

Die geladene Ressource wird über eine echte registrierte ID-Map gesucht. Der Frame enthält dafür einen kopierten Pointer auf ein externes Objekt. Zusammen mit den benannten Systemlisten ist das ein konkreter Ansatz für die originale `g38_items::ItemRegistry`.

Der Join zwischen Namensliste und T ist innerhalb des gefundenen Binderpfads nachgewiesen. Noch offen sind die Basisrelation zum separaten Registrar, ein unabhängig erreichbarer Besitzer der Instanz sowie Lebensdauer und Synchronisierung während Reloads oder Weltwechseln. Aus dem Lookup ist keine Hooknotwendigkeit ableitbar. Ebenso ist hier noch keine funktionsfähige unabhängige Runtimeauflösung nachgewiesen.
