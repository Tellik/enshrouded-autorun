---
{
  "id": "builtin-world-editor-rotationquarterturns",
  "locale": "de",
  "title": "Drehung vor dem Einfügen · rotationQuarterTurns",
  "summary": "Anfangsdrehung des aktiven Blueprints in Vierteldrehungen. 0, 1, 2 und 3 entsprechen 0°, 90°, 180° und 270°. F3 erhöht den Wert im Spiel.",
  "navigation": {
    "path": [
      "Eingebaute Mods · Archiv"
    ]
  }
}
---

# Drehung vor dem Einfügen

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Anfangsdrehung des aktiven Blueprints in Vierteldrehungen. 0, 1, 2 und 3 entsprechen 0°, 90°, 180° und 270°. F3 erhöht den Wert im Spiel.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("rotationQuarterTurns", fallback)`: `rotationQuarterTurns`.
- Datentyp: `number`.
- Startwert: `0`.
- Zahlenbereich: 0 bis 3.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `0`.
- Maximum: `3`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("rotationQuarterTurns", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
