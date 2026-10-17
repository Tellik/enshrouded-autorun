export function createNavigation({ locale, state, pageIds, docPageByRoute, $, $$, loadCatalog, loadMarkdownPage }) {
  function groupDocsForNavigation(pages) {
    const groups = new Map();
    for (const page of pages) {
      const label = page.navGroup?.[locale] || (locale === "de" ? "Modding-Referenz" : "Modding reference");
      const order = page.navOrder ?? 1000;
      const key = `${order}:${label}`;
      if (!groups.has(key)) groups.set(key, { label, order, items: [] });
      groups.get(key).items.push({ id: page.route, label: page.title[locale].replaceAll("`", ""), order: page.order ?? 1000 });
    }
    return [...groups.values()].sort((left, right) => left.order - right.order || left.label.localeCompare(right.label, locale))
      .map(group => ({ ...group, items: group.items.sort((left, right) => left.order - right.order || left.label.localeCompare(right.label, locale)) }));
  }

  function setPage(page, updateHash = true) {
    if (!pageIds.includes(page)) page = "home";
    state.page = page;
    $$(".view").forEach(view => view.classList.toggle("active", view.id === `view-${page}`));
    $$(".nav-item").forEach(item => item.classList.toggle("active", item.dataset.view === page));
    if (updateHash) history.replaceState(null, "", `#${page}`);
    $(".sidebar").classList.remove("open");
    window.scrollTo({ top: 0, behavior: "smooth" });
    if (page === "api") loadCatalog();
    const docPage = docPageByRoute.get(page);
    if (docPage) loadMarkdownPage(docPage);
  }

  return { groupDocsForNavigation, setPage };
}
