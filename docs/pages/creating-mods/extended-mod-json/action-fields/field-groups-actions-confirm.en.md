---
{
  "id": "field-groups-actions-confirm",
  "locale": "en",
  "title": "The `groups[].actions[].confirm` field",
  "summary": "Optional confirmation prompt before running an action.",
  "navigation": {
    "path": [
      "Groups and actions"
    ]
  }
}
---

# The `groups[].actions[].confirm` field

Optional confirmation prompt before running an action.

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
      "actions": [
        {
          "id": "reset",
          "label": "Reset",
          "confirm": "example"
        }
      ]
    }
  ]
}
```

## Source

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). The package reader checks additional relationships between fields.
