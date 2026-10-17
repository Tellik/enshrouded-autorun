---
{
  "id": "field-changelog",
  "locale": "en",
  "title": "The `changelog` field",
  "summary": "Short version notes on the mod page.",
  "navigation": {
    "path": [
      "extended.mod.json"
    ]
  }
}
---

# The `changelog` field

Short version notes on the mod page.

## Type and requirement

`array` · optional

## Allowed values and limits

- maximum items: `50`

## Example

```json
{
  "changelog": [
    "example"
  ]
}
```

## Source

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). The package reader checks additional relationships between fields.
