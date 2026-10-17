# The `settings.<key>.description` field

Short text shown on the mod page.

## Type and requirement

`string` · optional

## Allowed values and limits

- maximum length: `500`

## Example

```json
{
  "settings": {
    "example": {
      "value": true,
      "description": "example"
    }
  }
}
```

## Source

Schema: [extended.mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/extended.mod.schema.json). The package reader checks additional relationships between fields.
