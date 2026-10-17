---
{
  "id": "field-targets",
  "locale": "en",
  "title": "The `targets` field",
  "summary": "Selects the client process, the Dedicated Server, or both as mod execution targets.",
  "navigation": {
    "path": [
      "extended.mod.json"
    ]
  }
}
---

# The `targets` field

Selects the client process, the Dedicated Server, or both as mod execution targets.

## Type and requirement

`array` · optional

## Allowed values and limits

- list items: `"client"`, `"server"`
- minimum items: `1`
- Items must be unique.
- Processes this mod runs in. Legacy EML packages without targets default to client and server. ShroudForge packages default to client.

## Example

```json
{
  "targets": [
    "client"
  ]
}
```

## Source

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). The package reader checks additional relationships between fields.
