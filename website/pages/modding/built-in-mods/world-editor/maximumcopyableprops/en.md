# Maximum props per blueprint

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Maximum props per blueprint. Capture, save, load, and paste reject larger blueprints instead of silently dropping props.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("maximumCopyableProps", fallback)`: `maximumCopyableProps`.
- Type: `number`.
- Starting value: `60000`.
- Number range: 1 to 1000000.
- The control is inferred from the value type.
- Minimum: `1`.
- Maximum: `1000000`.

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("maximumCopyableProps", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
