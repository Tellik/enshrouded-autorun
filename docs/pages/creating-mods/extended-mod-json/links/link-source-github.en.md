---
{
  "id": "link-source-github",
  "locale": "en",
  "title": "Link ID `source-github`",
  "summary": "Recognized Modloader button for source-github.",
  "navigation": {
    "path": [
      "Link IDs"
    ]
  }
}
---

# Link ID `source-github`

Use this ID in `extended.mod.json → links.source-github` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "source-github": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
