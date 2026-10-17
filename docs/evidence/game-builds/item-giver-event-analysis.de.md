# Creative Mode 146, ItemGiver und der originale Ereignispfad

## Identität und Grenze

Statisch untersuchte Datei: `creative/creative.dll`, SHA256 `fe901f31fc61128e9e3e7b59c74366c4ef3cd6a5b0a7e95837f169bca28ff592`. Alle folgenden Code-RVAs beziehen sich auf diese DLL. Die Datei wurde nicht geladen oder ausgeführt. Kein Spielprozess wurde angesprochen. Die separat vorhandene Symboltabelle mit 147 Clientnamen gehört zu Steam-Build 23966345 und ist kein Beleg für aktuelle KFC4-Funktionsadressen.

## Geschlossene Ketten

```text
CreativeScreen take, 0x3e010
  -> 0x3e0e5 CALL 0xc8fb, Thunk auf 0xe8fc0
  -> ItemGiver: lokaler 24-Byte-Auftrag oder 32-Byte-Callbackauftrag

server.craftingActionSystem, vom Mod gehookt
  -> DLL 0xe57b0
  -> Originalfunktion aufrufen, Cursor restaurieren
  -> 0xe5879 CALL 0xb212, Thunk auf 0xe7ce0
  -> echte ECS-Query, Spielerzuordnung, eventPartition/eventQueue/pushEvent
  -> CraftRecipeEvent mit eingebettetem Modprotokoll

server.craftAndCommit, vom Mod gehookt
  -> DLL 0xe5e90
  -> Modprotokoll erkennen und dekodieren
  -> 0xe6188 CALL 0x9d81, Thunk auf 0xe7a20
  -> originale Inventorytransaktion
```

Die DLL ersetzt die Serverpfade durch eigene Interposition. Das Modprotokoll ist kein normaler Engine-Itemgive-RPC. Ohne passenden Modempfänger folgt aus dem Absenden eines CraftRecipeEvent keine generische Itemvergabe.

## Gebundene Enginefunktionen und Daten

Installer **`0xe62f0`** löst Namen aus dem eigenen Symbolkatalog. Es handelt sich um Bezeichnungen dieses Mods, nicht automatisch um reflektierte vollständige C++-Signaturen.

| Feld im ItemGiver-Objekt | Symbolname |
| --- | --- |
| `+0x00`, `+0x08` | `ecs.queryBegin`, `ecs.queryNext` |
| `+0x10` | `ecs.ownerEntity` |
| `+0x18`, `+0x20`, `+0x28` | `ecs.eventPartition`, `ecs.eventQueue`, `ecs.pushEvent` |
| `+0x30`, `+0x38` | `inventory.transactionConstruct`, `inventory.transactionSetup` |
| `+0x40`, `+0x48` | `inventory.addItem`, `inventory.moveToInventory` |
| `+0x50`, `+0x58`, `+0x60` | `inventory.commit`, `inventory.rollback`, `inventory.transactionDestruct` |
| `+0x68` | `inventory.anySlot` |

Weitere Einträge liefern Querygröße, Query-Kontext und EventWriter-Offset, CraftRecipeEvent-Hash und -Feldoffsets, Carrier-Rezept, CraftRequest-Konfiguration und Transaktionsgröße. Der Installer erzeugt Hooks für `server.craftingActionSystem`, `server.craftAndCommit` und `ecs.spawn`. Erfolgsprüfungen bei `0xe710b..0xe714b` betreffen alle drei Hookobjekte.

## Client-/UI-Auftrag

`0xe8fc0` erhält `RCX=ItemGiver`, `EDX=Item-ID`, `R8D=Menge` und zusätzliche Instanz-/Optionsdaten. Die Menge wird auf `0xffff` begrenzt. Ohne genutzten Remote-Callback wird unter einem Modmutex ein 24-Byte-Auftrag in die Queue bei `this+0x1e0` geschrieben. Der Auftrag enthält Item-ID, UInt16-Menge, optionales Zielslot-/Inventarmerkmal und weitere Instanzoptionen.

Mit vorhandenem Callback bei `this+0x140` kann die Methode stattdessen einen 32-Byte-Envelope an dessen `vtable+0x10` übergeben, Call bei **`0xe9183`**. Die zusätzliche Entscheidung über **`0xe9a10`** prüft, ob der Serverhook in den letzten 2.000 ms gesehen wurde. Sie ist kein hier bewiesener Netzwerkautoritätsvertrag.

Der Envelope beginnt mit **`0x014d4345`**, also `ECM` und Protokollversion 1. Byte `+4` ist Requestkind 1. Er kodiert unter anderem Item-ID bei `+8`, Menge im unteren Halbwort von `+0x18` und Optionsbits bei `+0x1c`. Andere Versionsnummern werden im Empfänger ausdrücklich abgewiesen und gemeldet.

**Offener Remoteanschluss:** Der durch die UI-Analyse belegte Wrapper `0x108000` ruft bei `0x108042` den gebundenen `gameui.queueCraft`-Pointer aus API `+0x540` auf. Für diesen Wrapper und seinen Thunk `0x15a78` wurden bislang keine erreichbaren Aufrufer oder gespeicherten Funktionspointer nachgewiesen. Seine Existenz schließt daher die Verbindung vom ItemGiver-Callback zum tatsächlichen Engine-/Netzwerktransport noch nicht. `0x100870` ist selbst lediglich eine Modqueue für 32-Byte-Aufträge, mit optionalem Zusammenfassen nach Requestkind.

## Native Ereignisreservierung

Der Post-Hook `0xe57b0` speichert den ursprünglichen Systemcursor, ruft den ursprünglichen Serverpfad und restauriert den Cursor. Danach verarbeitet **`0xe7ce0`** die lokalen Aufträge in derselben Systemphase. Diese Routine ruft `queryBegin` und wiederholt `queryNext` mit dem echten Systemzeiger, eigenem Querybuffer und gebundener Querygröße auf.

Aus dem Querybuffer werden Kontext und EventWriter bezogen. `ownerEntity` liefert die zugeordnete Spieler-ID. Vor der Reservierung folgen `eventPartition(EventWriter)` und `eventQueue(EventWriter)`. Der Aufruf bei **`0xe81a3`** entspricht dem hier beobachteten Windows-x64-Registervertrag:

```text
RCX = Queue aus eventQueue
EDX = gebundener CraftRecipeEvent-Typwert
R8D = gebundene Eventgröße
R9W = Partition aus eventPartition
RAX = beschreibbarer Ereignispayload oder null
```

Bei null wird die volle Eventqueue gemeldet. Sonst setzt die Routine Operator und Station auf die Spieler-ID, das Rezept auf den gebundenen Carrier-Rezeptwert und Amount auf 1. Zwei 16-Byte-Kopien bei `0xe8329/0xe8331` legen den Mod-Envelope im Bereich ab, den der Symbolkatalog als `craftRecipeEvent.spare` bezeichnet.

**Dieser Name ist keine Paddinggarantie.** Die aktuelle KFC4-Reflexion nennt bei `keen::ecs::CraftRecipeEvent` den qualifizierten Hash `0xa4766fb7`, Größe `0x38` und die Felder `craftingOperatorId+8`, `craftingStationId+0x0c`, `recipeId+0x10`, `recipeAmount+0x14` und **`inputCategorySelections+0x18`**. Letzteres ist `keen::RecipeInputCategorySelections`, Typindex 2708, Größe 32. Ein dort angesetzter Envelope zweckentfremdet das tatsächliche Auswahlfeld. Aktuelle Offset-/ABI-Kompatibilität des Modsymbolkatalogs wurde nicht angenommen.

## Empfang und tatsächliche Inventaränderung

Der Hook **`0xe5e90`** erhält die originalen Craft-Aufrufargumente in RCX/RDX. Über den gebundenen Offset `craftRequest.spare` liest er den Envelope aus dem CraftRequest. Bei fremden Daten fällt er zum ursprünglichen `craftAndCommit` zurück. Bei passendem `ECM`, Version 1 und Requestkind 1 dekodiert er die Daten, ruft `0xe7a20` und kehrt über seinen eigenen Rückgabepfad zurück. Der beobachtete Statuswert 2 wird hier nicht mit einem ungeprüften Engine-Enum-Namen versehen.

`0xe7a20` verwendet **`0xe13f0`** für optionales Freimachen eines Zielslots und **`0xe15b0`** für die Itemvergabe. Beide reservieren eigenen, auf 16 Bytes ausgerichteten Transaktionsspeicher anhand von `inventory.transactionSize`. `transactionConstruct` erhält diesen Speicher. `transactionSetup` erhält zusätzlich **die eingebettete Konfiguration aus dem originalen CraftRequest plus dem gebundenen Offset `craftRequest.config`**. Damit stammt der benötigte Inventar-/Servicekontext aus dem regulären Serveraufruf.

Die Itemvergabe ruft bei **`0xe170d`** `inventory.addItem` mit Ergebnisbuffer, Transaktion, Spieler-ID, Zielslot beziehungsweise `anySlot`, Item-ID, Menge und optionalen Level-/Raritätsangaben auf. Bei Ergebnistag null wird `commit`, sonst `rollback` aufgerufen. `transactionDestruct` folgt in beiden Fällen. Der Ergebnisbuffer enthält außerdem die nicht eingefügte Restmenge, die separat gemeldet wird.

Die dritte Interposition, `ecs.spawn`, verwendet im Give-Ablauf einen TLS-Zustand für zusätzliche Instanzeigenschaften. Diese Untersuchung erklärt dadurch keine allgemeine direkte Spawn- oder beliebige Komponenten-ABI.

## Bedeutung für einen Zugriff ohne Codeänderung

ECS-Query, Eventreservierung und Inventorytransaktion sind echte Enginevorgänge. Ihr eigener Ablauf benötigt keinen von diesem Mod erzeugten Ersatzcode. Die Creative-DLL beschafft Phase, Cursor und CraftRequest-Konfiguration jedoch durch ihre Hooks und implementiert den Empfänger ihres eigenen Protokolls ebenfalls per Hook.

Als übertragbare Belege bleiben die typisierte Ereignisstruktur, der beobachtete Queue-Aufrufvertrag und die Transaktionskonfiguration aus dem originalen CraftRequest. Ein vollständig nachgewiesener, unabhängig erreichbarer und ohne Spielcodeänderung nutzbarer Itemgive-Einstieg entsteht daraus bisher nicht. Der Remote-Transportanschluss und sämtliche aktuellen KFC4-Funktionsadressen bleiben gesondert zu belegen.
