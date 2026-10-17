---
{
  "id": "builtin-sf-production-time-seconds",
  "locale": "en",
  "title": "Base production time (seconds) · seconds",
  "summary": "Sets the base duration in seconds for timed production recipes. Set the same value on the client and host or server before starting the game.",
  "navigation": {
    "path": [
      "Built-in mods",
      "Archive",
      "SF Production Time"
    ]
  }
}
---

# Base production time (seconds)

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Sets the base duration in seconds for timed production recipes. Set the same value on the client and host or server before starting the game.

## Meaning and default

Sets timed production recipes to a chosen base duration. World speed settings still apply.

- Mod: SF Production Time (`sf-production-time`).
- Key read with `shroudforge.settings.get("seconds", fallback)`: `seconds`.
- Type: `number`.
- Starting value: `1`.
- Number range: 0.1 to 3600.
- The control is inferred from the value type.
- Minimum: `0.1`.
- Maximum: `3600`.

## Source of this information

This page is based on `reference-mods-0.9/sf-production-time/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("seconds", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-production-time/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-production-time/README.md)
