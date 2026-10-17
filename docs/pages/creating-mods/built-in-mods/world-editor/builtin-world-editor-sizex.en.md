---
{
  "id": "builtin-world-editor-sizex",
  "locale": "en",
  "title": "Size X (manual fallback) · sizeX",
  "summary": "Manual capture extent along X in voxel cells. Used only when there is no complete selection marked by cursor corners.",
  "navigation": {
    "path": [
      "Built-in mods",
      "Archive",
      "World Editor"
    ]
  }
}
---

# Size X (manual fallback)

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Manual capture extent along X in voxel cells. Used only when there is no complete selection marked by cursor corners.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("sizeX", fallback)`: `sizeX`.
- Type: `number`.
- Starting value: `8`.
- Number range: 1 to 65536.
- The control is inferred from the value type.
- Minimum: `1`.
- Maximum: `65536`.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("sizeX", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
