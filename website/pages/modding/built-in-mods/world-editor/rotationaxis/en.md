# Blueprint up axis (stored on capture)

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Up axis for new captures. It is stored in the blueprint, and existing blueprints keep their own axis.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("rotationAxis", fallback)`: `rotationAxis`.
- Type: `string`.
- Starting value: `"y"`.
- Allowed values: `x` for X axis, `y` for Y axis, `z` for Z axis.
- The control is inferred from the value type.
- Options: `x` (X axis), `y` (Y axis), `z` (Z axis).

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("rotationAxis", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
