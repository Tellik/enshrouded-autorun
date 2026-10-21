# World Editor: registrierte Originaloperationen am 21. Oktober 2026

## Auftrag und untersuchter Lauf

Ziel sind Prop-, Terrain- und Undo-Aufträge aus Lua über originale Spieloperationen. Neue Hooks, Codepatches, feste Funktions-RVAs und manuell gewählte Funktionshashes sind dafür ausgeschlossen. Alte Adressen dienen ausschließlich als Untersuchungsanker.

Untersucht wurde der laufende Client `enshrouded.exe`, PID 28372, Build 25988036, ECS-Weltsitzung 5. Die Untersuchung verglich Originalcode aus der EXE mit den entsprechenden geladenen Codebereichen. Solche Vergleiche belegen die untersuchte Implementierung. Sie ersetzen keinen erfolgreich abgeschlossenen Weltauftrag.

Die aktuellen Codezuständigkeiten stehen weiterhin in [code-ownership.en.md](../../development/code-ownership.en.md). Dieses Dokument hält einen datierten Untersuchungs- und Implementierungsstand fest.

## Tatsächlich ausgeführte Live-Prüfungen

| Prüfung | Ergebnis in dieser Sitzung | Weltänderung |
| --- | --- | --- |
| `world-editor.registered-impact-ab` | Vier Fälle für `IsSameEntity` bestanden, einschließlich Null und voller uint32-Werte | Keine |
| `world-editor.api-read-ab` | 37 Cursorlesungen, 20 gültige Cursor, 37 stabile Propvergleiche, 9 stabile Voxelvergleiche, keine gemeldeten Fehler | Keine |
| `world-editor.typed-inputs`, alte Diagnose | Versionspaare lesbar, Bitmasken fehlerhaft als Nichtskalare abgewiesen | Keine |
| Korrigierte typisierte Diagnose, Schema 2 | Zustand `observed`, Cursor- und Consumerbitmasken als benannte Werte lesbar | Keine |

Die API-Leseprüfung meldete null passende Client-/Authority-Cursorvergleiche. Daraus wird kein positiver Abgleich dieser beiden Welten abgeleitet. Die früher genannten 20 Terrain- und 57 Propvergleiche gehören zum Lauf vom 19. Oktober.

Die korrigierte Diagnose kopiert ausschließlich bekannte Maskenfelder in begrenzte Namenslisten. Leere Masken sind gültig. Erfolgreiche Runden verlangen nun lesbare Eingabemasken und Consumerfelder zusätzlich zu stabilen Versionspaaren. Das ist weiterhin eine Leseprüfung. Sie belegt keine Zustellung oder Verarbeitung eines selbst veröffentlichten Eingabeauftrags.

Die Diagnoseeinstellungen wurden nach den temporären Prüfungen wieder deaktiviert. Der monotone Anforderungszähler wurde beibehalten. Die für den Live-Test vorübergehend angelegten benutzerdefinierten Jobdateien wurden entfernt, ihre Exporte bleiben erhalten.

## Prop-Löschung über die registrierte Originaloperation

Die Originalregistrierung `impact::ImpactSystem` enthält `destroyEntity` mit einem Argument. Der vollständig untersuchte Aufrufpfad liest eine Entity-ID aus eigenen VM-Argumenten und hängt einen 16-Byte-Befehl vom Typ 1 an die originale Entity-Commandqueue an. Der gewählte Pfad enthält keine TLS-Abfrage, Speicherallokation oder Warteoperation. Der originale Commanddrain führt die Löschung später aus.

Die neue Anbindung besitzt folgende Zuständigkeiten:

| Datei im Modul `game-runtime` | Handlung und Aufrufer |
| --- | --- |
| `native/impact/owned_operation_arguments.h` | Gemeinsame eigene VM-Daten für die getrennt zugelassenen Adapter. Verwendet von ID-Vergleich und Prop-Löschung |
| `native/impact/resolve_registered_operation.*` | Originaloperation anhand ihres Namens in der registrierten Map finden und unveränderte Registrierung prüfen |
| `native/ecs/resolve_entity_command_queue.*` | Über den benannten Dienst `ecs::Tasks` originale Queue, Besitzer, Workerbezüge und feste Puffer validieren |
| `native/impact/submit_entity_destruction.*` | Ausschließlich den typisierten originalen `destroyEntity`-Aufruf zulassen und dessen exakten Queueeffekt prüfen |
| `native/building/remove_building_prop.cpp` | Bestehenden Handle-Löschpfad an den Adapter anbinden und anschließend echte Abwesenheit bestätigen |

Der Lua-Aufruf bleibt `runtime.world.entity.destroy(entityHandle)`. Er verwendet die vorhandene Rust-Brücke und `KfcRuntimeWorldEntityDestroyHandle`. Die Lua-Verfügbarkeitsprüfung verlangt für diese Handlung ihren eigenen nativen Kontext und den Prop-Lesepfad. Sie hängt nicht mehr zusätzlich an noch fehlenden Spawn-, Place- und Finish-Operationen.

Vor dem Originalaufruf werden die Sitzung gepinnt und das originale Scene-Gate exklusiv erworben. Das vorhandene opaque Handle muss mit seiner gespeicherten Entity-ID, Generation, Sitzung und originalen Tabellenmitgliedschaft übereinstimmen. Die Queue stammt aus der aktuellen Originalregistrierung. Ihre Besitzer, Puffer, Kapazität, Metadaten und Nachbarqueues werden wiederholt geprüft. Der Adapter verändert weder die Systemkennung noch einen Schedulerframe.

Die eigenen Query-Daten wählen den belegten Originalzweig mit Quell-ID null. Die Originalfunktion liest dabei keine Entityliste. Nach dem Aufruf müssen Eingaben und Guardbereiche unverändert sein, Ergebnis und Rückgabezeiger stimmen und exakt ein Befehl für die erwartete Ziel-ID vorliegen. Ein unklarer Ausgang sperrt weitere native Schreibzugriffe dieser Weltsitzung. Es gibt danach keinen automatischen Rückfall auf einen zweiten Ausführungspfad.

`Submitted` bedeutet ausschließlich, dass dieser Befehl korrekt eingereiht wurde. Erst nachdem alle Scene- und Sitzungsgates freigegeben wurden, prüft der vorhandene sitzungsgebundene Prop-Leser die Entity. Nur Status 6, bestätigte Abwesenheit beziehungsweise geänderte Generation, gilt als abgeschlossene Löschung. Ein Lesefehler gilt nicht als Abwesenheit. Bleibt der Ausgang nach dem begrenzten Warten unbestätigt, werden weitere native Schreibzugriffe ebenfalls gesperrt.

Die abschließende Prüfung fand zwei weitere Fehlerfälle und schloss sie vor dem Liveversuch. Vorübergehend fehlende Besitzer können beim Completionreader ebenfalls Status 4 auslösen. Die Sperre für einen bereits publizierten Auftrag wird deshalb ausschließlich anhand seines bestehenden Epoch unter dem ECS-State-Mutex gesetzt und benötigt keine erneute Besitzerauflösung. Ein tatsächlich neuer Epoch bleibt unberührt. Auch Exceptions beim allokierenden Completionreader sowie nach Eintritt in die Originaloperation führen zur Sperre. Ein solcher Ausgang fällt niemals auf einen zweiten Löschpfad zurück.

**Live-Grenze:** Während des ersten dafür nötigen Spielneustarts aktualisierte Steam auch das Spiel. Die ursprünglichen Live-Leseprüfungen gehören weiterhin zu Build 25988036. Nach erneuter Profilprüfung und DLL-Einsatz wurde die unten dokumentierte Kiste in Build 26082781 erfolgreich gelöscht. Reines `destroyEntity` legt keinen Ingame-Undo-Eintrag an.

## Prop-Spawn und originale Baukomponenten

Der registrierte `spawnEntity`-Callback besitzt 13 Argumente. Seine Eingaben enthalten keine freie Baukomponenten-Override-Liste und keinen Tracking-ItemId-Eingang. Der originale Bauhelper ergänzt dagegen ausdrücklich `CurrentTransform` und `DisableRandomLootOnDestroy` sowie bei gesetztem Trackingwert `UsedItem`.

Die konkrete gelesene Prop-UUID `4d47cb9eb0a72b46 / 62f20d8c90e7c3b2` enthält `UsedItem = 2610555646` bereits als originalen Template-Default. Der Originalconsumer kopiert solche Defaults in die neue Entity, wenn kein Override für denselben Typ vorliegt. Der fehlende Loot-Marker lässt sich daraus jedoch nicht ableiten.

Die erfasste Authority-Template-Map wurde vollständig auf ihre Defaultlisten geprüft. Von 12.267 belegten Definitionen besitzen 652 `UsedItem`. Keine besitzt `DisableRandomLootOnDestroy`, entsprechend besitzt keine beide benötigten Defaults. Es traten keine Lesefehler auf. Die Mapheader, Schlüssel, Werte und Belegungsdaten wurden wiederholt gleich gelesen. Dieser Suchumfang bezieht sich auf diese erfasste Map und ersetzt keine atomare Weltaufnahme.

Die beiden Einträge der originalen Queuevtable sind löschende Destruktoren. Sie sind kein zusätzlicher Spawn-/AddComponent-Aufruf. Die aktuellen ComponentRegistry-Einträge der beiden genannten Komponenten enthalten ebenfalls keine ausführbaren Callbackslots. Der noch fehlende Vertrag ist damit die veröffentlichte Einreichung der originalen Bau-Overrides. Der bestehende Lua-Spawn wird nicht auf einen unvollständigen Template-Spawn umgestellt.

## Terrain und exakte Voxelwerte

Das originale `ecs::VoxelGrid` besitzt sechs veröffentlichte Callbackslots. Sie lesen Zellen beziehungsweise Ursprung, prüfen Bereitschaft und prüfen Belegung. Slot 5 erhält keinen Zellpuffer und ist kein Writer. Die beiden gefundenen Originalkonstruktoren registrieren dieselben sechs Funktionen.

Ein echter interner Schreibpfad wurde aus `player_building_blueprints` bis zum Store-Commit verfolgt. Er verarbeitet originale 8³-Blöcke, besitzt ein eigenes Store-Gate und aktualisiert Dirtyzustand sowie Folgeverarbeitung. Er ist kein veröffentlichter Writer des VoxelGrid-Interfaces. Seine interne Adresse wird nicht als neue Laufzeitoperation eingebaut.

Zusätzlich ist `terraforming` mit neun Argumenten namentlich registriert. Sein originaler Add-Zweig erreicht den echten Store über Readback, Brushberechnung und Commit. Dafür benötigt er einen größeren eigenen Impactkontext, originale benannte Dienste, einen geladenen ItemId-Kontext und originale Ausgabe-/Zeitkontexte. Das Material wird im Add-Zweig aus dem originalen Item ermittelt. Der Callback nimmt keinen beliebigen Puffer exakter Terrainzellen entgegen. Ein Add-Brush ist daher kein Ersatz für den bestehenden F7-Pastevertrag mit exakten Zellwerten.

Ein Aufruf ohne Playerkontext legt dort keinen bestätigten Ingame-Undo-Eintrag an. F7s Prüfung auf einen verifizierten exakten Terrainwriter bleibt deshalb bestehen.

Zusätzlich wurde die originale Schedulerreihenfolge aus dem laufenden Client gelesen. Der gespeicherte Plan umfasst 258 Gruppen und blieb über zwei Lesungen bytegleich. `clear_hit_candidates` läuft in Gruppe 106, `impact_simulation` und `impact_hit_conclusion` erst in Gruppe 241. Der Add-Callback veröffentlicht regulär in die Hauptqueue. Ein zwischen Frames erzeugter HitCandidate würde damit im nächsten Clear vor seinem Consumer verworfen, obwohl der Terraincommit bereits synchron erfolgt wäre. Der Scene-Guard allein stellt für diesen Terrainaufruf folglich noch keinen vollständigen Ausführungskontext her. Ein allgemeiner Add-Brush wird auf dieser Grundlage ebenfalls nicht freigegeben.

Für Build 26082781 wurden außerdem der vollständige originale `VoxelWorld`- und `Store`-Objektkörper sowie fünf Allocator- und drei Reserve-Callbackverträge untersucht. Keiner veröffentlicht einen Writer für einen beliebigen Zellbereich. Eine interne Terrain-Clear-Liste setzt ausschließlich ausgewählte Materialien auf null. Ihr Consumer liest einen anderen `VoxelWorld`-Zeiger als das registrierte Payload. Sie ist daher kein F7-Publisher für diesen World-Kontext und enthält keinen Undo-Auftrag. 9.205 Bytes aus 21 untersuchten Prozeduren stimmen im laufenden PID 31676 mit der Original-EXE überein. Der vollständige Nachweis liegt unter `dev/profiles/captures/enshrouded-client-26082781-static/voxel-world-store-retained-interface-contract-20261021.json`.

## Originaler Undo-Pfad

Der untersuchte Originalpfad lautet:

`PlayerInput → ServerInputConsumer → player_control → originale ActionSequence → actor::ChangeVoxel(Undo) → player_building_undo`

Im aktuellen Build ist `BuildingUndo` der Eingabewert 40. Wert 37 bezeichnet `Glider`. Der alte hookbasierte `client-player-input-v1`-Adapter verwendet Wert 37 und alte Feldoffsets. Er ist für den aktuellen Build nicht als Undo-Einstieg geeignet und wird dafür nicht aktiviert.

Der Originalconsumer verarbeitet einen gehaltenen Undo-Press einmal. Er prüft `logicalStates[40]` und `consumedInputs[40]` und hält den Verbrauch bis zum Release fest. Dieser digitale Auftrag besitzt keine `VersionedData`-Version. Das Verändern einer beliebigen Versionsnummer schafft deshalb keinen bestätigten Undo-Auftrag.

Für andere versionierte Bauaktionen ist der Originalvergleich lediglich `input.version != consumed.version`. Ein erneut eintreffender Clientwert kann einen selbst überschriebenen Authoritywert ersetzen. Wiederholtes Schreiben bis zu einem vermeintlichen ACK kann deshalb erneut einen alten Wert als neuen Auftrag erscheinen lassen.

Offen bleibt der sichere Publishervertrag der originalen Mainclientquelle einschließlich Press, eindeutig zugehörigem Release und Ergebnisbestätigung. Ein geratenes Systemframe oder ein manuell gewählter ActionSequence-Hash wird dafür nicht verwendet. Der bestehende World-Editor-Recoveryjournalpfad bleibt erhalten.

Die erneute Prüfung in Build 26082781 verfolgte die Mainclientquelle bis zum originalen Device-Eventpump. Der Root-Vtableslot `+0x70` ist ein vorhandener Consumer, verarbeitet Eingaben aber unmittelbar ohne eigenes Mainclient-Owner-Gate. Die originale Plattformqueue besitzt 104-Byte-Events und einen Ring-Lock. Dessen Schutz endet vor Teilen der restlichen Deviceverarbeitung und pinnt die Queue-Objektlebensdauer nicht. Der Original-Enqueue ist bisher nur durch direkte interne Aufrufe erreichbar, nicht über eine erhaltene benannte oder virtuelle Producerregistrierung.

Der original registrierte Fenster-Callback bietet ebenfalls keinen zulässigen Ersatzauftrag. Echte Tastenevents werden über `WM_INPUT` und `GetRawInputData` mit einem gültigen `HRAWINPUT` verarbeitet. `WM_KEYDOWN` und `WM_KEYUP` gelangen dagegen zum DefWindowProc-Zweig. Es wurden weder RawInput-Handles erfunden noch Inputevents injiziert. 29 untersuchte Originalprozeduren stimmen im aktuellen Prozess vollständig mit der EXE überein. Das schafft weiterhin keinen freigegebenen Lua-Publisher.

## Große Blueprints und die native Undo-Grenze

Die originale Undo-Historie besitzt fünf Gruppen mit jeweils höchstens 32 gespeicherten Operationen. Das sind keine fünf Items. Eine Terrainoperation kann einen eigenen Puffer vieler Zellen enthalten. Der originale Pool, Gruppenring, Inventarzustand und variable Terrainpuffer haben zusammengehörige Lebensdauern. Beliebige Operationen über die feste Gruppenbegrenzung hinweg in diese Strukturen einzutragen ist kein freigegebener Bauauftrag.

Der World Editor verwaltet bereits einen eigenen `undo_state` pro Paste mit dem vorherigen Zellpuffer, den neuen Entityhandles, ersetzten Props und getrennten Fortschrittsindizes. `operations.lua` fügt vor jedem Löschversuch das vollständige Originalprop ein. `history.lua` entfernt beim Rückgängigmachen die gespeicherten neu gesetzten Handles direkt über `runtime.world.entity.destroy`, restauriert vorherige Terrainwerte über den nativen Writer und stellt ersetzte Props über Spawn wieder her. Dieser Editorvorgang verlangt keine native Undo-Taste pro Item. Er ist keine atomare Batchoperation und gegenwärtig auch keine bestätigte Serie mehrerer nativer Löschaufrufe. Die fünf nativen Historygruppen begrenzen seine Listen nicht.

Das Journal bleibt bei fehlender Bestätigung oder Konflikten erhalten. Es ist aktuell ein sitzungsgebundener Zustand. Der vorliegende Lauf beweist weder dauerhaftes Journal-Recovery nach einem Clientneustart noch einen vollständigen großen Paste-/Undo-Zyklus. Dafür fehlen weiterhin der exakte Terrainwriter und die korrekte Prop-Wiederherstellung. Direktes Entity-Destroy allein restauriert keine Terrain-, Block- oder Voxelzellen.

## Lokale Prüfungen und ausstehender Live-Schritt

Die privaten Queue- und Submissionfixtures verwenden eigene Objekte und kontrollierte Callbackfunktionen. Sie öffnen keinen Spielprozess und beweisen keinen abgeschlossenen Ingameauftrag.

- Queuefixture: 51 Prüfungen bestanden.
- Submissionfixture: zuletzt 1.890 Prüfungen bestanden, darunter Eingabeveränderung, Guardverletzung, falscher Queueeffekt, native Exception, C++-Exception vor und nach dem Originalaufruf sowie Sceneverlust.
- Diagnosefixtures: vier gezielte World-Editor-Tests bestanden.
- Rust-Fehlermeldungen: ein gezielter Test bestanden. Veraltete Sitzung und unklarer Originalauftrag werden getrennt gemeldet.
- Lua-Impact-Eingaben: zwei gezielte Tests bestanden. Nicht zugelassene Operationen bleiben dort abgewiesen.
- Bestehende World-Kontextfixture: 53 Prüfungen bestanden.
- Service- und Sitzungsfixture mit echter Handle-Auflösung: zuletzt 527 Prüfungen bestanden, einschließlich Generation, Epoch, Tabellenmitgliedschaft, unveränderter Fehleroutputs und Sperre bei vorübergehend verlorener Besitzerauflösung.
- Resourcefixture: 795 Prüfungen bestanden.

Der native Runtimebuild und der Releasebuild des Workspace wurden erfolgreich ausgeführt. Die gebauten DLLs liegen unter `build/x64`. Die vorhandenen Compilerwarnungen zu Padding und Parserimports bleiben bestehen.

Die unten bestätigte Entity-Abwesenheit nach dem Originaldrain ist der positive Prop-Löschungsnachweis. Der Nutzer hat eine Kiste als entbehrliches Testobjekt bestimmt. Spawn mit korrekten Itemkomponenten, exakter F7-Terrainpaste und nativer Undo sind durch die bisherigen Prüfungen weiterhin nicht als vollständig umgesetzt belegt.

## Neustart, Steamupdate und reparierte Startbasis

Steam spielte am 21. Oktober um 10:15:59 UTC beim Beenden des ursprünglichen Clients Build 26082781 ein. Die aktuelle EXE besitzt SHA256 `d64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882`. Die neue KFC trägt Version `1273043` vom 20. Oktober, die vorhandene Arbeitsbackup-Datei noch `1269579` vom 16. Oktober. Der Start wurde deshalb vor dem Laden einer Welt mit einem korrekt abgewiesenen Baseline-Versionskonflikt beendet.

Ein vorübergehender Rücktausch der Rust-Runtime änderte den Fehler nicht. Eine neue Rustregression ist damit nicht belegt. Beide neuen DLLs wurden anschließend wieder eingesetzt.

Die aktuellen EXE-, KFC- und Resource-Dateien wurden gegen das lokale Steamdepotmanifest `1203621_4708593645837114731.manifest` geprüft. Vollständige Dateigrößen und Content-SHA1 sowie sämtliche 248 Chunks stimmen. Die aktuellen Dateien sind damit Steam-original. Der vorhandene Updaterauftrag `capture-verified-gamefile-originals` nahm das Paar unter `shroudforge/backups/originals/client/35338312-1792577757/` unverändert auf. Die beiden alten Arbeitsbackups wurden unter `shroudforge/backups/legacy-working-baselines/before-26082781-122526/` erhalten. Die kanonische Aktion `shroudforge prepare` legte passende neue Arbeitsbackups an und beendete die Modvorbereitung erfolgreich.

Der danach gestartete Client PID 26184 trägt den Fenstertitel `Enshrouded` und hat beide neuen DLLs geladen. Seine nativen Zugriffe bleiben mangels eines exakt validierten Profils für den neuen Build zunächst gesperrt. Der Startfehler ist behoben. Vor dem Kistentest müssen die Besitzer-, Gate-, Registrierungs- und Callbackverträge für Build 26082781 erneut bestätigt werden.

## Erneute Profilprüfung für Build 26082781

Die neue EXE wurde anschließend statisch und in PID 26184 lesend geprüft. Alle 19 profilgebundenen Bildanker wurden einzeln neu hergeleitet. Es wurde keine gemeinsame Adressverschiebung angenommen. Alle 12.646 Reflectionlayouts sind unverändert. Die aktuelle Besitzerkette, die bereite Authorityszene, originale Threadregistrierungen und Callbackkontexte stimmen mit den neuen Konstruktoren überein.

Die Prüfung erfasste 640 Komponentenregistrierungen, 69 benannte DI-Dienste, 16 unabhängige Entitylayouts und sämtliche 4.207 ItemInfo-Zuordnungen der aktuellen Map. GameSettings besitzt genau das erwartete mutable Binding des originalen Systems `admin_player`. Die ursprünglichen Trackerbytes und A/B-Kategorien wurden erneut geprüft. 21 relevante Originalprozeduren mit insgesamt 17.511 Bytes stimmen mit der geladenen EXE überein. Die externen Aufnahmen sind keine atomaren Snapshots. Der Produktcode muss seine bestehenden Besitzer- und Mitgliedschaftsprüfungen unter den originalen Guards wiederholen.

Das neue Scene-Shared-Gate umfasst weiterhin die Systemausführung und den originalen Commanddrain. Der vollständige untersuchte Aufrufweg führt von der geschützten Sceneschleife über die Ausführung bis zum Drain, bevor das Gate freigegeben wird. Die aktuelle `destroyEntity`-Registrierung besitzt weiterhin genau ein Argument. VM-, Query-, Iterator- und Queueformate sowie der 16-Byte-Löschbefehl sind erneut am Originalcode belegt. 55 primäre Queue-/Registrierungsprüfungen meldeten keine Fehler. Spätere ungegatete Queuezähler änderten sich erwartungsgemäß und werden nicht als Lebensdauergarantie verwendet.

Das kanonische Clientprofil `modules/game-runtime/profiles/enshrouded/client/26082781.json` verwendet diese neu geprüften Besitzer- und Guardanker. Das zwischenzeitlich hinzugekommene Profil mit alten Ankern wurde vorher als Untersuchungsartefakt erhalten. Die Referenzen zeigen auf diesen datierten Nachweis. Der Attributinterpreter wurde für diesen Build nicht semantisch geprüft. Sein geerbtes `attributeCalculationModel` und die zugehörige Freigabe wurden deshalb nicht übernommen. Es wurden keine neuen Hook-, Patch- oder festen Operationsdispatch-Einträge ergänzt.

Die Aufnahme dieses Profils allein belegt keinen Lua-Löschauftrag. Erst der nachfolgende Liveversuch bestätigt die konkrete Löschung. Spawn, exakter Terrainpaste und nativer Undo bleiben offen.

Die korrigierte DLL wurde um 12:44:44 Berliner Zeit bei vollständig beendetem Spiel eingesetzt. Ihr SHA256 lautet `20db62c364b5f8a3e308e69ece4285c92fd850b8eb04f413b2c8dd2d7f4fd6e4`. PID 31676 hat beide Runtime-DLLs geladen. In der geladenen Welt melden ECS und ResourceService Bereitschaft, der neue exakte Profilname wird erkannt. Die temporäre Kistenprüfung ist zunächst ausschließlich lesend und meldete in ihrer ersten Aufnahme keinen Prop am vorherigen Cursorpunkt. Diese Aufnahme bestätigt keinen Löschauftrag.

## Erfolgreicher einzelner Lua-Löschversuch

Der Nutzer platzierte die Testkiste selbst und bestätigte das Anvisieren mit dem Bauhammer. Die lesende Prüfung fand am zugehörigen Cursorpunkt genau eine Kiste mit geladenem ItemInfo-Namen `Prop_Storage_24_T1_Box`. Identität und vollständige Transformation wurden wiederholt gleich gelesen.

| Eigenschaft | Beobachteter Wert |
| --- | --- |
| Prozess und Sitzung | PID 31676, Sitzung 5 |
| Opaque Handle und Entity-ID | 2, 1031 |
| Item-ID | 213939880 |
| Template-UUID | `454b022a65baa890 / d46cc0b82cb9b79c` |
| Position in Weltmetern | `3749.611083984375, 842.8125, 1433.54052734375` |
| Rotation und Skalierung | `0, 0, 0, 1` und `1, 1, 1` |

Der einmalige temporäre Lua-Job prüfte vor dem Aufruf Sitzung, exakt diese Identität und Transformation sowie die Nähe des aktuellen Cursors zur beobachteten Kiste. Ein persistenter Versuchsmarker wurde vor dem Eintritt in die native API geschrieben. Genau ein Aufruf von `runtime.world.entity.destroy(2)` gab `true` zurück. Die anschließende Lua-Lesung gab ausdrücklich `entity-absent` zurück. Die Sitzung blieb 5. Der Bericht lautet `native-delete-confirmed`, `attempts=1`, `after_present=false`. Der Nutzer bestätigte zusätzlich, dass die Kiste sichtbar verschwunden ist.

Damit ist die konkrete native Prop-Löschung über die originale, anhand ihres Namens aufgelöste Operation live bestätigt. Weder ein Spielneustart mit Save-Reload noch die native Wiederherstellung der Kiste wurde geprüft. Dieser Versuch bestätigt keinen Spawn, Terrainpaste oder nativen Undo. Die Diskrepanz zwischen zeitweise positiver nativer Operationsverfügbarkeit und weiterhin negativem Lua-`has` wird gesondert untersucht.

Der nachfolgende rein lesende Verfügbarkeitsvergleich wechselte die Aufrufreihenfolge über 40 Runden. In allen 20 Runden mit `has` zuerst war `has=true` und die unmittelbar folgende native Verfügbarkeitsabfrage `false`. In allen 20 Runden mit nativer Abfrage zuerst war diese `true` und das folgende `has=false`. Prop-Query war 40-mal bereit, GetTransform-Status ebenfalls. Damit ist das zuvor negative `has` kein Beleg für einen veralteten Rustzweig. Beide Abfragen prüfen den ursprünglichen exklusiven Scene-Guard jeweils neu und besitzen keinen Readinesscache. Verfügbarkeit reserviert keinen Ausführungskontext für den nächsten API-Aufruf. Ein bereits originaler Spielthread kann das Gate zwischen Abfragen übernehmen. Diese Erklärung ist eine Schlussfolgerung aus Reihenfolge-A/B und Guardcode, keine Ablaufverfolgung des Threadschedulers.

Die Diagnose wurde danach gestoppt und ihre vorherigen Einstellungen wiederhergestellt. `enabled` und `sessionActive` sind wieder `false`. Der monotone Anforderungszähler steht auf 7. Nur die vier für diesen Versuch neu angelegten benutzerdefinierten Jobdateien wurden entfernt. Versuchsmarker und Berichte bleiben im vorgesehenen Diagnoseexport erhalten.

Nach dem zusätzlichen Verfügbarkeitsvergleich wurden auch dessen beide temporären Jobdateien entfernt und die vorherigen Diagnoseeinstellungen erneut wiederhergestellt. Der Anforderungszähler steht nun auf 8.

## Lokale Untersuchungsartefakte

Die Rohdaten liegen unter den ignorierten Entwicklungscaptures und den installierten Diagnoseexporten. Sie sind kein zweiter Laufzeitprofilkatalog.

- `dev/profiles/captures/world-editor-runtime-ab-20261021/`: aktuelle Lua-Berichte einschließlich typisierter Masken.
- `dev/profiles/captures/world-editor-runtime-ab-20261021/crate-native-delete-confirmed-26082781.json`: positiver einzelner Lua-Löschversuch mit Abwesenheits- und Sitzungsbestätigung.
- `dev/profiles/captures/world-editor-runtime-ab-20261021/destroy-readiness-order-live-26082781.json`: 40 rein lesende Runden mit wechselnder Aufrufreihenfolge.
- `dev/profiles/captures/enshrouded-client-26082781-static/building-task-spawn-publisher-contract-20261021.json`: vollständige originale Bau-, Inventar- und Undo-Kette sowie Grenzen der gespeicherten Historie.
- `dev/profiles/captures/enshrouded-client-26082781-static/mainclient-input-publisher-admission-20261021.json`: Mainclient-Inputqueue, Besitzerlebensdauer und ausgeschlossene Fenster-/Input-Consumer.
- `dev/profiles/captures/enshrouded-client-26082781-static/owners-gates-static-contract-20261021.json`: neue Besitzer-, Guard-, Registry-, Resource- und ItemMap-Verträge.
- `dev/profiles/captures/enshrouded-client-26082781-static/impact-queue-destroy-contract-20261021.json`: aktueller Originalaufruf, Queueformat und geschützter Drainpfad.
- `dev/profiles/captures/enshrouded-client-26082781-static/steam-original-files-verified-20261021.json`: vollständige Dateiprüfung und 248 originale Steamchunks.
- `dev/profiles/captures/enshrouded-client-25988036-static/impact-spawn-undo-live-20261021.json`: 107 aktuelle Originalregistrierungen und benannte Dienste.
- `dev/profiles/captures/enshrouded-client-25988036-static/impact-spawn-undo-live-code-identity-20261021.json`: Vergleich der untersuchten Codebereiche mit der geladenen EXE.
- `dev/profiles/captures/enshrouded-client-25988036-static/registered-entry-queue-lifecycle-live-20261021.json`: originale Queuebesitzer und Lebensdauer.
- `dev/profiles/captures/registered-voxel-interface-contract-20261021.json`: getrennte Verträge der sechs veröffentlichten Voxelcallbacks.
- `dev/profiles/captures/enshrouded-client-25988036-static/terraforming-named-operation-contract-20261021.json`: Add-Argumente, Dienstbesitzer, Store-Commit und Grenzen.
- `dev/profiles/captures/enshrouded-client-25988036-static/terraforming-hit-scheduler-order-live-20261021.json`: tatsächliche Reihenfolge von Hit-Clear und Consumer.
- `dev/profiles/captures/enshrouded-client-25988036-static/registered-building-undo-input-contract-20261021.json`: Press-/Consume-/Releasevertrag und fehlende Publishergrenze.
- `dev/profiles/captures/enshrouded-client-25988036-static/registered-prop-spawn-semantics-contract-20261021.json`: originale Bau-Overrides, Template-Defaultprüfung und Queuevtable-Rollen.
- `dev/profiles/captures/enshrouded-client-26082781-static/voxel-world-store-retained-interface-contract-20261021.json`: vollständige World-/Store-Objektkörper, Reserve-Callbacks und Gegenbeleg zur getrennten Terrain-Clear-Liste.

## Blueprint-Zyklus im laufenden Client

Die aktive Bibliothek enthält `capture-1.sfbp` mit 12 Zellen, davon sechs belegt, und null Props. Die Datei ist ein gespeichertes V7-Blueprint mit den Abmessungen 1×6×2. Ihr SHA-256 vor dem nächsten Live-Zyklus lautet `F857804123AE011A9D909265F380F302FA5E4FEDDE28824E61D67C0C50D9D995`. Das ist ein vorhandener Capture- und Speichernachweis aus dem vorherigen Lauf, keine neue F8-Aufnahme in Sitzung 5.

Der aktuelle Lauf lud dieses Blueprint in PID 31676, Sitzung 5. Der Nutzer löste F7 aus. Der Logeintrag um 10:54:27 UTC zeigt, dass `runtime.world.voxel.write` vor jeder Weltänderung abgewiesen wurde: „No verified native ECS terrain write operation is available for this world session. No world changes were sent.“ Damit entstand kein Paste-Journal, das F4 rückgängig machen könnte. Für diese Sitzung ist kein abgeschlossener F7-/F4-Zyklus belegt.

Am 21. Oktober um 11:16:17 UTC markierte der Nutzer eine neue Auswahl von `3731,843,1441` bis `3736,845,1446` und löste F8 aus. Der Leser erfasste 605 Terrainzellen. Danach brach `runtime.world.entity.query_props_in_bounds` sofort mit `native recipe-bounds prop query failed` ab. Das Log enthält keinen erfolgreichen Export. `editor-state.txt` meldet `saveState=error`, `processState=error` und `processPhase=Scanning props`. Im Blueprintordner blieb ausschließlich das vorherige `capture-1.sfbp` mit Zeitstempel 11:18 Uhr Berliner Zeit aus dem früheren Lauf. Es entstand kein neues Blueprint und keine Weltänderung.

Für Sitzung 5 hatte derselbe Modlauf zuvor 1.774 ItemInfo-Platzierungsrezepte aus dem KFC-Dateifallback registriert. Der unmittelbare Bounds-Abfragefehler kann daher nicht mit einem leeren oder nicht registrierten Rezeptkatalog erklärt werden. Die native C-ABI liefert für mehrere unterschiedliche Fehlerbedingungen denselben `SIZE_MAX`-Wert. Der Lua-Fehlertext unterscheidet diese Fälle nicht. Der genaue native Abbruchgrund ist mit diesem Lauf noch offen.

Ein F8-Aufruf um 11:20:39 UTC wurde noch vor einer Weltabfrage abgewiesen, weil nur Ecke A markiert war. Danach setzte der Nutzer die Auswahl neu. Die letzte protokollierte Markierung lautet A `3731.5,841.5,1448.5` und B `3736,845,1446`. Dies ist ein anderer Bereich als bei der fehlgeschlagenen Auswahl um 11:16:17 UTC und daher kein zeitlicher A/B-Vergleich derselben Bounds.

Um 11:21:08 UTC wurde F8 für die neue Auswahl erfolgreich abgeschlossen. Der erste native Bounds-Aufruf meldete „still scanning“. Eine Folgeabfrage endete 2 ms später mit `matched_props=0`. Das gespeicherte `capture-2.sfbp` enthält V7, den Terrainbereich `10×8×6`, 480 Zellen mit 167 belegten Werten und null Props. Das Log bestätigt Encoding, Dateiexport und Bibliotheksaktualisierung. Die Datei im Spiel-Export hat 2.663 Bytes und SHA-256 `89457087E2883C0F767B090267750F84DF9E2ACE1191F6CEF8BE17C1E1E77B46`. Eine identische Belegkopie liegt unter `dev/profiles/captures/world-editor-runtime-ab-20261021/capture-2-live-26082781.sfbp`.

Der Nutzer löste F7 um 11:21:30 UTC zweimal mit `capture-2` aus. Beide Aufrufe wurden vor einer Mutation abgewiesen: `runtime.world.voxel.write` hat keinen verifizierten nativen Writer für diese Sitzung. Beide Logeinträge sagen ausdrücklich, dass keine Weltänderung gesendet wurde. F4 wurde um 11:20:22 und 11:20:40 UTC davor ausgelöst. Beide Aufrufe meldeten, dass es keine bestätigte Blueprint-Platzierung zum Rückgängigmachen gibt. Nach den beiden abgewiesenen F7-Aufrufen drückte der Nutzer F4 erneut um 11:26:21 UTC. Auch dieser Aufruf meldete, dass es keine bestätigte Blueprint-Platzierung zum Rückgängigmachen gibt. Ein weiterer F7-Aufruf um 11:26:44 UTC wurde wieder vor einer Mutation abgewiesen und meldete, dass keine Weltänderung gesendet wurde. Es existiert kein Terrain-Pastejournal. Ein erfolgreicher F4-Undo von Terrain, Blöcken oder Voxeln wurde nicht ausgeführt.

Damit ist in Sitzung 5 der Ablauf Capture und Speichern für `capture-2` live bestätigt. F7 wurde dreimal live geprüft und sicher vor Mutation abgewiesen. F4 wurde dreimal ohne Pastejournal als No-op bestätigt, zweimal vor und einmal nach den ersten beiden F7-Versuchen. Der spätere dritte F7-Aufruf erfolgte nach diesem F4. Der vollständige Paste-/Undo-Zyklus bleibt offen, weil der verifizierte native Terrainwriter weiterhin fehlt. Der frühere Propscanfehler gehört zu einer anderen Auswahl und wurde durch diesen späteren erfolgreichen Propscan nicht als reproduzierbarer Fehler bestätigt.

## Neue Anbindung und erfolgreicher Terrainzyklus in PID 5352

Dieser spätere Lauf ersetzt die früheren negativen Ergebnisse nicht. Er verwendet die neu gebauten und bei beendetem Client installierten Bibliotheken. Der Client startete um 12:14:47 UTC, seine lokale ECS-Sitzung ist 5.

Der neue Decoder verfolgt originale gerichtete Aufrufketten ab den registrierten Namen. Zydis dekodiert auch die überprüften originalen Switchtabellen des Terraformingpfads. Es werden keine festen Writer- oder Spawnadressen als Dispatchziele eingetragen. Der Voxeladapter prüft den vollständigen originalen Blockkopierer, den Store-Commit, die Codebelege und die aktuellen Besitzer unter den ursprünglichen Gates. Er bewahrt benachbarte Zellen in erweiterten 8³-Blöcken und verifiziert das Ergebnis durch Rücklesen. Der Propadapter verwendet den originalen Bauhelper mit eigenen Argumenten und bestätigt anschließend die exakt zugeteilte Entity-ID einschließlich Baukomponenten. Statische Vertragsauswahl und ein erfolgreicher Terrainauftrag bestätigen noch keinen Prop-Spawn.

| UTC am 21. Oktober | Tatsächlicher Livebefund |
| --- | --- |
| 12:18:35 bis 12:18:40 | F7 mit `capture-1`, vorherigen Terrainzustand gesichert, 12 Zellen geschrieben und native Bestätigung erhalten, null Props |
| 12:18:51 bis 12:18:53 | F4 wartete zunächst auf den Terrainleser, restaurierte dann den vorherigen Zellpuffer und meldete `Undo verified` |
| 12:21:20 | F8 speicherte `capture-3`, 13×9×10 beziehungsweise 1.170 Zellen, davon 205 belegt, ein Prop |
| 12:21:29 | F7 mit `capture-3` wurde durch die Lua-Bereitschaftsprüfung vor Mutation abgewiesen |

Damit ist erstmals ein tatsächlicher F7-/F4-Zyklus für Terrainzellen im laufenden Spiel bestätigt. Save-Reload, vollständiger gemischter Blueprint und Mehrfach-Prop-Löschung sind dadurch nicht geprüft. Die gefilterten Originalmeldungen und die unveränderte Blueprintkopie liegen unter `dev/profiles/captures/world-editor-runtime-ab-20261021/` mit den Namen `f7-undo-live-5352-before-readiness-fix.log` und `capture-3-live-5352.sfbp`.

### Separater einzelner Voxel-A/B-Test und Wiederherstellung

Der ausdrücklich autorisierte Diagnosejob speicherte den vollständigen Chunk ab `7480,1672,2800` mit 512 Zellen. Der No-op wurde bestätigt. Anschließend wurde genau die Zelle am Chunkursprung von 65282 auf null gesetzt und wiederholt zurückgelesen. Alle 511 anderen Zellen waren unverändert. Der Restoreversuch wurde mit `voxel write was rejected before changing the world` abgewiesen. Der Job wiederholte keine Mutation und behielt seine Baseline sowie den Zustand `recovery-required` bei.

Ein eigener einmaliger Recoveryjob überprüfte dieselbe Sitzung und vor dem Schreiben den gesamten Chunk. Er akzeptierte ausschließlich die eine bekannte geleerte Zelle als Unterschied. Ein Wiederherstellungsaufruf gab `true` zurück. Drei vollständige Rücklesungen ergaben jeweils null Unterschiede zur ursprünglichen Baseline. Sein Abschluss lautet `complete`, `restored_verified=true` und `recovery_required=false`. Die ursprünglichen Diagnosebelege wurden nicht überschrieben. Belegkopien heißen `voxel-write-ab-incomplete-5352-report.json` und `voxel-recovery-confirmed-5352-report.json`.

### Ursache der späteren Bereitschaftsabweisung

Die native Verfügbarkeitsabfrage versucht das originale Scene-Gate nicht blockierend zu erwerben. Ein belegtes Gate ergibt vorübergehend `false`. Die Lua-Funktion `native_mutation_unavailable_reason` behandelte zuvor jedes negative Ergebnis bei direkten Mutationen als grundsätzlich fehlenden Adapter. Dadurch wurde das vorhandene Warten in `require_world_feature` umgangen. Außerdem wurde die Bereitschaft nur bis zum ersten Gesamterfolg beziehungsweise die erste Wartemeldung protokolliert. Eine Verfügbarkeitsabfrage reserviert das Gate weiterhin nicht für den späteren Write.

Die Korrektur lässt vorübergehende Bereitschaftsfehler vor der ersten Mutation im bestehenden begrenzten Warteablauf weiterlaufen und aktualisiert den Editorstatus periodisch. Ein zusätzlicher nativer Retry darf ausschließlich Ergebnis 1, sicher vor Dispatch abgewiesen, innerhalb desselben Sitzungsmarkers wiederholen. Erfolgreiche, zurückgerollte oder unklare Mutationen werden nicht erneut eingereicht.

## Bestätigter gemischter lokaler Zyklus und gedrehte Platzierung

In PID 27804, ECS-Sitzung 5, wurde das gespeicherte `capture-3` mit 1.170 Terrainzellen und einem Prop vollständig gesetzt und rückgängig gemacht. F7 begann um 13:02:32,538 UTC. Nach etwa sieben Sekunden erster Bereitschafts- und Codeauflösung war der Undo-Snapshot um 13:02:39,772 gesichert. Um 13:02:39,778 waren die Terrainzellen geschrieben, um 13:02:40,391 war die gemischte Platzierung bestätigt. Die eigentliche Mutation dauerte etwa 0,61 Sekunden, nicht der gesamte erste F7-Auftrag.

F4 begann um 13:02:53,235 UTC und bestätigte um 13:02:53,768 die Wiederherstellung des vorherigen Terrains und die Entfernung des gesetzten Props. Ein weiterer Auftrag setzte `capture-3` um 13:04:14,645 mit insgesamt 180° Drehung. Vor dem zusätzlich ausgelösten Vierteldrehen hatte der Nutzer bereits 90° eingestellt. Der Diagnosebericht bestätigt diesen Auftrag mit `rotation_turns=2` und etwa 0,97 Sekunden akkumuliertem Callbackdelta. Weitere Nutzeraktionen bestätigten gedrehte Platzierung sowie Terrain-Undo. Die Einzelberichte und der gefilterte Originalauszug liegen unter `dev/profiles/captures/world-editor-runtime-ab-20261021/`.

Der Editor-Callback ist auf 100 ms gestellt. Zusätzliche Bereitschaftsbeobachtungen erfolgen höchstens einmal pro zwei akkumulierten Callbacksekunden und pausieren während aktiver Aufträge. Die separate P2P-Zielprüfung behält ihr bestehendes Intervall. Die zusätzlich eingeführte zweite sichtbare Runtimezeile wurde wieder entfernt. Diese Änderungen wurden für den nächsten Clientstart installiert, der Lua-Lebenszyklustest bestand. Daraus folgt keine Messung eines exakten Wandzeitintervalls des übergeordneten Schedulers.

## Dedicated Server und P2P: Fehler und erfolgreicher Livezyklus

Geprüft wurden die aktuellen Enshrouded- und ShroudForge-Logs in beiden Installationen `Enshrouded` und `EnshroudedServer`. Der Clientauftrag um 13:17:26 UTC kam vollständig beim autorisierten Server an. Der Server validierte V7, 1.170 Zellen und ein Prop und meldete erst um 13:18:32 `server-world-runtime-did-not-become-ready`. Der Versand und der Rückkanal funktionierten bereits vor der Korrektur.

Die zusätzliche Serverprüfung verlangte unabhängig von der tatsächlichen Propgröße immer `runtime.world.entity.set_scale`. Außerdem sollten mehrere voneinander unabhängige Scene-Bereitschaftsabfragen gemeinsam erfolgreich sein. In einem späteren rein lesenden Vergleich im neu gestarteten Server, PID 10356, Sitzung 5, war `set_scale` in allen 20 Runden nicht verfügbar. Writer, Spawn, Destroy und Leser lieferten dagegen wiederholt positive Proben. Dieser Vergleich führte keine Weltmutation aus. Der Cursor ist auf dem Headless-Server ebenfalls nicht verfügbar und wird dort für den übertragenen Zielpunkt nicht gebraucht.

Ein weiterer Nutzerauftrag um 13:23:24 UTC zeigte einen zweiten Fehler. Eine vorübergehend negative Grid-Abfrage wurde schon vor der kanonischen Platzierung als ungültige Zielausrichtung abgewiesen: `server-rejected-target-grid-alignment: KFC Runtime world operation is not ready: runtime.world.voxel.grid_spec`.

`operations.lua` übergibt jetzt den bereits vom Transport validierten Weltpunkt an die kanonische Platzierung. Diese besitzt Drehung, Gridprüfung, Bounds, tatsächlich benötigte Features und begrenzte Bereitschaftswiederholungen. Die Serverkoordination enthält keine zweite Featureliste. `history.lua` ruft für Server-F4 denselben abgesicherten Undo-Ablauf auf und entfernt seine zusätzlichen Leser-/Writer-Proben. Authentifizierung, Übertragungsformat, Sitzungsschutz, unveränderte Original-Guards und serverseitiger Transaktionsbesitz bleiben erhalten.

Der Client-Reload gelang sofort. Der erste Server-Reload wurde vom ausstehenden Cleanup von `sf-crafting-essentials` abgewiesen. Nach dessen protokolliertem Abschluss gelang der erneute Reload um 13:26:54 UTC. Beide Installationen verwenden dieselben korrigierten Lua-Dateien, die nativen DLLs wurden für diese Korrektur nicht geändert.

| UTC am 21. Oktober | Livebefund nach der Korrektur |
| --- | --- |
| 13:27:09,929 | Client beginnt den einmaligen F7-Test mit `capture-3`, Client-Sitzung 8 |
| 13:27:10,417 | Server erhält den vollständigen autorisierten Auftrag |
| 13:27:11,072 | Server bestätigt den Write von 1.170 Terrainzellen |
| 13:27:11,087 | Server bestätigt Terrain und ein neu gesetztes Prop bei `3728,843,1436.5` |
| 13:27:11,179 | Client erhält die positive Serverantwort mit Undo-Token |
| 13:27:34,853 | Server erhält den F4-Auftrag für diese Transaktion |
| 13:27:34,886 | Server bestätigt restauriertes Terrain und Entfernung des gesetzten Props |
| 13:27:42,722 | Weiterer Nutzer-F7 nach einer Vierteldrehung bestätigt erneut Terrain und ein Prop |
| 13:27:48,083 | Auch der anschließende Nutzer-F4 wird vom Server bestätigt |

Die einmaligen Diagnosejobs melden für F7 und F4 jeweils `world-operation-confirmed`, `attempts=1`. Dazwischen wurde ein weiterer F7 um 13:27:31 mit dem bestehenden Undo-Token abgewiesen. Das schützt die noch offene Servertransaktion. Der bestätigte F4 gab sie anschließend frei. Die erfolgreichen Aufträge bestätigen keinen Save-Reload und keine Mehrfach-Prop-Platzierung.

Rohlogs aller vier Quellen, der lesende Vergleich, Installationsbelege und die bestätigten F7-/F4-Berichte liegen unter `dev/profiles/captures/world-editor-server-p2p-20261021/`. Die temporären Jobs wurden nach Sicherung entfernt. Diagnoseeinstellungen sind wieder wie zuvor: Client deaktiviert, Server aktiviert ohne aktive Sitzung, beide auf `runtime.overview`. Die monotonen Anforderungszähler wurden beibehalten.

## Online-F8 und vollständiger Zyklus mit zwei Props

Der Nutzer-F8 um 13:28:13 UTC versuchte weiterhin, `runtime.world.voxel.read` im replizierten Client auszuführen. Um 13:28:35 wurde der lesende Auftrag nach dem Bereitschaftstimeout beendet. Es war kein Capture-Auftrag zum Server geschickt worden. Die zuvor bestätigte P2P-Anbindung umfasste F7 und F4.

Online-F8 überträgt jetzt die beiden markierten Weltpunkte und die Blueprintachse über denselben autorisierten Modkanal. Der Server verwendet `copy_voxels(false)` mit den vorhandenen Terrain-, Grid- und Prop-Abfragen. Er speichert kein zusätzliches Blueprint. Das fertige V7 wird mit den gemeinsam verwendeten Chunk- und Integritätsprüfungen zurückgeschickt. Der Client prüft Peer, Request-ID, Länge, Prüfsumme, aktuelle Sitzung und V7-Inhalt. Erst danach verwendet `library.lua::save_capture` denselben freien Dateinamen und Speicherablauf wie ein lokales F8. Fehlende Chunks, falsche Absender, beschädigte Inhalte und abgelaufene Sitzungen erzeugen keine Blueprintdatei. F6 beendet einen lesenden Capture, ohne ein vorhandenes Baujournal aufzugeben.

| UTC am 21. Oktober | Livebefund |
| --- | --- |
| 13:52:25,022 | Server erhält den autorisierten F8-Auftrag `client-8-6c94a73e-0-1` |
| 13:52:26,130 | Capture bestätigt 648 Terrainzellen, davon 311 belegt, und zwei Props ab Gridposition `7503,1685,2855` |
| 13:52:26,133 | Server gibt das vollständige V7 mit 4.010 Bytes zurück |
| anschließend | Client speichert `capture-4.sfbp`, Abmessungen `8×9×9`, und wählt es aus |
| 13:52:34,286 | Server bestätigt F7 mit allen 648 Zellen und beiden Props |
| 13:52:51,252 | Server bestätigt F4 mit restauriertem Terrain und Entfernung beider gesetzten Props |

Die unveränderte Datei, Rohlogs, Installationsbelege und Diagnoseberichte liegen unter `dev/profiles/captures/world-editor-server-capture-20261021/`. Damit sind Capture, Rückübertragung, lokales Speichern, serverseitiges Setzen und Undo eines Blueprints mit zwei Props live belegt. Save-Reload und mehrere echte gleichzeitig verbundene Spielclients sind damit weiterhin nicht geprüft.

## Getrennte Clients, Warteschlange und erneutes F7

Die vorherige Implementierung wies weitere Aufträge bei `server_busy` ab und besaß nur ein serverweites Undo-Journal. Sie blockierte außerdem das nächste F7 selbst nach bestätigter Platzierung. Beispielsweise endete die Platzierung um 13:54:30,878 UTC erfolgreich. Der nächste Nutzer-F7 um 13:54:36 wurde dennoch als `server-has-an-unresolved-world-editor-transaction` abgewiesen. Diese Sperre war keine Aussage über einen weiterhin laufenden nativen Auftrag.

`p2p.lua` besitzt jetzt eine FIFO-Warteschlange für vollständig empfangene F8-, F7- und F4-Anfragen. Genau ein Weltauftrag wird gleichzeitig ausgeführt. Schlüssel bestehen aus authentifiziertem Peer und Request-ID. Parallele Uploads haben getrennte Puffer. Es gelten höchstens 32 wartende Anfragen, eine wartende Anfrage je Peer, 32 MiB wartende Blueprintinhalte und separat 32 MiB deklarierte Uploadinhalte. Vor Dispatch werden Berechtigung, Sitzung und eine Wartefrist von 45 akkumulierten Callbacksekunden erneut geprüft. Wartende Anfragen erhalten periodische Fortschrittsmeldungen. Doppelte aktive Anfragen werden nicht erneut ausgeführt, die letzten 128 Abschlussbelege bleiben für Wiederholungen erhalten. F6 kann nur eine passende eigene wartende Anfrage beziehungsweise eigene Transaktion freigeben.

`operations.lua` und `history.lua` halten jetzt das jeweils letzte Journal getrennt je Client. Nur während eines serialisierten Undo-Auftrags ist das betreffende Journal im kanonischen Undo-Ablauf aktiv. Ein weiterer erfolgreicher F7 ersetzt ausschließlich das vorherige bestätigte Journal dieses Clients. F4 nimmt anschließend dessen letzte Platzierung zurück, frühere Platzierungen bleiben stehen. Eine tatsächlich unvollständige Wiederherstellung sperrt weiterhin das nächste F7 desselben Clients. Es werden höchstens 32 Clientjournale in einer Sitzung behalten. Die vorhandenen Konfliktprüfungen schützen spätere Terrain- und Propänderungen auch bei überlappenden Bereichen.

Die bestehenden Lua-Prüfungen wurden um die echte Transport- und Modkoordination mit kontrollierten nativen Ergebnissen erweitert. Geprüft sind mehrteilige V7-Übertragung, gleichzeitige Uploads verschiedener Peers mit derselben Request-ID, FIFO-Ausführung ohne Überschneidung, doppelte aktive und abgeschlossene Anfragen, verspätete Abschlusscallbacks, erneute Autorisierung, Sitzungswechsel, Ablauf, Queuegrenzen und Abbruch vor Dispatch. Getrennte Journale, fremde Undo-/Freigabetoken, wiederholtes F7 und die verbleibende Wiederherstellungssperre sind ebenfalls geprüft. Dies ist eine automatisierte Prüfung mit mehreren simulierten Peers, kein Liveversuch mit einem zweiten Spielclient. Die drei Prüfungen des gebündelten Lua-Testtargets bestanden.

### Wiederholtes F7 nach dem Einspielen live bestätigt

Nach dem letzten Lua-Reload bestätigten Client und Server um 14:09:33,412 und 14:09:37,523 UTC zwei direkt aufeinanderfolgende Platzierungen desselben aktiven `capture-4`, jeweils mit 648 Zellen und zwei Props. Dazwischen wurde kein F4 verlangt. Um 14:09:39,803 bestätigte der Server den anschließenden Undo der letzten Platzierung. Vier weitere F7-Aufträge um 14:09:47,012, 14:09:49,619, 14:09:51,904 und 14:09:56,533 wurden ebenfalls erfolgreich abgeschlossen. Damit ist der frühere Undo-Zwang für bestätigte Platzierungen auch im laufenden Spiel behoben.

Die Tests verwendeten unterschiedliche Cursorziele desselben Blueprints. Es wird keine Rücknahme aller dieser Platzierungen behauptet. F4 betrifft das zuletzt behaltene Journal. Die Belegkopien heißen `EnshroudedServer-repeat-f7-confirmed.log` und `Enshrouded-repeat-f7-confirmed.log` im Captureordner dieses Laufs. Die Diagnoseeinstellungen wurden wiederhergestellt und die temporäre Vorbereitung für den Reload archiviert.
