---
{
  "id": "link-website",
  "locale": "en",
  "title": "Link ID `website`",
  "summary": "Recognized Modloader button for website.",
  "navigation": {
    "path": [
      "Link IDs"
    ]
  }
}
---

# Link ID `website`

Use this ID in `extended.mod.json → links.website` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "website": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
