---
{
  "id": "link-store",
  "locale": "en",
  "title": "Link ID `store`",
  "summary": "Recognized Modloader button for store.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link IDs",
      "Supported IDs"
    ]
  }
}
---

# Link ID `store`

Use this ID in `extended.mod.json → links.store` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "store": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
