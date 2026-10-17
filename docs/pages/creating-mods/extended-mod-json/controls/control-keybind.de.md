---
{
  "id": "control-keybind",
  "locale": "de",
  "title": "Steuerelement `keybind`",
  "summary": "Für eine Tastenkombination, die der Modloader als Schlüsselwert speichert.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Steuerelemente",
      "Steuerelementtypen"
    ]
  }
}
---

# Steuerelement `keybind`

Für eine Tastenkombination, die der Modloader als Schlüsselwert speichert. Lege es bei einer Einstellung als `control` fest.

```json
{
  "settings": {
    "example": {"value": "F7", "label": "Tastenkürzel", "control": "keybind"}
  }
}
```

Der Wert in `value` sollte zum Steuerelement passen. Weitere Regeln stehen unter [settings.<key>.control](#doc-field-settings-key-control).
