---
{
  "id": "builtin-world-editor-panelwidth",
  "locale": "de",
  "title": "Breite des Blueprint-Fensters · panelWidth",
  "summary": "Breite des World-Editor-Fensters in logischen Pixeln. Änderungen gelten während des laufenden Spiels.",
  "navigation": {
    "path": [
      "Eingebaute Mods · Archiv"
    ]
  }
}
---

# Breite des Blueprint-Fensters

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Breite des World-Editor-Fensters in logischen Pixeln. Änderungen gelten während des laufenden Spiels.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("panelWidth", fallback)`: `panelWidth`.
- Datentyp: `number`.
- Startwert: `1180`.
- Zahlenbereich: 760 bis 1920.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `760`.
- Maximum: `1920`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("panelWidth", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
