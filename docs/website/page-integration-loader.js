export function createPageIntegrationLoader({ locale, copy, esc, state, copyText }) {
  const loaded = new Set();
  const loading = new Map();

  function loadPageIntegration(page, route) {
    if (!page.integration || loaded.has(page.id)) return Promise.resolve();
    if (loading.has(page.id)) return loading.get(page.id);

    const task = (async () => {
      const root = document.getElementById(`view-${route}`);
      if (!root) throw new Error(`Page container is missing: ${route}`);
      const integration = await import(new URL(page.integration, import.meta.url));
      if (typeof integration.mountPage !== "function") throw new Error(`${page.integration} must export mountPage()`);
      await integration.mountPage({ root, page, locale, copy, esc, state, copyText });
      loaded.add(page.id);
    })();
    loading.set(page.id, task);
    return task.finally(() => loading.delete(page.id));
  }

  return { loadPageIntegration };
}
