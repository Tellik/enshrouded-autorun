---
{
  "id": "control-number",
  "locale": "en",
  "title": "The `number` control",
  "summary": "For numeric values. min, max, and step can constrain input.",
  "navigation": {
    "path": [
      "Controls"
    ]
  }
}
---

# The `number` control

For numeric values. min, max, and step can constrain input. Set it as `control` on a setting.

```json
{
  "settings": {
    "example": {"value": 1, "label": "Number field", "control": "number"}
  }
}
```

The value in `value` should match the selected control. See [settings.<key>.control](#doc-field-settings-key-control) for the schema rules.
