---
{
  "id": "field-groups-description",
  "locale": "de",
  "title": "Feld `groups[].description`",
  "summary": "Kurzer Text für die Modseite.",
  "navigation": {
    "path": [
      "Gruppen und Aktionen"
    ]
  }
}
---

# Feld `groups[].description`

Kurzer Text für die Modseite.

## Datentyp und Pflicht

`string` · optional

## Erlaubte Werte und Grenzen

- Höchstlänge: `500`

## Beispiel

```json
{
  "groups": [
    {
      "label": "Example",
      "settings": [],
      "description": "example"
    }
  ]
}
```

## Quelle

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
