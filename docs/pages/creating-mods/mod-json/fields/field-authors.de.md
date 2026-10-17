---
{
  "id": "field-authors",
  "locale": "de",
  "title": "Feld `authors`",
  "summary": "Namen der Personen, die den Mod erstellt haben.",
  "navigation": {
    "path": [
      "mod.json"
    ]
  }
}
---

# Feld `authors`

Namen der Personen, die den Mod erstellt haben.

## Datentyp und Pflicht

`array` · optional

## Erlaubte Werte und Grenzen

- Einträge müssen eindeutig sein.

## Beispiel

```json
{
  "authors": [
    "Your Name"
  ]
}
```

## Quelle

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
