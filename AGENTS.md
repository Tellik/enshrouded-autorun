# Arbeitsregeln für Codex in ShroudForge

## Verantwortung des Hauptagenten

Der gestartete Codex-Agent bleibt für den Auftrag verantwortlich. Er erfasst den Ist-Zustand, liest die betroffenen Implementierungen, entscheidet über Zuständigkeiten, setzt den beauftragten Umfang um und prüft das Ergebnis. Er gibt die Verantwortung nicht ohne ausdrücklichen Auftrag an einen Unteragenten ab.

Diese Datei ist dauerhafte Projektanweisung für Codex. Sie erzeugt weder einen eigenen Agenten in der ChatGPT-Oberfläche noch einen GitHub-Actions-Workflow.

## Vor Änderungen

- Prüfe `git status --short` und erhalte alle bereits vorhandenen Änderungen.
- Lies betroffene Dateien vollständig und verfolge ihre tatsächlichen Aufrufer, Datenwege, Speicherorte, Build-Einstiege, Paketierung und dynamischen Registrierungen.
- Verwende `docs/development/code-ownership.en.md` für die aktuelle Codeübersicht. Dateien in `docs/evidence/` dokumentieren datierte Untersuchungen und frühere Arbeitsstände, keine aktuellen Zuständigkeiten.
- Trenne belegten Ist-Zustand, Nutzerziel, Vorschlag und offene Frage. Erfinde keine Verantwortlichkeiten oder Verhaltensweisen aus Ordnernamen.

## Dateien und Zuständigkeiten

Für jede neue, verschobene oder aufgeteilte Datei bestimme vorab die konkrete Handlung, das zuständige Modul, den vollständigen Zielpfad, einen funktionsbeschreibenden Dateinamen, die Aufrufer und den Grund, warum vorhandener Code die Handlung nicht bereits besitzt.

Veröffentlichte Website-Artikel liegen als Markdown-Paare unter `docs/pages/`. Ihre Dokumentköpfe bestimmen Kennung, Sprache und Menügruppe. Der relative Ordner- und Dateiname bestimmt die Reihenfolge. Der Website-Build findet neue Paare automatisch. Führe keine zweite Routenliste, `page.json`-Sammlung oder kopierte Website-Artikel ein. Bewahre vorhandene Artikelinhalte vor dem Verschieben. `build/website/` ist erzeugte Ausgabe.

Seitenbezogene interaktive Website-Elemente und ihre Bedienlogik liegen gemeinsam unter `docs/website/page-integrations/<page-id>/`. Der Einstieg heißt `mount-page.js`. Der Build ordnet ihn anhand der Seitenkennung automatisch zu. Gemeinsame Darstellung und Navigation bleiben in den vorhandenen Website-Modulen. Füge keine Registrierung in einer weiteren Liste hinzu.

Eine Handlung hat eine kanonische Implementierung und ein zuständiges Modul. Aufrufer wie UI, CLI und Hintergrundprozess übergeben Eingaben und zeigen Ergebnisse. Sie duplizieren keine Geschäftslogik. Benenne Dateien nach ihrer Aufgabe. Vermeide unklare Sammelnamen wie `misc`, `helpers`, `utils` und `common`. Teile nicht allein nach Zeilenzahl auf und erzeuge keine bloßen Weiterleitungen.

Jedes Feature ist ein Modul. `modules/transport/` ist ebenfalls ein Modul. Ein Modul besitzt seine Fachlogik und den dazugehörigen UI-Teil. Geteilte UI-Ressourcen gehören nur dann in ein gemeinsames Modul, wenn mehrere Module sie tatsächlich verwenden.

Halte die belegten Repository-Grenzen ein:

- Mods liegen direkt unter `mods/<mod-id>/`. Erfinde keine zusätzlichen Modkategorien.
- `kfc-parser/` ist das direkt eingebundene Parser-Submodul. Sein eigener Code und seine Crates gehören in dieses Submodul. Füge weder `kfc-parser/kfc-parser/` noch einen zusätzlichen `vendor/`-Ordner ein.
- Füge keine zusätzliche `src/`-Zwischenebene ein, außer ein konkretes Sprach- oder Buildsystem verlangt sie.
- Ordne Runtime- und native Dateien anhand der tatsächlich vorhandenen Strukturen, Aufrufe und Zuständigkeiten ein. Erfinde keine Kategorien, bevor der Code sie belegt.
- Entwicklungsaktionen gehören zu den vorhandenen Einstiegspunkten unter `dev/`. Ergänze nur eine konkrete, wiederverwendbare Aktion. Führe kein allgemeines Tool- oder Jobsystem ohne belegten Bedarf ein.

## Verhaltensgrenzen

- Verfolge die vorhandene Konfiguration sowie alle Leser und Schreiber, bevor du Speicherformate oder Zuständigkeiten änderst. Erzeuge kein paralleles Einstellungsformat ohne konkreten, belegten Bedarf.
- `ensure_eml_extension.rs` gehört zur EML-Kompatibilität und ergänzt eine fehlende `extended.mod.json` bei der ersten Änderung an einem EML-Mod. `update_mod_configuration.rs` schreibt Mod-Einstellungen und den Aktivierungsstatus. Es liest vorhandene Erweiterungen direkt und ruft die EML-Ergänzung nur auf, wenn die Datei bei einem EML-Mod fehlt.
- Installierte Mods sind entpackte Ordner. Das Entpacken eines heruntergeladenen Archivs gehört zum Modinstallationsablauf. Verlege Archivzugriffe nicht in den laufenden Modbetrieb, solange aktueller Code und Auftrag das nicht verlangen.
- Untersuche beim Updater die tatsächlichen Aufrufstellen und Netzwerkzugriffe. Unterscheide Release-Metadaten, die GitHub-Hashdatei, Asset-Download, Staging und Installation. Belege Änderungen an wiederholten Anfragen durch die tatsächliche Anzahl der Zugriffe.
- Erhalte die Koordination automatischer und manueller Systemupdateprüfungen pro Installation, den Cooldown automatischer Prüfungen und das gespeicherte Prüfergebnis für UI und Headless-Prozess, sofern der aktuelle Code diese Semantik weiterhin verwendet. Leite daraus keine ungeprüfte Cache-Dauer für eine andere Datei ab.

Bei Refactoring bleiben Verhalten, öffentliche Schnittstellen, Datenformate, Reihenfolgen, Fehlerbehandlung, Sperren, Cooldowns, Transaktionsgrenzen und Paketpfade erhalten, sofern der Auftrag keine gezielte Änderung verlangt. Eine erfolglose Textsuche beweist nicht, dass Code verwaist ist. Prüfe dynamisches Laden, Skripte, Build- und Paketpfade vor Verschiebungen oder Löschungen. Verändere keine lokalen Daten, Einstellungen, Logs, Profile oder Build-Eingaben ohne geklärten Eigentümer und geklärte Verwendung.

## Prüfung und Rückmeldung

Führe die zum Auftrag passenden vorhandenen Prüf- und Buildaktionen aus. Unterscheide klar zwischen ausgeführten und nicht ausgeführten Prüfungen. Berichte bei Codeänderungen, was die betroffenen Dateien tun, welchem Modul sie gehören, welche Aufrufer den Ablauf verwenden, welche Verhaltensgrenzen geprüft wurden und was offenbleibt.

Bei einem ausdrücklich verlangten vollständigen Repository-Audit inventarisiere zuerst den vereinbarten Umfang. Lies jede Quelldatei inhaltlich und trenne vollständig gelesene Dateien von gezielt durchsuchten, generierten, vendorten oder ausgeschlossenen Dateien. Behaupte keine Vollständigkeit ohne diesen Nachweis.

Schreibe auf Deutsch mit Umlauten und `ß`. Verwende im deutschen Fließtext keine Strichpunkte.
