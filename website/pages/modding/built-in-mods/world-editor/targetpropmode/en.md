# Props at target

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Controls whether props at the target are kept or safely verified intersecting props are replaced.

## Meaning and default

Copy, rotate, save, and paste parts of the world.

- Mod: World Editor (`world-editor`).
- Key read with `shroudforge.settings.get("targetPropMode", fallback)`: `targetPropMode`.
- Type: `string`.
- Starting value: `"keep"`.
- Allowed values: `keep` for Keep existing props, `replace` for Replace intersecting props.
- The control is inferred from the value type.
- Options: `keep` (Keep existing props), `replace` (Replace intersecting props).

## Source of this information

This page is based on `reference-mods-0.9/world-editor/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("targetPropMode", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/src/mod.lua).

[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/world-editor/README.md)
