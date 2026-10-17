---
{
  "id": "field-settings-key-label",
  "locale": "en",
  "title": "The `settings.<key>.label` field",
  "summary": "Player-facing name displayed in the Modloader.",
  "navigation": {
    "path": [
      "Settings"
    ]
  }
}
---

# The `settings.<key>.label` field

Player-facing name displayed in the Modloader.

## Type and requirement

`string` · optional

## Allowed values and limits

- minimum length: `1`
- maximum length: `120`

## Example

```json
{
  "settings": {
    "example": {
      "value": true,
      "label": "example"
    }
  }
}
```

## Source

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). The package reader checks additional relationships between fields.
