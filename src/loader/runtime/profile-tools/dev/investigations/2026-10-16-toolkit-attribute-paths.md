# Toolkit-Anker bis zu originalen Attributfeldern

Stand: 16. Oktober 2026. Aktuelles Client-Image mit SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Untersuchung der EXE als Datei, bereits erfasster Toolkit-Registrierungen und Reflection. Kein Toolkitcode wurde geladen und kein neuer Spielspeicherzugriff geschrieben.

## Ergebnisübersicht

| Toolkitfunktion | Aktueller Anker | Originaler Besitzer und veränderte Daten | Übernahme in die typisierte API |
| --- | --- | --- | --- |
| Infinite Stamina | `0x2762ab` | `network_player_attributes`, `Stamina`-Root, Current bekommt Maximum | `update_attribute(entity, "Stamina", maximum)` erreicht dieselbe originale Root. Periodischer Aufruf besitzt einen anderen Ausführungszeitpunkt als der Toolkitpatch. |
| Fast Health Regen | `0x2364fc`, Eingriff bei `0x236521` | `health_recharge` bei `0x235eb0`, addiert einen geänderten Delta auf `Health` und berechnet die Root neu | `Health`, `Health_Recharge` und `HealthRechargeMod` sind typisiert zugänglich. Ein direkter Refill ist keine identische Änderung des originalen Regenereignisses. |
| Fast Mana Regen | `0x237a7c`, Eingriff bei `0x237a83` | `mana_regeneration` bei `0x2373f0`, addiert einen geänderten Delta auf `Mana` und berechnet die Root neu | `Mana`, `Mana_Recharge` und `ManaRechargeMod` sind typisiert zugänglich. Genaue Änderung der Regenrate verlangt Prüfung der zugehörigen Rampen-/Timeoutberechnung. |
| Infinite Mana | `0x237654` | Liest `Mana`-Current bei `0x23765a`, Toolkit schreibt anschließend die Konstante 100 an dieselbe Speicherstelle | `update_attribute(entity, "Mana", value)` erreicht den Originalwert mit vollständiger Rootberechnung. Refill auf Maximum ist nicht identisch zum Toolkitwert 100. |
| No Enemy Damage | `0x21be03` und `0x3e7df3` | Zwei Codekopien des generischen Signed-Attributadders. Toolkit merkt sich die Adresse des letzten nichtnegativen Deltas und überspringt negative Deltas an dieser Adresse | Kein belegter einzelner „No Enemy Damage“-Attributwert. Ein Health-Refill ersetzt diese Ereignissperre nicht. |
| Damage Multiplier | dieselben beiden Treffer | Derselbe Signed-Adder. Toolkit merkt sich zwei Adressen mit nichtnegativen Deltas und skaliert negative Deltas anderer Adressen | Kein Nachweis, dass dies dem Attribut `DamageMod_Multiplier` entspricht. Beide Mechanismen müssen anhand ihrer tatsächlichen Konsumenten unterschieden werden. |

## Bekannter eigener Stamina-Patch und aktueller Originalpfad

Das alte Clientprofil `1076226.json` enthält `runtime.patch.auto_stamina_refill`. Der Patch liegt im ursprünglichen `network_player_attributes`-Pfad. Seine Nutzlast liest den dword acht Bytes hinter dem aktuellen Attributwert und schreibt ihn an dessen Adresse. Die deklarierte Wirkung lautet entsprechend Maximum nach Current kopieren. Dies ist ein vorhandener Patchbeleg. Für diese Fortsetzung wurde die alte EXE nicht erneut geladen.

Im aktuellen Image liefert die Toolkit-Signatur genau `0x2762ab`. Der umgebende Originalcode belegt die Beziehung unabhängig von der Funktionsbezeichnung:

1. `0x2761ff` lädt die originale Attribut-ID `0x04b6aa8b`. Der Resolver `0x121b2a0` liefert ihre Beschreibung. `0x27621f..0x276223` vergleicht die Root-ID der realen Komponente mit der Beschreibung.
2. `0x27622c` liest den originalen Attributindex, `0x276230` den reflektierten `storageOffset`. `0x27623c` liest daraus `Stamina`, Index 0.
3. `0x27626e` lädt `0xf443c410`, die Parserdefinition `Stamina_Max`. `0x27629b..0x2762ab` liest ihren Index 2 über dieselbe Root und denselben Storagebeginn.
4. Die Toolkit-Cave liest den Wert an dieser Maximumadresse und schreibt ihn acht Bytes davor. Bei vier Bytes je Scalar sind das exakt zwei Indizes Abstand, also Current bei Index 0.
5. `0x2768dc` schreibt Maximum nach `NetworkPlayerAttributes+0x8e`, `0x2768f4` Current nach `+0x8c`. Beide werden dabei auf den Bereich 0 bis 65.535 begrenzt.

Die aktuelle Reflection benennt diese beiden Zielfelder als `staminaMax` bei Offset 142 und `stamina` bei Offset 140, jeweils `keen::uint16`. Der Ausgangspointer wird bei `0x276812` aus dem ursprünglichen Iteratorfeld geladen. Die Enginebeschreibung bei `0x1dc69c0` deklariert `Stamina`, `Health` und `Mana` als Runtime-Leseabhängigkeiten und `NetworkPlayerAttributes` als Runtime-Schreibabhängigkeit.

Damit ist der Übertragungsweg vom echten Attributspeicher in den originalen Netzwerksnapshot belegt. Eine tatsächliche Lua-Änderung muss weiterhin mit unabhängigem Lesen beider Originalspeicher und Apply-/Restore-Prüfung bestätigt werden. Externe Veröffentlichung, Netzwerkautorität und UI-Aktualisierung werden nicht allein aus der Abhängigkeitsbeschreibung zugesichert.

## Rootstruktur und tatsächliche Berechnung

Die originale `keen::ecs::Stamina`-Komponente ist 40 Bytes groß. Der reflektierte Header enthält `rootId` bei 0, `storageOffset` bei 4, `flags` bei 6 und `storageCount` bei 7. Die acht Signed-Scalars beginnen bei Offset 8. Die Zuordnung wird je Entity durch Root-ID und Storageform bestätigt.

| Index | Parsername | Bedeutung im Berechnungsprogramm |
| --- | --- | --- |
| 0 | `Stamina` | Current, am Ende auf Minimum und Maximum begrenzt |
| 1 | `Stamina_Min` | Untere Grenze |
| 2 | `Stamina_Max` | `Max_Base + Max_Adder`, inklusive originaler Skalierung von Current |
| 3 | `Stamina_RechargeRampup` | Gespeicherter Eingang |
| 4 | `Stamina_RechargeTimeout` | Gespeicherter Eingang |
| 5 | `Stamina_Recharge` | Auf 0 bis 65.536 begrenzter Eingang |
| 6 | `Stamina_Max_Base` | Gespeicherter Eingang für Maximum |
| 7 | `Stamina_Max_Adder` | Gespeicherter Eingang für Maximum |

Die Programme stammen aus dem Parser. Maximum verwendet `LoadRef(Current)`, `Load(Min)`, `Load(oldMax)`, `Load(Base)`, `Load(Adder)`, `Add`, `ScaleToNewMax`. Der originale Signed-Recompute bei `0x2315c0` läuft die Attributdefinitionen absteigend durch und ruft bei `0x23167f` den Interpreter `0x21c510` auf. `0x231696` schreibt das jeweilige Ergebnis in den ausgewählten Storageindex. Die eigene API verwendet dieselben Parserprogramme und dieselbe Reihenfolge auf einem vollständigen Snapshot. Das vorhandene Modell wurde mit 4.720 Originalemulationsvektoren verglichen.

Für einen reinen Stamina-Refill bleibt Maximum unverändert. Der API-Aufruf setzt Current und führt das gesamte Rootprogramm aus. Für eine Maximumänderung reicht das Schreiben des isolierten Maximumwerts nicht, weil dessen eigenes Programm Base und Adder auswertet und Current anteilig anpasst. Der bestehende kontrollierte Schreibtest verändert deshalb `Stamina_Max_Adder` und stellt denselben Eingang wieder her.

## Gemeinsamer Signed-Adder und Grenzen der Schadensfunktionen

Die beiden Treffer `0x21be03` und `0x3e7df3` gehören zu den Originaleintritten `0x21bda0` und `0x3e7d90`. Beide erhalten Komponentenpointer, Attributdefinitionskontext, Signed-Delta und Attribut-ID. Der Resolver bestimmt den Index, die Komponente liefert `storageOffset`, der Delta wird an genau diesen Scalar addiert. Anschließend werden die Rootprogramme absteigend über denselben Signed-Interpreter ausgeführt.

Der Originalcode enthält an diesen Stellen keinen festen `Health`-Typ, keine Feindprüfung und keinen festen Schadensmultiplikator. Die Toolkit-Caves ergänzen eine Adresshistorie und behandeln positive sowie negative Deltas unterschiedlich. Die Mehrdeutigkeit der Signatur bleibt deshalb als zwei belegte Codekopien sichtbar. Ein willkürlich ausgewählter Treffer wäre kein vollständiger Nachweis.

Das erklärt auch, weshalb ein weiterer pauschaler Hash- oder Funktionsadapter keine fertige fachliche API wäre. Für `Health`, `Stamina` und `Mana` ist die Rootwirkung nun konkret beschrieben. Eine API für eingehende oder ausgehende Schadensereignisse braucht zusätzlich den tatsächlichen Aufrufer, dessen Zielentity und die Auswahl des betreffenden Attributs. Die bestehenden Attribute `DamageMod_*` und `DamageSusceptibility_*` können erst dann als gleichwertige Steuerung bezeichnet werden, wenn ihr Originalkonsument auf dieser Wirkungskette belegt ist.

## Belege

- `devdata/embervale-toolkit-3.1.0/complete-registration-catalog.json` enthält die original erfassten Toolkit-Registrierungen und Cavebytes.
- `devdata/embervale-toolkit-3.1.0/toolkit-signature-scan.json` enthält alle Treffer im aktuellen exakten Client-Image.
- `build/stamina-network-current-disassembly.txt` enthält den aktuellen vollständigen Originalpfad einschließlich ausgelagerter Unwind-Fragmente.
- `build/toolkit-attribute-anchor-disassembly.txt` enthält die Regen-, Mana- und Signed-Adder-Anker.
- `build/attribute-native-recalculate-disassembly.txt` enthält den originalen absteigenden Signed-Recompute.
- `target/enshrouded-attribute-resolution.json` enthält Parserdefinitionen, Programme und reflektierte Storageform.
- `devdata/creative-mode-146/current-attribute-vm-summary-20261016.json` dokumentiert den Originalemulationsvergleich.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ausgangspunkt | Vorhandene eigene Patchbytes und tatsächlich erfasste Toolkit-Anker. |
| Datenfluss | Current, Maximum, Originalberechnung und Netzwerksnapshot anhand gültiger Instruktionen und Reflection verfolgt. |
| Architektur | Bestehende typisierte Attribut-API, kein neuer Attributkatalog und keine zusätzliche Runtime. |
| Prüfung | Statische Originalanalyse und bestehender Originalemulationsvergleich. Native Kernprüfung zuletzt 89 erfolgreiche ECS-Fälle. |
| Offen | Livewirkung des neuen Lua-Schreibpfads, unveränderter Ereigniszeitpunkt gegenüber Toolkit und genaue Schadensereignisscope. |
