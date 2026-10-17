---
{
  "id": "builtin-sf-auto-stamina-refill-autorefill",
  "locale": "en",
  "title": "Auto Refill · autoRefill",
  "summary": "Turns repeated stamina refill to maximum on or off.",
  "navigation": {
    "path": [
      "Built-in mods",
      "Archive",
      "SF Auto Stamina Refill"
    ]
  }
}
---

# Auto Refill

> **Historical reference:** This package is not among the mods currently included under `mods/`. The page remains available to explain the former package.

> Historical reference mod. This package is not one of the two currently bundled mods. Source links point to the verified repository revision before its removal.

Turns repeated stamina refill to maximum on or off.

## Meaning and default

Repeatedly refills stamina to maximum.

- Mod: SF Auto Stamina Refill (`sf-auto-stamina-refill`).
- Key read with `shroudforge.settings.get("autoRefill", fallback)`: `autoRefill`.
- Type: `boolean`.
- Starting value: `true`.
- Allowed values: `true` or `false`.
- The control is inferred from the value type.

## Source of this information

This page is based on `reference-mods-0.9/sf-auto-stamina-refill/extended.mod.json`. Lua reads the current value with `shroudforge.settings.get("autoRefill", fallback)`. Modloader saves player changes in the extension file. [Lua source file](https://github.com/bonsaibauer/shroudforge/blob/81f9c5fe2d70ea4ba10896fb22a75b2d082800a6/reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua).
