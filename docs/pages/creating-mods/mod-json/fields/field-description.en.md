---
{
  "id": "field-description",
  "locale": "en",
  "title": "The `description` field",
  "summary": "Short text shown on the mod page.",
  "navigation": {
    "path": [
      "mod.json",
      "Basic fields"
    ]
  }
}
---

# The `description` field

Short text shown on the mod page.

## Type and requirement

`string` · optional

## Allowed values and limits

- maximum length: `2000`

## Example

```json
{
  "description": "example"
}
```

## Source

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). The package reader checks additional relationships between fields.
