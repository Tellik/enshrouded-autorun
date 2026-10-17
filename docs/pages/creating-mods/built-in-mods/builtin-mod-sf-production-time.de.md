---
{
  "id": "builtin-mod-sf-production-time",
  "locale": "de",
  "title": "SF Production Time",
  "summary": "Legt eine Basisdauer für zeitgesteuerte Produktionsrezepte fest. Weltgeschwindigkeitseinstellungen gelten weiterhin.",
  "navigation": {
    "path": [
      "Eingebaute Mods",
      "Archiv",
      "SF Production Time"
    ],
    "overview": true,
    "label": "Übersicht"
  }
}
---

# SF Production Time

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Legt eine Basisdauer für zeitgesteuerte Produktionsrezepte fest. Weltgeschwindigkeitseinstellungen gelten weiterhin.

## Paketangaben

- Mod-ID: `sf-production-time`
- Zielprozesse: Client und Dedicated Server
- Laufzeitberechtigung: nein
- Deklarierte spielerseitige Einstellungen: 1

## Einstellungen

## Produktionszeit

Lege vor dem Spielstart denselben Wert auf Client und Host oder Server fest. Weltgeschwindigkeitseinstellungen multiplizieren diese Basisdauer. Sofortrezepte und Inventarmengen bleiben unverändert.

- [Basis-Produktionszeit in Sekunden](#doc-builtin-sf-production-time-seconds)

Die Einstellungen werden in `extended.mod.json` des Mod-Pakets gespeichert. Der Mod liest sie über `shroudforge.settings.get(key, fallback)`. Weitere Details stehen in der [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-production-time/src/mod.lua).
