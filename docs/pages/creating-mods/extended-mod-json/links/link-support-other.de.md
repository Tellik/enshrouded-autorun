---
{
  "id": "link-support-other",
  "locale": "de",
  "title": "Link-ID `support-other`",
  "summary": "Bekannte Modloader-Schaltfläche für support-other.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link-IDs",
      "Unterstützte IDs"
    ]
  }
}
---

# Link-ID `support-other`

Diese ID wird in `extended.mod.json → links.support-other` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "support-other": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
