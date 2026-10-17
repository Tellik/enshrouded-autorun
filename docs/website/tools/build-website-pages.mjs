import fs from "node:fs";
import path from "node:path";
import { collectWebsitePages, readDocumentMetadata } from "./read-doc-pages.mjs";

const root = path.resolve(import.meta.dirname, "../../..");
const documentsRoot = path.join(root, "docs");
const output = path.join(root, "build", "website", "pages", "index.js");
const allPages = collectWebsitePages(documentsRoot);
const integrationRoot = path.join(root, "docs", "website", "page-integrations");
const pageIds = new Set(allPages.map(page => page.id));
for (const entry of fs.readdirSync(integrationRoot, { withFileTypes: true }).filter(entry => entry.isDirectory())) {
  if (!pageIds.has(entry.name)) throw new Error(`Page integration has no matching Markdown page: ${entry.name}`);
  const integration = path.join(integrationRoot, entry.name, "mount-page.js");
  if (!fs.existsSync(integration)) throw new Error(`${path.relative(root, integration)} is required for this page integration`);
  const page = allPages.find(candidate => candidate.id === entry.name);
  page.integration = `page-integrations/${entry.name}/mount-page.js`;
}
const pages = allPages.filter(page => !page.view);
const viewPages = allPages.filter(page => page.view);
const views = { de: {}, en: {} };
for (const page of viewPages) {
  if (views.de[page.view] || views.en[page.view]) throw new Error(`Duplicate website view: ${page.view}`);
  for (const locale of ["de", "en"]) {
    const file = path.join(documentsRoot, page.path.slice("docs/".length), page.files[locale]);
    const parsed = readDocumentMetadata(file);
    views[locale][page.view] = parsed.body;
  }
}
const copiedDirectories = new Set();
for (const page of pages) {
  const relativeDirectory = page.path.slice("docs/".length).replace(/\/$/, "");
  if (copiedDirectories.has(relativeDirectory)) continue;
  copiedDirectories.add(relativeDirectory);
  const source = path.join(documentsRoot, relativeDirectory);
  const destination = path.join(root, "build", "website", "docs", relativeDirectory);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.cpSync(source, destination, {
    recursive: true,
    filter: file => !file.endsWith(".page-source.json"),
  });
}

fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, `export default ${JSON.stringify(allPages, null, 2)};\n`);
fs.writeFileSync(path.join(path.dirname(output), "views.js"), `export default ${JSON.stringify(views, null, 2)};\n`);
console.log(`Indexed ${pages.length} bilingual documentation routes and ${viewPages.length} localized website views from docs/pages/.`);
