# Documentation

This is the single source tree for repository guides and published website pages. Website output is generated from these files and the source modules under `website/`.

## Contents

- [`pages/`](pages/) contains bilingual Markdown pages published on the website. The build discovers them recursively. `navigation.path` chooses the menu group, while the page's folder and filename determine its order.
- [`website/`](website/) contains the website shell, shared behavior, page-specific interactive integrations, source data, and build commands. It does not contain a second copy of article text.
- [`development/`](development/) explains how to find code ownership and perform repository development tasks.
- [`modding/`](modding/) explains mod package behavior and multiplayer constraints.
- [`runtime/`](runtime/) explains runtime APIs, game profiles, and system updates.
- [`examples/`](examples/) contains the copyable starter mod and small reference examples.
- [`evidence/`](evidence/) keeps dated investigations and archived source material. Use the current code and guides for present-day behavior.

## Add a website page

Create matching `<page-id>.de.md` and `<page-id>.en.md` files under the appropriate folder in `pages/`. The build adds the pair to the website menu without a separate route list. The page header supplies the stable ID, locale, localized title and summary, and menu group. Folder and filename order the entries. Add page-specific behavior only when needed under `website/page-integrations/<page-id>/`.

See [adding and ordering website pages](website/add-and-order-website-pages.en.md) for the exact format and build command. Generated files belong in `build/website/` and must not be edited by hand.

## Build and validate

Run this command from the repository root:

```powershell
npm run site:check
```
