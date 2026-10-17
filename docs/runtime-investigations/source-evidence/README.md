# Geschlossene historische Quellbelege

Diese Dateien dokumentieren frühere Untersuchungen eines bestimmten Spielbuilds oder einer bestimmten lokalen Sitzung. Sie sind als Text erhalten. Sie werden weder importiert noch als aktuelle Entwicklungsaufträge angeboten.

Die ursprünglichen 22 Python-Dateien im lokalen Capturebereich wurden inhaltlich geprüft. Davon wurde der wiederverwendbare Export beobachteter Registrierungen nach `dev/profiles/export_observed_registry.py` übernommen. Die übrigen 21 Dateien bleiben hier bytegleich erhalten. Dazu gehört auch das unvollständige Laden eines inzwischen entfernten Buildskripts mittels `exec`. Diese Abhängigkeit wird nicht in einen aktiven Auftrag übernommen.

Zusätzlich bleibt der frühere `audit-mod-origins.py` als Beleg erhalten. Seine feste Deckungsprüfung für inzwischen entfernte Modpakete gehört nicht zur aktuellen Runtimeprüfung. Die wiederverwendbaren Payload-, Patch-, Hook- und World-Prüfungen liegen als tatsächliche Aktionsimplementierungen unter `dev/profiles/`.

[manifest.json](manifest.json) hält für jede historische Quelldatei den früheren Pfad, den heutigen Pfad und die SHA-256-Prüfsumme fest. Namen, alte Pfade und feste Prozessnummern innerhalb dieser Belege wurden absichtlich unverändert erhalten.

Originale EXE-/KFC-Dateien und die zugehörigen Aufnahmen bleiben lokal unter `dev/profiles/captures/`. Das dortige Erhaltungsmanifest bestätigt die Verschiebung mit Prüfsummen. Diese Daten werden nicht als Compilerprodukte behandelt.
