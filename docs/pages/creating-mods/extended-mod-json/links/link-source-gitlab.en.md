---
{
  "id": "link-source-gitlab",
  "locale": "en",
  "title": "Link ID `source-gitlab`",
  "summary": "Recognized Modloader button for source-gitlab.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link IDs",
      "Supported IDs"
    ]
  }
}
---

# Link ID `source-gitlab`

Use this ID in `extended.mod.json → links.source-gitlab` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "source-gitlab": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
