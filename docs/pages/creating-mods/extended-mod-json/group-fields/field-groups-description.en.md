---
{
  "id": "field-groups-description",
  "locale": "en",
  "title": "The `groups[].description` field",
  "summary": "Short text shown on the mod page.",
  "navigation": {
    "path": [
      "Groups and actions"
    ]
  }
}
---

# The `groups[].description` field

Short text shown on the mod page.

## Type and requirement

`string` · optional

## Allowed values and limits

- maximum length: `500`

## Example

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

## Source

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). The package reader checks additional relationships between fields.
