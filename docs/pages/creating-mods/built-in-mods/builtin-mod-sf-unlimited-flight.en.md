---
{
  "id": "builtin-mod-sf-unlimited-flight",
  "locale": "en",
  "title": "SF Unlimited Flight",
  "summary": "Five adjustable settings for flight duration, yaw rate, forward movement, and air resistance.",
  "navigation": {
    "path": [
      "Built-in mods · SF Unlimited Flight"
    ]
  },
  "source": {
    "file": "mods/sf-unlimited-flight/extended.mod.json",
    "pointer": "/settings"
  }
}
---

# SF Unlimited Flight

SF Unlimited Flight provides five flight settings. The package targets the client and Dedicated Server and declares the `runtime` capability.

| Setting | Starting value | Range and effect |
| --- | --- | --- |
| `unlimitedFlying` | `false` | Checkbox. Enables upward steering without glider stamina depletion. |
| `turnSpeed` | `100` | 25 to 400 percent in steps of 5. Multiplies the original glider yaw rate. |
| `moveSpeed` | `100` | 25 to 400 percent in steps of 5. Multiplies the original forward acceleration. |
| `liftSpeed` | `100` | 25 to 400 percent in steps of 5. Multiplies longitudinal air resistance. Higher values add drag. |
| `descendSpeed` | `100` | 25 to 400 percent in steps of 5. Multiplies vertical air resistance. Higher values add drag. |

The mod reads these keys from its `extended.mod.json`. The `unlimitedFlying` setting requires pitch-speed controls and one player in the authoritative world. The four percentage controls use the active glider's coefficients.

The [mod guide](https://github.com/bonsaibauer/shroudforge/blob/HEAD/mods/sf-unlimited-flight/README.md) describes technical limits and restoration rules.
