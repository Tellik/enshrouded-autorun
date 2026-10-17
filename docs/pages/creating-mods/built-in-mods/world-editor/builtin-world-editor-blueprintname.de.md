---
{
  "id": "builtin-world-editor-blueprintname",
  "locale": "de",
  "title": "Dauerhafter Blueprint-Name · blueprintName",
  "summary": "Standardname für das Speichern eines Blueprints und den ersten Namen in den Mod-Aktionen zum Umbenennen oder Duplizieren.",
  "navigation": {
    "path": [
      "Eingebaute Mods",
      "Archiv",
      "World Editor"
    ]
  }
}
---

# Dauerhafter Blueprint-Name

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Standardname für das Speichern eines Blueprints und den ersten Namen in den Mod-Aktionen zum Umbenennen oder Duplizieren.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("blueprintName", fallback)`: `blueprintName`.
- Datentyp: `string`.
- Startwert: `"my_blueprint"`.
- Textlänge: 1 bis 64 Zeichen.
- Das Steuerelement wird aus dem Werttyp abgeleitet.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("blueprintName", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
