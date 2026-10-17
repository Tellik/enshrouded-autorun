---
{
  "id": "builtin-world-editor-rotationquarterturns",
  "locale": "en",
  "title": "Initial paste rotation (F3 cycles in game, 0–3) · rotationQuarterTurns",
  "summary": "Initial rotation of the active blueprint in quarter turns. 0, 1, 2, and 3 mean 0°, 90°, 180°, and 270°. F3 advances it in game.",
  "navigation": {
    "path": [
      "Built-in mods",
      "Archive",
      "World Editor"
    ]
  }
}
---

# Initial paste rotation (F3 cycles in game, 0–3)

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Initial rotation of the active blueprint in quarter turns. 0, 1, 2, and 3 mean 0°, 90°, 180°, and 270°. F3 advances it in game.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("rotationQuarterTurns", fallback)`: `rotationQuarterTurns`.
- Type: `number`.
- Starting value: `0`.
- Number range: 0 to 3.
- The control is inferred from the value type.
- Minimum: `0`.
- Maximum: `3`.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("rotationQuarterTurns", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
