---
{
  "id": "control-segmented",
  "locale": "de",
  "title": "Steuerelement `segmented`",
  "summary": "Zeigt Auswahlwerte als eine Reihe von Schaltflächen.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Steuerelemente",
      "Steuerelementtypen"
    ]
  }
}
---

# Steuerelement `segmented`

Zeigt Auswahlwerte als eine Reihe von Schaltflächen. Lege es bei einer Einstellung als `control` fest.

```json
{
  "settings": {
    "example": {"value": "easy", "label": "Segmentierte Auswahl", "control": "segmented", "options": {"easy": "Einfach", "hard": "Schwer"}}
  }
}
```

Die Optionsschlüssel sind gespeicherte Werte. Bei `multiselect` muss `value` eine Liste sein. Weitere Regeln stehen unter [settings.<key>.control](#doc-field-settings-key-control).
