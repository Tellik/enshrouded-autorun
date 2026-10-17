---
{
  "id": "builtin-world-editor-rotationaxis",
  "locale": "de",
  "title": "Obere Blueprint-Achse · rotationAxis",
  "summary": "Obere Achse neuer Aufnahmen. Sie wird im Blueprint gespeichert, bereits gespeicherte Blueprints behalten ihre eigene Achse.",
  "navigation": {
    "path": [
      "Eingebaute Mods · Archiv"
    ]
  }
}
---

# Obere Blueprint-Achse

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Obere Achse neuer Aufnahmen. Sie wird im Blueprint gespeichert, bereits gespeicherte Blueprints behalten ihre eigene Achse.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("rotationAxis", fallback)`: `rotationAxis`.
- Datentyp: `string`.
- Startwert: `"y"`.
- Erlaubte Werte: `x` für X-Achse, `y` für Y-Achse, `z` für Z-Achse.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Optionen: `x` (X-Achse), `y` (Y-Achse), `z` (Z-Achse).

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("rotationAxis", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
