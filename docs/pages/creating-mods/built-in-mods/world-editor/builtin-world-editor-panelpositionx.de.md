---
{
  "id": "builtin-world-editor-panelpositionx",
  "locale": "de",
  "title": "Fensterposition X, minus eins zentriert · panelPositionX",
  "summary": "Horizontale Position relativ zum Spielfenster. Der Wert −1 zentriert das Fenster, nichtnegative Werte legen den linken Abstand fest.",
  "navigation": {
    "path": [
      "Eingebaute Mods",
      "Archiv",
      "World Editor"
    ]
  }
}
---

# Fensterposition X, minus eins zentriert

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Horizontale Position relativ zum Spielfenster. Der Wert −1 zentriert das Fenster, nichtnegative Werte legen den linken Abstand fest.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("panelPositionX", fallback)`: `panelPositionX`.
- Datentyp: `number`.
- Startwert: `-1`.
- Zahlenbereich: -1 bis 8192.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `-1`.
- Maximum: `8192`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("panelPositionX", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
