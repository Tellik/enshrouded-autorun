# Add and order website pages

Website article text belongs in `docs/pages/`. The website build reads its headers, creates the route index, and copies the source pages into `build/website/`. There is no second article tree, route table, or `page.json` file.

## Add a bilingual page

Create two files with the same page ID and filename stem, for example:

```text
docs/pages/creating-mods/my-feature.en.md
docs/pages/creating-mods/my-feature.de.md
```

Start each file with a JSON header:

```markdown
---
{
  "id": "my-feature",
  "locale": "en",
  "title": "My feature",
  "summary": "A short description shown with this page.",
  "navigation": {
    "path": ["MODDING"],
    "label": "My feature"
  }
}
---

Write the page content here in Markdown.
```

Use `"locale": "de"` and translated `title`, `summary`, `navigation.path`, and `navigation.label` in the German file. The `id`, filename stem, and folder must match between both files. Keep English as the default language for shared documentation. Use German where the audience is German-speaking and mark the filename with `.de.md`.

## Menu placement and order

`navigation.path` places a page in a localized menu group. `navigation.label` changes the displayed page label. The page's relative folder and filename determine its order in the generated menu. Use descriptive English folder and file names. Add a page in the desired location and the next build includes it automatically. Do not add numeric ordering fields or a manual route list.

The build checks that each page has both translations, matching IDs and filenames, valid headers, and localized titles and summaries. Optional `apiSymbols` and `apiPrefixes` link API entries to relevant pages. Use `view` for a complete website view whose HTML is stored in the paired Markdown files.

## Add page-specific behavior

Most pages need only Markdown. If a page has custom controls, put their localized HTML templates and behavior under `docs/website/page-integrations/<page-id>/`. The convention entry point is `mount-page.js`. The build discovers the matching module from the page ID, so there is no registration list to update. Keep queries and event handlers scoped to the supplied page root. Put shared behavior in an existing website module only when more than one page uses it.

For example, the manifest editor's files live under `docs/website/page-integrations/website-manifests/`. Its Markdown page stays under `docs/pages/creating-mods/`.

## Build and validate

Run from the repository root:

```powershell
npm run site:check
```

The command builds the website, indexes pages and page integrations, generates the runtime reference, and validates the publication output. Inspect the result in `build/website/`. Never edit that generated directory directly.
