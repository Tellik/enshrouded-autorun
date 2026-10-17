---
{
  "id": "control-slider",
  "locale": "de",
  "title": "Steuerelement `slider`",
  "summary": "Für Zahlen, die Spieler innerhalb eines Bereichs einstellen.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Steuerelemente",
      "Steuerelementtypen"
    ]
  }
}
---

# Steuerelement `slider`

Für Zahlen, die Spieler innerhalb eines Bereichs einstellen. Lege es bei einer Einstellung als `control` fest.

```json
{
  "settings": {
    "example": {"value": 1, "label": "Regler", "control": "slider"}
  }
}
```

Der Wert in `value` sollte zum Steuerelement passen. Weitere Regeln stehen unter [settings.<key>.control](#doc-field-settings-key-control).
