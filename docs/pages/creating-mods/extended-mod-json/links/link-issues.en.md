---
{
  "id": "link-issues",
  "locale": "en",
  "title": "Link ID `issues`",
  "summary": "Recognized Modloader button for issues.",
  "navigation": {
    "path": [
      "Link IDs"
    ]
  }
}
---

# Link ID `issues`

Use this ID in `extended.mod.json → links.issues` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "issues": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
