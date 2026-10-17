---
{
  "id": "link-source-github",
  "locale": "de",
  "title": "Link-ID `source-github`",
  "summary": "Bekannte Modloader-Schaltfläche für source-github.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link-IDs",
      "Unterstützte IDs"
    ]
  }
}
---

# Link-ID `source-github`

Diese ID wird in `extended.mod.json → links.source-github` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "source-github": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
