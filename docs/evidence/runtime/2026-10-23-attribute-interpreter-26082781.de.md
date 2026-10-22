# Attributinterpreter des Clientbuilds 26082781

## Befund

Das genaue Clientprofil `enshrouded-client-26082781` verwendet die EXE mit SHA-256 `d64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882`, PE-Zeitstempel `1792507660` und Image-Größe `44163072` Bytes. Der laufende ShroudForge-Prozess meldete SF Survival als geladen. Sein Status stand mit `attribute calculations are not validated for this executable profile` auf `waiting`. Die fehlende Profilfreigabe verhinderte die vier Attributschreibvorgänge.

Die drei Interpreter wurden in der exakten EXE anhand ihrer unwind-begrenzten Funktionsbereiche und des gemeinsamen Programmspan-Aufrufs lokalisiert:

| Skalar | RVA | Disassemblierungsmerkmal |
| --- | ---: | --- |
| `sint32` | `0x21c9a0` | Attributindex im oberen 16-Bit-Opcode, 32-Bit-Wortprogramm, vorzeichenbehaftetes `idiv` |
| `uint32` | `0x24db80` | Gleicher Programmspan-Aufruf, vorzeichenlose `div`-Operation |
| `float32` | `0x231b70` | Gleicher Programmspan-Aufruf, Float32-Arithmetik einschließlich `divss` |

Die Ergebnisse liegen bei allen drei Roots im 32-Bit-Skalarfeld `Result+8`. Der Erfolgsstatus liegt bei `Result+0`. Die Interpreter wurden nicht im Spielprozess aufgerufen.

## Differenzvergleich

Die frischen Ressourcen stammen aus `dev/profiles/captures/sf-survival-26082781/resources.json`. Der bestehende Exporter meldete 17 gelesene Ressourcen. Darin befinden sich 59 Attributroots, 307 Definitionen und 66 nichtleere Rechenprogramme. Der Offline-Oracle emulierte die Originalfunktionen mit privaten Unicorn-Seiten und erzeugte für die aktuelle EXE 2.112 Vektoren. Alle nativen Statuswerte waren erfolgreich.

Der gezielte Test `original_engine_differential` verglich jeden nativen Programmausgang und jede Referenzänderung mit dem Rust-Modell. Ergebnis: `2112 independent original-engine differential vectors matched`. Der Test lief in einem abgetrennten Checkout, weil ein bereits vorhandener Compilefehler in den lokalen, nicht zu dieser Änderung gehörenden Modloader-Änderungen den normalen Workspace-Build verhindert.

Damit ist `attribute-command-v1` für die exakte EXE und die frisch exportierten KFC4-Programme durch den differentiellen Offlinevergleich bestätigt. Die Änderung der Profilfreigabe aktiviert keine Engine-Hooks und ändert keine Rechenlogik.

## Grenze der Prüfung

Der Offlinevergleich bestätigt das Rechenmodell. Er bestätigt nicht die sichtbare Spielwirkung oder die Synchronisierung in Mehrspielerwelten. Der Liveprozess mit der zuvor installierten ShroudForge-Version muss einen neu gebauten Runtime-Provider laden, bevor die vier SF-Survival-Einstellungen mit dieser Profilfreigabe geprüft werden können.
