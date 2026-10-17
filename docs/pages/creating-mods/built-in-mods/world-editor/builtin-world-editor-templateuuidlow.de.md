---
{
  "id": "builtin-world-editor-templateuuidlow",
  "locale": "de",
  "title": "Untere Hälfte der Template-UUID in Hexadezimal · templateUuidLow",
  "summary": "Untere hexadezimale Hälfte der nativen Template-UUID für die erweiterte Aktion Queue prop spawn.",
  "navigation": {
    "path": [
      "Eingebaute Mods",
      "Archiv",
      "World Editor"
    ]
  }
}
---

# Untere Hälfte der Template-UUID in Hexadezimal

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Untere hexadezimale Hälfte der nativen Template-UUID für die erweiterte Aktion Queue prop spawn.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("templateUuidLow", fallback)`: `templateUuidLow`.
- Datentyp: `string`.
- Startwert: `"0000000000000000"`.
- Textwert. Das Mod-Paket legt keine weitere Längenbegrenzung fest.
- Das Steuerelement wird aus dem Werttyp abgeleitet.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("templateUuidLow", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
