import fs from "node:fs";
import path from "node:path";

export function readDocumentMetadata(file) {
  const text = fs.readFileSync(file, "utf8").replace(/^\uFEFF/, "");
  const match = text.match(/^---\s*\r?\n([\s\S]*?)\r?\n---\s*(?:\r?\n|$)/);
  if (!match) return null;
  try {
    return { metadata: JSON.parse(match[1]), body: text.slice(match[0].length) };
  } catch (error) {
    throw new Error(`${path.relative(process.cwd(), file)}: invalid JSON document header: ${error.message}`);
  }
}

export function collectWebsitePages(documentsRoot) {
  const pagesRoot = path.join(documentsRoot, "pages");
  const files = walk(pagesRoot).filter(file => file.endsWith(".md"));
  const locales = new Map();
  for (const file of files) {
    const relative = path.relative(documentsRoot, file).replaceAll("\\", "/");
    const parsed = readDocumentMetadata(file);
    if (!parsed?.metadata || (!parsed.metadata.id && !parsed.metadata.locale)) continue;
    const { metadata } = parsed;
    if (!metadata.id || !["de", "en"].includes(metadata.locale)) {
      throw new Error(`${relative}: website documents require an id and locale de or en`);
    }
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(metadata.id)) throw new Error(`${relative}: invalid page id ${metadata.id}`);
    if (typeof metadata.title !== "string" || typeof metadata.summary !== "string") {
      throw new Error(`${relative}: website documents require localized title and summary strings`);
    }
    if (!metadata.view && (!metadata.navigation || !Array.isArray(metadata.navigation.path) || metadata.navigation.path.some(item => typeof item !== "string" || !item.trim()))) {
      throw new Error(`${relative}: article documents require a navigation.path array`);
    }
    const page = locales.get(metadata.id) || { id: metadata.id, locales: {} };
    if (page.locales[metadata.locale]) throw new Error(`${relative}: duplicate ${metadata.locale} document for ${metadata.id}`);
    page.locales[metadata.locale] = { file, relative, metadata };
    locales.set(metadata.id, page);
  }

  const pages = [];
  for (const page of locales.values()) {
    const de = page.locales.de;
    const en = page.locales.en;
    if (!de || !en) throw new Error(`${page.id}: missing ${de ? "en" : "de"} translation`);
    const deMeta = de.metadata;
    const enMeta = en.metadata;
    const shared = ["view", "source", "apiSymbols", "apiPrefixes"];
    for (const key of shared) {
      if (JSON.stringify(deMeta[key] ?? null) !== JSON.stringify(enMeta[key] ?? null)) {
        throw new Error(`${page.id}: ${key} differs between de and en documents`);
      }
    }
    const deOrder = de.relative.replace(/\.de\.md$/, "");
    const enOrder = en.relative.replace(/\.en\.md$/, "");
    if (deOrder !== enOrder) throw new Error(`${page.id}: de and en pages must share the same folder and file name`);
    const relativeDirectory = path.posix.dirname(de.relative);
    pages.push({
      id: page.id,
      title: { de: deMeta.title, en: enMeta.title },
      summary: { de: deMeta.summary, en: enMeta.summary },
      navLabel: {
        de: deMeta.navigation?.label ?? deMeta.title,
        en: enMeta.navigation?.label ?? enMeta.title,
      },
      navGroup: {
        de: deMeta.navigation?.path?.slice(0, -1).join(" / ") || deMeta.navigation?.path?.[0] || "",
        en: enMeta.navigation?.path?.slice(0, -1).join(" / ") || enMeta.navigation?.path?.[0] || "",
      },
      sourceOrder: deOrder,
      apiSymbols: deMeta.apiSymbols,
      apiPrefixes: deMeta.apiPrefixes,
      view: deMeta.view,
      path: `docs/${relativeDirectory === "." ? "" : `${relativeDirectory}/`}`,
      files: { de: path.posix.basename(de.relative), en: path.posix.basename(en.relative) },
    });
  }
  return pages.sort((left, right) => left.sourceOrder.localeCompare(right.sourceOrder, "en"));
}

function walk(directory) {
  if (!fs.existsSync(directory)) return [];
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const file = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(file) : [file];
  });
}
