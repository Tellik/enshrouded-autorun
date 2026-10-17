---
{
  "id": "builtin-world-editor-panelwidth",
  "locale": "en",
  "title": "Blueprint panel width · panelWidth",
  "summary": "Width of the World Editor window in logical pixels. Changes apply while the game is running.",
  "navigation": {
    "path": [
      "Built-in mods · Archive"
    ]
  }
}
---

# Blueprint panel width

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Width of the World Editor window in logical pixels. Changes apply while the game is running.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("panelWidth", fallback)`: `panelWidth`.
- Type: `number`.
- Starting value: `1180`.
- Number range: 760 to 1920.
- The control is inferred from the value type.
- Minimum: `760`.
- Maximum: `1920`.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("panelWidth", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
