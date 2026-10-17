# Basis-Produktionszeit in Sekunden

> Historischer Referenzmod. Dieses Paket gehört nicht zu den zwei aktuell mitgelieferten Mods. Die Quellenlinks zeigen den nachgewiesenen Repository-Stand vor seiner Entfernung.

Legt die Basisdauer in Sekunden für alle zeitgesteuerten Produktionsrezepte fest. Stelle denselben Wert auf Client und Host oder Server ein, bevor das Spiel startet.

## Bedeutung und Standard

Legt eine Basisdauer für zeitgesteuerte Produktionsrezepte fest. Weltgeschwindigkeitseinstellungen gelten weiterhin.

- Mod: SF Production Time (`sf-production-time`).
- Schlüssel für `shroudforge.settings.get("seconds", fallback)`: `seconds`.
- Datentyp: `number`.
- Startwert: `1`.
- Zahlenbereich: 0.1 bis 3600.
- Das Steuerelement wird aus dem Werttyp abgeleitet.
- Minimum: `0.1`.
- Maximum: `3600`.

## Woher die Angaben kommen

Diese Seite basiert auf `reference-mods-0.9/sf-production-time/extended.mod.json`. Lua liest den aktuellen Wert mit `shroudforge.settings.get("seconds", fallback)`. Spieleränderungen speichert der Modloader in der Erweiterungsdatei. [Lua-Quelldatei](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-production-time/src/mod.lua).

[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-production-time/README.md)
