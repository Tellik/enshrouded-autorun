---
{
  "id": "builtin-mod-sf-survival",
  "source": {
    "file": "mods/sf-survival/extended.mod.json",
    "pointer": "/settings"
  },
  "locale": "de",
  "title": "SF Survival",
  "summary": "Acht optionale Überlebenseinstellungen aus dem aktuellen Modpaket.",
  "navigation": {
    "path": [
      "Eingebaute Mods"
    ]
  }
}
---

# SF Survival

SF Survival bietet acht einzeln schaltbare Überlebensfunktionen. Die Mod verwendet die Laufzeitberechtigung `runtime` und ist für Client und Dedicated Server deklariert.

| Einstellung | Startwert | Wirkung laut Modpaket |
| --- | --- | --- |
| `noFallDamage` | `false` | Schaltet die Anpassung des Fallschadens aus oder ein. |
| `fastHealthRegen` | `false` | Erhöht die vorhandene Gesundheitsregeneration. Ohne natürliche Regeneration hat diese Einstellung keine Wirkung. |
| `fastManaRegen` | `false` | Erhöht die vorhandene Manaregeneration. Ohne natürliche Regeneration hat diese Einstellung keine Wirkung. |
| `infiniteStamina` | `false` | Hält Ausdauer über die vorhandene Attributgrenze auf ihrem aktuellen Maximum. |
| `infiniteUnderwaterBreath` | `false` | Hält den Atemvorrat unter Wasser auf seinem aktuellen Maximum. |
| `infiniteWeaponDurability` | `false` | Schaltet den Haltbarkeitsverlust für die gehostete Welt aus. Das betrifft auch Gegenstände anderer Spieler. |
| `infiniteFrostTimer` | `false` | Hält Körperwärme über die vorhandene Attributgrenze auf ihrem aktuellen Maximum. |
| `infiniteMana` | `false` | Hält Mana über die vorhandene Attributgrenze auf seinem aktuellen Maximum. |

Die Mod liest ihre Werte aus `extended.mod.json`. Sie besitzt keine Abhängigkeit von SF Unlimited Flight. Die [Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/HEAD/mods/sf-survival/README.md) beschreibt die Wirkungsbereiche und die Wiederherstellung der ursprünglichen Spielwerte.
