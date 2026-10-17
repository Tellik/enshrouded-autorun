---
{
  "id": "builtin-world-editor-panelheight",
  "locale": "de",
  "title": "Höhe des Blueprint-Fensters · panelHeight",
  "summary": "Höhe des World-Editor-Fensters in logischen Pixeln. Änderungen gelten während des laufenden Spiels.",
  "navigation": {
    "path": [
      "Eingebaute Mods · Archiv"
    ]
  }
}
---

# Höhe des Blueprint-Fensters

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Höhe des World-Editor-Fensters in logischen Pixeln. Änderungen gelten während des laufenden Spiels.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("panelHeight", fallback)`: `panelHeight`.
- Datentyp: `number`.
- Startwert: `190`.
- Zahlenbereich: 140 bis 400.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `140`.
- Maximum: `400`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("panelHeight", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
