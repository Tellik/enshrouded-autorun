export function createNavigation({ locale, state, pageIds, docPageByRoute, integrationPageByRoute, loadPageIntegration, $, $$, loadCatalog, loadMarkdownPage }) {
  function groupDocsForNavigation(pages) {
    const groups = new Map();
    const compareDescending = (left, right) => right.order.localeCompare(left.order, "en");
    const compareAscending = (left, right) => left.order.localeCompare(right.order, "en");
    const createGroup = (label, order) => ({ label, order, firstOrder: order, pages: [], children: new Map(), pageCount: 0 });

    for (const page of pages) {
      const fallback = page.navGroup?.[locale] || (locale === "de" ? "Modding-Referenz" : "Modding reference");
      const groupPath = page.navPath?.[locale]?.length ? page.navPath[locale] : [fallback];
      const order = page.sourceOrder ?? page.id;
      let group = groups.get(groupPath[0]);
      if (!group) {
        group = createGroup(groupPath[0], order);
        groups.set(groupPath[0], group);
      }
      group.pageCount += 1;
      if (order.localeCompare(group.order, "en") > 0) group.order = order;
      if (order.localeCompare(group.firstOrder, "en") < 0) group.firstOrder = order;

      for (const label of groupPath.slice(1)) {
        let child = group.children.get(label);
        if (!child) {
          child = createGroup(label, order);
          group.children.set(label, child);
        }
        child.pageCount += 1;
        if (order.localeCompare(child.order, "en") > 0) child.order = order;
        if (order.localeCompare(child.firstOrder, "en") < 0) child.firstOrder = order;
        group = child;
      }

      group.pages.push({
        type: "page",
        id: page.route,
        label: (page.navLabel?.[locale] || page.title[locale]).replaceAll("`", ""),
        order,
        overview: page.navOverview === true,
      });
    }

    function prepareGroup(group, root = false) {
      group.pages.sort((left, right) => Number(right.overview) - Number(left.overview) || compareAscending(left, right));
      const children = [...group.children.values()].sort((left, right) => left.firstOrder.localeCompare(right.firstOrder, "en")).map(child => prepareGroup(child));
      return {
        type: root ? "root" : "group",
        label: group.label,
        order: group.order,
        pageCount: group.pageCount,
        hasOverview: group.pages.some(page => page.overview),
        entries: [...group.pages.filter(page => page.overview), ...children, ...group.pages.filter(page => !page.overview)],
      };
    }

    return [...groups.values()].sort(compareDescending).map(group => prepareGroup(group, true));
  }

  function setPage(page, updateHash = true) {
    if (!pageIds.includes(page)) page = "home";
    state.page = page;
    $$(".view").forEach(view => view.classList.toggle("active", view.id === `view-${page}`));
    $$(".nav-item").forEach(item => item.classList.toggle("active", item.dataset.view === page));
    $$("details.nav-subgroup[open]").forEach(group => { group.open = false; });
    let openGroup = $$(".nav-item").find(item => item.dataset.view === page)?.closest("details.nav-subgroup");
    while (openGroup) {
      openGroup.open = true;
      openGroup = openGroup.parentElement?.closest("details.nav-subgroup");
    }
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
