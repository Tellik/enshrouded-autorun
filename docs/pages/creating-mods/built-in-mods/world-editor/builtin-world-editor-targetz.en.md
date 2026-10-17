---
{
  "id": "builtin-world-editor-targetz",
  "locale": "en",
  "title": "Target cell Z · targetZ",
  "summary": "Manual Z target coordinate in voxel cells for a blueprint with voxels when neither the cursor nor a marked target is used.",
  "navigation": {
    "path": [
      "Built-in mods",
      "Archive",
      "World Editor"
    ]
  }
}
---

# Target cell Z

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Manual Z target coordinate in voxel cells for a blueprint with voxels when neither the cursor nor a marked target is used.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("targetZ", fallback)`: `targetZ`.
- Type: `number`.
- Starting value: `0`.
- Numeric value. The mod package declares no additional limit.
- The control is inferred from the value type.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("targetZ", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
