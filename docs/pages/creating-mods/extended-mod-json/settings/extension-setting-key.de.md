---
{
  "id": "extension-setting-key",
  "locale": "de",
  "title": "Schlüssel `settings.<key>`",
  "summary": "Der Schlüssel benennt eine Einstellung und ist zugleich der Name, den Lua zum Lesen ihres aktuellen Werts verwendet.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Einstellungen",
      "Einstellungsfelder"
    ]
  }
}
---

# Schlüssel `settings.<key>`

Der Platzhalter `<key>` wird durch einen Namen ersetzt, den der Mod-Autor wählt. Zum Beispiel heißt `settings.flightSpeed` eine Einstellung `flightSpeed`.

Erlaubt sind 1 bis 80 Buchstaben, Zahlen, Punkte, Bindestriche und Unterstriche. Groß- und Kleinschreibung zählt. Lua verwendet exakt denselben Namen: `shroudforge.settings.get("flightSpeed", 1.0)`.

Dieser Schlüssel ist kein globaler ShroudForge-Schlüssel. Jeder Mod hat seine eigenen Einstellungswerte.
