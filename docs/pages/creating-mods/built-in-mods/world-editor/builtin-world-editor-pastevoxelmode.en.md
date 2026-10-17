---
{
  "id": "builtin-world-editor-pastevoxelmode",
  "locale": "en",
  "title": "Voxel mode · pasteVoxelMode",
  "summary": "Controls how voxels are combined during paste. `replace` replaces target cells, while `add` adds only occupied source cells.",
  "navigation": {
    "path": [
      "Built-in mods · Archive"
    ]
  }
}
---

# Voxel mode

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Controls how voxels are combined during paste. `replace` replaces target cells, while `add` adds only occupied source cells.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("pasteVoxelMode", fallback)`: `pasteVoxelMode`.
- Type: `string`.
- Starting value: `"replace"`.
- Allowed values: `replace` for Replace target voxels, `add` for Add occupied source voxels.
- The control is inferred from the value type.
- Options: `replace` (Replace target voxels), `add` (Add occupied source voxels).

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("pasteVoxelMode", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
