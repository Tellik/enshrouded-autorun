---
{
  "id": "builtin-world-editor-feedbackid",
  "locale": "de",
  "title": "Material-Feedback-ID nur für Platzierung · feedbackId",
  "summary": "Material-Feedback-ID aus der ItemInfo-Platzierungsdefinition. Die erweiterte Aktion Place at coordinates benötigt sie, Destroy at coordinates nicht.",
  "navigation": {
    "path": [
      "Eingebaute Mods",
      "Archiv",
      "World Editor"
    ]
  }
}
---

# Material-Feedback-ID nur für Platzierung

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Material-Feedback-ID aus der ItemInfo-Platzierungsdefinition. Die erweiterte Aktion Place at coordinates benötigt sie, Destroy at coordinates nicht.

## Bedeutung und Standard

Weltbereiche kopieren, drehen, speichern und einfügen.

- Mod: World Editor (`world-editor`).
- Schlüssel für `shroudforge.settings.get("feedbackId", fallback)`: `feedbackId`.
- Datentyp: `number`.
- Startwert: `0`.
- Zahlenwert. Das Mod-Paket legt keine weitere Grenze fest.
- Das Steuerelement wird aus dem Werttyp abgeleitet.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/world-editor/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("feedbackId", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
