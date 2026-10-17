---
{
  "id": "link-support-paypal",
  "locale": "de",
  "title": "Link-ID `support-paypal`",
  "summary": "Bekannte Modloader-Schaltfläche für support-paypal.",
  "navigation": {
    "path": [
      "Link-IDs"
    ]
  }
}
---

# Link-ID `support-paypal`

Diese ID wird in `extended.mod.json → links.support-paypal` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "support-paypal": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
