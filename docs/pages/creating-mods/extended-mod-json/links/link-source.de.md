---
{
  "id": "link-source",
  "locale": "de",
  "title": "Link-ID `source`",
  "summary": "Bekannte Modloader-Schaltfläche für source.",
  "navigation": {
    "path": [
      "Link-IDs"
    ]
  }
}
---

# Link-ID `source`

Diese ID wird in `extended.mod.json → links.source` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "source": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
