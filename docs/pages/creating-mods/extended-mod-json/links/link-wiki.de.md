---
{
  "id": "link-wiki",
  "locale": "de",
  "title": "Link-ID `wiki`",
  "summary": "Bekannte Modloader-Schaltfläche für wiki.",
  "navigation": {
    "path": [
      "extended.mod.json",
      "Link-IDs",
      "Unterstützte IDs"
    ]
  }
}
---

# Link-ID `wiki`

Diese ID wird in `extended.mod.json → links.wiki` verwendet. Trage darunter eine HTTPS-Adresse ein. Der Modloader zeigt den Link mit dem zugehörigen Symbol und übersetzten Namen.

```json
{
  "links": {
    "wiki": "https://example.com/project"
  }
}
```

Die IDs und ihre Reihenfolge sind in [order.json](https://github.com/bonsaibauer/shroudforge/blob/HEAD/modules/modloader/mod_link_order.json) definiert.
