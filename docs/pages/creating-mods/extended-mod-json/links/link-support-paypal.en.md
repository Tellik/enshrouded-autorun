---
{
  "id": "link-support-paypal",
  "locale": "en",
  "title": "Link ID `support-paypal`",
  "summary": "Recognized Modloader button for support-paypal.",
  "navigation": {
    "path": [
      "Link IDs"
    ]
  }
}
---

# Link ID `support-paypal`

Use this ID in `extended.mod.json → links.support-paypal` and provide an HTTPS address. The Modloader displays the link with its registered icon and translated label.

```json
{
  "links": {
    "support-paypal": "https://example.com/project"
  }
}
```

The IDs and their order are defined in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json).
