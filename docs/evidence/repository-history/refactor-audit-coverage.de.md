# Abdeckung und Grenzen der Repository-Analyse

Dieser Bericht dokumentiert den damaligen Struktur- und Auditstand. Die anschließende Reduzierung mit aktuellen Dateinamen, Löschungen und Prüfungen steht im [Reduzierungsbericht](reduction-report.md). Prüfsummen und Pfadlisten dieser Strukturphase sind historische Momentaufnahmen.

Stand: 17. Oktober 2026. Ausgangscommit: `81f9c5fe2d70ea4ba10896fb22a75b2d082800a6`.

| Umfang | Bestand | Vollständig inhaltlich gelesen | Aussage |
| --- | ---: | ---: | --- |
| Hauptrepository | 1.040 getrackte Pfade einschließlich 2 Gitlinks | Nicht pauschal alle | Jede Position hat eine Inventar- und Zielentscheidung. |
| Eigene manuell gepflegte Codedateien | 287 | 287 | 85.865 physische Zeilen, 100 % Leseanteil. |
| Generierte codeförmige Dateien | 4 | 0 als eigene manuelle Implementierung | Erzeuger und Veröffentlichung separat eingeordnet. |
| src/parser/kfc-parser | 243 Blattpfade, 169 codeförmige Dateien | 10 | 33.321 Quellcodezeilen im Inventar. Individuelle Fremdimplementierungen außerhalb dieses Leseanteils. |
| src/loader/runtime/native/json | 1163 Blattpfade, 487 codeförmige Dateien | 0 | 144.855 Quellcodezeilen im Inventar. Individuelle Fremdimplementierungen außerhalb dieses Leseanteils. |

Die ursprüngliche Zusatzlektüre betrifft `src/parser/kfc-parser/crates/mod-loader-bridge/src/lib.rs` mit 432 Zeilen. Für die Strukturkorrektur wurden zusätzlich konkrete Parserverträge vollständig gelesen. Die unten aufgeführten Dateien ersetzen keine Vollprüfung des externen Parserprojekts. Die geplante Übernahme eigener Parserformat-/Schema-/Snapshotfunktionen ins Submodul ist im Hauptbericht ausgewiesen.

## Methode

Alle 287 Dateien wurden in vollständigen, begrenzten Abschnitten gelesen. Aufrufer, Cargo-/CMake-/Einbettungspfade und ausgewählte Schema-/Generatorverträge wurden zusätzlich verfolgt. Pro Datei sind SHA-256, Zeilenzahl, 15 Prüffragen, Befunde und Evidenzgrenzen im [Einzeldateianhang](refactor-source-audit.md) dokumentiert. Der abschließende Hashvergleich ergab bei allen 287 Dateien Übereinstimmung mit den Leseprotokollen.

Die vollständige Hauptinventarliste beruht auf dem Ausgangsbestand von 1.040 getrackten Pfaden. Neu geschriebene Berichtanhänge sind Ergebnisdateien und erweitern diesen historischen Nenner nicht. Zwei Gitlinks werden im Hauptinventar je einmal geführt, ihre 1406 Blattpfade ausschließlich im [Abhängigkeitsinventar](refactor-dependency-inventory.csv).

## Separat behandelte erzeugte Dateien

- `src/loader/modules/modloader-ui/ui/dist/index.html`
- `src/loader/modules/modloader-ui/ui/src/generated-flags.ts`
- `website/data/enshrouded-1076226---game38-branches-ea-update-08-2026-06-29t10-27-39-052394z/types.lua`
- `website/pages/index.js`

Weitere erzeugte oder veröffentlichte Daten wie JSON-Snapshots und Schemakopien sind ebenfalls inventarisiert. Artikeltexte, Grafiken, Videos, Binär-DLLs und alle sonstigen Begleitdateien haben eine dokumentierte Entscheidung. Ihre Inhalte wurden nicht pauschal vollständig gelesen oder funktional validiert. Undurchsichtige Sidecar-DLLs wurden nicht dekompiliert.

## Ausgeschlossene lokale Ausgaben

Ignorierte Verzeichnisse wie `target/`, `build/`, `x64/`, `node_modules/` und lokale installierte `shroudforge/`-Daten sind Arbeits-/Buildausgaben. Sie sind kein zusätzlicher gepflegter Quellbestand. Die Leseprotokolle unter `target/` sind Hilfsmittel dieses Audits. Dauerhafte Ergebnisse liegen unter `docs/sf/`. Kein genereller Nachweis, dass beliebige lokale Ausgaben gelöscht werden dürften.

## Ausgeführte Prüfungen

Dateiinventar, Vollständigkeitsabgleich 287/287, SHA-256-Abgleich, CSV-Rücklesen, 15 Prüffelder pro Datei, relative Dokumentlinks und Prüfung der vorgeschlagenen Abhängigkeitsrichtungen. `git diff --check` meldete keine Whitespacefehler. `cargo metadata --no-deps --offline --locked --format-version 1` war im ursprünglichen Audit erfolgreich und erfasste 13 Workspacepakete einschließlich automatisch aufgenommener Mitglieder. Das ist keine Kompilierung. Tests, Builds, Spieleläufe, Prozessinjektionen und Installer wurden nicht ausgeführt. Vorhandene Tests sind statischer Schutzbestand und keine in diesem Audit nachgewiesenen grünen Tests.

## Aussagegrenzen

Statische Erreichbarkeit beweist keine tatsächliche Nutzung durch alle Anwender. Ein fehlender Suchtreffer beweist bei dynamischem Lua-/IPC-/CLI-Laden keinen toten Code. Laufzeit-, Fehler- und Performancebefunde sind entsprechend als statisch belegt oder abgeleitet zu lesen. Der Zielbaum beschreibt einen Entwurf. Seine Buildfähigkeit kann erst während der Umsetzung geprüft werden.

## Nachprüfung der Struktur am 17. Oktober 2026

Die ursprünglichen 287 eigenen Codedateien bleiben vollständig gelesen. In der Nachprüfung wurden Modul-/UI-Einbettungen, Parserverbraucher und die Zuordnung der Lua-Aufrufe zu ihren nativen Datenbesitzern nochmals gezielt verfolgt. Große bereits gelesene Dateien wurden dabei an diesen Kontrollflüssen nachgeprüft, nicht als vollständig erneut gelesen gezählt. Kleine Vertragsdateien wurden vollständig nachgelesen.

Zusätzlich vollständig gelesene externe Codedateien:

- `src/parser/kfc-parser/crates/kfc/src/lib.rs`
- `src/parser/kfc-parser/crates/kfc-base/src/lib.rs`
- `src/parser/kfc-parser/crates/kfc-resource/src/lib.rs`
- `src/parser/kfc-parser/crates/mod-loader-base/src/env.rs`
- `src/parser/kfc-parser/crates/kfc-base/src/reflection/registry.rs`
- `src/parser/kfc-parser/crates/kfc-base/src/reflection/serde.rs`
- `src/parser/kfc-parser/crates/kfc-base/src/reflection/mod.rs`
- `src/parser/kfc-parser/crates/mod-loader-lua/src/definition/generator.rs`
- `src/parser/kfc-parser/crates/kfc-base/src/container/mod.rs`

Der verbleibende externe Code ist weiterhin nur inventarisiert beziehungsweise gezielt durchsucht. Der eigene Leseanteil bleibt 287/287 = 100 %. Im aktualisierten Hauptinventar sind 31 eigene Codedateien als Aufteilungskandidaten ausgewiesen.
