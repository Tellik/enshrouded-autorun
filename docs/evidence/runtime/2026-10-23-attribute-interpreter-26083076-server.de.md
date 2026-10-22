# Attributinterpreter des Serverbuilds 26083076

## Geprüfte EXE und Ressourcen

Geprüft wurde die installierte `enshrouded_server.exe` mit SHA-256
`d7baa768eb00aa895d9f5a1cd8747ccf66858c7b542f50983092b2e3261588ee`,
PE-Zeitstempel `1792507668` und Image-Größe `31264768` Bytes. Diese Identität
stimmt mit `modules/game-runtime/profiles/enshrouded/server/26083076.json`
überein.

Der vorhandene KFC-Exporter hat die Attributressourcen frisch aus den zum Server
gehörenden KFC-Dateien gelesen. Er meldete 17 Ressourcen. Darunter befinden
sich 59 Attributroots mit 307 Definitionen und 66 nichtleeren Rechenprogrammen.
Weder ein Spielprozess noch dessen Speicher wurde geöffnet oder verändert.

## Interpreter der exakten Server-EXE

Die drei Funktionen wurden anhand der PE-Unwind-Bereiche und ihres
Programmspan-Zugriffs in der exakten Server-EXE lokalisiert. Ihre ausführbaren
Bereiche passen strukturell zu den jeweiligen Interpreterfunktionen des
Clientbuilds. Der Float32-Bereich hat im normalisierten Befehlsvergleich eine
kleine compilerbedingte Abweichung. Seine Originalausgabe wurde deshalb
zusätzlich direkt gegen das Rust-Modell differenziell geprüft.

| Skalar | Server-RVA | Funktionsgröße | Merkmal |
| --- | ---: | ---: | --- |
| `sint32` | `0x3a550` | 3016 Bytes | `idiv`, 48 Aufrufe und 39 Sprünge im Programminterpreter |
| `uint32` | `0x61230` | 3048 Bytes | `div`, 48 Aufrufe und 39 Sprünge im Programminterpreter |
| `float32` | `0x4f240` | 2928 Bytes | Float32-Arithmetik mit `divss` |

Der Erfolgsstatus liegt bei allen drei Interpreterergebnissen an Offset `+0`.
Der Ergebniswert liegt bei Offset `+8`. Die Vectorprüfung bestätigte diese
Felder für den exakten Serverbuild.

## Differenzialvergleich

Die Generatorprüfung hat die EXE ausschließlich als Daten in isolierten
Unicorn-Seiten verwendet. Sie hat die drei ursprünglichen Serverfunktionen
ausgeführt, nicht als Windows- oder Spielprozess geladen. Alle nativen
Statuswerte waren erfolgreich. Die unabhängige Rust-Prüfung verglich für jeden
Vektor sowohl den Ergebniswert als auch sämtliche geänderten Skalarwerte.

Insgesamt stimmten 3992 Vektoren überein:

- 2112 normale Programmauswertungen mit 32 Vektoren für jedes nichtleere
  Programm.
- 1320 zusätzliche Grenzwertvektoren mit 20 Vektoren für jedes nichtleere
  Programm.
- 32 zusätzliche Skalierungsgrenzwerte für die vier Programme mit
  `ScaleToNewMax`.
- 528 Schritte aus acht vollständigen, absteigenden Neuberechnungen der
  Attributroots.

Der Test `original_engine_differential` bestand mit
`3992 independent original-engine differential vectors matched`. Er lief in
einem abgetrennten Checkout, weil bereits vorhandene Änderungen im lokalen
Modloader den normalen Workspace-Build mit einem unabhängigen Compilefehler
blockieren.

Damit ist `attribute-command-v1` für die exakte Server-EXE und ihre frisch
exportierten KFC4-Programme offline differenziell bestätigt. Das belegt keine
sichtbare Gameplaywirkung oder Mehrspielersynchronisierung. Diese bleiben einer
separaten Liveprüfung vorbehalten.
