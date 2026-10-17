KFC4-Profilanalyse für Enshrouded

Steam-Build 25966634. KFC-Version 1267041, Format KFC4.

Aktuelle EXE SHA-256: `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`.
Alter Vergleichs-Capture SHA-256: `af2f5a1227911d8aa06b3908d6bd0211838211cae14ea91099cb57d0df990781`.

Die Auswertung kombiniert alte KFC3-Reflection und Code-Captures mit der frisch extrahierten KFC4-EXE und der aktuellen Live-Registry.

Alle 541 Laufzeittypen sind über ihre tatsächlichen Deskriptorverweise zugeordnet. Offene Typzuordnungen: 0. Die 640 Registrierungen enthalten zusätzlich 99 reine Vorlagen.
62 Registrierungen verwenden eigene Dynamic*-Laufzeittypen. Die früheren 55 Größenkonflikte und sieben scheinbar passenden Namen entstanden durch eine falsche manuelle TSV-Zwischenkonvertierung. Der strukturierte Export erhält Runtime- und Vorlagentyp getrennt.

Der native Produktionsleser bestätigt 640 Registrierungen und 53 Callback-Adressen. Der Codevergleich nutzt 10.425 gegenseitig eindeutige statische Vergleichsanker.
17 Profiloperationen haben einen bevorzugten Prozedurkandidaten. Für building.version besteht zusätzlich ein eindeutiger Byte-Treffer. Alle Kandidaten bleiben semantisch ungeprüft und callable=false.

Der unabhängige native Byte-Scanner bestätigt 56 erzeugte Einstiegssignaturen einschließlich Originalbytes und erwarteter Zieladresse. allFunctionCandidatesUnique=true gilt ausschließlich für diese Byteprüfung.

| Profiloperation | Status | Ziel-RVA |
| --- | --- | --- |
| `hook.game_thread` | Statische Zuordnung als Kandidat | `0x002D2DB0` |
| `hook.entity_manager` | Statische Zuordnung als Kandidat | `0x0027DF80` |
| `hook.world_prop_update` | Statische Zuordnung als Kandidat | `0x002D4B10` |
| `hook.world_actor_placement` | Statische Zuordnung als Kandidat | `0x002D4B10` |
| `hook.world_building_dispatch` | Bevorzugter Kandidat aus Aufrufgraph | `0x0048B4E0` |
| `hook.world_cursor` | Statische Zuordnung als Kandidat | `0x0028F760` |
| `runtime.world.context.active` | Offen | Offen |
| `runtime.world.entity.spawn` | Kandidaten ohne eindeutige Zuordnung | Offen |
| `runtime.world.entity.place` | Bevorzugter Kandidat aus Codestruktur | `0x0048FAC0` |
| `runtime.world.entity.destroy` | Bevorzugter Kandidat aus Codestruktur | `0x0048FC00` |
| `runtime.world.entity.finish_building` | Bevorzugter Kandidat aus Aufrufgraph | `0x0048B4E0` |
| `runtime.world.voxel.read` | Bevorzugter Kandidat aus Aufrufgraph | `0x01094EE0` |
| `runtime.world.voxel.write` | Bevorzugter Kandidat aus Codestruktur | `0x0109DC50` |
| `runtime.world.building.input` | Statische Zuordnung als Kandidat | `0x0028F760` |
| `runtime.world.building.version` | Eindeutiger Byte-Treffer ohne Funktionsumfang | `0x009B0830` |
| `runtime.patch.infinite_item_split` | Bevorzugter Kandidat aus Aufrufgraph | `0x00411990` |
| `runtime.patch.infinite_item_use` | Mehrdeutig | Offen |
| `runtime.patch.no_fall_damage` | Statische Zuordnung als Kandidat | `0x00273D60` |
| `runtime.patch.no_resource_cost` | Bevorzugter Kandidat aus Aufrufgraph | `0x00409AC0` |
| `runtime.patch.auto_stamina_refill` | Statische Zuordnung als Kandidat | `0x00275C10` |
| `runtime.patch.unlimited_flight` | Statische Zuordnung als Kandidat | `0x00269D30` |

Der frühere Schluss aus consistentSnapshot=false auf eine fehlende geladene Welt war unbelegt. Das Feld kennzeichnet einen nicht atomaren Snapshot des laufenden Prozesses.

Die verbleibenden offenen Fälle sind globaler Weltkontext, entity.spawn und infinite_item_use. Für die beiden Funktionen liegen gerankte Kandidaten vor. Der Kontext ist eine globale Daten-/Zeigerstruktur und benötigt eine separate Live-Zuordnung.

Gleiche Codefragmente und Aufrufbeziehungen beweisen keine nativen Argumenttypen, Thread-Anforderungen oder Spielwirkung. Die neuen Einstiegssignaturen bestimmen keine Hook-Einfügestelle und keinen Patch-Offset. Die alten Captures enthalten direkte CALL/JMP, aber keine vollständigen Instruktionen und keine bedingten oder indirekten Kontrollflusskanten. Eine alte Client-EXE liegt im Workspace nicht vor.

Der Profilentwurf bleibt `draft-requires-live-review`. Seine strukturelle Prüfung besteht. Die Profilprüfung meldet erwartungsgemäß nur einen Entwurf. Er wird nicht als Produktionsprofil installiert.

Zusätzliche Hashauflösung mit der alten Reflection:

| Alter KFC3-Hash | Eindeutiger Typ | Größe |
| --- | --- | ---: |
| `0x47a54aff` | `keen::VfxMeshData` | 40 |
| `0x8bea0afc` | `keen::ColorPaletteCollectionResource` | 8 |

Diese Typnamen sind im aktuellen KFC4-Schema entfernt. Die alte Reflection löst ihre Identität für das KFC3-Paket auf. Ressourceninhalte wurden dabei nicht neu exportiert.

Prüfung: 14 vorhandene Python-Tests bestanden. Native Registry-Prüfung einschließlich negativer Prüfungen an lokalen Kopien bestanden. 56 erzeugte Einstiegssignaturen unabhängig im nativen Scanner eindeutig gefunden. JSON-Syntax und git diff --check bestanden.

Dateien:

- `cross-build-with-signatures.json` · Funktionsvergleich mit Signaturen
- `components-runtime.json` · Komponenten mit Runtime-/Template-Belegen
- `enshrouded-client-25966634-kfc4.profile-draft.json` · Profilentwurf
- `candidate-entry-byte-scan.json` · Unabhängige Byteprüfung
- `live-discovery/runtime-bindings.json` · Vollständiger Runtime-Katalog
- `kfc3-resource-hash-resolution.json` · Alte Ressourcenhashes
