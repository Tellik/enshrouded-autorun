---
{
  "id": "link-wiki",
  "locale": "en",
  "title": "Link ID `wiki`",
  "summary": "Recognized Modloader button for wiki.",
  "navigation": {
    "path": [
      "Link IDs"
    ]
  }
}
---

# Link ID `wiki`

Use this ID in `extended.mod.json → links.wiki` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "wiki": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
