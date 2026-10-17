---
{
  "id": "builtin-mod-sf-production-time",
  "locale": "en",
  "title": "SF Production Time",
  "summary": "Sets timed production recipes to a chosen base duration. World speed settings still apply.",
  "navigation": {
    "path": [
      "Built-in mods · Archive"
    ]
  }
}
---

# SF Production Time

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Sets timed production recipes to a chosen base duration. World speed settings still apply.

## Package details

- Mod ID: `sf-production-time`
- Target processes: client and Dedicated Server
- Runtime capability: no
- Declared player settings: 1

## Settings

## Production time

Apply the same value to client and host/server before starting the game. World speed settings multiply this base duration. Instant recipes and inventory quantities are preserved.

- [Base production time (seconds)](#doc-builtin-sf-production-time-seconds)

Settings are stored in the mod package's `extended.mod.json`. The mod reads them with `shroudforge.settings.get(key, fallback)`. See the [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-production-time/src/mod.lua) for implementation details.
