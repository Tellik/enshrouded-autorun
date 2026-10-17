---
{
  "id": "builtin-world-editor-panelpositiony",
  "locale": "de",
  "title": "Fensterposition Y ab oberem Rand · panelPositionY",
  "summary": "Vertikaler Abstand des World-Editor-Fensters vom oberen Rand des Spielfensters.",
  "navigation": {
    "path": [
      "Eingebaute Mods · Archiv"
    ]
  }
}
---

# Fensterposition Y ab oberem Rand

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Vertikaler Abstand des World-Editor-Fensters vom oberen Rand des Spielfensters.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("panelPositionY", fallback)`: `panelPositionY`.
- Datentyp: `number`.
- Startwert: `92`.
- Zahlenbereich: 0 bis 8192.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `0`.
- Maximum: `8192`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("panelPositionY", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
