---
{
  "id": "link-support-other",
  "locale": "en",
  "title": "Link ID `support-other`",
  "summary": "Recognized Modloader button for support-other.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link IDs",
      "Supported IDs"
    ]
  }
}
---

# Link ID `support-other`

Use this ID in `extended.mod.json → links.support-other` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "support-other": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
