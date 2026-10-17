---
{
  "id": "field-capabilities",
  "locale": "en",
  "title": "The `capabilities` field",
  "summary": "Access categories requested by the mod.",
  "navigation": {
    "path": [
      "mod.json",
      "Capabilities"
    ],
    "overview": true,
    "label": "Overview"
  }
}
---

# The `capabilities` field

Access categories requested by the mod.

## Type and requirement

`array` · optional

## Allowed values and limits

- list items: `"patch"`, `"export"`, `"runtime"`, `"runtime-register-dll"`
- Items must be unique.

## Example

```json
{
  "capabilities": [
    "runtime"
  ]
}
```

## Source

Schema: [mod.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/manifest/manifest.schema.json). The package reader checks additional relationships between fields.
