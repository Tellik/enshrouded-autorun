---
{
  "id": "field-settings-key-min",
  "locale": "de",
  "title": "Feld `settings.<key>.min`",
  "summary": "Kleinster zulässiger Eingabewert.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Einstellungen",
      "Einstellungsfelder"
    ]
  }
}
---

# Feld `settings.<key>.min`

Kleinster zulässiger Eingabewert.

## Datentyp und Pflicht

`number` · optional

## Erlaubte Werte und Grenzen

Keine zusätzlichen Werte oder Grenzen im Schema angegeben.

## Beispiel

```json
{
  "settings": {
    "example": {
      "value": 1,
      "control": "number",
      "min": 1
    }
  }
}
```

## Quelle

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
