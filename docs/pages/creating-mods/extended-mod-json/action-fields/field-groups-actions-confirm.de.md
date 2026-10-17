---
{
  "id": "field-groups-actions-confirm",
  "locale": "de",
  "title": "Feld `groups[].actions[].confirm`",
  "summary": "Optionale Rückfrage vor dem Ausführen einer Aktion.",
  "navigation": {
    "path": [
      "Gruppen und Aktionen"
    ]
  }
}
---

# Feld `groups[].actions[].confirm`

Optionale Rückfrage vor dem Ausführen einer Aktion.

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
      "actions": [
        {
          "id": "reset",
          "label": "Reset",
          "confirm": "example"
        }
      ]
    }
  ]
}
```

## Quelle

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
