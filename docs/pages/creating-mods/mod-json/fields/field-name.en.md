---
{
  "id": "field-name",
  "locale": "en",
  "title": "The `name` field",
  "summary": "Name players see in the Modloader.",
  "navigation": {
    "path": [
      "mod.json"
    ]
  }
}
---

# The `name` field

Name players see in the Modloader.

## Type and requirement

`string` · required

## Allowed values and limits

- minimum length: `1`
- maximum length: `120`

## Example

```json
{
  "name": "Example Mod"
}
```

## Source

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). The package reader checks additional relationships between fields.
