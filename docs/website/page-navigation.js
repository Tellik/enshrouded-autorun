export function createNavigation({ locale, state, pageIds, docPageByRoute, integrationPageByRoute, loadPageIntegration, $, $$, loadCatalog, loadMarkdownPage }) {
  function groupDocsForNavigation(pages) {
    const groups = new Map();
    for (const page of pages) {
      const label = page.navGroup?.[locale] || (locale === "de" ? "Modding-Referenz" : "Modding reference");
      const order = page.sourceOrder ?? page.id;
      if (!groups.has(label)) groups.set(label, { label, order, items: [] });
      const group = groups.get(label);
      if (order.localeCompare(group.order, "en") < 0) group.order = order;
      group.items.push({ id: page.route, label: (page.navLabel?.[locale] || page.title[locale]).replaceAll("`", ""), order });
    }
    return [...groups.values()].sort((left, right) => left.order.localeCompare(right.order, "en"))
      .map(group => ({ ...group, items: group.items.sort((left, right) => left.order.localeCompare(right.order, "en")) }));
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
    const integrationPage = integrationPageByRoute.get(page);
    if (docPage) {
      loadMarkdownPage(docPage).then(() => {
        if (integrationPage) loadPageIntegration(integrationPage, page).catch(error => console.error(`Could not load page integration for ${integrationPage.id}`, error));
      });
    } else if (integrationPage) {
      loadPageIntegration(integrationPage, page).catch(error => console.error(`Could not load page integration for ${integrationPage.id}`, error));
    }
  }

  return { groupDocsForNavigation, setPage };
}
