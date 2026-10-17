# Website source

Website pages are authored in [`../pages/`](../pages/). This directory contains the code and data that render and publish them. Article text is not copied here.

## Source files

- `start-website.js` starts the browser application and connects shared modules.
- `page-navigation.js` builds the menu from the generated page index and opens routes.
- `markdown-rendering.js` loads and renders Markdown articles.
- `reference-search.js` searches the generated Lua API, type, and resource catalogs.
- `page-integrations/<page-id>/` contains markup and code used only by one interactive page. Its `mount-page.js` entry is discovered by the website build. Keep localized UI templates, markup hooks, and behavior scoped to their own page root.
- `content/de.js` and `content/en.js` contain shared interface labels, not article text.
- `data/` contains the captured game snapshot used to build the runtime reference.
- `tools/` contains named build, indexing, sanitizing, and validation actions.
- `de/`, `en/`, `styles.css`, and `media/` contain the language entry pages, shared design, and site images.

The build generates `build/website/`. Do not edit generated files by hand.

## Add a page

Add the German and English Markdown pair in the right folder under `docs/pages/`. The build discovers the pair, places it in the menu group from its header, and sorts it by folder and filename. No route list or `page.json` is maintained.

When a page needs custom interaction, add a matching `page-integrations/<page-id>/` module. Keep page-only markup in a clearly named HTML template and behavior in `mount-page.js`. Pages without custom behavior need no integration folder. Shared behavior belongs in an existing top-level module only when multiple pages use it.

See [adding and ordering website pages](add-and-order-website-pages.en.md) for the page header, menu rules, and validation command.

## Build and validate

Run from the repository root:

```powershell
npm run site:check
```
