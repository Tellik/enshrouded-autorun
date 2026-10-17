---
{
  "id": "builtin-sf-auto-stamina-refill-autorefill",
  "locale": "de",
  "title": "Ausdauer automatisch auffüllen · autoRefill",
  "summary": "Schaltet die wiederholte Auffüllung der Ausdauer bis zum Maximum ein oder aus.",
  "navigation": {
    "path": [
      "Eingebaute Mods",
      "Archiv",
      "SF Auto Stamina Refill"
    ]
  }
}
---

# Ausdauer automatisch auffüllen

> **Historische Referenz:** Dieses Modpaket gehört nicht zu den aktuell unter `mods/` enthaltenen Mods. Die Seite bleibt als Erklärung des früheren Pakets erhalten.

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Schaltet die wiederholte Auffüllung der Ausdauer bis zum Maximum ein oder aus.

## Bedeutung und Standard

Füllt die Ausdauer wiederholt bis zum Maximum auf.

- Mod: SF Auto Stamina Refill (`sf-auto-stamina-refill`).
- Schlüssel für `shroudforge.settings.get("autoRefill", fallback)`: `autoRefill`.
- Datentyp: `boolean`.
- Startwert: `true`.
- Erlaubte Werte: `true` oder `false`.
- Das Steuerelement wird aus dem Werttyp abgeleitet.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/sf-auto-stamina-refill/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("autoRefill", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua).
