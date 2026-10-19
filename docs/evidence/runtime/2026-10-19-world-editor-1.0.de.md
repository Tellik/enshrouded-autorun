# World Editor und die native Spielwelt in Enshrouded 1.0

Untersuchung vom 19. Oktober 2026. Dieser Bericht hält den untersuchten Stand fest. Die aktuelle Modulübersicht bleibt unter `docs/development/code-ownership.en.md`.

## Aktueller Status: typgebundener F8-Leser eingebaut, Schreibpfade offen

Der nachfolgend dokumentierte Voxel-/Entity-Versuch verwendete feste Funktionsadressen, Byteprüfungen und Ergänzungen je Spielbuild. Der Nutzer verlangt die Auflösung über Typnamen und originale ECS-/Operationsregistrierungen. Auch ein dynamischer Suchlauf nach internen Funktionsadressen erfüllt diese Vorgabe nicht. Der Versuch wurde deshalb verworfen. Seine vier neuen Adapterdateien, Produkt-Buildanschlüsse, World-Routen, Profilparser und die zusätzliche Bounded-Tracking-Policy wurden aus dem Produkt entfernt. Die neuen Profil- und Schemaergänzungen wurden zurückgenommen. Drei anschließend erstellte Offline-Funktionssucher wurden ebenfalls aus dem aktiven Quellbaum entfernt und im ignorierten Archiv gesichert. Die ursprüngliche allgemeine Owner-/Serviceanbindung blieb unverändert.

Der eingefrorene Quellstand und der Diff vor dieser Bereinigung liegen ausschließlich im ignorierten Entwicklungsarchiv `dev/profiles/captures/world-editor-rejected-rva-approach-20261019-054825/`. Die nachfolgende Aufrufkette und die historischen Livebeobachtungen dokumentieren diesen verworfenen Versuch. Sie sind keine Freigabe für Voxel-Schreib-, Spawn- oder Delete-Unterstützung.

Der neue F8-Leser löst `ecs::VoxelGrid` und `VoxelWorld` bei jedem Aufruf aus der echten ECS-Dienstregistrierung auf und verwendet die originalen registrierten Interface-Rollen für Ursprung, Tile-Bereitschaft und Zell-Lesen. Er sucht keine Funktionsadresse und verwendet keine Byte-Signatur. Der sitzungsgebundene Lua-Aufruf `runtime.world.voxel.read` verlangt ABI 12 und verwendet keinen Rückfall auf den alten Profilfunktionsaufruf. Das native Ergebnis bleibt bis zum erneuten Sitzungsabgleich in einem eigenen Puffer. Bei noch nicht lesbarem Scene- oder Tilezustand meldet es `Pending`, worauf Lua denselben F8-Auftrag innerhalb seiner bestehenden Frist fortsetzt.

Die allgemeine Owner- und Scene-Sperrenauflösung bleibt profilgestützt. Es wurden dafür keine Profil- oder Schemafelder ergänzt. Die Cursor-/Prop-Leser, der unterscheidbare Abwesenheitsstatus und die Lua-Recoveryjournale bleiben erhalten.

### Verfügbarkeits- und Rasterkorrektur im aktuellen Lauf

Der bisher installierte Leser war hinter einer falschen Verfügbarkeitsprüfung gesperrt. `OperationAvailable` fiel für Voxel-Lesen auf die alte Ausführungsprofilliste zurück. Der Rasteraufruf las ebenfalls noch Profilmetadaten. Beide nativen Einstiege verwenden jetzt den registrierten `ecs::VoxelGrid`-Leser. Die Weltklassifizierung verwendet den doppelt geprüften ursprünglichen ECS-Owner und benötigt dafür keinen Hook-Dispatcher. Der alte Voxel-Schreiber wird dadurch nicht freigeschaltet. `Status` und `Diagnostics` werden auch unter der vorhandenen ECS-Zustandssperre geprüft. Die Ownerklassifizierung fragt deshalb nicht rekursiv die ECS-Sitzung ab.

Die neue optionale ABI-12-Funktion `KfcRuntimeWorldGridGetSpecForSession` erhält die Sitzung explizit und unterscheidet Erfolg, vorübergehendes Warten, ungültige Eingaben, Sitzungswechsel, fehlenden Vertrag und einen bleibenden Readerfehler. Bei jedem Fehler bleibt der Ausgabepuffer unverändert. Die Rust-Bridge reicht `voxel-grid-pending` beziehungsweise `voxel-read-pending` bis Lua durch. F8 wartet auch bei noch nicht lesbaren Rasterdaten innerhalb derselben Gesamtfrist. Ungültige Rasterdaten werden nicht als Wartezustand behandelt. Für die bereits installierte Rust-Bridge bleibt der bisherige konkrete Pendinggrund zusätzlich erkennbar.

Aktuell ausgeführte Prüfungen:

| Prüfung | Ergebnis |
| --- | --- |
| Nativer Release-Build | Erfolgreich |
| Dienstfixture, normaler Lauf | 443 Checks bestanden |
| Dienstfixture, sitzungsgebundenes Lesen | 412 Checks bestanden |
| Dienstfixture, ungültiger Origin-Rückgabepointer | 420 Checks bestanden |
| Dienstfixture, schreibgeschützte Metadaten | 442 Checks bestanden |
| Weltkontextfixture | 53 Checks bestanden |
| `cargo check -p shroudforge-api -p shroudforge-runtime-bridge --lib` | Erfolgreich |
| Gebündelte Lua-Tests | Drei Tests bestanden, einschließlich F8-Pending, Raster-Pending, Deadline und V7-Kompatibilität |
| Verschobene Diagnostics-Prüfungen | Vier Tests bestanden, kanonische Probequellen und private Fixtures |
| Vollständiger Rust-Loader-Releasebuild | An bereits vorhandenen Compilerfehlern im geänderten Updater gescheitert, keine neue Rust-Loader-DLL installiert |

Nach einer weiteren parallel entstandenen Diagnostics-Umstellung scheiterte die erneute Cargo-Prüfung zusätzlich an uneinheitlichen `DiagnosticJob.execution`-/`runtime_probe`-Aufrufen. Diese Umstellung gehört nicht zur hier vorgenommenen Rasterkorrektur. Die zuletzt ergänzten Lua-Fälle wurden deshalb außerdem mit dem isolierten Lua-Runner unter `build/world-editor-lua-check/` gegen die unveränderten aktuellen Modquellen und `world_editor_lifecycle.lua` ausgeführt. Diese gezielte Prüfung bestand einschließlich Bereitschaftswechsel und dem booleschen Rasterprovider. Sie ersetzt keinen vollständigen Rust-Loader-Build.

Am 19.10.2026 um 14:37 Uhr wurde bei geschlossenem Spiel ausschließlich die geprüfte native DLL und `mods/world-editor/src/operations.lua` installiert. Die vorherigen Dateien liegen unter `shroudforge/backups/world-editor-native-read-20261019-143752/`. Die bestehende Rust-DLL, Spiel-EXE, Einstellungen und Profile wurden dabei nicht ersetzt. Die Einstellungen und die fünf unverändert übernommenen Lua-Module wurden zusätzlich byteweise mit der 0.9-Referenz verglichen. Der neue Spielprozess hat PID `10032`.

Der erste neue F8-Aufruf scheiterte noch bei der Lua-Rasterbereitschaft. Zwischen `runtime.has` und `runtime.status` kann sich die native Bereitschaft ändern. Ein bereiter zweiter Status besitzt keinen Fehlergrund, wodurch Lua fälschlich den endgültigen Ersatzgrund `native voxel grid metadata is unavailable` ausgab. Der Lua-Ablauf behandelt diesen Wechsel jetzt als begrenztes Warten. Die boolesche Rasterexportvariante der installierten Rust-Bridge bleibt ebenfalls innerhalb dieser Frist kompatibel. Der erste Live-Nachladeversuch wurde von noch ausstehenden Crafting-Ressourcenrücknahmen abgebrochen. Nach deren protokolliertem Abschluss bestätigte der zweite Nachladeversuch `reloaded`.

Um 14:46:39 Uhr schrieb F8 anschließend tatsächlich `capture-5.sfbp`. Das Log bestätigt Terrainlesen, den fortgesetzten Prop-Scan, Encoding, Export und Abschluss. Die separat gelesene Datei hat 530 Bytes, den unveränderten Header `SHROUDFORGE_WORLD_BLUEPRINT_V7`, die Dimensionen 8 × 6 × 2, genau 96 Zellwerte, davon 32 ungleich null, und null Props. Die unveränderte Belegkopie liegt unter `dev/profiles/captures/enshrouded-client-25988036/world-editor-registered-native-capture-5-20261019.sfbp`. Dieser Lauf bestätigt den registrierten F8-Produktleser und den Export, keinen Welt-Schreibaufruf.

Ein zweiter F8-Aufruf schrieb um 14:54:23 Uhr `capture-6.sfbp`, 12.142 Bytes mit 12 × 14 × 14 Zellen. Die Datei enthält genau 2.352 gültige uint16-Zellwerte, davon 586 belegt, und zwei Props. Alle 2.352 Coverageflags stimmen mit den jeweiligen Luft-/Belegungswerten überein. Das Log bestätigt erneut den abgeschlossenen Export, der Editorzustand meldet `saveState=complete` und `processState=complete`. Auch diese Datei wurde unverändert als `world-editor-registered-native-capture-6-20261019.sfbp` im bisherigen Captureverzeichnis gesichert.

Beim anschließenden F7 am selben Capture wartete der Editor zehn Sekunden und brach mit `The client world context did not become readable within the retry window` ab. Das Log zeigte weiter `source=ecs-local-authority` und `voxel_context=ready`, aber auch `profile=missing-or-ambiguous-profile` und `game_thread=unavailable`. Die Meldung war somit irreführend. `runtime.has` und der anschließende `runtime.status` können außerdem verschiedene Bereitschaftsmomente abbilden. Der World Editor meldet einen fehlenden nativen ECS-Schreibauftrag für eine bereits aktive direkte Sitzung nun sofort und führt keine Warteschleife aus. Bei F4 bleibt das Wiederherstellungsjournal erhalten und der Vorgang endet als Fehler. Die aktuelle Lua-Lifecycle-Fixture besteht mit diesen Fällen. `mod.lua`, `operations.lua` und `history.lua` wurden am 19.10.2026 um 15:04 Uhr mit Sicherung unter `shroudforge/backups/world-editor-f7-readiness-20261019-150458/` installiert. Der erste Reloadversuch wartete noch auf den Crafting-Cleanup. Nach dessen protokolliertem Abschluss bestätigte der zweite Versuch `reloaded`. Um etwa 15:06:42 Uhr löste der Nutzer F7 für `capture-6` aus. Das Log meldet sofort `No verified native ECS terrain write operation is available for this world session. No world changes were sent.` Es wurde weder gewartet noch ein Writer aufgerufen. Damit ist der falsche Kontextfehler beseitigt. Der native Schreibpfad ist weiterhin nicht implementiert und F7 hat keinen Block gesetzt.

### Neu eingegrenzter regulärer Hammerpfad

Eine frische lesende Probe des vorherigen Prozesses `5984` bestätigte, dass `PlayerInput` und `ClientPlayerInput` keine der beiden ursprünglichen Trackingkategorien besitzen. Die lokale Cliententity und die autoritative Entity stimmen in ID, Generation und Template-UUID überein. Der natürliche digitale Input wird vom Client zur Authority übertragen und anschließend vom ursprünglichen `ServerConsumedPlayerInput` verarbeitet. Die Auswahl eines vorhandenen Bauitems verlangt lediglich einen von der verbrauchten Aktion abweichenden Versionswert. Die alte profilierte Versionsfunktion ist für diesen Vergleich nicht erforderlich. Der autoritative Input kann jedoch vor seinem Verbrauch durch das nächste Clientpaket ersetzt werden. Eine zuverlässige Einreichung benötigt deshalb den noch zu bestätigenden Mainclient-Schreibschutz und eine ursprüngliche Verbrauchsbestätigung.

Mainclient und Authority verwenden dieselben 126 Blueprintobjekte der geladenen Ressource `3064d35d-7342-40ca-bdc9-aad58f83bf45`. Ein vorhandenes unkomprimiertes 8³-Item besitzt bereits 512 Dichtebytes. Die reflektierte Ressourcen-API kann einzelne vorhandene Dichtebytes ändern, ohne Größe, Arraydescriptor oder Registry zu erweitern. Diese Bytes bleiben aber bis zur eigentlichen Platzierung gemeinsam genutzter Input. Eine sofortige Wiederherstellung nach der Itemauswahl wäre zu früh. Zusätzlich besitzt der ursprüngliche Bauablauf Materialmasken und Überlappungsregeln. Daraus folgt noch keine exakte Umsetzung beliebiger gemischter 16-Bit-Voxelraster. In dieser Untersuchung wurden keine Blueprint- oder Inputwerte geändert.

Die weitere Prüfung des vollständigen `impact_simulation`-Deskriptors ergab genau eine registrierte Codefunktion, den ursprünglichen Systemlauf. Die übrigen belegten Pointerpaare enthalten Namenarrays für Komponenten, Dienste und Events. Ein registrierter Framekonstruktor oder Query-Prepare-Callback ist darin nicht vorhanden. Die nächsten Codepointer gehören bereits zum nächsten Deskriptor. In der untersuchten Reflection und den vollständig gelesenen DI-Namen ist außerdem kein Commandqueue-Typ mit reflektierten Queuefeldern oder benannter CommandPublisher belegt. Eine eigene Queuepublikation würde deshalb weiterhin interne Manageroffsets voraussetzen.

Auch die weitere Mainclient-Untersuchung lieferte noch keinen zulässigen Auftragseinstieg. Seine Inputqueue besitzt keine gemeinsame Publikations-/Abbausperre. Der Owner wird freigegeben, bevor sein Zeiger gelöscht wird, sodass doppelte Lesungen keine sichere fremde CAS erlauben. Die zusätzliche gesperrte Plattformqueue verarbeitet ausschließlich feste Plattformereignisse, keine eigenen Modcallbacks. Die überprüften 15 Taskworker liefern einen Workerrahmen, keinen ursprünglichen Scene-Queryframe. Diese Befunde schließen die geprüften Kandidaten aus, nicht jede unbekannte Schnittstelle der Engine. Für F7 wurde deshalb kein Ersatz über interne Adressen, erfundene Frames oder direkte Queueänderungen eingebaut.

## Typen und originale Datenkanäle nach der Korrektur

Eine frische Extraktion der Original-EXE erschließt 12.646 Typen und 39.081 Structfelder über die bestehende Reflection. Der vom bisherigen Parser gelesene 144-Byte-Basisbereich enthält keine direkten Codezeiger und keine Methodentabelle. Die weitere Untersuchung der originalen Registrierung zeigt jedoch drei zusätzliche Callbackfelder danach. Bei 2.116 DS-Typen sind diese belegt, bei den übrigen 2.685 DS-Typen und allen 7.845 Nicht-DS-Typen sind sie leer. Die nach Typnamen aufgelösten Callbacks von `ds::impact_nodes::SpawnEntity` und `DestroyEntity` konstruieren, kopieren oder bereinigen die reflektierten Nutzdaten. Sie führen die Graphoperationen nicht aus. Der bisherige Parser erfasst diese Lifecycle-Erweiterung noch nicht. Ein fehlender Methodenaufruf im JSON beweist daher keine fehlende Engineoperation.

Die vorhandene Lua-Anbindung löst Typnamen bereits mit `runtime.types.resolve` auf. `runtime.ecs.query`, `read_fields` und `write_fields` verwenden die tatsächliche Komponentenregistrierung und reflektierte Feldpfade. Ihr bisheriger Umfang umfasst keine Entity-Erzeugung, Komponentenaddition oder allgemeine Eventpublikation. Ein neuer Lua-Funktionsname allein würde diese fehlende native Handlung nicht bereitstellen.

| Typ oder Kanal | Belegter Umfang | Noch fehlender Aufrufvertrag |
| --- | --- | --- |
| `keen::ecs::CurrentTransform`, `UsedItem` | Registrierte Entity-Komponenten, bestehende generische Leser und eingeschränkter CAS-Schreiber | Tracking und bestätigtes Gameplayverhalten bei Änderungen |
| `VoxelWorldResource`, `VoxelWorldChunkResource` | Ressourcenlayout und Contentreferenzen | Zugriff auf den aktuellen dynamischen Weltstand und Cell-Schreiboperation |
| `VoxelBlueprintItemRegistryResource` | Typisierte Blueprintdaten mit Größe, Datenarray und Kompressionsflag | Eigene Live-Ressourcen veröffentlichen und originalen Bauauftrag bestätigen |
| `PlayerInput` und `ServerConsumedPlayerInput` | Originaler Eingang und Verbrauchszustand der Building-/Adminsysteme | Sitzungssicher publizieren und den tatsächlichen Weltänderungsausgang zuordnen |
| `AdminSpawnCommand` | UUID, Position und Spieloptionen mit Versionsverbrauch | Rotation, Scale, Item-/NoLoot-Overrides und Ergebnis-ID fehlen im Commandvertrag |
| Building-/Terraforming-Events | Reflektierte Ereignisdaten | Originale Eventqueue auflösen und nachgewiesen publizieren |
| `impact_nodes::SpawnEntity`, `DestroyEntity`, `Terraforming` | Benannte Graphoperationen mit markierten Eingängen, Ausgängen und Konfigurationen. Die echte Operationstabelle im originalen `impact::ImpactSystem` ist inzwischen belegt | Natürlicher ECS-Ausführungskontext sowie Veröffentlichungs- und Lebensdauervertrag für Weltänderungen |

`SpawnEntity` besitzt tatsächlich einen Transformations- und Templateeingang sowie einen Entity-ID-Ausgang. Der natürliche VM-Konsument des originalen `ImpactSystem` löst seine benannten Operationen über eine Tabelle aus 80-Byte-Records auf. Die laufende Welt enthält 107 eindeutige Namen mit Callback und Argumentanzahl, darunter `spawnEntity`, `destroyEntity` und `isSameEntity`. Die Auswahl erfolgt über den vollständigen Registrierungsnamen. Kein gespeicherter RVA oder Funktionshash wählt den Callback. Die vorhandene allgemeine DI-Ownerauflösung bleibt profilgestützt.

Die neue Lua-Anbindung `runtime.impact.call(type, inputs)` besitzt zunächst nur einen ausdrücklich geprüften Adapter für `keen::impact_nodes::IsSameEntity`. Er validiert die reflektierten Eingänge und `EntityId.id`, löst die echte Registrierung auf und ruft den Originalcallback nach Freigabe sämtlicher Sitzungs- und Scene-Sperren mit ausschließlich eigenen VM-, Argument- und Ergebnispuffern auf. Der Vergleich schreibt keine Spielwelt. Andere Operationen bleiben mit ausdrücklichem Grund gesperrt. Private Tests prüfen Eingabegrenzen und beide Ergebniszweige. Der separate Live-A/B-Prüfmod ist vorbereitet, sein Ergebnis liegt noch nicht vor.

Spawn und Delete verwenden zusätzlich den originalen ECS-Query-/Servicekontext der laufenden `impact_simulation`. Der benannte `ecs::Tasks`-Dienst ist vorhanden, stellt allein aber keinen gültigen Ausführungskontext bereit. Diese Weltaufrufe werden deshalb noch nicht freigegeben. Der Admin-Spawnkanal ist wegen seiner engeren Eingaben und fehlenden Ergebnis-ID kein gleichwertiger Ersatz für World-Editor-Paste.

Auch `ecs::VoxelGrid` ist als original gebundenes Interface belegt. Der Produktleser prüft die originale Dienstbindung, Tile-Handles und Taskgenerationen unter Session- und Scene-Schutz. Er gibt den World-Lock vor dem Aufruf der originalen Interface-Callbacks frei, liest nur in einen eigenen uint16-Puffer und bestätigt die vollständige Zellenzahl, bevor Lua das Ergebnis erhält. Die Produktroute ist gebaut und installiert. Der aktuelle F8-Liveexport ist im Abschnitt zur Verfügbarkeits- und Rasterkorrektur bestätigt.

Der vorherige Live-F8-Versuch brach bei `runtime.world.voxel.read` ab, bevor eine Blueprintdatei geschrieben wurde. Zusätzlich blieb der Editorfortschritt trotz Abbruch auf `running`. Der Lua-Abbruchpfad wurde korrigiert und ist mit dem neuen Paket installiert. Timeout, Sitzungsverlust, endgültige API-Verweigerung und F6 entfernen ihre Lesefortsetzungen und beenden den Fortschritt, ohne ein unvollständiges Blueprint zu speichern oder ein vorhandenes Recoveryjournal zu verlieren. Die Lifecycle-Prüfung deckt weitere 410 Sekunden nach dem Timeout und einen anschließenden neuen F8-Auftrag ab. Der neue native Leser ist bislang nur mit privaten ECS-Fixtures geprüft.

Die reinen Typprüfungen verwendeten weder ein Spielprofil noch eine Suche nach nativen Funktionsadressen, Bytesignaturen oder Funktionshashes. Lokale Berichte sind `typed-world-registration-contract-20261019.json` und `typed-voxel-data-channels-20261019.json`. Letzterer enthält 39 vollständige Typ-/Basistypbeschreibungen, 14 relevante erfasste Komponentenregistrierungen und acht Systemabhängigkeitslisten. Diese Aufnahmen rufen keine Engineoperation auf und verändern keine Welt.

Der vorherige Versuchsstand bestand den nativen und den Rust-Release-Build, 427 Dienstchecks sowie sechs damals gemeinsam gebündelte Lua-Tests. Die aktuellen Prüfungen und ihre Buildgrenze stehen im Abschnitt zur Verfügbarkeits- und Rasterkorrektur. Alle 37 Einstellungen und fünf Einstellungsgruppen bleiben gegenüber der Referenz unverändert. F8 mit dem neuen registrierten Reader ist im laufenden Spiel bestätigt. Paste/Undo bleiben offen.

## Ziel und geprüfte Eingaben

Das Paket aus `B:\reference-mods-0.9\world-editor` soll seine Einstellungen, Bedienung, SFBP-V7-Dateien, Rotation, Bibliothek, Transaktionsjournale und sein Steam-P2P-Protokoll behalten. Spielzugriffe sollen über die öffentliche Lua-API und native ECS-Anbindung erfolgen, ohne Spielcode zu verändern.

Untersucht wurde die installierte originale Client-EXE aus Steam-Build `25988036`, interne Version `1269579`, `release_1.0`.

```text
SHA-256: 82e5020a8736348cd0374f004b2f5e5ef456b5bd27aa5420df34c7dd241cfeeb
PE timestamp: 1792148057
PE image size: 44146688
```

Die lokale Spielwelt wurde mit Zustimmung des Nutzers zunächst über den bestehenden lesenden Prozesszugriff untersucht. Anschließend wurden die neue DLL und der Lua-Mod in einer geladenen Testwelt ausgeführt. Der erste Canary und F8 lasen nur die Welt und schrieben Exportdateien. Spätere manuelle F7-/F4-Aufrufe bestätigten Terrain-Paste und eine Terrain-Rücknahme im damals installierten, inzwischen verworfenen Backend. Der separate automatische Ein-Zellen-Schreibcanary führte keine Mutation aus und wurde deaktiviert. Es wurden keine Spielcode-Hooks installiert.

Der letzte protokollierte F7-Versuch mit `capture-4.sfbp`, 6.440 Voxelzellen und neun Props, wurde vor der Mutation wegen `runtime.world.entity.spawn unavailable` abgelehnt. Dieser konkrete Fehler bleibt ohne passenden nativen Spawnvertrag ungelöst. Frühere erfolgreiche Terrainaufrufe belegen weder einen konformen Typaufruf noch vollständiges Paste/Undo.

## Historisch untersuchter und später verworfener Zugriff

Der Referenzmod enthält bereits ausschließlich öffentliche `runtime.world.*`- und Asset-Aufrufe. Die dort verwendeten nativen Implementierungen besitzen jedoch alte profilierte Funktionsadressen und Dispatcher, deren Ausführungsphase über Spielcode-Hooks entsteht. Eine Modkopie allein wäre deshalb keine Migration der Spielweltanbindung.

Die aktuellen 1.0-Profile besitzen eine eigenständig geprüfte originale Ownerkette sowie ECS-, Ressourcen- und Schreibsperren. Sie besitzen keine freigegebenen alten `worldOperations` oder Hookdefinitionen. Ihre allgemeine ECS-Anbindung liest Originalregistrierungen und installiert keine Spielcode-Hooks.

| Handlung | Umsetzung im untersuchten Stand | Grenze |
| --- | --- | --- |
| Einstellungen, UI-Aktionen, SFBP V7, Bibliothek, Rotation, P2P und Journale | Paket unter `mods/world-editor/`, identische Einstellungen und bestehende Modulaufrufe | Kein Nachweis einer erfolgreichen Weltänderung |
| Lokaler Cursor | Originaler Mainclient-ECS-Owner, dynamische `ClientPlayerInput`-Registrierung, reflektiertes `data.cursorInput` | Sitzung, Mitgliedschaft, Generation, Komponentenspeicher und stabile Kopien müssen auf jedem Aufruf stimmen |
| Props abfragen und Transform lesen | Originaler ECS-Owner und dynamische Registrierungen von `CurrentTransform` und `UsedItem` | Vollständige Ergebnisse erst nach inkrementellem Scan, keine atomare Gesamtsicht der Welt |
| Prop-Skalierung | Kanonischer ECS-Snapshot und maskierter CompareExchange | Keine Umgehung der Sperre für getrackte Komponenten, kein Nachweis für Kollision oder Persistenz |
| Voxel lesen | Originaler VoxelWorld-Dienst der lokalen Authority, Mode-6-Reader unter Sitzungs-, Scene- und Lebensdauerschutz | 20 stabile Einzelzellen-/8³-Block-Vergleiche und ein F8-Capture bestätigt, keine Freigabe für einen Client ohne lokale Authority |
| Voxel schreiben | Originaler 8³-Writer mit erhaltenden Teiländerungen, vollständiger Rücklesung und geprüftem Rollback eingebaut | Statische ABI-, Speicher- und Commitprüfung erfolgt, Live-Schreibtest steht noch aus |
| Props erzeugen, platzieren und entfernen | Bestehende Lua-Verträge erhalten | Kein freigegebener patchfreier 1.0-Adapter |
| Vollständiges Paste und Undo | Bestehende Ablauf- und Recoverylogik erhalten | Nicht als funktionsfähige 1.0-Weltänderung bestätigt |

## Cursor und getrennte ECS-Kontexte

In der untersuchten laufenden Welt enthielt der Mainclient genau eine Entity mit `ClientPlayerInput` und `ClientCursor`. Ihre originale Hashtabellenmitgliedschaft, Generation, alle 84 gesetzten Archetypkomponenten und die stabile Cursor-Kopie wurden gegen die vollständige Registrierung geprüft. Die autoritative Welt besaß eine getrennte Entitytabelle und eine `PlayerInput`-Cursorübertragung. Ihre 184 gesetzten Komponenten wurden ebenfalls abgeglichen. Die beiden primären Cursorpositionen stimmten überein.

`ClientPlayerInput` hat in diesem Build 1.536 Bytes. Der eingebettete `ClientCursorInput` hat 168 Bytes. Positionen sind Q32.32-Integer. Cursorflags haben 32 Bits, Material und Objekttyp jeweils ein Byte. Der alte 160-Byte-Hooksnapshot ist kein gültiger Ersatz für dieses Layout.

`ResolveClientEcs` wählt ausdrücklich den originalen Mainclient. Die bestehende allgemeine ECS-Sitzung kann parallel die lokal gehostete autoritative Szene auswählen. Registrierungsindices und Entityhandles dieser beiden Manager dürfen nicht vermischt werden.

Die neue optionale ABI-12-Funktion `KfcRuntimeWorldCursorReadInput` liefert nur eine eigene Kopie der vollständigen lokalen Eingabekomponente. Der Lua-Adapter löst ihre Feldnamen über die aktive Reflection auf und erhält die bisherige Cursor-Ergebnisform. Der Discovery-Scan ist begrenzt und gibt einen wiederholbaren Wartezustand aus. Ein gelesener Cursor erteilt keine Voxel-Schreibberechtigung.

## Originale Systeme und VoxelWorld

Die Originaldeskriptoren wurden frisch aus der EXE gelesen. `client_cursor` bindet `VoxelWorld` lesend an Serviceposition 17. `server_cursor_no_prediction` bindet ihn lesend an Position 12. Die Systeme `player_building_blueprints`, `player_building_dismantle` und `player_building_undo` binden ihn schreibend an Position 8, `player_building_place_prop` an Position 9.

In beiden Live-Kontexten stimmten die normalen und alternativen VoxelWorld-Bindungen mit dem jeweiligen eindeutigen originalen VoxelWorld-Servicerecord überein. Insgesamt wurden 14 dieser Bindungen verglichen. Die VoxelWorld-Objekte von Mainclient und autoritativer Welt sind verschieden. Andere Servicepositionen sind teilweise leer oder weichen in der jeweils fremden Prozessrolle ab. Ein vorhandener Systemrecord beweist deshalb keine vollständige Ausführbarkeit dieses Systems in beiden Kontexten. Die untersuchten 15 Systemprozeduren entsprachen im laufenden Prozess den Originalbytes der EXE.

### Manuell bestätigter Hammerblock

Der Nutzer bestätigte am 19.10.2026, dass in der geladenen Testwelt ein einzelner Terrainblock mit dem Bauhammer gesetzt wurde. Es wurde keine weitere Platzierung angefordert. Die verfügbare Prozessaufnahme `world-editor-operations-live-hammer-after-place-20261019.json` zeigt einen stabil gelesenen Cursor in Mainclient und lokaler Authority sowie die passende `player_building_blueprints`-Bindung. Sie enthält jedoch weder `PlayerDigitalInput` noch einen Vorher-/Nachher-Voxelvergleich. Ihr `worldChanged: false` bedeutet nur, dass der lesende Prüfprozess die Spielwelt nicht geändert hat.

Der 30-Sekunden-Wächter `entity-native-queue-hammer-current-20261019.json` meldet keine ECS-Entity-Queue-Ereignisse. Er beobachtet weder den digitalen Hammerinput noch Änderungen im VoxelWorld und kann deshalb die bestätigte Platzierung nicht widerlegen oder als Commit nachweisen. Die Reflection benennt `keen::ecs::PlayerInputType.MainhandAction` als Eingabeindex 0. Die Originaldeskriptoren zeigen `player_building_blueprints` als System, das `PlayerInput` liest und `VoxelWorld` schreibt. Damit ist der natürliche Spielpfad eingegrenzt, der konkrete Input-zu-Voxel-Übergang dieses einzelnen Blocks aber nicht synchron mitgeschnitten.

Im gleichzeitig laufenden ShroudForge-Log steht `profile=missing-or-ambiguous-profile`, `game_thread=unavailable` und `voxel_context=waiting`. Die aktuelle `runtime.world.building.input`-Implementierung hängt an einem profilspezifischen Cursor-Hook und festen Layout-Offsets. Der alte `runtime.world.voxel.write`-Pfad ruft einen profilgebundenen Funktionszeiger auf. Beide Wege sind für die verlangte patchfreie, typnamenbasierte Produktanbindung ungeeignet und im beobachteten Lauf nicht verfügbar.

Die statisch verfolgte aktuelle Lesekette lautet:

```text
client_cursor callsite 0x290e14
  -> VoxelWorld service accessor 0xa8ade0 (+8)
  -> cursor voxel consumer 0x109ca80
  -> voxel reader 0x1095520, mode 6
```

Diese RVAs dokumentieren den konkreten Originalbeleg. Sie sind keine im Mod eingebauten Adressen und keine Freigabe, eine Funktion auf einem beliebigen Thread aufzurufen. Ein ECS-Dienstname allein beweist weder dessen native ABI noch das Schreib-, Streaming- oder Persistenzprotokoll.

Die ursprüngliche Schreibkette von `player_building_blueprints` wurde ebenfalls verfolgt:

```text
0x2d30d0 -> 0x486ef0 -> 0xa837b0 -> 0xa77680
  -> 0x109e290 -> 0x109df00 -> 0x109d2b0 -> 0x10b0e30
```

Der Reader verwendet einen Welt-Leselock bei `world + 0x174` und einen DynamicStorage-Leselock bei `[world + 0x1220] + 0x118`. Der Writer verarbeitet 8³-Chunks und nimmt den DynamicStorage-Lock exklusiv. Danach folgen Serialisierung beziehungsweise Kompression, Änderungen an Chunkbitmaps, eine Dirtymarkierung bei `storage + 0x2c1` sowie Listen für geänderte Bounds und niedrigere Auflösungen bei `storage + 0x50d0` und `storage + 0x5300`.

Der Consumer `0x109a500` verteilt den Dirtyzustand auf 64 Kanäle und serialisiert Delta-Payloads. Sein ursprünglicher Aufrufer ist `0xe4cf40`. Die Save-StateMachine `0xe4d9d0` ruft über `0x109c540` und `0x109bc80` einen getrennten Chunkmap-Serialisierungsweg auf. Daraus folgt noch kein Persistenznachweis für beliebige vom Editor veränderte Weltbereiche.

Der neue Adapter in `modules/game-runtime/native/voxels/read_and_write_native_voxels.cpp` prüft die exakte Imageidentität, das eindeutige originale VoxelWorld-Servicerecord und dessen tatsächliche Bindung im System `server_cursor_no_prediction`. Er hält die originale lokale Authority-Sitzung und den Scene-Schutz während des Aufrufs. Der Mode-6-Reader übernimmt seine internen Welt-, Storage- und Ressourcenpins selbst. Keine dieser nativen Adressen wird an Lua oder den Mod weitergegeben. Das genaue Profil enthält den Readervertrag unter `voxelReadLayout`. Die ausgegebenen Gridmetadaten haben 0,5 m Zellgröße und begrenzen den freigegebenen positiven Koordinatenraum auf 0 bis 32.767. Die ursprüngliche Cache-Eviction dekodiert gepackte Koordinaten als signed16.

Der Writer prüft zusätzlich die schreibende VoxelWorld-Bindung von `player_building_blueprints`. Seine DynamicStorage-Arena besitzt in der aktuellen Welt die Vtable `0x13b23a8`. Allocate und Free verwenden eine eigene Arena-Sperre. Die vollständig verfolgten normalen Speicher- und Arraywachstumspfade benötigen weder TLS noch die Threadregistrierung oder wartende ECS-Jobs. Auch der globale Fallback-Allocator verwendet dieselbe Arena-Vtable. Beide besitzen aktuell keinen externen Backing-Allocator. Die geänderten Bounds liegen in einem Inlinepuffer mit 16 Einträgen. Bei Erschöpfung verdichtet die ursprüngliche Funktion diese Bounds im vorhandenen Puffer. Der Adapter wiederholt diese Bindungsprüfungen unter dem Scene-Schutz.

Teiländerungen erweitern ihren Bereich auf vollständige 8³-Chunks. Alle eigenen Puffer entstehen vor dem Erwerb der Engine-Sperren. Der Adapter liest den Ausgangszustand, ersetzt ausschließlich angeforderte Zellen, ruft den ursprünglichen Writer und vergleicht anschließend den vollständigen erweiterten Bereich. Nach einem normalen, aber nicht bestätigten Schreibausgang stellt er den Ausgangszustand wieder her und liest ihn zurück. Nach einer nativen Exception erfolgt kein weiterer nativer Voxelaufruf in diesem Prozess, weil eine interne Sperre gehalten geblieben sein könnte. Diese Prüfung des Codes ersetzt den noch ausstehenden Live-Schreibtest nicht.

Im Live-A/B-Canary wurde zuerst eine Einzelzelle gelesen, danach derselbe Ursprung als 8³-Block und anschließend erneut als Einzelzelle. Nur bei unveränderter Einzelzelle wurde ihr Wert mit dem ersten Blockeintrag verglichen. `world-editor-native-ground-read-ab-20261019.json` dokumentiert 20 passende Vergleiche in belegtem Terrain, 20 gültige Cursor-Samples und 57 stabile Prop-Vergleiche zwischen `get_transform` und reflektierten ECS-Feldlesern. Es enthält keine protokollierten Fehler. Diese A/B-Prüfung bestätigt die Konsistenz unterschiedlicher Lesegrößen desselben nativen Readers, keine Weltmutation.

F8 erfasste und speicherte `capture-1.sfbp` im unveränderten SFBP-V7-Format. Die gesicherte Kopie `world-editor-native-capture-1-20261019.sfbp` hat 538 Bytes, 8 × 6 × 2 Zellen, davon 36 belegt, und null Props. Dieser Produktionslauf bestätigt den Lua-/DLL-Lesepfad einschließlich Export. Er ist kein Nachweis für Paste, Undo oder Prop-Erzeugung.

Die originalen Dienste `ecs::Tasks`, `ecs::EventQuery` und `ecs::VoxelGrid` sind in beiden Kontexten vorhanden. Die Untersuchung belegt noch keine registrierbare Callback- oder Commandqueue-ABI für diese opaken Dienste. Ihre Namen erteilen keine native Aufrufberechtigung.

## Originale Entity-Commandqueues

Die weitere Untersuchung hat einen vom alten Actorframe unabhängigen Originalpfad für Entity-Erzeugung und Löschung ergeben. Der EntityManager besitzt bei `+0xeb9f08` ein Array aus Commandqueue-Records mit 64 Bytes und bei `+0xeb9f10` dessen Anzahl. Der Manager-Konstruktor `0x984300` reserviert je Queue einen eigenen festen Buffer mit 1 MiB. Die fünf beobachteten Queues sind über die ursprünglichen Taskstates bei `manager + 0x528` und deren 224-Byte-Struktur eindeutig auf denselben Manager zurückgebunden.

`0x954250(queue, sourceEntityId, targetEntityId)` stellt einen 16-Byte-Löschbefehl ein. Der gemeinsame Appender `0x952400` richtet den Beginn auf 16 Bytes aus und verändert nur den vorhandenen Buffer sowie dessen belegte Länge und Befehlsanzahl. Er allokiert nicht und wartet nicht. Das dritte Argument ist die Ziel-Entity und kein Löschgrund. Ein Ziel von null löst keinen Befehl aus.

Der normale Consumer `0xa15960` sammelt und sortiert die Befehle, ruft `0xa144f0` auf und leert anschließend ihre Zähler. Der Typ-1-Zweig ruft `0x995ad0(manager, targetEntityId, 0, 0)` auf. Diese ursprüngliche Löschfunktion behandelt die Entity-Lebensdauer, die ursprünglichen Authority-Callbacks und Netzwerkänderungen. Die Building-Funktion `0x489300` verwendet denselben Appender. Ihr zusätzlicher Verlauf führt außerdem Spiel-Undo und Feedbackevents aus. Der Editor besitzt seine eigene Undo-Verwaltung. Ob jeder betroffene Prop nach Laden, Replikation und Speichern genau dem bisherigen Verhalten entspricht, wird davon nicht allein bewiesen.

Die originale Erzeugung `0x485a80` erzeugt Komponenten-Overrides für `CurrentTransform`, `DisableRandomLootOnDestroy` und `UsedItem`. Der gemeinsame Original-Appender `0x953800` kopiert den UUID-Deskriptor und die vollständigen Override-Payloads in denselben Commandbuffer. Er reserviert über den ursprünglichen Zähler bei `[queue + 0x38]` die tatsächliche Entity-ID. Das ist die später direkt abzufragende ID und kein asynchroner Timeline-Token. Der Typ-0-Consumer lädt die UUID-Ressource und führt die ursprüngliche ECS-Erzeugung durch `0x97b790` aus. Die separate 192-Byte-Templatequeue über `0x953da0` ist dafür nicht erforderlich.

Der GameServer-Aufruf `0xe47710` besitzt den ursprünglichen Shared-Lock bei `Scene + 1496` für die Ausführung über `0xe46070`, `0xa1e210` und `0xa1e2e0`. Die Systemausführung wartet auf ihre geplanten Aufgaben, bevor der Commanddrain läuft. Ein exklusiver Scene-Guard kann deshalb die kurze Einstellung in den vorhandenen Buffer schützen. Der neue Adapter muss zusätzlich die aktuelle Sitzung, die originale Queue-Vtable, Task-Backreferences, den Manager-Zählerzeiger, null aktive Systemjobs, Kapazität, Originalcode und die vollständige Veröffentlichung des Befehls prüfen. Die ursprünglichen Gates müssen vor jedem Warten und vor der Rücklesung wieder freigegeben werden.

Der reproduzierbare lokale Beleg liegt unter `entity-native-queue-20261019.py` mit der gleichnamigen JSON-Datei im bisherigen Capture-Verzeichnis. Er enthält 17 vollständig disassemblierte Prozeduren und 27.694 Originalbytes. Die verglichenen Livebytes im Prozess `32276` stimmten alle überein. In Mainclient und autoritativer Szene stimmten jeweils fünf Queue-Vtables, Zählerzeiger, Taskstate-Backreferences und doppelte Headerkopien. Diese Aufnahme war lesend und stellte selbst keinen Befehl ein.

## ItemInfo und Modkompatibilität

Ein frischer Export über den vorhandenen KFC4-Parser las 4.262 `keen::ItemInfo`-Ressourcen. Der unveränderte Modfilter erzeugte 1.774 eindeutige Placement-Rezepte. Die verwendeten Pfade für `equipment`, `placedEntity`, Placement-AABBs, Collider-Varianten und Materialfeedback sind in 1.0 vorhanden.

Alle sieben Lua-Dateien des Referenzpakets wurden vollständig gelesen, insgesamt 3.802 Zeilen. Die fünf zusätzlichen Paketdateien wurden ebenfalls gelesen. Neben Anpassungen an Schemapfad, LuaLS-Pfad, Dokumentation und dem lokalen `props`-Deklarationsfehler im Replace-Ablauf wurde die Abwesenheitsprüfung für destruktive Abläufe abgesichert. Einstellungen, Bedienung und Dateiformat bleiben erhalten. Dies war kein vollständiges Repository-Audit.

## Reproduzierbare Belege und offene Prüfung

Die konkrete Entwicklungsaktion liegt unter `dev/profiles/inspect/inspect_world_editor_operations.py`. Sie verwendet die vorhandenen PE-, Deskriptor-, Registrierungs- und Prozessleser. Lokale, ignorierte Ergebnisdateien liegen unter `dev/profiles/captures/enshrouded-client-25988036/`:

- `world-editor-operations-live-20261019.json` enthält Originalsysteme, Servicebindungen und stabile Live-Cursorbeobachtungen.
- `world-editor-voxel-callchain-20261019.json` enthält die aktuelle statische Lesekette und den Vergleich der ausgewählten Originalbytes.
- `world-editor-voxel-storage-live-20261019.json` hält getrennte originale Welt-/Storagezeiger und momentane Lockwerte fest. Ein beobachteter Lockwert ist kein erworbener Lock und kein Lebensdauernachweis.
- `world-editor-voxel-mutation-flow-20261019.json` enthält den früheren statischen Stand mit 18 vollständig zerlegten Prozeduren, 59 Compilerfragmenten und 33.733 Originalbytes. Die erfassten Livebytes stimmten mit der EXE überein. Die damalige Einstufung als nicht aufrufbar wurde für den separat geprüften Mode-6-Reader durch die folgenden Livebelege ergänzt.
- `world-editor-writer-allocation-review-20261019.json` ergänzt 36 vollständig gelesene Prozeduren, 4.330 Instruktionen und 16.013 Originalbytes der Speicher- und Arraypfade. `world-editor-writer-arrays-live-20261019.json` prüft die frisch aufgelösten Speicherbindungen gegen dieselbe Ownerkette. Diese beiden Aufnahmen änderten die Welt nicht.
- `world-editor-iteminfo-20261019.json` enthält den frischen ItemInfo-Export.
- `world-editor-native-ground-read-ab-20261019.json` enthält den abgeschlossenen lesenden Live-A/B-Canary mit 20 passenden Terrainvergleichen und 57 passenden Prop-Vergleichen.
- `world-editor-native-capture-1-20261019.sfbp` enthält die tatsächlich durch F8 erzeugte V7-Datei mit 96 Zellen, 36 belegten Zellen und null Props.

Die zusätzliche optionale ABI-12-Funktion `KfcRuntimeWorldEntityReadTransformForSession` unterscheidet nun bestätigte Abwesenheit von Lesefehlern. Status 6 entsteht nur bei bekanntem Handle in derselben Sitzung und zwei übereinstimmenden originalen Hash-Lookups. Ungültige oder veraltete Handles, unlesbare Daten, bewegte Allokationen, geänderte Owner und ein unklarer Schreibausgang ergeben andere Statuswerte. Lua erhält bei bestätigter Abwesenheit `nil, "entity-absent"`. Ein älterer boolescher Provider kann weiterhin vorhandene Props liefern, aber aus seinem Fehlschlag wird keine Abwesenheit abgeleitet.

Replace und Undo akzeptieren ausschließlich diesen ausdrücklichen Abwesenheitsgrund als Löschbestätigung. Der Restore-Eintrag entsteht bereits vor dem Löschversuch. Kann dessen Ergebnis nicht gelesen werden, bleibt das Journal erhalten. Ist das unveränderte Original nach einem nicht bestätigten Löschversuch noch vorhanden, wird kein Duplikat erzeugt. Das macht die Recoverygrenze eindeutig, schaltet aber keinen neuen nativen Delete- oder Spawnadapter frei.

Der lesende Produktionsaufruf des neuen Lua-/DLL-Pfads und F8-Capture sind bestätigt. Ein vollständiger Capture-/Paste-/Undo-Zyklus, Kollision, Replikation und Persistenz nach erneutem Laden benötigen getrennte Nachweise. Ein erfolgreicher Build oder eine private Speicherfixture ersetzt sie nicht.

Der hier ausgeführte lesende Aufruf war:

```powershell
$env:PYTHONPATH = 'A:\Github\shroudforge\dev\profiles\captures\world-editor-python-dependencies'
python dev/profiles/inspect/inspect_world_editor_operations.py 'C:\Program Files (x86)\Steam\steamapps\common\Enshrouded\enshrouded.exe' --capture dev/profiles/captures/enshrouded-client-25988036/native-function-discovery --profile modules/game-runtime/profiles/enshrouded/client/25988036.json --pid 31620 --out dev/profiles/captures/enshrouded-client-25988036/world-editor-operations-live-20261019.json
```

Die PID gehört zur dokumentierten Aufnahme und muss für eine neue Spielsitzung neu ermittelt werden. Die Python-Abhängigkeiten entsprechen der vorhandenen Discovery-Anforderung und liegen hier in einem lokalen ignorierten Verzeichnis.

## Historische Prüfungen am untersuchten Versuchsstand

| Prüfung | Ergebnis |
| --- | --- |
| `dev/build/build_kfc_runtime.ps1` | Erfolgreich, Runtime-DLL gebaut |
| `dev/build/build_modloader.ps1 -UseInstalledDependencies -SkipUiBuild` | Erfolgreich, Rust-Workspace im Releaseprofil gebaut, Frontend nicht neu gebaut |
| `cargo test -p shroudforge-api --test bundled_lua --offline` | Vier Tests bestanden, einschließlich echter World-Editor-Module und des Ein-Zellen-Canarys mit privaten Erfolgs-, Fehler-, Sitzungswechsel- und Recoveryfällen |
| `kfc-runtime-test-ecs-read.exe` | 252 Checks bestanden, einschließlich getrennter Client-/Authority-Fixtures, dynamischer Registrierungen, Cursor, gesicherter Skalierung, Prop-Lesestatus und Voxelkoordinaten-/Chunkgrenzen |
| `kfc-runtime-test-prop-query.exe` | 35 Checks bestanden |
| `kfc-runtime-test-world-context.exe` | 53 Checks bestanden |
| Aktuelle Manifestschemas und Release-Lua-Tokenprüfung | Bestanden |
| Python-Syntax, Rustformat und gezielter Git-Diffcheck | Bestanden |
| Live-Lua-/DLL-Canary | 20 stabile Terrainvergleiche, 20 gültige Cursor-Samples, 57 stabile Prop-Vergleiche, keine protokollierten Fehler |
| F8-Capture und V7-Export | 538-Byte-Datei mit 96 Zellen, 36 belegt, null Props |

Die nativen Tests verwenden eigene private Speicherfixtures und öffnen keinen Spielprozess. Die damaligen Artefakte wurden anschließend in der Testinstallation geladen und mit dem getrennt dokumentierten lesenden Live-Canary geprüft. Die späteren manuellen Terrainänderungen und die anschließende Verwerfung des Backends sind im aktuellen Vorspann eingeordnet. Die 37 Einstellungen und fünf Gruppen der Referenz wurden unverändert übernommen. Nebenläufig entstandene Updater- und Übersetzungsänderungen sind nicht Teil dieser Migration.
