# Ausdauer automatisch auffüllen

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
