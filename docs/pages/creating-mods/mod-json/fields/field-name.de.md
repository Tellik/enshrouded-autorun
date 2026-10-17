---
{
  "id": "field-name",
  "locale": "de",
  "title": "Feld `name`",
  "summary": "Name, den Spieler im Modloader sehen.",
  "navigation": {
    "path": [
      "mod.json",
      "Grundfelder"
    ]
  }
}
---

# Feld `name`

Name, den Spieler im Modloader sehen.

## Datentyp und Pflicht

`string` · erforderlich

## Erlaubte Werte und Grenzen

- Mindestlänge: `1`
- Höchstlänge: `120`

## Beispiel

```json
{
  "name": "Example Mod"
}
```

## Quelle

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
