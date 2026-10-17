# Creative Mode 146, statische UI- und Ladepfadanalyse

## Umfang und Grenzen

Untersuchte Quelle ist der lokal bereitgestellte Ordner `Creative Mode 146 1.2 2026-10-13T10-28Z U89dZfAAM`.
Die DLLs wurden ausschließlich als Dateien gelesen. Keine DLL wurde geladen, ausgeführt oder installiert. Es gab keine Spielzugriffe und keine Änderungen an Spielcode oder Spielspeicher.
Alle Adressen in dieser Notiz sind RVAs der jeweiligen DLL. Ein statischer erreichbarer Codepfad belegt die Implementierung, keine tatsächlich erfolgreiche Ausführung mit dem aktuellen Spielbuild.

## Ergebnis

Die nahtlose Creative-Oberfläche verwendet tatsächlich Engine-UI-Funktionen und deren echte Parameterlayouts. Das lässt sich bis zu aufgerufenen Buttons und Slidern verfolgen. Die untersuchte Mod gewinnt ihren HUD- und Input-Einstieg über SafetyHook und verändert dafür den ausführbaren Zielcode. Sie belegt somit enginenahes UI und echte Engineaufrufe. Sie liefert bislang keinen Nachweis eines vollständig ohne Codeänderungen registrierten UI-Einstiegs.

Für ShroudForge ist daran der gemeinsame Katalog aus Symboladressen und Layoutwerten sowie der belegbare Aufrufkontext verwertbar. Die Tatsache, dass diese Mod einen Hook verwendet, beweist keine allgemeine Notwendigkeit eines Hooks für unser Ziel. Der gesuchte Weg ohne Änderungen am Spielcode bleibt gesondert nachzuweisen.

## PE-Inventar

| Datei | SHA256 | Dateigröße | Unwind-Funktionsbereiche |
| --- | --- | --- | --- |
| dinput8.dll | `0912baf25dc15e0eac9a571893ca9a7ee35bbe549d7d9d4df97dfdf36c8558d5` | 1683968 | 3914 |
| creative.dll | `fe901f31fc61128e9e3e7b59c74366c4ef3cd6a5b0a7e95837f169bca28ff592` | 4651520 | 11681 |

`creative.dll` hat keine Exporte. Sie importiert aus USER32 lediglich GetForegroundWindow, GetAsyncKeyState, GetClientRect, GetKeyState und FindWindowW. Es gibt keine direkten DXGI-/D3D-Importe. Das Importinventar allein wäre kein ausreichender Beweis gegen ein Overlay. Die weiter unten belegten erreichbaren Engine-Widgetaufrufe sind der entscheidende Nachweis.

`dinput8.dll` exportiert die sechs erwartbaren Proxyfunktionen DirectInput8Create, DllCanUnloadNow, DllGetClassObject, DllRegisterServer, DllUnregisterServer und GetdfDIJoystick. Die Funktion bei `0x1e830` ermittelt den Programmpfad mit GetModuleFileNameW, ergänzt `creative/creative.dll` und lädt sie bei `0x1ea88` mit LoadLibraryW. Die Funktion `0x18030` baut außerdem einen Pfad aus GetSystemDirectoryW und `dinput8.dll`, um die originale Systembibliothek zu laden.

Belege: `dinput8-ui-static.json`, `proxy-load-functions.json`, `creative-ui-static.json`.

## Gemeinsame UI-Bindung

Binder `creative.dll+0x1080b0` löst 189 `gameui.*`-Schlüssel auf. 70 werden über den Symboladresskatalog und 119 über den Offsetkatalog gelesen. Eine Symboladresse kann ein Funktions- oder Variablenanker sein, diese 70 sind daher nicht pauschal 70 Funktionen.

Symbollookup `0x17a94` ist ein Thunk zu `0x9c250`. Der Lookup sucht im modinternen Katalog nach dem Namen und liefert den gespeicherten Wert aus dem gefundenen Knoten bei `+0x30`. Der Offsetlookup `0xeeb7` führt zu `0xa00c0` und sucht im zweiten Wörterbuch bei Katalog `+0x40`. Das ist kein Namenslookup in einer öffentlich exportierten Engine-API.

Fehlende Einträge schalten einen gemeinsamen Erfolgsstatus ab. Bei Erfolg kopiert der Binder `0x7c0` Byte in das Bindingobjekt und setzt dessen Verfügbarkeitsbyte bei `+0x7c0`. Bei Fehlschlag bleibt dieses Byte false.

| Schlüssel | Platz im Bindingobjekt | Verbrauch |
| --- | --- | --- |
| gameui.uiSystem | +0x0 | Adresse eines Engine-Kontextankers |
| gameui.uiContext | +0x8 | Adresse des aktuellen UI-Kontextankers |
| gameui.doButton | +0x3a8 | Native Widgetfunktion |
| gameui.settingSlider | +0x728 | Native Widgetfunktion |
| gameui.queueCraft | +0x540 | Native Aktionsfunktion, Erreichbarkeit dieses Wrappers noch offen |

Beleg für doButton: String `0x33c008`, Lookupcall `0x1086b2`, Ergebnisstore `0x1086e2`. Aus dem Stacklayout des Binders folgt Bindingoffset `+0x3a8`.
Beleg für settingSlider: String `0x33cc60`, Lookupcall `0x10ad16`, Ergebnisstore `0x10ad46`, Bindingoffset `+0x728`.

Belege: `ui-api-bindings.json`, `ui-functions.json`, `ui-helpers.json`.

## Erreichbarer HUD- und Widgetpfad

1. Der CreativeScreen-Konstruktor bei `0x127080` übergibt einen eigenen Callback an die Hookerzeugung. Der Callbackthunk `0x1a00` führt zu `0x12b000`.
2. `0x12b000` prüft den Menüstatus und ruft den Creative-Menüaufbau bei `0x12a790` auf.
3. `0x12a790` erzeugt eine std::function mit Vtable `0x33edf8`. Deren operator() ist `0x1294b0`. Er übergibt den Callback über Thunk `0x16e55` an den Root-Widgetbuilder `0x300dc0`.
4. `0x300dc0` ruft die übergebene std::function bei `0x300ff4` über Vtable `+0x10` auf.
5. Ein erreichbarer Pfad führt von `0x1294b0` über `0x143dd0` und `0x143b20` zum Sliderwrapper `0x10e2d0`. Dieser ruft bei `0x10e538` die gebundene Enginefunktion `[Bindingobjekt+0x728]` auf.
6. Ein weiterer Pfad führt von `0x1294b0` über `0x13f1d0`, `0x13f7b0` und `0x302fa0` zum Buttonwrapper `0x106df0`. Dieser ruft bei `0x106f34` `[Bindingobjekt+0x3a8]` auf.

Der Buttonwrapper baut die Engineparameter in einem lokalen Puffer anhand der gebundenen Offsets auf. Er befüllt unter anderem Kind, enabled, Style, Breite, Höhe, Skalierung, visible und Text. Für den Text wird FNV-1a mit Startwert `0xcbf29ce484222325` und Multiplikator `0x100000001b3` berechnet. Anschließend erfolgt der echte doButton-Aufruf.

Der Sliderwrapper prüft die gebundene Parametergröße, befüllt Label und Beschreibung sowie weitere Layoutwerte und ruft die echte settingSlider-Funktion. Während des Aufrufs setzt er im aktuellen Engine-UI-Kontext das gebundene input.blocked-Feld temporär und stellt dessen vorigen Wert danach wieder her.

Belege: `ui-hook-and-callback.json`, `ui-hook-details.json`, `ui-widget-callback-reachability.json`, `ui-root-widget-builder.json`, `ui-context-and-widgets.json`, `ui-engine-call-sites.json`.

## Einstieg dieser Mod verändert Code

Der HUD-Hookpfad lautet `0x127080 -> 0xb14a -> 0xd4fb0 -> 0x1ac090 -> 0x1aaa60 -> 0x1aa8d0`. Die Erzeugung baut eine ausführbare Trampolinstruktur und aktiviert sie über `0x1a9d20`. Dieser Aktivierungspfad bereitet einen Callback mit Vtable `0x345760` vor. Dessen operator() bei `0x1a8830` schreibt abhängig von der gewählten Hookart einen Sprung an die Zieladresse.

Bei Art 1 schreibt `0x1a9b90` ein `E9 rel32` und füllt gegebenenfalls weitere verdrängte Bytes mit `0x90`. Bei Art 2 erzeugen `0x1a9c50` und `0x1aa1c0` einen `FF 25`-Sprung mit RIP-relativem Zeiger. Der Aktivierungspfad erreicht VirtualProtect über `0x1aea20`. RTTI nennt zusätzlich ausdrücklich `safetyhook::InlineHook::enable`, `disable` und SafetyHook-Allocator-Klassen.

Ohne Ausführung ist nicht festgestellt, welche der beiden Sprungarten der aktuelle HUD-Einstieg wählen würde. Beide Varianten sind direkte Änderungen an ausführbaren Zielbytes. Das ist eine Aussage über diese Modimplementierung, nicht über alle möglichen Integrationswege.

Auch der InputInterceptor konstruiert bei `0xda8a0` einen Hook für `input.dispatch`, liest dafür InputMessage-Layouts aus dem Katalog und protokolliert den Erfolg. Die für GameThread verwendete Funktion `0xfdb40` löst `hud.build` auf und legt ebenfalls einen Hook an.

Belege: `ui-hook-install.json`, `ui-hook-write.json`, `ui-hook-writer-callback.json`, `ui-hook-patch-bytes.json`, `ui-hook-api-paths.json`, `creative-ui-static.json`.

## UIaktion und noch offene Transportfrage

Wrapper `0x108000` enthält einen konkreten Aufruf von gameui.queueCraft bei `0x108042`. Er ermittelt zunächst den aktuellen Spieler, liest einen Spielerwert mit dem gebundenen Feld bei Bindingobjekt `+0x588` und übergibt den aktuellen UI-Systemkontext sowie die Aktionsparameter an die gebundene Enginefunktion bei `+0x540`.

Für diesen Wrapper und seinen Thunk `0x15a78` wurden keine direkten Aufrufer, RIP-relativen Adressreferenzen oder statisch gespeicherten absoluten Funktionszeiger gefunden. Er kann eine vom Linker behaltene, ungenutzte Methode sein. Der Wrapper allein beweist deshalb nicht den tatsächlich verwendeten Transport der Creative-Itemanfrage. Die serverseitige Inventar-/Craft-Ereigniskette wird gesondert analysiert.

Belege: `ui-craft-calls.json`, `ui-craft-callers.json`.

## KI-Codex-Check

| Bereich | Ergebnis |
| --- | --- |
| Ziel | Nahtlose UI bis Enginewidget und Einstieg verfolgt. |
| Belege | Native Calls, Binder, Callback-Vtables und Schreibpfad statisch geprüft. |
| Änderungen | Ausschließlich ignorierte Analyseartefakte erstellt. |
| Dateicheck | Zwei PE-Dateien inventarisiert. Relevante Funktionsbereiche gezielt disassembliert, keine Behauptung vollständiger semantischer Dekompilierung. |
| Prüfung | Statische PE-, Import-, String-, Xref- und Kontrollflussanalyse ausgeführt. Keine DLL-Ausführung und kein Ingame-Test. |
| Offen | Patchfreier UI-Einstieg und tatsächlicher Client-Requesttransport weiterhin separat nachzuweisen. |
