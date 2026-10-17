---
{
  "id": "link-store",
  "locale": "de",
  "title": "Link-ID `store`",
  "summary": "Bekannte Modloader-Schaltfläche für store.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link-IDs",
      "Unterstützte IDs"
    ]
  }
}
---

# Link-ID `store`

Diese ID wird in `extended.mod.json → links.store` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "store": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
