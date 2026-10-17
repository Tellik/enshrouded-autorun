---
{
  "id": "field-settings-key-minlength",
  "locale": "de",
  "title": "Feld `settings.<key>.minLength`",
  "summary": "Mindestlänge eines Textwerts.",
  "navigation": {
    "path": [
      "Einstellungen"
    ]
  }
}
---

# Feld `settings.<key>.minLength`

Mindestlänge eines Textwerts.

## Datentyp und Pflicht

`integer` · optional

## Erlaubte Werte und Grenzen

- Minimum: `0`

## Beispiel

```json
{
  "settings": {
    "example": {
      "value": "hello",
      "control": "text",
      "minLength": 3
    }
  }
}
```

## Quelle

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
