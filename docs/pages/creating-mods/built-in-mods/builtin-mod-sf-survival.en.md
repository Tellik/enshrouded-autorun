---
{
  "id": "builtin-mod-sf-survival",
  "source": {
    "file": "mods/sf-survival/extended.mod.json",
    "pointer": "/settings"
  },
  "locale": "en",
  "title": "SF Survival",
  "summary": "Eight optional survival settings from the current mod package.",
  "navigation": {
    "path": [
      "Built-in mods"
    ]
  }
}
---

# SF Survival

SF Survival provides eight individually switchable survival features. The mod declares the `runtime` capability and targets both the client and Dedicated Server.

| Setting | Starting value | Effect declared by the package |
| --- | --- | --- |
| `noFallDamage` | `false` | Turns the fall-damage adjustment on or off. |
| `fastHealthRegen` | `false` | Increases existing health regeneration. It has no effect without natural regeneration. |
| `fastManaRegen` | `false` | Increases existing mana regeneration. It has no effect without natural regeneration. |
| `infiniteStamina` | `false` | Keeps stamina at its current maximum through the existing attribute bound. |
| `infiniteUnderwaterBreath` | `false` | Keeps the underwater breath reserve at its current maximum. |
| `infiniteWeaponDurability` | `false` | Disables durability loss for the hosted world. This also affects other players' items. |
| `infiniteFrostTimer` | `false` | Keeps body warmth at its current maximum through the existing attribute bound. |
| `infiniteMana` | `false` | Keeps mana at its current maximum through the existing attribute bound. |

The mod reads its values from `extended.mod.json`. It does not depend on SF Unlimited Flight. The [mod guide](https://github.com/bonsaibauer/shroudforge/blob/HEAD/mods/sf-survival/README.md) describes effect scopes and restoration of original game values.
