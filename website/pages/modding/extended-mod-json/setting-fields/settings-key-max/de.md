# Feld `settings.<key>.max`

Größter zulässiger Eingabewert.

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
      "max": 1
    }
  }
}
```

## Quelle

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
