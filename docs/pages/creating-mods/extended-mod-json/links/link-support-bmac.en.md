---
{
  "id": "link-support-bmac",
  "locale": "en",
  "title": "Link ID `support-bmac`",
  "summary": "Recognized Modloader button for support-bmac.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link IDs",
      "Supported IDs"
    ]
  }
}
---

# Link ID `support-bmac`

Use this ID in `extended.mod.json → links.support-bmac` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "support-bmac": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
