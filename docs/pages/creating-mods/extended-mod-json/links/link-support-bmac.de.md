---
{
  "id": "link-support-bmac",
  "locale": "de",
  "title": "Link-ID `support-bmac`",
  "summary": "Bekannte Modloader-Schaltfläche für support-bmac.",
  "navigation": {
    "path": [
      "Link-IDs"
    ]
  }
}
---

# Link-ID `support-bmac`

Diese ID wird in `extended.mod.json → links.support-bmac` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "support-bmac": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
