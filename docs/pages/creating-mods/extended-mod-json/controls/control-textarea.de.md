---
{
  "id": "control-textarea",
  "locale": "de",
  "title": "Steuerelement `textarea`",
  "summary": "Für längere Texte, die über mehrere Zeilen bearbeitet werden.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Steuerelemente",
      "Steuerelementtypen"
    ]
  }
}
---

# Steuerelement `textarea`

Für längere Texte, die über mehrere Zeilen bearbeitet werden. Lege es bei einer Einstellung als `control` fest.

```json
{
  "settings": {
    "example": {"value": "Longer text", "label": "Mehrzeiliges Textfeld", "control": "textarea"}
  }
}
```

Der Wert in `value` sollte zum Steuerelement passen. Weitere Regeln stehen unter [settings.<key>.control](#doc-field-settings-key-control).
