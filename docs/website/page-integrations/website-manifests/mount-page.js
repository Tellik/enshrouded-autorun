import { createModEditor } from "./render-mod-manifest-editor.js";
import { createManifestValidation } from "./validate-mod-manifest.js";
import { makeModZip, downloadJson } from "./download-mod-package.js";
import { createExampleModManifests } from "./create-example-mod-manifests.js";

export async function mountPage({ root, locale, copy, esc, state, copyText }) {
  if (!root || root.dataset.integrationMounted === "true") return;
  const integrationSlot = root.querySelector('[data-page-integration="mod-manifest-editor"]');
  if (!integrationSlot) throw new Error("The mod manifest editor page is missing its integration slot");
  const template = locale === "de" ? "mod-manifest-editor.de.html" : "mod-manifest-editor.en.html";
  const response = await fetch(new URL(template, import.meta.url));
  if (!response.ok) throw new Error(`Could not load ${template}: HTTP ${response.status}`);
  integrationSlot.outerHTML = await response.text();
  const $ = selector => selector === "#view-manifests" ? root : root.querySelector(selector);
  const $$ = selector => [...root.querySelectorAll(selector)];
  const editorSession = { uploadedIcon: null, editingSettingKey: null };
  const { validateManifests } = createManifestValidation({ locale });
  const { rebuildManifestStudio, changeEditor, updatePreview, parseCurrent, setBuilderTab, readManifestIntoBuilder, showUploadedIcon, updateExtended, syncTargetPicker, renderBadgeBuilder } = createModEditor({ locale, copy, esc, state, $, $$, editorSession, validateManifests });
  const { exampleManifest, exampleExtended } = createExampleModManifests();
  state.manifest = JSON.stringify(exampleManifest, null, 2);
  state.extended = JSON.stringify(exampleExtended, null, 2);
  rebuildManifestStudio();
  root.addEventListener("click", event => {
      const editSetting = event.target.closest("[data-edit-setting]");
      if (editSetting) {
        const data = parseCurrent() || {}, key = editSetting.dataset.editSetting, raw = data.settings?.[key];
        const item = raw && typeof raw === "object" && !Array.isArray(raw) ? raw : { value: raw };
        editorSession.editingSettingKey = key;
        $("#setting-key").value = key;
        $("#setting-label").value = item.label || key;
        $("#setting-description").value = item.description || "";
        $("#setting-control").value = item.control || (item.options ? "select" : typeof item.value === "boolean" ? "toggle" : typeof item.value === "number" ? "number" : "text");
        $("#setting-value").value = Array.isArray(item.value) ? item.value.join(", ") : String(item.value ?? "");
        $("#setting-options").value = Object.entries(item.options || {}).map(([value, label]) => `${value} = ${label}`).join("\n");
        $("#setting-group").value = data.groups?.find(group => group.settings?.includes(key))?.label || "";
        for (const field of ["min", "max", "step", "minLength", "maxLength"]) $(`#setting-${field.replace("Length", "-length").toLowerCase()}`).value = item[field] ?? "";
        $("#add-setting").textContent = locale === "de" ? "Einstellung speichern" : "Save setting";
        $("#setting-key").scrollIntoView({ behavior: "smooth", block: "center" });
      }
      const removeSetting = event.target.closest("[data-remove-setting]");
      if (removeSetting) updateExtended(extended => {
        delete extended.settings?.[removeSetting.dataset.removeSetting];
        for (const group of extended.groups || []) group.settings = (group.settings || []).filter(key => key !== removeSetting.dataset.removeSetting);
      });
      const removeGroup = event.target.closest("[data-remove-group]");
      if (removeGroup) updateExtended(extended => { extended.groups?.splice(Number(removeGroup.dataset.removeGroup), 1); });
      const removeAction = event.target.closest("[data-remove-action]");
      if (removeAction) {
        const [groupIndex, actionIndex] = removeAction.dataset.removeAction.split(":").map(Number);
        updateExtended(extended => { extended.groups?.[groupIndex]?.actions?.splice(actionIndex, 1); });
      }
      const removeChangelog = event.target.closest("[data-remove-changelog]");
      if (removeChangelog) updateExtended(extended => { extended.changelog?.splice(Number(removeChangelog.dataset.removeChangelog), 1); });
      const removeDependency = event.target.closest("[data-remove-dependency]");
      if (removeDependency) {
        const index = Number(removeDependency.dataset.removeDependency);
        let mod; try { mod = JSON.parse(state.manifest); } catch { return; }
        mod.dependencies?.splice(index, 1);
        state.manifest = JSON.stringify(mod, null, 2); $("#manifest-editor").value = state.manifest;
        readManifestIntoBuilder(); updatePreview();
      }
      const editorTab = event.target.closest("[data-editor-tab]");
      if (editorTab) changeEditor(editorTab.dataset.editorTab);
  });

  $("#mod-icon-file")?.addEventListener("change", event => {
    const file = event.currentTarget.files?.[0];
    if (!file) return;
    const extension = file.name.split(".").pop()?.toLowerCase();
    if (!["png", "jpg", "jpeg", "webp", "svg"].includes(extension)) {
      alert(locale === "de" ? "Erlaubt sind PNG, JPG/JPEG, WebP und SVG." : "Supported formats are PNG, JPG/JPEG, WebP, and SVG.");
      event.currentTarget.value = "";
      return;
    }
    if (file.size > 2 * 1024 * 1024) {
      alert(locale === "de" ? "Das Icon darf höchstens 2 MiB groß sein." : "The icon must be 2 MiB or smaller.");
      event.currentTarget.value = "";
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      const safeName = `icon.${extension}`;
      const renamed = new File([file], safeName, { type: file.type, lastModified: file.lastModified });
      showUploadedIcon(renamed, String(reader.result));
      $("#mod-icon").value = safeName;
      $("#apply-mod-info").click();
    };
    reader.readAsDataURL(file);
  });
  $("#download-icon")?.addEventListener("click", () => {
    if (!editorSession.uploadedIcon) return;
    const href = URL.createObjectURL(editorSession.uploadedIcon.file);
    const link = document.createElement("a"); link.href = href; link.download = editorSession.uploadedIcon.file.name; link.click();
    setTimeout(() => URL.revokeObjectURL(href), 1000);
  });
  
  $("#download-package")?.addEventListener("click", async event => {
    const button = event.currentTarget;
    button.disabled = true;
    try {
      const manifest = JSON.parse(state.manifest), extended = JSON.parse(state.extended);
      const errors = validateManifests(manifest, extended);
      if (errors.length) { alert(errors[0]); return; }
      if (manifest.icon && (!editorSession.uploadedIcon || manifest.icon !== editorSession.uploadedIcon.file.name)) {
        alert(locale === "de" ? "Das eingetragene Icon ist nicht im Paket. Lade die Bilddatei hoch oder entferne den Icon-Pfad." : "The referenced icon is not included in the package. Upload that image or remove the icon path.");
        return;
      }
      const file = await makeModZip(state.manifest, state.extended, editorSession.uploadedIcon), href = URL.createObjectURL(file), link = document.createElement("a");
      const base = (manifest.id || "mod").replace(/[^A-Za-z0-9._-]/g, "_");
      link.href = href; link.download = `${base}.zip`; link.click(); setTimeout(() => URL.revokeObjectURL(href), 1500);
    } catch { alert(locale === "de" ? "Bitte korrigiere zuerst beide JSON-Dateien." : "Please fix both JSON files first."); }
    finally { button.disabled = false; }
  });
  $("#add-dependency")?.addEventListener("click", () => {
    const id = $("#dependency-id").value.trim();
    const version = $("#dependency-version").value.trim();
    if (!/^[A-Za-z0-9._-]{1,80}$/.test(id) || !version) { alert(locale === "de" ? "Bitte trage die gültige Mod-Kennung aus deren mod.json und einen SemVer-Versionsbereich ein." : "Enter the dependency ID from its mod.json and a SemVer version requirement."); return; }
    let mod; try { mod = JSON.parse(state.manifest); } catch { return; }
    if (id === mod.id) { alert(locale === "de" ? "Ein Mod kann nicht von sich selbst abhängen." : "A mod cannot depend on itself."); return; }
    mod.dependencies ||= [];
    if (mod.dependencies.some(item => item.id === id)) { alert(locale === "de" ? "Diese Mod ist bereits als Abhängigkeit eingetragen." : "This mod is already listed as a dependency."); return; }
    const dependency = { id, version };
    if ($("#dependency-optional").checked) dependency.optional = true;
    mod.dependencies.push(dependency);
    state.manifest = JSON.stringify(mod, null, 2);
    $("#manifest-editor").value = state.manifest;
    $("#dependency-id").value = ""; $("#dependency-version").value = ""; $("#dependency-optional").checked = false;
    readManifestIntoBuilder();
    updatePreview();
  });
  $("#apply-mod-info")?.addEventListener("click", () => {
    let mod;
    try { mod = JSON.parse(state.manifest); } catch { alert(locale === "de" ? "Bitte korrigiere zuerst mod.json." : "Please fix mod.json first."); return; }
    mod.id = $("#mod-id").value.trim();
    mod.name = $("#mod-name").value.trim();
    mod.version = $("#mod-version").value.trim();
    const authors = $("#mod-authors").value.split(";").map(value => value.trim()).filter(Boolean);
    if (authors.length) mod.authors = authors; else delete mod.authors;
    for (const [key, selector] of [["description", "#mod-description"], ["license", "#mod-license"], ["icon", "#mod-icon"]]) {
      const value = $(selector).value.trim();
      if (value) mod[key] = value; else delete mod[key];
    }
    mod.capabilities = $$(`[data-capability]:checked`).map(input => input.value);
    state.manifest = JSON.stringify(mod, null, 2);
    $("#manifest-editor").value = state.manifest;
    updatePreview();
  });
  
  $("#add-group")?.addEventListener("click", () => {
    const label = $("#group-label").value.trim();
    if (!label) return;
    updateExtended(extended => { extended.groups ||= []; const group = { label, settings: [] }; const description = $("#group-description").value.trim(); if (description) group.description = description; extended.groups.push(group); });
    $("#group-label").value = "";
    $("#group-description").value = "";
  });
  $("#add-action")?.addEventListener("click", () => {
    const id = $("#action-id").value.trim();
    const label = $("#action-label").value.trim();
    if (!id || !label) { alert(locale === "de" ? "Bitte trage eine Knopf-ID und einen Knopftext ein." : "Please enter a button ID and button text."); return; }
    updateExtended(extended => {
      extended.groups ||= [];
      const groupName = $("#action-group").value.trim() || (locale === "de" ? "Einstellungen" : "Settings");
      let group = extended.groups.find(item => item.label === groupName);
      if (!group) { group = { label: groupName, settings: [] }; extended.groups.push(group); }
      group.actions ||= [];
      const action = { id, label, style: $("#action-style").value };
      const confirm = $("#action-confirm").value.trim();
      if (confirm) action.confirm = confirm;
      group.actions.push(action);
    });
  });
  $("#add-changelog")?.addEventListener("click", () => {
    const item = $("#changelog-item").value.trim();
    if (!item) return;
    updateExtended(extended => { extended.changelog ||= []; extended.changelog.push(item); });
    $("#changelog-item").value = "";
  });
  
  $("#apply-extended-core")?.addEventListener("click", () => updateExtended(extended => { extended.enabled = $("#extended-enabled").checked; }));$("#add-setting")?.addEventListener("click", () => {
    const extended = parseCurrent();
    if (!extended) return;
    const key = $("#setting-key").value.trim();
    const control = $("#setting-control").value;
    if (!/^[A-Za-z0-9._-]{1,80}$/.test(key)) { alert(locale === "de" ? "Bitte nutze für den Einstellungsnamen 1 bis 80 Buchstaben, Zahlen, Punkte, Bindestriche oder Unterstriche." : "Use 1 to 80 letters, numbers, dots, hyphens, or underscores for the setting name."); return; }
    let options = {};
    for (const line of $("#setting-options").value.split(/\r?\n/).map(value => value.trim()).filter(Boolean)) {
      const [option, ...label] = line.split("=");
      options[option.trim()] = (label.join("=").trim() || option.trim());
    }
    const raw = $("#setting-value").value.trim();
    let value = raw;
    if (["toggle", "checkbox"].includes(control)) value = raw.toLowerCase() === "true";
    else if (["number", "slider"].includes(control)) value = Number(raw || 0);
    else if (control === "multiselect") value = raw ? raw.split(",").map(item => item.trim()).filter(Boolean) : [];
    else if (control === "color") value = raw || "#c9ad6e";
    else if (control === "keybind") value = raw || "F";
    else if (["select", "radio", "segmented"].includes(control) && Object.keys(options).length) value = Object.keys(options).includes(raw) ? raw : Object.keys(options)[0];
    const setting = { value, label: $("#setting-label").value.trim() || key, control };
    const description = $("#setting-description").value.trim();
    if (description) setting.description = description;
    for (const field of ["min", "max", "step", "minLength", "maxLength"]) {
      const element = $(`#setting-${field.replace("Length", "-length").toLowerCase()}`);
      if (element.value !== "") setting[field] = Number(element.value);
    }
    if (Object.keys(options).length) setting.options = options;
    extended.settings ||= {};
    if (editorSession.editingSettingKey && editorSession.editingSettingKey !== key) {
      delete extended.settings[editorSession.editingSettingKey];
      for (const group of extended.groups || []) group.settings = (group.settings || []).filter(value => value !== editorSession.editingSettingKey);
    }
    extended.settings[key] = setting;
    if (!Array.isArray(extended.groups)) extended.groups = [];
    if (!extended.groups.length) extended.groups.push({ label: locale === "de" ? "Einstellungen" : "Settings", settings: [] });
    const groupName = $("#setting-group").value.trim();
    const targetGroup = groupName ? (extended.groups.find(group => group.label === groupName) || (extended.groups.push({ label: groupName, settings: [] }), extended.groups.at(-1))) : extended.groups[0];
    targetGroup.settings ||= [];
    if (!targetGroup.settings.includes(key)) targetGroup.settings.push(key);
    state.extended = JSON.stringify(extended, null, 2);
    $("#extended-editor").value = state.extended;
    editorSession.editingSettingKey = null;
    $("#add-setting").textContent = locale === "de" ? "Einstellung hinzufügen +" : "Add setting +";
    $("#setting-key").value = "newSetting";
    $("#setting-label").value = locale === "de" ? "Neue Einstellung" : "New setting";
    $("#setting-value").value = "";
    $("#setting-description").value = "";
    $("#setting-options").value = "";
    updatePreview();
  });
  $("#badge-builder")?.addEventListener("change", event => {
    const target = event.target;
    if (target.matches("[data-badge]")) {
      const url = $(`[data-badge-url="${target.dataset.badge}"]`);
      url.disabled = !target.checked;
      if (!target.checked) { url.value = ""; updateExtended(extended => { if (extended.links) delete extended.links[target.dataset.badge]; if (extended.links && !Object.keys(extended.links).length) delete extended.links; }); }
      else url.focus();
    }
    if (target.matches("[data-badge-url]")) {
      const extended = parseCurrent();
      if (!extended) return;
      const { badgeUrl } = target.dataset;
      extended.links ||= {};
      if (target.value.trim()) extended.links[badgeUrl] = target.value.trim();
      else delete extended.links[badgeUrl];
      if (!Object.keys(extended.links).length) delete extended.links;
      state.extended = JSON.stringify(extended, null, 2);
      $("#extended-editor").value = state.extended;
      updatePreview();
    }
  });
  $("#badge-builder")?.addEventListener("input", event => {
    if (event.target.matches("[data-badge-url]")) event.target.dispatchEvent(new Event("change", { bubbles: true }));
  });
  
  $("#manifest-editor").value = state.manifest;
  $("#extended-editor").value = state.extended;
  setBuilderTab("mod");
  readManifestIntoBuilder();
  renderBadgeBuilder();
  $("#manifest-editor").addEventListener("input", () => { state.manifest = $("#manifest-editor").value; try { readManifestIntoBuilder(); const mod = JSON.parse(state.manifest); if (editorSession.uploadedIcon && mod.icon !== editorSession.uploadedIcon.file.name) { $("#mod-icon-file").value = ""; showUploadedIcon(null, ""); } } catch {} updatePreview(); });
  $("#mod-builder").addEventListener("input", event => { if (event.target.matches("#mod-icon") && editorSession.uploadedIcon && event.target.value !== editorSession.uploadedIcon.file.name) { $("#mod-icon-file").value = ""; showUploadedIcon(null, ""); } if (event.target.matches("#mod-id, #mod-name, #mod-version, #mod-authors, #mod-description, #mod-license, #mod-icon, [data-capability]")) $("#apply-mod-info").click(); });
  $("#mod-builder").addEventListener("change", event => { if (event.target.matches("[data-capability]")) $("#apply-mod-info").click(); });
  $("#extended-enabled").addEventListener("change", () => $("#apply-extended-core").click());
  $("#target-client")?.addEventListener("change", () => updateExtended(extended => {
    const targets = [$("#target-client").checked ? "client" : null, $("#target-server").checked ? "server" : null].filter(Boolean);
    if (targets.length) extended.targets = targets;
    else { extended.targets = ["client"]; syncTargetPicker(extended); }
  }));
  $("#target-server")?.addEventListener("change", () => updateExtended(extended => {
    const targets = [$("#target-client").checked ? "client" : null, $("#target-server").checked ? "server" : null].filter(Boolean);
    if (targets.length) extended.targets = targets;
    else { extended.targets = ["client"]; syncTargetPicker(extended); }
  }));
  $("#extended-editor").addEventListener("input", () => { updatePreview(); try { $("#extended-enabled").checked = Boolean(parseCurrent()?.enabled); renderBadgeBuilder(); } catch {} });
  
  $('#copy-mod-json')?.addEventListener('click', event => copyText(state.manifest, event.currentTarget));
  $('#copy-extended-json')?.addEventListener('click', event => copyText(state.extended, event.currentTarget));
  
  $('#download-mod-json')?.addEventListener('click', () => downloadJson(state.manifest, 'mod.json'));
  $('#download-extended-json')?.addEventListener('click', () => downloadJson(state.extended, 'extended.mod.json'));
  $('#reset-example').addEventListener('click', () => {
    state.manifest = JSON.stringify(exampleManifest, null, 2); state.extended = JSON.stringify(exampleExtended, null, 2);
    $('#manifest-editor').value = state.manifest; $('#extended-editor').value = state.extended;
    editorSession.editingSettingKey = null; $('#add-setting').textContent = locale === 'de' ? 'Einstellung hinzufügen +' : 'Add setting +';
    $('#mod-icon-file').value = ''; showUploadedIcon(null, '');
    readManifestIntoBuilder(); $('#extended-enabled').checked = Boolean(exampleExtended.enabled); renderBadgeBuilder(); updatePreview();
  });
  $("#mod-preview").addEventListener("input", event => {
    if (event.target.matches('input[type="range"]')) event.target.nextElementSibling.value = event.target.value;
  });
  updatePreview();
  root.dataset.integrationMounted = "true";
}
