# Creative Mode 146, echte Engine-UI und geladene KFC4-Ressourcen

## Ergebnis

Die untersuchte Creative-Mod verwendet originale UI-Funktionen des Spiels. Buttons und Slider werden über eigene Parameterstrukturen an die Engine übergeben. Der statisch verfolgte HUD-Callback erreicht diese Widgets tatsächlich. Ihre nahtlose Darstellung ist damit erklärt.

Die Mod enthält zugleich Änderungen an ausführbarem Spielcode. Sie installiert unter anderem Hooks für HUD, Eingaben und Crafting. Ihre Buildtabellen enthalten feste Funktionsadressen und Strukturwerte. Die Mod beweist daher keine bereits vorhandene universelle Auflösung aller Enginefunktionen ohne Codeeingriff. Sie liefert konkrete zusätzliche Aufruf- und Datenverträge für die weitere Untersuchung.

Unabhängig davon konnte die laufende KFC4-ItemRegistry über eine aus dem Originalspiel hergeleitete Besitzkette erreicht werden. Alle **4.207 belegten Einträge** wurden lesend geprüft. Ihre gespeicherte Item-ID stimmt jeweils mit dem Map-Schlüssel überein. Bei 3.498 auch im älteren Creative-Katalog enthaltenen IDs stimmen zusätzlich sämtliche GUIDs überein. Vier geladene Gleiter wurden einschließlich ihrer acht reflektierten Gleiterparameter gelesen. Das ist ein Laufzeitnachweis für den Ressourcen-Lesezugriff. Eine sichere Datenänderung, ihre tatsächliche Spielwirkung und ein allgemeiner Engine-Ausführungspfad ohne Hooks sind damit noch nicht bewiesen.

## Dateien und Identität

Untersucht wurde der vom Nutzer benannte Ordner:

```text
A:\17 Cloud\OneDrive\Downloads\Creative Mode 146 1.2 2026-10-13T10-28Z U89dZfAAM
```

| Datei | Größe | SHA256 |
| --- | ---: | --- |
| `dinput8.dll` | 1.683.968 Bytes | `0912baf25dc15e0eac9a571893ca9a7ee35bbe549d7d9d4df97dfdf36c8558d5` |
| `creative/creative.dll` | 4.651.520 Bytes | `fe901f31fc61128e9e3e7b59c74366c4ef3cd6a5b0a7e95837f169bca28ff592` |
| `creative/items.json` | 2.545.263 Bytes | `b1a9e83242827c1d6e3c8d7e29bfeb5016f4c2e28f67c55b7e3f402ea453f09b` |
| `creative/sigs/6A4236C8-2DA7000.json` | 18.474 Bytes | `8c783d13caf0061de5a6f9007ad0a66eac0ea5cedec297a1f0e16d525a524718` |
| `creative/sigs/69FDECC9-1DA7000.json` | 14.504 Bytes | `741e75b9a7cb2ea8d10cffb331b165818ee6019ce7c2626130a68df5a1e26934` |

Der Clientkatalog nennt Steam-Build **23966345**, 147 Symbole und 295 Layoutwerte. Der Serverkatalog nennt Build **23178631**, 40 Symbole und 297 Layoutwerte. Die Symbolwerte sind RVAs. Die Dateinamen verwenden PE-Zeitstempel und Imagegröße. Das sind keine dynamischen Funktionssignaturen oder Typbeschreibungen.

Der aktuell untersuchte Client ist Build **25966634**, EXE-SHA256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`, Zeitstempel `0x6ad1080d`, Imagegröße `0x2a19000`. Seine entsprechende Tabellendatei wäre `6AD1080D-2A19000.json`. Sie ist nicht mitgeliefert. Die alten RVAs werden deshalb nicht für das aktuelle Spiel übernommen.

`items.json` enthält 3.520 katalogisierte Items, sechs zusätzliche Itemdefinitionen und 24 Gruppen. Dieser mitgelieferte Katalog ist keine vollständige Bestandsaufnahme des aktuellen KFC4-Spiels.

## Echte UI-Funktionen und ihre Aufrufer

Der Binder bei DLL-RVA `0x1080b0` löst für seine UI-Schnittstelle 70 Symboladressen und 119 Layoutwerte auf. Bei Erfolg kopiert er ein `0x7c0` Byte großes Bindingobjekt und setzt dessen Verfügbarkeitsflag. Die gemeinsamen Lookupfunktionen liegen bei `0x9c250` und `0xa00c0`.

| Implementierung | Konkreter Aufruf |
| --- | --- |
| Buttonwrapper `0x106df0` | `0x106f34` ruft die gebundene Enginefunktion `gameui.doButton` aus `API+0x3a8` auf. |
| Sliderwrapper `0x10e2d0` | `0x10e538` ruft `gameui.settingSlider` aus `API+0x728` auf. |
| Craftwrapper `0x108000` | `0x108042` ruft `gameui.queueCraft` aus `API+0x540` auf. Seine Erreichbarkeit im tatsächlichen Remote-Requestpfad bleibt offen. |

Die Widgetparameter werden anhand der Buildoffsets aufgebaut. Der Slider verwendet außerdem den aktuellen Engine-UI-Kontext und behandelt dessen Eingabesynchronisierung. Die Engine stellt Darstellung, Stile und Widgetverarbeitung bereit.

Die Erreichbarkeit von Button und Slider wurde über den tatsächlichen Callbackaufbau verfolgt:

```text
HUD-Callback 0x12b000
  → 0x12a790
  → std::function mit Operator 0x1294b0
  → Rootwidgetbuilder 0x300dc0, Callbackaufruf bei 0x300ff4
  → CreativeScreen
  → Buttonwrapper beziehungsweise Sliderwrapper
  → gebundene Engine-UI-Funktion
```

Der HUD-Einstieg wird mit SafetyHook verändert. Die Installation führt von `0x127080` über die Hookkonstruktion bis zur Aktivierung bei `0x1a9d20`. Die Schreibpfade unterstützen sowohl einen relativen `E9`-Sprung als auch einen indirekten `FF25`-Sprung. Welche Variante für ein konkretes Spielziel gewählt würde, wurde nicht durch Ausführen der Mod geprüft. Beide Pfade ändern ausführbaren Code.

Die eigenen `GameThread`- und Eingabeintegrationen besitzen ebenfalls Hookpfade. Ein selbst benannter API- oder Queue-Typ innerhalb der Mod ist deshalb kein Nachweis eines öffentlich registrierbaren originalen Engine-Schedulers.

## Events, Inventar und Kosten

Der ItemGiver bindet originale ECS-Funktionen für Querybeginn, Iteration, Entityzuordnung, Eventpartition, Eventqueue und Eventanlage. Zusätzlich bindet er Konstruktion, Setup, Itemeinfügung, Commit, Rollback und Destruktion von Inventartransaktionen.

Der serverseitig verfolgte Ablauf lautet:

```text
Hook des Crafting-Systems
  → Originalsystem und eigene Requestverarbeitung
  → originale ECS-Query und CraftRecipeEvent-Anlage
  → Hook des Craft-and-Commit-Pfads
  → eigenes Protokoll erkennen
  → originale Inventartransaktion ausführen
```

Die Mod nutzt ein 32 Byte großes Protokoll mit Kennung `0x014d4345`. Der in ihrer Konfiguration als `spare` bezeichnete Bereich bei `CraftRecipeEvent+0x18` heißt in der aktuellen KFC4-Reflection tatsächlich **`inputCategorySelections`**. Das ist ein vorhandenes fachliches Eventfeld. Seine Umnutzung setzt den eigenen empfangenden Hook voraus. Das Verfahren ist daher kein nachgewiesener unveränderter Engine-RPC zum freien Erzeugen beliebiger Items.

Die aktuelle Reflection bestätigt `keen::ecs::CraftRecipeEvent`, qualifizierter Hash `0xa4766fb7`, Größe `0x38`, mit Operator bei `+8`, Station bei `+0xc`, Rezept bei `+0x10`, Menge bei `+0x14` und Kategorien bei `+0x18`. Gleiche Eventlayouts beweisen weder gleiche Aufrufadressen noch identische Querykontexte.

Der Kostenpfad liefert zusätzliche Semantik:

- Der Recipe-Callback `0x1027c0` setzt für den Originalaufruf vorübergehend ein Flag in der Transaktion und stellt den ursprünglichen Wert danach wieder her. Die mitgelieferte Tabelle bezeichnet den Offset `0xb0` und Flagwert `8` als `transaction.flags` und `transactionFlags.free`.
- Der Building-Callback `0x101cf0` kann einen Erfolg zurückgeben und den ursprünglichen Bezahlpfad überspringen.
- Der RemoveItems-Callback `0x102910` unterscheidet bestimmte Consumable-Aufrufstellen anhand ihrer Rücksprungadressen und kann deren Entfernung unterdrücken.

Diese unterschiedlichen Mechanismen dürfen nicht als eine generische, bereits aufgelöste Ressourcenoption zusammengefasst werden. Das Transaktionsflag ist ein Ansatz für reguläre eigene Transaktionen. Sein Einsatz erfordert weiterhin den richtigen Transaktionsbesitzer, gültige Argumente und eine zulässige Enginephase.

## Nachweis im laufenden KFC4-Spiel

Die Besitzkette wurde aus Konstruktoren, Stores, Vtables und dem originalen Singletongetter des aktuellen Spiels hergeleitet. Die erste Szene-Variante lieferte zur Messzeit keine Einträge. Eine zunächst unvollständige alternative Kette wurde verworfen. Die erfolgreiche Kette enthält den gesonderten Controller:

```text
EXE + 0x1fe1cb0, direktes Singletonobjekt
  → [ + 0x258 ] ClientOwner
  → [ + 0x35280 ] Controller
  → [ + 0x1b8 ] MainOwner
  → [ + 0x47d4e0 ] aktiver Kontext
  → [ + 0x3e10 ] g38_items::ItemRegistry
  → originale ID-Map bei +0x38
  → geladenes ItemInfo
```

Singleton-, Client- und Controller-Vtable sowie der Controller-Backpointer wurden vor der Map-Auswertung geprüft. Der Registrykonstruktor verbindet seinen Wrapper mit `keen::ItemRegistryResource`, dessen aktueller qualifizierter Hash `0x72ab39b6` ist. Der originale Itemgetter `0xe7a250` verwendet dieselbe Map. Die zuvor untersuchten Gleitersysteme benutzen diesen Getter und lesen anschließend die reflektierten `ItemInfo.equipment.gliderConfig`-Felder.

| Prüfung am Client PID 33644 | Ergebnis |
| --- | --- |
| Mapkapazität | 16.384 Slots |
| Gemeldete und tatsächlich belegte Einträge | Jeweils 4.207 |
| Gespeicherte Item-ID gegen Map-Schlüssel | Alle 4.207 stimmen überein |
| GUID-Vergleich mit vorhandenen IDs des älteren Creative-Katalogs | Alle 3.498 stimmen überein |
| Gelesene Gleiter | T1, T2, T3 und T4-Rewardsatz |
| Identitätskonflikte | Keine |
| Wiederholter Vergleich | Besitzkette, Mapheader, Belegungsbits, Schlüssel- und Pointerarrays unverändert |
| Umfang des vollständigen Map-Abgleichs | 481.768 Bytes, 4.231 gezielte Leseaufrufe |

Beispielsweise besitzt T3 im gelesenen Datensatz `accelerationForward=1`, longitudinalen Widerstand ungefähr `0,215`, vertikalen Widerstand ungefähr `1,6` und `yawAngleSpeed=60`. Das sind die gelesenen Ressourcenwerte. Sie sind kein gemessener Flugversuch und kein Nachweis, dass der Nutzer diesen Gleiter gerade ausgerüstet hatte.

Die Leseprüfung verwendete ausschließlich den bestehenden `discovery.live.Process` mit lesenden Prozessrechten. Sie installierte keine Hooks, führte keine Enginefunktion aus und schrieb keine Spielwerte. Im verglichenen Spiel war bereits das vom Nutzer installierte Embervale-Toolkit geladen. Dessen vorhandene Eingriffe werden nicht als unveränderter Ausgangszustand des Spiels ausgegeben.

### Verbindung zum allgemeinen ResourceService

Die originale Lademethode `0xea8720` bezieht `keen::ItemRegistryResource` über den ResourceService und speichert den Datenpointer im Registry-Wrapper bei `+0x20`. Sie löst die darin enthaltenen Item-GUIDs mit dem Typwert `0xb5ce8765`, also `keen::ItemInfo`, ebenfalls über den ResourceService auf. Anschließend trägt sie genau die zurückgegebenen Pointer in die Item-ID-Map ein. Sie erzeugt an dieser Stelle keine gesonderten ItemInfo-Kopien.

Auch die generische Handleauflösung wurde gezielt lesend abgeglichen:

| Prüfung | Ergebnis |
| --- | --- |
| ResourceService-Auflösungscallback | Pointer entspricht EXE-RVA `0x10629c0` |
| Handle der geladenen ItemRegistryResource | 80.640 |
| Einträge im zugehörigen Index | 91.812, keine Aussage über die Anzahl aktiver geladener Ressourcen |
| Ermittelter Ressourcenumfang | 247.432 Bytes |
| Ermittelter Datenpointer | Exakt identisch mit dem vom originalen Ladepfad gehaltenen Pointer bei Registry `+0x20` |
| Item-Referenzen im geladenen Ressourcenheader | 4.207 |
| Zusätzlich gelesener Umfang | 180 Bytes |

Die Auflösung wurde durch Lesen der ursprünglichen Indextabelle und Chunkzuordnung nachvollzogen. Der Callback wurde nicht aufgerufen. Indexbesitzer, Eintrag und Chunkbasis wurden erneut verglichen. Das belegt für diese Ressource die Verbindung zwischen Engine-Handle und den tatsächlich geladenen Ressourcenbytes. Eine allgemeine schreibbare API entsteht daraus noch nicht.

Der originale Schlüssel dieses Eintrags wurde ebenfalls gelesen. Er besteht aus GUID, Typwert `0x72ab39b6`, Teilindex null und acht reservierten Nullbytes. Das entspricht exakt dem bereits vorhandenen 32 Byte großen `ResourceId` in `kfc-parser/crates/kfc-base/src/guid/resource.rs`. Auch dessen `StaticHash` stimmt mit der rekonstruierten originalen Hashbildung überein. Für die Liveauflösung ist damit derselbe kanonische Schlüssel wie im Parser verwendbar. Ein zusätzliches Hashprotokoll ist nicht erforderlich.

Abschließend wurde die vollständige Vorwärtsauflösung geprüft. Aus GUID, Typ und Teilindex wurde jeweils der originale StaticMap-Bucket bestimmt, der vollständige Schlüssel verglichen, der Handle ermittelt und anschließend der geladene Datenbereich über die Chunkzuordnung gelesen. Für ItemRegistryResource und alle vier Gleiter lieferte dieser generische Weg jeweils exakt den schon über die Item-ID-Map bestätigten Pointer. Dabei wurden Schlüssel-, Handle- und Chunkgrenzen sowie Ressourcenlängen geprüft. Dieser zusätzliche Abgleich benötigte 796 Bytes in 57 Leseaufrufen und rief keine Enginefunktion auf. Die dazu verwendete Besitzkette bleibt auf die untersuchte EXE bezogen.

## Umsetzung der read-only Ressourcen-API

Die vorhandene Ressourcen-API hat jetzt ein explizites Runtime-Backend. In der ingame Phase rufen Mods `game.assets.get_loaded_resource(guid, type, part)` oder `game.assets.get_loaded_resources_by_type(type)` auf. Die native Seite validiert das exakte Client-Image und die ClientOwner/Controller/MainOwner/ItemRegistry-Kette. Sie prüft den Registry-Handle erneut gegen den tatsächlich geladenen ResourceService-Eintrag und liest danach Schlüssel, Chunk und Bytebereich. Die Rust-Seite bildet den Schlüssel mit `ResourceId::static_hash()` aus dem eingebundenen KFC-Parser.

Die zurückgegebenen Werte verwenden dieselbe KFC4-Reflection wie Dateiressourcen. `resource.source` und `resource.loaded` unterscheiden die Runtimekopie von `kfc-file`. Runtimewerte sind schreibgeschützt. Der Setter weist `resource.data = ...` zurück und `ResourceInfo::apply` gibt für Runtimewerte niemals eine Dateiänderung aus. Eine neue Listenabfrage erstellt neue Resource-Objekte. Jedes Objekt liest und dekodiert seine Daten beim ersten Zugriff auf `data` und hält danach diesen Snapshot.

Der neue Profil-Eintrag `kfc-runtime/profiles/enshrouded/client/25966634-resource-read.json` ist `read-validated`. Er wird separat von Ausführungsprofilen eingebettet und erlaubt nur diesen lesenden Pfad für das EXE mit SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Die ResourceService-Diagnose weist `resourceEngineCalls=false`, `resourceWrites=false` und `resourceHooksInstalled=false` aus. Das beschreibt ausschließlich dieses Backend. Bestehende ECS- und World-Runtimepfade werden separat initialisiert. Es gibt weiterhin kein KFC4-Ausführungsprofil.

Der World Editor nutzt nun zuerst `get_loaded_resources_by_type("keen::ItemInfo")`, um Platzierungsrezepte aus den tatsächlich geladenen ItemInfo-Werten zu bilden. Ist die Runtime-API nicht verfügbar, verwendet er die KFC-Dateiressourcen als Fallback und protokolliert diese Quelle. Die World-Editor-Aktionen für Cursor, ECS, Voxel und Entities bleiben eigenständige Profiloperationen mit ihren bisherigen Runtimekontexten.

### Prüfung der neun vorhandenen Mods

| Modgruppe | Anzahl | Ergebnis der Quellprüfung |
| --- | ---: | --- |
| `sf-production-time`, `sf-unlock-blueprints` | 2 | Ändern reflektierte KFC-Dateiressourcen in der Pregame-Assetphase. Der Live-Reader macht aus ihnen keine Ingame-Schreibmodifikationen. |
| `sf-auto-stamina-refill`, `sf-infinite-item-split`, `sf-infinite-item-use`, `sf-no-fall-damage`, `sf-no-resource-cost`, `sf-unlimited-flight` | 6 | Verlangen `runtime.gameplay.patch` und rufen `runtime.functions.bind_modifier` mit einem benannten Modifier auf. Die neue Ressourcen-API ersetzt deren Ausführungspatches nicht. |
| `world-editor` | 1 | Liest `ItemInfo`-Ressourcen, um Prop-Rezepte zu ermitteln. Diese Eingabe nutzt jetzt bevorzugt den Live-Reader. Weltänderungen laufen weiterhin über `runtime.world.*`. |

Damit gibt es jetzt eine generische, modübergreifende ResourceService-Lese-API ohne pro Mod eingebettete Hashwerte oder Spielcodepatches. Der Nachweis deckt **Ressourcenlesen** ab. Er beweist nicht, dass veränderte Bytes im Spiel wirksam würden. Er löst weder die originalen ECS-Ausführungskontexte noch die sechs Modifier-Patches auf. Dafür müssen die jeweiligen Empfänger und ihre Ausführungswege separat verstanden werden.

## Artefakte und Prüfgrenzen

Alle Binärdateien wurden als Dateien gelesen und inventarisiert. Die drei JSON-Dateien wurden vollständig eingelesen. PE-Strukturen, Strings und relevante Aufrufketten wurden gezielt analysiert. Eine vollständige semantische Dekompilierung sämtlicher DLL-Funktionen wird nicht behauptet. Die fremden DLLs wurden weder ausgeführt noch installiert.

Lokale, ignorierte Artefakte liegen unter `profile-tools/devdata/creative-mode-146/`:

- `creative-ui-static.json`, `dinput8-ui-static.json` für PE-, Import-, Export- und Stringbefunde.
- `ui-api-bindings.json`, `ui-engine-call-sites.json`, `ui-widget-callback-reachability.json` und `ui-root-widget-builder.json` für die echte UI-Aufrufkette.
- `resource-and-item-functions.json` und `resource-cost-callbacks.json` für Registry- und Kostenpfade.
- `ui-integration-analysis.de.md` und `item-giver-event-analysis.md` für die vertieften Aufruf- und Kontextverträge.
- `current-client-items-validated-chain.json` für die ersten bestätigten Gleiterdaten.
- `current-item-map-complete.json` für alle 4.207 Einträge.
- `loaded-registry-resource-link.json` und `loaded-resource-handle-proof.json` für die Verbindung zum allgemeinen ResourceService.
- `loaded-resource-key-proof.json` für den originalen `ResourceId` des geprüften Handle-Eintrags.
- `resource-id-live-resolution.json` für GUID/Typ/Teilindex → Handle → geladener Datenbereich, an fünf Ressourcen vorwärts geprüft.

Die statische Besitzerherleitung steht außerdem in `devdata/embervale-toolkit-3.1.0/item-registry-owner-provenance.json` und der Service-/Gleiterpfad in `glider-service-provenance.md`. Der Profilentwurf mit dem validierten Lese-Layout ist `../../../profiles/enshrouded/client/25966634-resource-read.json`.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Ziel und Umfang | Generisches KFC4 ResourceService-Lesen in `game.assets` umgesetzt und alle neun vorhandenen Mods nach Ressourcen-, Modifier- und World-API-Aufrufen eingeordnet. |
| Repository und Belege | Alle acht Nicht-Editor-Mod-Einstiegspunkte gelesen, World-Editor-Ressourcenpfad sowie alle ResourceService-Aufrufstellen gezielt geprüft. |
| Architektur | Parser-`ResourceId` und `StaticHash` wiederverwendet. Neue API-Exporte sind optionale ABI-12-Erweiterungen. Keine Schreib-, Enginecall- oder Hookfunktion ergänzt. |
| Prüfung | Native Release-DLL gebaut und `cargo check -p shroudforge-api` erfolgreich. Liveprüfung mit neuem Build steht nach Neustart und Weltladen aus. |
| Offen | Schreibwirkung, Ressourcen-Refresh innerhalb eines zurückgegebenen Objekts, neue EXE-Builds, originaler ECS-Aufrufkontext und die sechs Modifier-Ausführungspfade. |

## Testinstallation am 16.10.2026

Der aktuelle Branch ist mit Tag `1.6.8` markiert, während die vom Paketbau gelesene Datei `VERSION` noch `1.6.5` enthielt. `VERSION` wurde auf `1.6.8` gesetzt. Der Release-Build `kfc4-resource-read-20261016` liegt unter `build/shroudforge-1.6.8-kfc4-resource-read-20261016.zip`, SHA-256 `e6661e10a56bfdceefe8b2ab8b5e9e6204a56de15f01c4c45a981f3fcda3e28d`.

Das Paket wurde mit dem transaktionalen Updater in `C:/Program Files (x86)/Steam/steamapps/common/Enshrouded` installiert. Alle 59 verwalteten Dateien stimmen bytegenau mit dem entpackten Paket überein. Die aktuelle `enshrouded.exe` hat SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a` und stimmt mit dem Leseprofil überein. Die Proxykonfiguration wählt `shroudforge`. Die zuvor vorhandene `dbghelp.dll` wurde in `shroudforge/updates/backups/1792114377/dbghelp.dll` gesichert. `emm.exe` blieb unverändert.

Vor der Installation lagen in diesem Spielordner weder `shroudforge/version.json` noch `shroudforge/kfc-runtime.dll`. Der dort vorgefundene alte Runtime-Hash aus dem vorherigen Status ließ sich deshalb an diesem Pfad nicht bestätigen. Die Runtime wurde nach der Installation nicht gestartet. Der erste Liveabgleich muss nach einem Spielstart in einer geladenen Welt erfolgen.

Die Installation aktiviert noch nicht alle Modfunktionen über die neue API. Der Ressourcenreader ist read-only. Die sechs Modifier-Mods verwenden weiterhin `runtime.functions.bind_modifier`. Die Funktionsinventur markiert unbestätigte Einträge ausdrücklich als nicht nativ aufrufbar, ohne vollständige Signatur, Enginekontext oder bestätigte Gameplaywirkung. Die beiden Pregame-Mods ändern KFC-Dateien. Die World-Editor-Weltaktionen bleiben profilgestützte Runtimeoperationen. Das sind separate Implementierungsarbeiten und keine fehlende Dateiübertragung.

## Fehlerkorrektur und gemeinsamer ECS-Besitzer am 16.10.2026

Der anschließende Livebetrieb zeigte einen Fehler im implementierten Ressourcenreader. Er behandelte allocatorbasierte Datenobjekte fälschlich als polymorphe Objekte und verlangte dort eine EXE-Vtable. Außerdem fehlte die Dereferenzierung des bei Kontext `+0x3e10` gespeicherten ItemRegistry-Pointers. Der Reader wies deshalb den tatsächlichen Besitzer mit `main-owner-vtable-outside-image` zurück. Das war ein Implementierungsfehler und kein Beleg für eine ungeladene Welt.

Die Korrektur prüft die belegten Singleton-, Client- und Controller-Vtables sowie den Controller-Backpointer. MainOwner, Reader und ResourceService erhalten keine erfundene Vtable-Anforderung. Die ursprünglichen Servicecallbacks bei `+0x7d8` und `+0x7e8` werden gegen die exakten EXE-RVAs geprüft, ohne sie aufzurufen. Ein ausschließlich lesender vollständiger Abgleich fand 91.812 eindeutige Ressourcenschlüssel, darunter 4.262 ItemInfo-Einträge, mit gültigen StaticHash-Buckets und Chunkbereichen. Der World Editor verwirft bei einem Fehler während der vollständigen Live-Dekodierung auch bereits abgeleitete Teilrezepte und versucht anschließend die vorhandene Dateiressource.

Ressourcen- und ECS-Zugriff verwenden nun denselben ursprünglichen Besitzerresolver `provider/engine_context.cpp`. Das separate `ownerLayout` enthält die gemeinsame Kette. Das `ecsReadLayout` ergänzt den Originalpfad zum Registrybesitzer und EntityManager. Es enthält keine kopierte Komponentenliste. Alle 640 Registrierungen, 541 Laufzeitspeichertypen und 99 reinen Template-Registrierungen stammen aus dem ursprünglichen Register und werden mit der aktuellen Parserreflection verbunden.

Für eine offline oder lokal gehostete Welt ist die aktive Szenenliste entscheidend. Sie liegt unter `Client+0x630`, enthält Pointer und Szenen-UUID in Einträgen von `0x90` Bytes und wird vom Originalcode bei RVA `0xe3adf0` iteriert. Nur der geprüfte Zustand 5 bezeichnet hier eine bereite Szene. Aus deren Kontext führt `+0x1d0` zum Ausführungsbesitzer R2, `R2+0x928` zum Registrybesitzer und `R2+0xe15dc0` zum eingebetteten EntityManager. Wenn keine lokale Szene existiert, kann derselbe Leser die Clientreplik verwenden. Mehrdeutige oder wechselnde Szenen werden zurückgewiesen.

Die vorhandenen Lua-Funktionen `runtime.ecs.query`, `resolve`, `read`, `read_bytes`, `read_fields` und `read_attributes` verwenden diesen Pfad für stabile Snapshots ohne Dispatcherhook. Die Abfrage prüft Komponentenpräsenz, der eigentliche Lesezugriff zusätzlich Spalte und exakte reflektierte Größe. Präsenz ohne gültige Spalte wird nicht pauschal als leeres Tag behandelt. Besitzer, Tabelle, Entitymitgliedschaft, Generation und Payload werden erneut geprüft. Ein KFC3-Ausführungsprofil kann keinen KFC4-Prozess mehr durch strukturellen Fallback freigeben.

Die unabhängigen Prozesscaptures und produktiven Lua-Codecprüfungen stehen im [ECS- und Attributbericht](2026-10-16-kfc4-ecs-attribute-read.md). Sie belegen Daten aus Client und Host, einschließlich 1.488 eindeutig zugeordneten Rootsnapshots aus 57 Roots. Sie ersetzen nicht die noch erforderliche Ausführung der neuen DLL und Lua-Abfragen nach einem Spielneustart.

### Originaler Scheduler und verbleibender Schreibpfad

Die echte TaskQueue liegt bei `R2[+0xe15d60]`. Submit bei RVA `0x8e0ff0`, der Descriptor, Worker und Folgeabhängigkeiten sind nachvollzogen. Submit mit Wait bei `0x8df860` ist keine automatische Threadfreigabe. Das originale Wait kann Aufgaben auf seinem aufrufenden Thread ausführen. Dieser Pfad ist in der aktuellen Workergruppe aktiviert. Ein Loaderthread könnte damit seine eigene Aufgabe ausführen.

Der originale Systembuilder unterscheidet außerdem sechs Komponentenlisten. Die Listen bei Descriptor `+0x38/+0x48` werden zu Lesemasken, die bei `+0x58/+0x68` zu Schreibmasken. Das Kompatibilitätsprädikat erlaubt Read/Read, verweigert aber überlappende Read/Write- und Write/Write-Gruppen. Ein unabhängig eingereihter Task gehört dadurch noch nicht automatisch zum geschützten ECS-Graphen. Die noch fehlende Graphintegration und Besitzerlebensdauer bleiben Voraussetzung für generische Schreibaufträge. Es wurden keine Tasks eingereiht und keine Spielbytes verändert.
