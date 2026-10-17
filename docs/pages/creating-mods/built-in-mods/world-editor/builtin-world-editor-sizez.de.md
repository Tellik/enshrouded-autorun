---
{
  "id": "builtin-world-editor-sizez",
  "locale": "de",
  "title": "Auswahlgröße Z als manueller Ersatzwert · sizeZ",
  "summary": "Manuelle Ausdehnung der Aufnahme entlang Z in Voxel-Zellen. Wird nur verwendet, wenn keine vollständige Auswahl über Cursor-Ecken vorliegt.",
  "navigation": {
    "path": [
      "Eingebaute Mods · Archiv"
    ]
  }
}
---

# Auswahlgröße Z als manueller Ersatzwert

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Manuelle Ausdehnung der Aufnahme entlang Z in Voxel-Zellen. Wird nur verwendet, wenn keine vollständige Auswahl über Cursor-Ecken vorliegt.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("sizeZ", fallback)`: `sizeZ`.
- Datentyp: `number`.
- Startwert: `8`.
- Zahlenbereich: 1 bis 65536.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `1`.
- Maximum: `65536`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("sizeZ", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
