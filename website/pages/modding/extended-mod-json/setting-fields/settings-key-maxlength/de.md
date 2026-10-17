# Feld `settings.<key>.maxLength`

Höchstlänge eines Textwerts.

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
      "maxLength": 5
    }
  }
}
```

## Quelle

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
