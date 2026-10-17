---
{
  "id": "builtin-mod-sf-unlimited-flight",
  "locale": "de",
  "title": "SF Unlimited Flight",
  "summary": "Fünf veränderbare Einstellungen für Flugdauer, Drehrate, Vorwärtsbewegung und Luftwiderstand.",
  "navigation": {
    "path": [
      "Eingebaute Mods · SF Unlimited Flight"
    ]
  },
  "source": {
    "file": "mods/sf-unlimited-flight/extended.mod.json",
    "pointer": "/settings"
  }
}
---

# SF Unlimited Flight

SF Unlimited Flight bietet fünf Einstellungen für das Fliegen. Das Paket zielt auf Client und Dedicated Server und verwendet die Laufzeitberechtigung `runtime`.

| Einstellung | Startwert | Bereich und Wirkung |
| --- | --- | --- |
| `unlimitedFlying` | `false` | Kontrollkästchen. Ermöglicht die aufwärts gerichtete Steuerung ohne Verbrauch der Gleiter-Ausdauer. |
| `turnSpeed` | `100` | 25 bis 400 Prozent, in 5er-Schritten. Multipliziert die ursprüngliche Gleiter-Drehrate. |
| `moveSpeed` | `100` | 25 bis 400 Prozent, in 5er-Schritten. Multipliziert die ursprüngliche Vorwärtsbeschleunigung. |
| `liftSpeed` | `100` | 25 bis 400 Prozent, in 5er-Schritten. Multipliziert den Luftwiderstand in Längsrichtung. Ein höherer Wert bedeutet mehr Widerstand. |
| `descendSpeed` | `100` | 25 bis 400 Prozent, in 5er-Schritten. Multipliziert den vertikalen Luftwiderstand. Ein höherer Wert bedeutet mehr Widerstand. |

Die Mod liest die Schlüssel aus ihrer `extended.mod.json`. Die Einstellung `unlimitedFlying` setzt Pitch-Speed-Steuerung und einen Spieler in der maßgeblichen Welt voraus. Die vier Prozentregler verwenden die Koeffizienten des aktiven Gleiters.

Die [Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/HEAD/mods/sf-unlimited-flight/README.md) enthält die technischen Grenzen und Wiederherstellungsregeln.
