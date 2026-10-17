# Gleiterfelder in der aktuellen KFC4-Reflexion

Die vier Toolkit-Offsets passen vollständig auf die eingebettete `GliderConfig` in `keen::ItemInfo`. Der reflektierte Anfang dieses Teilobjekts liegt bei `ItemInfo + 0x104 + 0x3bc = ItemInfo + 0x4c0`. Das ist ein belegter Layouttreffer. Die Herkunft der Register `r12` und `rdi` aus genau diesem Objekttyp ist mit den hier ausgewerteten Reflexionsdaten allein noch nicht bewiesen.

## Quellen und Identität

- Discoverydump: `../enshrouded-client-25966634-kfc4/live-discovery/reflection.json`.
- Parserdump: `../enshrouded-client-25966634-kfc4/reflection_data.json`.
- Erfasste Toolkit-Konstruktorargumente: `patch-constructor-capture.json`.
- Bereits vorhandene Suche der Toolkit-Bytefolgen in der aktuellen EXE: `toolkit-signature-scan.json`.
- Toolkit-Zeichenketten: `strings.json` und `glider-label-xrefs.json`.

Discoverydump und Signatursuche beziehen sich auf `enshrouded.exe`, SHA256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`, PE-Zeitstempel `1792083981`, Imagegröße `44142592`. KFC-Format ist 4. Die Version lautet `1267041|^/game38/branches/release_1.0|2026-10-15T17:07:04.598331Z`.

Beide Reflexionsdumps enthalten 12.646 Typen. Für alle Typen wurden Index, qualifizierter Name, Größe, Primitivtyp, `innerType` sowie die geordnete Folge aus Feldname, Feldtypindex und Feldoffset normalisiert verglichen. Es bestehen **null Unterschiede in diesen Layoutdaten**. Attribute, Defaultbytes und weitere nicht genannte Metadaten gehören nicht zu diesem Vergleich. Leere oder ausgelassene `attributes`-Objekte erklären Unterschiede eines unverarbeiteten JSON-Vergleichs. Die Dumps sind zwei Auswertungen derselben EXE-Reflexion und keine unabhängigen Laufzeitbeweise.

## Exakter Einbettungspfad

| Typ | Typindex | Größe | Metadaten-RVA | Feld und Einbettung |
| --- | --- | --- | --- | --- |
| `keen::ItemInfo` | 4109 | `0x828` | `0x18bbc70` | `equipment` bei `0x104`, Typ `keen::EquipmentSetup` |
| `keen::EquipmentSetup` | 4073 | `0x440` | `0x1965370` | `gliderConfig` bei `0x3bc`, Typ `keen::GliderConfig` |
| `keen::GliderConfig` | 4042 | `0x20` | `0x192f8e0` | Acht `keen::float32`-Felder, jeweils vier Bytes |

`GliderConfig.innerType` verweist auf `keen::UniqueItemAttributeBase`, Index 4022, Größe `0x1`. Diese Basisklasse hat keine reflektierten Felder. Die expliziten `dataOffset`-Werte der Gleiterfelder beginnen bei null. Die Basisklassengröße wird deshalb nicht nochmals zu diesen Feldadressen addiert.

| Vollständiger Pfad ab `keen::ItemInfo` | Offset in `GliderConfig` | Gesamtoffset ab `ItemInfo` | Primitivtyp |
| --- | --- | --- | --- |
| `equipment.gliderConfig.accelerationForward` | `0x00` | `0x4c0` | `FLOAT32` |
| `equipment.gliderConfig.airResistanceLongitudinal` | `0x04` | `0x4c4` | `FLOAT32` |
| `equipment.gliderConfig.airResistanceLateral` | `0x08` | `0x4c8` | `FLOAT32` |
| `equipment.gliderConfig.airResistanceVertical` | `0x0c` | `0x4cc` | `FLOAT32` |
| `equipment.gliderConfig.yawAngleSpeed` | `0x10` | `0x4d0` | `FLOAT32` |
| `equipment.gliderConfig.pitchAngleSpeed` | `0x14` | `0x4d4` | `FLOAT32` |
| `equipment.gliderConfig.rollAngleSpeed` | `0x18` | `0x4d8` | `FLOAT32` |
| `equipment.gliderConfig.updraftStrengthChargeStartValue` | `0x1c` | `0x4dc` | `FLOAT32` |

Für alle acht Felder ist der reflektierte Typindex 45, `keen::float32`, Größe `0x4`. Das Teilobjekt wird durch die beiden `STRUCT`-Felder direkt eingebettet. Für diesen Pfad ist weder eine Arrayauflösung noch das Verfolgen eines Pointers erforderlich. `ItemInfo` und `EquipmentSetup` enthalten zusätzlich Blobfelder an anderen Stellen. Dies verändert die hier explizit reflektierten Inline-Offsets nicht.

## Zuordnung zu den vier Toolkit-Leseanweisungen

| Toolkit-Bezeichnung | Gefundene RVA in der aktuellen EXE | Ursprüngliche Bytefolge | Dekodierte Leseanweisung | Passender KFC4-Feldpfad bei `Register = ItemInfo*` |
| --- | --- | --- | --- | --- |
| Glider Move Speed | `0x42b6d0` | `F3 41 0F 10 BC 24 C0 04 00 00` | `movss xmm7, [r12+0x4c0]` | `equipment.gliderConfig.accelerationForward` |
| Glider Lift Speed | `0x42b8fb` | `F3 41 0F 10 94 24 C4 04 00 00` | `movss xmm2, [r12+0x4c4]` | `equipment.gliderConfig.airResistanceLongitudinal` |
| Glider Descend Speed | `0x42b886` | `F3 41 0F 10 94 24 CC 04 00 00` | `movss xmm2, [r12+0x4cc]` | `equipment.gliderConfig.airResistanceVertical` |
| Glider Turn Speed | `0x26ae36` | `F3 0F 10 97 D0 04 00 00` | `movss xmm2, [rdi+0x4d0]` | `equipment.gliderConfig.yawAngleSpeed` |

Die vorhandene Signatursuche meldet für jede dieser vier Bytefolgen genau einen Treffer in der oben identifizierten aktuellen EXE. Die Konstruktorerfassung ordnet die Originalbytefolgen den angegebenen Toolkit-Bezeichnungen zu. Die Toolkit-Zeichenketten enthalten zusätzlich die originalen Pfadnamen `equipment.gliderConfig.accelerationForward`, `airResistanceLongitudinal`, `airResistanceLateral`, `airResistanceVertical`, `yawAngleSpeed`, `pitchAngleSpeed`, `rollAngleSpeed` und `updraftStrengthChargeStartValue`, jeweils mit dem Präfix `equipment.gliderConfig.`. Diese Zeichenketten stützen die Feldzuordnung. Eine Zeichenkette beweist keine konkrete Registerherkunft am Hook.

Die Bezeichnungen des Toolkits beschreiben die Feldsemantik nur grob. **Lift Speed entspricht in diesem Layout `airResistanceLongitudinal`, also einem longitudinalen Luftwiderstandsparameter. Descend Speed entspricht `airResistanceVertical`, also einem vertikalen Luftwiderstandsparameter.** Move Speed adressiert eine Vorwärtsbeschleunigung. Aus dem Feldnamen allein folgen weder Maßeinheit noch Simulationsgleichung, Vorzeichen, zulässiger Wertebereich oder lineare Wirkung auf die tatsächlich erreichte Fluggeschwindigkeit.

## Abweichendes DS-Layout

| Typ | Typindex | Größe | Relevantes Feld |
| --- | --- | --- | --- |
| `keen::ds::ItemInfo` | 10355 | `0xaf0` | `equipment` bei `0x128` |
| `keen::ds::EquipmentSetup` | 10321 | `0x5b0` | `gliderConfig` bei `0x528` |
| `keen::ds::GliderConfig` | 10287 | `0x20` | Gleiche acht Feldnamen und lokale Offsets |

Im DS-ItemInfo beginnt die Gleiterkonfiguration bei **`0x128 + 0x528 = 0x650`**. Die entsprechenden absoluten Offsets lauten `0x650`, `0x654`, `0x658`, `0x65c`, `0x660`, `0x664`, `0x668` und `0x66c`. Die Offsets `0x4c0` bis `0x4d0` dürfen deshalb nicht auf einen DS-ItemInfo-Pointer übertragen werden. Im DS-EquipmentSetup beginnt bei `0x4c0` stattdessen `voxelSnappingConfig`, Typ `keen::ds::VoxelSnappingConfigOld`, Größe `0x1c`.

| Typfamilie | Gleicher `internalHash` | Verschiedene `qualifiedHash`-Werte |
| --- | --- | --- |
| `ItemInfo` und `ds::ItemInfo` | `0x3bb29fc4` | `0xb5ce8765` und `0x4b56a174` |
| `EquipmentSetup` und `ds::EquipmentSetup` | `0x83f09013` | `0xbc62befb` und `0xc7a57822` |
| `GliderConfig` und `ds::GliderConfig` | `0x7792aaff` | `0x2c91079d` und `0x748baae4` |

Ein identischer interner Hash beweist somit kein identisches Speicherlayout. Eine belastbare Zuordnung muss die qualifizierte Typidentität, Rolle und tatsächliche Layoutdaten berücksichtigen.

## Mehrdeutigkeiten durch andere reflektierte Strukturen

Alle 12.646 Typen wurden an den vier Zieloffsets über Inline-Structfelder, Vererbung, Typedefs und statische Arrayelemente verfolgt. DS-, Blob- und Objekt-Referenzgrenzen wurden nicht dereferenziert. Als Treffer galt ausschließlich ein exakt am Zieloffset beginnendes `FLOAT32`-Blatt. 96 Wurzeltypen besitzen an mindestens einem Zieloffset einen solchen Float. **65 Wurzeltypen besitzen an allen vier Offsets Floatblätter.** Dazu gehören große Animations- und Shaderarrays. Numerische Offsetgleichheit allein ist daher kein Typbeweis.

Auch semantisch zum Gleiter gehörende Kamerastrukturen ergeben weitere Treffer:

| Kandidat | `0x4c0` | `0x4c4` | `0x4cc` | `0x4d0` |
| --- | --- | --- | --- | --- |
| `keen::CameraModifierConfig` | `gliderSettings.swaySlowVerticalFrequency` | `gliderSettings.swaySlowVerticalAmplitude` | `gliderSettings.swayMediumHorizontalAmplitude` | `gliderSettings.swayMediumVerticalFrequency` |
| `keen::CameraStateConfig` | `config.gliderSettings.swaySlowHorizontalFrequency` | `config.gliderSettings.swaySlowHorizontalAmplitude` | `config.gliderSettings.swaySlowVerticalAmplitude` | `config.gliderSettings.swayMediumHorizontalFrequency` |

Die jeweiligen `keen::ds::`-Kameravarianten haben hier dieselben Offsets. `CameraStatesManager.cameraStates[1]` und die statischen Arraytypen um `CameraStateConfig` reproduzieren diese Kameratreffer ebenfalls. Die Kamerafelder heißen Sway-Frequenz beziehungsweise Sway-Amplitude. Der ItemInfo-Pfad passt zu allen vier Toolkit-Reglern und zu den originalen `equipment.gliderConfig.*`-Zeichenketten. Dennoch bleibt der Nachweis des konkreten Registerobjekts erforderlich.

Vorhandene ECS-Gleiterdaten sind beispielsweise `keen::ecs::ActorRotation.gliderRotationSpeed` bei `0x50` sowie `keen::ecs::DynamicLocomotion.gliderUpstreamFactor` bei `0x188`, `gliderUpstreamUpwardsFactor` bei `0x18c` und `gliderTurbulence` bei `0x190`. Diese Komponenten sind `0x68` beziehungsweise `0x1c8` Bytes groß. Keiner dieser beiden Typen kann bei einem Pointer auf seinen eigenen Anfang einen Zugriff auf `+0x4c0` enthalten. `ItemInfo` ist selbst ein Ressourcentyp und keine `keen::ecs::*`-Komponente.

## Was bewiesen und was noch offen ist

Bewiesen sind die ursprünglichen qualifizierten Typ- und Feldnamen, die dreistufige Inline-Einbettung, die lokalen und absoluten Offsets, der Float32-Speichertyp, die Übereinstimmung der benannten Layoutdaten in beiden Dumps und die vier bereits erfassten eindeutigen Bytefolgetreffer in der aktuellen EXE.

Noch offen ist der registerbezogene Datenfluss bis zu einer typgeprüften ItemInfo-Ressourcenauflösung. Für `r12` muss die Herkunft des Rückgabewerts des vorgelagerten Helpers geklärt werden. Für `rdi` gilt das gesondert an der Turn-Stelle. Ebenfalls nicht bewiesen sind die vollständigen Funktionssignaturen, Pointerlebensdauer, Thread- und Netzwerkauthorität, konkrete Simulationsgleichungen und Effekte im laufenden Spiel. In dieser Analyse wurde kein Hook installiert und kein Speicherwert verändert.

Für die weitere Auflösung sollte der Caller-/Helper-Datenfluss mit der ItemInfo-Ressourcen- und Typauflösung verbunden werden. Danach kann der geladene Ressourceninhalt mit dem reflektierten `equipment.gliderConfig`-Teilobjekt verglichen werden. Eine Anwendung sollte Felder über qualifizierte Typen und Feldpfade auflösen. Die hier berechneten Summen liefern Belege, aber keine universellen Konstanten für andere EXE- oder DS-Layouts.

## SHA256 der untersuchten Dateien

| Datei | SHA256 |
| --- | --- |
| `live-discovery/reflection.json` | `4a79d0266d89681aa8b9b60bcf6beee734478a38feef077944387e6ea071df9f` |
| `reflection_data.json` | `effa657bc3793a2455d13a7aebfa3b0ca9502c725e8154d6e1a86a1f07f4e137` |
| `patch-constructor-capture.json` | `27a53e958f42599638b3f27d4b9c51209ec4aeab0e607dba64706e3d978f47cb` |
| `toolkit-signature-scan.json` | `6944c6102fcb58bd259936f19ae3e0a54b8a95204438462a44de0c53b57d6116` |

## KI-Codex-Check

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ziel und Umfang | Vier Toolkit-Offsets mit beiden aktuellen KFC4-Reflexionsdumps abgeglichen. |
| Repository und Belege | Layoutdaten und vorhandene Toolkit-Artefakte gelesen. Registerprovenienz ausdrücklich offen. |
| Architektur und Struktur | Nur Bericht im ignorierten Devdata-Ordner ergänzt. |
| Konsistenz und Umfang | Ressourcen-, DS-, Kamera- und ECS-Layouts unterschieden. |
| Dateicheck | JSON-Dokumente strukturiert geladen und gezielt ausgewertet, keine vollständige semantische Prüfung aller Ressourcentypen behauptet. |
| Prüfung | Statische Datenanalyse und Layoutvergleich ausgeführt. Keine Tests oder Spielmodifikationen ausgeführt. |
| Offen / nicht geprüft | Registertypen, Lebensdauer, Authority, Gleichungen und Spieleffekte. |
