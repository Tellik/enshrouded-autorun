---
{
  "id": "link-support-github",
  "locale": "en",
  "title": "Link ID `support-github`",
  "summary": "Recognized Modloader button for support-github.",
  "navigation": {
    "path": [
      "Link IDs"
    ]
  }
}
---

# Link ID `support-github`

Use this ID in `extended.mod.json → links.support-github` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "support-github": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
