---
{
  "id": "builtin-world-editor-blueprintnewname",
  "locale": "en",
  "title": "New name for rename or duplicate · blueprintNewName",
  "summary": "Destination name for the mod actions that rename or duplicate a blueprint.",
  "navigation": {
    "path": [
      "Built-in mods · Archive"
    ]
  }
}
---

# New name for rename or duplicate

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Destination name for the mod actions that rename or duplicate a blueprint.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("blueprintNewName", fallback)`: `blueprintNewName`.
- Type: `string`.
- Starting value: `"my_blueprint_copy"`.
- Text length: 1 to 64 characters.
- The control is inferred from the value type.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("blueprintNewName", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
