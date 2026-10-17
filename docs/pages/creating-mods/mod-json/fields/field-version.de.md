---
{
  "id": "field-version",
  "locale": "de",
  "title": "Feld `version`",
  "summary": "Version dieser Mod-Veröffentlichung im SemVer-Grundformat.",
  "navigation": {
    "path": [
      "mod.json"
    ]
  }
}
---

# Feld `version`

Version dieser Mod-Veröffentlichung im SemVer-Grundformat.

## Datentyp und Pflicht

`string` · erforderlich

## Erlaubte Werte und Grenzen

- Muster: `^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$`

## Beispiel

```json
{
  "version": "1.0.0"
}
```

## Quelle

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). Der Paketleser prüft zusätzliche Beziehungen zwischen Feldern.
