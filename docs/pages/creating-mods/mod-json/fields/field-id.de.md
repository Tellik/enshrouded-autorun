---
{
  "id": "field-id",
  "locale": "de",
  "title": "Feld `id`",
  "summary": "Stabile Kennung, auf die andere Mods verweisen können.",
  "navigation": {
    "path": [
      "mod.json"
    ]
  }
}
---

# Feld `id`

Stabile Kennung, auf die andere Mods verweisen können.

## Datentyp und Pflicht

`string` · erforderlich

## Erlaubte Werte und Grenzen

- Höchstlänge: `80`
- Muster: `^[A-Za-z0-9._-]+$`

## Beispiel

```json
{
  "id": "author.example-mod"
}
```

## Quelle

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
