# Enshrouded-Mod-Dateibaum exportieren

`export_enshrouded_mod_catalog.ps1` liest die Enshrouded-Mods über die Nexus-Mods-API aus und erstellt aus den verfügbaren Datei-Vorschauen einen Baum aller aufgelisteten Archivdateien. Es filtert nicht nach Dateiendungen. Das Skript lädt keine Modarchive herunter.

## Voraussetzungen

- PowerShell unter Windows
- Ein Nexus-Mods-Konto und ein Personal API Key aus den [Nexus API Key Settings](https://www.nexusmods.com/settings/api-keys)
- Netzwerkzugriff auf die Nexus-API und deren Vorschau-CDN

Der API-Schlüssel ist ein persönliches Geheimnis. Nicht in Git einchecken und nicht in den Chat oder eine Projektdatei kopieren. Falls ein Schlüssel versehentlich geteilt wurde, ihn in den Nexus-Kontoeinstellungen widerrufen und einen neuen erstellen.

## Start

Eine PowerShell im ShroudForge-Projektstamm öffnen. Den Schlüssel verdeckt eingeben und das Skript im selben Fenster starten:

```powershell
$secureKey = Read-Host 'Nexus Personal API Key' -AsSecureString
$env:NEXUS_API_KEY = [System.Net.NetworkCredential]::new('', $secureKey).Password
./dev/export_enshrouded_mod_catalog.ps1
Remove-Item Env:NEXUS_API_KEY
Remove-Variable secureKey
```

Die Standardausgabe ist:

```text
build/nexusmods/enshrouded-latest-code-files.tree.txt
```

`build/` enthält generierte Ausgaben und wird nicht als Quelltext gepflegt.

## Optionen

Ohne Optionen verarbeitet das Skript alle gefundenen Enshrouded-Mods. Es liest die gelisteten Dateien der Kategorien `MAIN`, `UPDATE`, `OPTIONAL` und `MISCELLANEOUS`, einschließlich mehrerer Dateien beziehungsweise Releases pro Mod. Aus jeder Datei-Vorschau kommen alle Pfade in den Baum, unabhängig von der Dateiendung. Dazu gehören zum Beispiel DLLs, EXEs, Bilder, JSON-Dateien und Skripte.

| Option | Bedeutung |
| --- | --- |
| `-Category <Name>` | Beschränkt die Mod-Suche auf den exakten Namen einer Nexus-Kategorie. |
| `-OutputPath <Pfad>` | Ändert den Ausgabepfad. Relative Pfade beziehen sich auf den Projektstamm. |
| `-PageSize <Zahl>` | Anzahl Mods pro GraphQL-Seite, Standard `100`, erlaubter Bereich `1` bis `500`. |
| `-DelayMilliseconds <Zahl>` | Pause zwischen Mods, Standard `250`. |
| `-MaxMods <Zahl>` | Begrenzt die Verarbeitung zum Ausprobieren. `0` bedeutet alle Mods. |

Beispiel mit Begrenzung und eigenem Ausgabepfad:

```powershell
./dev/export_enshrouded_mod_catalog.ps1 -MaxMods 5 -OutputPath 'build/nexusmods/enshrouded-sample.txt'
```

Die Optionen können mit dem Startbefehl kombiniert werden. Zum Beispiel:

```powershell
./dev/export_enshrouded_mod_catalog.ps1 -Category 'Kategoriename' -MaxMods 10
```

## Ergebnis und Grenzen

Während des Laufs meldet das Skript den Fortschritt und schreibt Fehler beim Lesen einzelner Vorschauen in den Ergebnisbaum. Es verarbeitet sowohl das ältere Baumformat als auch das neuere JSON-Format mit `file_path`. Wenn die API keinen `content_preview_link` liefert, wird das ausdrücklich als Vorschaufehler ausgewiesen. Eine geladene Vorschau ohne Dateipfade bleibt davon unterscheidbar. Die Vorschau enthält Metadaten zum Archivinhalt. Das Skript lädt die Archive selbst nicht herunter.
