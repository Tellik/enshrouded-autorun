---
{
  "id": "builtin-world-editor-panelpositionx",
  "locale": "en",
  "title": "Panel position X (−1 = centered) · panelPositionX",
  "summary": "Horizontal position relative to the game window. A value of −1 centers the window, nonnegative values set its left offset.",
  "navigation": {
    "path": [
      "Built-in mods · Archive"
    ]
  }
}
---

# Panel position X (−1 = centered)

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Horizontal position relative to the game window. A value of −1 centers the window, nonnegative values set its left offset.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("panelPositionX", fallback)`: `panelPositionX`.
- Type: `number`.
- Starting value: `-1`.
- Number range: -1 to 8192.
- The control is inferred from the value type.
- Minimum: `-1`.
- Maximum: `8192`.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("panelPositionX", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
