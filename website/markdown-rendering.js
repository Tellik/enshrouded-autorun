export function createMarkdown({ locale, esc, pageIds, $$ }) {
  async function loadMarkdownPage(page) {
    const article = document.querySelector(`[data-doc-page="${page.route}"]`);
    if (!article || article.dataset.locale === locale) return;
    article.innerHTML = `<div class="doc-loading">${locale === "de" ? "Artikel wird geladen …" : "Loading article …"}</div>`;
    try {
      const response = await fetch(new URL(`${page.path}${locale}.md`, import.meta.url));
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const markdown = await response.text();
      article.innerHTML = renderMarkdown(markdown, page);
      article.dataset.locale = locale;
      addCopyControls();
    } catch (error) {
      article.innerHTML = `<div class="callout error"><b>${locale === "de" ? "Artikel konnte nicht geladen werden" : "Could not load article"}</b><span>${esc(error.message)}</span></div>`;
    }
  }

  function renderMarkdown(markdown, page) {
    const lines = markdown.replaceAll("\r", "").split("\n");
    const blocks = [];
    let index = 0;
    while (index < lines.length) {
      const line = lines[index];
      if (!line.trim()) { index++; continue; }
      const fence = line.match(/^\s*```([\w-]*)\s*$/);
      if (fence) {
        const codeLines = [];
        index++;
        while (index < lines.length && !/^\s*```\s*$/.test(lines[index])) codeLines.push(lines[index++]);
        index++;
        blocks.push(`<pre><code${fence[1] ? ` class="language-${esc(fence[1])}"` : ""}>${esc(codeLines.join("\n"))}</code></pre>`);
        continue;
      }
      const heading = line.match(/^(#{1,4})\s+(.+)$/);
      if (heading) {
        const level = heading[1].length;
        blocks.push(`<h${level}>${renderMarkdownInline(heading[2], page)}</h${level}>`);
        index++;
        continue;
      }
      if (/^\s*(?:---+|\*\*\*+)\s*$/.test(line)) { blocks.push("<hr>"); index++; continue; }
      if (line.startsWith("> ")) {
        const quote = [];
        while (index < lines.length && lines[index].startsWith("> ")) quote.push(lines[index++].slice(2));
        blocks.push(`<blockquote><p>${renderMarkdownInline(quote.join(" "), page)}</p></blockquote>`);
        continue;
      }
      if (line.includes("|") && index + 1 < lines.length && /^\s*\|?\s*:?-{3,}/.test(lines[index + 1])) {
        const rows = [line, lines[index + 1]];
        index += 2;
        while (index < lines.length && lines[index].includes("|")) rows.push(lines[index++]);
        const cells = row => row.trim().replace(/^\||\|$/g, "").split("|").map(cell => cell.trim());
        const header = cells(rows[0]);
        const tableRows = rows.slice(2).map(cells);
        blocks.push(`<div class="doc-table-wrap"><table class="doc-table"><thead><tr>${header.map(cell => `<th>${renderMarkdownInline(cell, page)}</th>`).join("")}</tr></thead><tbody>${tableRows.map(row => `<tr>${row.map(cell => `<td>${renderMarkdownInline(cell, page)}</td>`).join("")}</tr>`).join("")}</tbody></table></div>`);
        continue;
      }
      if (/^\s*[-*+]\s+/.test(line)) {
        const items = [];
        while (index < lines.length && /^\s*[-*+]\s+/.test(lines[index])) items.push(lines[index++].replace(/^\s*[-*+]\s+/, ""));
        blocks.push(`<ul>${items.map(item => `<li>${renderMarkdownInline(item, page)}</li>`).join("")}</ul>`);
        continue;
      }
      if (/^\s*\d+\.\s+/.test(line)) {
        const items = [];
        while (index < lines.length && /^\s*\d+\.\s+/.test(lines[index])) items.push(lines[index++].replace(/^\s*\d+\.\s+/, ""));
        blocks.push(`<ol>${items.map(item => `<li>${renderMarkdownInline(item, page)}</li>`).join("")}</ol>`);
        continue;
      }
      const paragraph = [line.trim()];
      index++;
      while (index < lines.length && lines[index].trim() && !/^(#{1,4})\s+/.test(lines[index]) && !/^\s*```/.test(lines[index]) && !/^\s*[-*+]\s+/.test(lines[index]) && !/^\s*\d+\.\s+/.test(lines[index]) && !lines[index].startsWith("> ")) paragraph.push(lines[index++].trim());
      blocks.push(`<p>${renderMarkdownInline(paragraph.join("\n"), page)}</p>`);
    }
    return blocks.join("\n");
  }

  function renderMarkdownInline(value, page) {
    const codeParts = [];
    let html = esc(value).replace(/`([^`]+)`/g, (_, content) => {
      const token = `\u0000CODE${codeParts.length}\u0000`;
      codeParts.push(`<code>${content}</code>`);
      return token;
    });
    html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_, alt, href) => `<img src="${esc(resolveMarkdownUrl(href, page))}" alt="${alt}">`);
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, label, href) => {
      if (href.startsWith("#")) {
        const target = href.slice(1);
        return `<a href="${esc(href)}"${pageIds.includes(target) ? ` data-view="${esc(target)}"` : ""}>${label}</a>`;
      }
      const external = /^https?:\/\//i.test(href);
      return `<a href="${esc(resolveMarkdownUrl(href, page))}"${external ? ' target="_blank" rel="noreferrer"' : ""}>${label}</a>`;
    });
    html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>").replace(/\*([^*]+)\*/g, "<em>$1</em>");
    return html.replace(/\u0000CODE(\d+)\u0000/g, (_, number) => codeParts[Number(number)]);
  }

  function resolveMarkdownUrl(href, page) {
    try { return new URL(href, new URL(page.path, import.meta.url)).href; }
    catch { return href; }
  }

  async function copyText(text, button) {
    try { await navigator.clipboard.writeText(text); }
    catch { const area = document.createElement('textarea'); area.value = text; document.body.append(area); area.select(); document.execCommand('copy'); area.remove(); }
    const original = button.textContent; button.textContent = locale === 'de' ? 'Kopiert ✓' : 'Copied ✓';
    setTimeout(() => button.textContent = original, 1400);
  }

  function addCopyControls() {
    $$("pre").forEach(block => {
      if (block.parentElement.classList.contains("code-block")) return;
      const wrapper = document.createElement("div"); wrapper.className = "code-block";
      const bar = document.createElement("div"); bar.className = "code-block-bar";
      const label = document.createElement("span"); label.textContent = block.dataset.label || (locale === "de" ? "CODEBEISPIEL" : "CODE EXAMPLE");
      const button = document.createElement("button"); button.type = "button"; button.className = "reset-button copy-code"; button.textContent = block.dataset.copyLabel || (locale === "de" ? "Kopieren" : "Copy");
      button.addEventListener("click", () => copyText(block.innerText, button));
      bar.append(label, button); block.before(wrapper); wrapper.append(bar, block);
    });
    $$(".folder-tree").forEach(tree => {
      if (tree.querySelector(".copy-code")) return;
      const button = document.createElement("button"); button.type = "button"; button.className = "reset-button copy-code tree-copy";
      button.textContent = locale === "de" ? "Ordnerstruktur kopieren" : "Copy folder structure";
      button.addEventListener("click", () => { const copy = tree.cloneNode(true); copy.querySelector(".tree-copy")?.remove(); copyText(copy.innerText.trim(), button); });
      tree.append(button);
    });
    $$(".command").forEach(command => {
      if (command.parentElement.classList.contains("command-copy")) return;
      const wrapper = document.createElement("span"); wrapper.className = "command-copy";
      const button = document.createElement("button"); button.type = "button"; button.className = "reset-button copy-code"; button.textContent = locale === "de" ? "Befehl kopieren" : "Copy command";
      button.addEventListener("click", () => copyText(command.innerText, button));
      command.before(wrapper); wrapper.append(command, button);
    });
  }

  return { loadMarkdownPage, copyText, addCopyControls };
}
