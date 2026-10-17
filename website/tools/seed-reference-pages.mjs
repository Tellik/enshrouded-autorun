import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "../..");
const pagesRoot = path.join(root, "website", "pages", "modding");
const packageRoot = path.join(root, "modules", "modloader", "manifest");
const manifestSchema = readJson(path.join(packageRoot, "manifest.schema.json"));
const extensionSchema = readJson(path.join(packageRoot, "extended.mod.schema.json"));
const loaderSchema = readJson(path.join(root, "modules", "loader-config", "loader.schema.json"));
const loaderDefaults = readJson(path.join(root, "modules", "loader-config", "loader.default.json"));
const overwriteSeededPages = process.argv.includes("--overwrite");


const referenceCopy = Object.fromEntries(["de", "en"].map(locale => [locale,
  JSON.parse(fs.readFileSync(new URL(`../content/reference/${locale}.json`, import.meta.url), "utf8")),
]));

function referenceText(locale, key, ...values) {
  return referenceCopy[locale][key].map((part, index) =>
    index ? `${values[index - 1]}${part}` : part).join("");
}

const pages = [];
const slug = value => String(value).normalize("NFKD").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase();
const titleFromKey = key => key.replace(/([a-z0-9])([A-Z])/g, "$1 $2").replaceAll("_", " ").replace(/^./, value => value.toUpperCase());
const code = value => `\x60${String(value)}\x60`;

function addPage({ id, folder, navGroup, navOrder = 100, order = 100, titleDe, titleEn, summaryDe, summaryEn, bodyDe, bodyEn, apiSymbols, apiPrefixes }) {
  const page = {
    id,
    title: { de: titleDe, en: titleEn },
    summary: { de: summaryDe, en: summaryEn },
    navGroup: { de: navGroup.de, en: navGroup.en },
    navOrder,
    order,
  };
  if (apiSymbols?.length) page.apiSymbols = apiSymbols;
  if (apiPrefixes?.length) page.apiPrefixes = apiPrefixes;
  const directory = path.join(pagesRoot, folder);
  fs.mkdirSync(directory, { recursive: true });
  writeSeedFile(path.join(directory, "page.json"), `${JSON.stringify(page, null, 2)}\n`);
  writeSeedFile(path.join(directory, "de.md"), bodyDe.trim() + "\n");
  writeSeedFile(path.join(directory, "en.md"), bodyEn.trim() + "\n");
  pages.push(page);
}

function writeSeedFile(file, content) {
  if (overwriteSeededPages || !fs.existsSync(file)) fs.writeFileSync(file, content);
}

function articleIntro(title, summary) {
  return `# ${title}\n\n${summary}\n`;
}

const core = [
  {
    id: "mod-json-guide", folder: "mod-json", navGroup: { de: referenceText("de", "mod-json-guide.navGroup.de.1"), en: referenceText("en", "mod-json-guide.navGroup.en.1") }, navOrder: 10, order: 0,
    titleDe: referenceText("de", "mod-json-guide.titleDe.2"), titleEn: referenceText("en", "mod-json-guide.titleEn.2"),
    summaryDe: referenceText("de", "mod-json-guide.summaryDe.3"),
    summaryEn: referenceText("en", "mod-json-guide.summaryEn.3"),
    bodyDe: referenceText("de", "mod-json-guide.bodyDe.4"),
    bodyEn: referenceText("en", "mod-json-guide.bodyEn.4"),
  },
  {
    id: "extended-mod-json-guide", folder: "extended-mod-json", navGroup: { de: referenceText("de", "extended-mod-json-guide.navGroup.de.5"), en: referenceText("en", "extended-mod-json-guide.navGroup.en.5") }, navOrder: 20, order: 0,
    titleDe: referenceText("de", "extended-mod-json-guide.titleDe.6"), titleEn: referenceText("en", "extended-mod-json-guide.titleEn.6"),
    summaryDe: referenceText("de", "extended-mod-json-guide.summaryDe.7"),
    summaryEn: referenceText("en", "extended-mod-json-guide.summaryEn.7"),
    bodyDe: referenceText("de", "extended-mod-json-guide.bodyDe.8"),
    bodyEn: referenceText("en", "extended-mod-json-guide.bodyEn.8"),
  },
  {
    id: "setting-controls", folder: "extended-mod-json/settings-controls", navGroup: { de: referenceText("de", "setting-controls.navGroup.de.9"), en: referenceText("en", "setting-controls.navGroup.en.9") }, navOrder: 30, order: 0,
    titleDe: referenceText("de", "setting-controls.titleDe.10"), titleEn: referenceText("en", "setting-controls.titleEn.10"),
    summaryDe: referenceText("de", "setting-controls.summaryDe.11"),
    summaryEn: referenceText("en", "setting-controls.summaryEn.11"),
    bodyDe: referenceText("de", "setting-controls.bodyDe.12"),
    bodyEn: referenceText("en", "setting-controls.bodyEn.12"),
  },
  {
    id: "setting-groups-actions", folder: "extended-mod-json/groups-actions", navGroup: { de: referenceText("de", "setting-groups-actions.navGroup.de.13"), en: referenceText("en", "setting-groups-actions.navGroup.en.13") }, navOrder: 30, order: 1,
    titleDe: referenceText("de", "setting-groups-actions.titleDe.14"), titleEn: referenceText("en", "setting-groups-actions.titleEn.14"),
    summaryDe: referenceText("de", "setting-groups-actions.summaryDe.15"),
    summaryEn: referenceText("en", "setting-groups-actions.summaryEn.15"),
    bodyDe: referenceText("de", "setting-groups-actions.bodyDe.16"),
    bodyEn: referenceText("en", "setting-groups-actions.bodyEn.16"),
  },
  {
    id: "eml-migration", folder: "eml-migration", navGroup: { de: referenceText("de", "eml-migration.navGroup.de.17"), en: referenceText("en", "eml-migration.navGroup.en.17") }, navOrder: 40, order: 0,
    titleDe: referenceText("de", "eml-migration.titleDe.18"), titleEn: referenceText("en", "eml-migration.titleEn.18"),
    summaryDe: referenceText("de", "eml-migration.summaryDe.19"),
    summaryEn: referenceText("en", "eml-migration.summaryEn.19"),
    bodyDe: referenceText("de", "eml-migration.bodyDe.20"),
    bodyEn: referenceText("en", "eml-migration.bodyEn.20"),
  },
  {
    id: "api-guides", folder: "lua-api", navGroup: { de: referenceText("de", "api-guides.navGroup.de.21"), en: referenceText("en", "api-guides.navGroup.en.21") }, navOrder: 50, order: 0,
    titleDe: referenceText("de", "api-guides.titleDe.22"), titleEn: referenceText("en", "api-guides.titleEn.22"),
    summaryDe: referenceText("de", "api-guides.summaryDe.23"),
    summaryEn: referenceText("en", "api-guides.summaryEn.23"),
    bodyDe: referenceText("de", "api-guides.bodyDe.24"),
    bodyEn: referenceText("en", "api-guides.bodyEn.24"),
  },
  {
    id: "api-notifications", folder: "lua-api/modloader-notifications", navGroup: { de: referenceText("de", "api-notifications.navGroup.de.25"), en: referenceText("en", "api-notifications.navGroup.en.25") }, navOrder: 50, order: 1,
    titleDe: referenceText("de", "api-notifications.titleDe.26"), titleEn: referenceText("en", "api-notifications.titleEn.26"),
    summaryDe: referenceText("de", "api-notifications.summaryDe.27"),
    summaryEn: referenceText("en", "api-notifications.summaryEn.27"),
    bodyDe: referenceText("de", "api-notifications.bodyDe.28"),
    bodyEn: referenceText("en", "api-notifications.bodyEn.28"),
  },
  {
    id: "api-settings", folder: "lua-api/settings", navGroup: { de: referenceText("de", "api-settings.navGroup.de.29"), en: referenceText("en", "api-settings.navGroup.en.29") }, navOrder: 50, order: 2,
    titleDe: referenceText("de", "api-settings.titleDe.30"), titleEn: referenceText("en", "api-settings.titleEn.30"),
    summaryDe: referenceText("de", "api-settings.summaryDe.31"),
    summaryEn: referenceText("en", "api-settings.summaryEn.31"),
    bodyDe: referenceText("de", "api-settings.bodyDe.32"),
    bodyEn: referenceText("en", "api-settings.bodyEn.32"),
  },
  {
    id: "api-ui-actions", folder: "lua-api/ui-actions", navGroup: { de: referenceText("de", "api-ui-actions.navGroup.de.33"), en: referenceText("en", "api-ui-actions.navGroup.en.33") }, navOrder: 50, order: 3,
    titleDe: referenceText("de", "api-ui-actions.titleDe.34"), titleEn: referenceText("en", "api-ui-actions.titleEn.34"),
    summaryDe: referenceText("de", "api-ui-actions.summaryDe.35"),
    summaryEn: referenceText("en", "api-ui-actions.summaryEn.35"),
    bodyDe: referenceText("de", "api-ui-actions.bodyDe.36"),
    bodyEn: referenceText("en", "api-ui-actions.bodyEn.36"),
  },
  {
    id: "api-logging", folder: "lua-api/logging", navGroup: { de: referenceText("de", "api-logging.navGroup.de.37"), en: referenceText("en", "api-logging.navGroup.en.37") }, navOrder: 50, order: 4,
    titleDe: referenceText("de", "api-logging.titleDe.38"), titleEn: referenceText("en", "api-logging.titleEn.38"),
    summaryDe: referenceText("de", "api-logging.summaryDe.39"),
    summaryEn: referenceText("en", "api-logging.summaryEn.39"),
    bodyDe: referenceText("de", "api-logging.bodyDe.40"),
    bodyEn: referenceText("en", "api-logging.bodyEn.40"),
  },
  {
    id: "api-files", folder: "lua-api/files-and-exports", navGroup: { de: referenceText("de", "api-files.navGroup.de.41"), en: referenceText("en", "api-files.navGroup.en.41") }, navOrder: 50, order: 5,
    apiPrefixes: ["io.", "buffer.", "Buffer:", "game.assets."],
    titleDe: referenceText("de", "api-files.titleDe.42"), titleEn: referenceText("en", "api-files.titleEn.42"),
    summaryDe: referenceText("de", "api-files.summaryDe.43"),
    summaryEn: referenceText("en", "api-files.summaryEn.43"),
    bodyDe: referenceText("de", "api-files.bodyDe.44"),
    bodyEn: referenceText("en", "api-files.bodyEn.44"),
  },
  {
    id: "api-runtime", folder: "lua-api/runtime-and-game", navGroup: { de: referenceText("de", "api-runtime.navGroup.de.45"), en: referenceText("en", "api-runtime.navGroup.en.45") }, navOrder: 50, order: 6,
    titleDe: referenceText("de", "api-runtime.titleDe.46"), titleEn: referenceText("en", "api-runtime.titleEn.46"),
    summaryDe: referenceText("de", "api-runtime.summaryDe.47"),
    summaryEn: referenceText("en", "api-runtime.summaryEn.47"),
    bodyDe: referenceText("de", "api-runtime.bodyDe.48"),
    bodyEn: referenceText("en", "api-runtime.bodyEn.48"),
  },
];

for (const page of core) addPage(page);

addPage({
  id: "api-network", folder: "lua-api/network-p2p", navGroup: { de: referenceText("de", "api-network.navGroup.de.49"), en: referenceText("en", "api-network.navGroup.en.49") }, navOrder: 50, order: 7,
  titleDe: referenceText("de", "api-network.titleDe.50"), titleEn: referenceText("en", "api-network.titleEn.50"),
  summaryDe: referenceText("de", "api-network.summaryDe.51"),
  summaryEn: referenceText("en", "api-network.summaryEn.51"),
  bodyDe: referenceText("de", "api-network.bodyDe.52"),
  bodyEn: referenceText("en", "api-network.bodyEn.52"),
});

addPage({
  id: "api-game-data", folder: "lua-api/game-data", navGroup: { de: referenceText("de", "api-game-data.navGroup.de.53"), en: referenceText("en", "api-game-data.navGroup.en.53") }, navOrder: 50, order: 8,
  titleDe: referenceText("de", "api-game-data.titleDe.54"), titleEn: referenceText("en", "api-game-data.titleEn.54"),
  summaryDe: referenceText("de", "api-game-data.summaryDe.55"),
  summaryEn: referenceText("en", "api-game-data.summaryEn.55"),
  bodyDe: referenceText("de", "api-game-data.bodyDe.56"),
  bodyEn: referenceText("en", "api-game-data.bodyEn.56"),
});

addPage({
  id: "api-lifecycle", folder: "lua-api/lifecycle", navGroup: { de: referenceText("de", "api-lifecycle.navGroup.de.57"), en: referenceText("en", "api-lifecycle.navGroup.en.57") }, navOrder: 50, order: 9,
  titleDe: referenceText("de", "api-lifecycle.titleDe.58"), titleEn: referenceText("en", "api-lifecycle.titleEn.58"),
  summaryDe: referenceText("de", "api-lifecycle.summaryDe.59"),
  summaryEn: referenceText("en", "api-lifecycle.summaryEn.59"),
  bodyDe: referenceText("de", "api-lifecycle.bodyDe.60"),
  bodyEn: referenceText("en", "api-lifecycle.bodyEn.60"),
});

const fieldPurpose = {
  id: ["Stabile Kennung, auf die andere Mods verweisen können.", "Stable identifier that other mods can depend on."],
  name: ["Name, den Spieler im Modloader sehen.", "Name players see in the Modloader."],
  version: ["Version dieser Mod-Veröffentlichung im SemVer-Grundformat.", "Version of this mod release using the basic SemVer format."],
  description: ["Kurzer Text für die Modseite.", "Short text shown on the mod page."],
  authors: ["Namen der Personen, die den Mod erstellt haben.", "Names of the people who created the mod."],
  license: ["Lizenzkennung, falls der Mod eine Lizenz erklärt.", "License identifier when the mod declares a license."],
  icon: ["Icon-Datei im Paketstamm oder unter assets/.", "Icon file in the package root or under assets/."],
  dependencies: ["Andere Mods, deren Verfügbarkeit oder Version der Loader vor dem Start prüfen muss.", "Other mods whose availability or version the loader checks before startup."],
  dependencyId: ["Exakte Mod-ID aus der mod.json des benötigten Mods.", "Exact mod ID from the required mod's mod.json."],
  dependencyVersion: ["SemVer-Bereich, der für die Version des benötigten Mods akzeptiert wird.", "SemVer range accepted for the required mod's version."],
  actionId: ["Eindeutiger Aktionsname, den der Mod in Lua registriert.", "Unique action name that the mod registers in Lua."],
  capabilities: ["Zugriffsarten, die der Mod anfordert.", "Access categories requested by the mod."],
  enabled: ["Gespeicherter Aktivierungszustand für diesen Mod.", "Saved enabled state for this mod."],
  schemaVersion: ["Version der extended.mod.json-Struktur.", "Version of the extended.mod.json structure."],
  targets: ["Wählt den Client-Prozess, den Dedicated Server oder beide als Ausführungsort des Mods.", "Selects the client process, the Dedicated Server, or both as mod execution targets."],
  launcher: ["Herkunft des Mod-Pakets für die Modloader-Anzeige.", "Package origin used in the Modloader display."],
  links: ["Bekannte Link-IDs und HTTPS-Adressen für Modloader-Schaltflächen.", "Recognized link IDs and HTTPS addresses for Modloader buttons."],
  changelog: ["Kurze Versionshinweise auf der Modseite.", "Short version notes on the mod page."],
  settings: ["Mod-eigene Einstellungswerte und ihre Anzeigenmetadaten.", "Mod-specific setting values and their display metadata."],
  groups: ["Gruppen für Einstellungen und Modloader-Aktionen.", "Groups for settings and Modloader actions."],
  value: ["Startwert und erwarteter Lua-Datentyp der Einstellung.", "Starting value and expected Lua data type of the setting."],
  label: ["Spielerfreundlicher Name, der im Modloader angezeigt wird.", "Player-facing name displayed in the Modloader."],
  control: ["Art des Eingabefelds, das der Modloader zeigt.", "Input control displayed by the Modloader."],
  min: ["Kleinster zulässiger Eingabewert.", "Smallest accepted input value."],
  max: ["Größter zulässiger Eingabewert.", "Largest accepted input value."],
  step: ["Schrittweite für Zahlen- und Reglerfelder.", "Increment for number and slider inputs."],
  minLength: ["Mindestlänge eines Textwerts.", "Minimum length of a text value."],
  maxLength: ["Höchstlänge eines Textwerts.", "Maximum length of a text value."],
  options: ["Erlaubte Auswahlwerte mit den Namen, die Spieler sehen.", "Allowed choices and the labels players see."],
  actions: ["Aktionsknöpfe, deren IDs Lua-Callbacks aufrufen.", "Action buttons whose IDs call Lua callbacks."],
  groupSettings: ["Schlüssel vorhandener Mod-Einstellungen, die in dieser Gruppe angezeigt werden.", "Keys of existing mod settings displayed in this group."],
  style: ["Darstellung des Aktionsknopfs, ohne die Funktion zu ändern.", "Appearance of the action button, without changing its behavior."],
  confirm: ["Optionale Rückfrage vor dem Ausführen einer Aktion.", "Optional confirmation prompt before running an action."],
  optional: ["Ob eine fehlende oder inkompatible Abhängigkeit den Start blockiert.", "Whether a missing or incompatible dependency blocks startup."],
};

function constraints(schema, locale) {
  if (!schema || typeof schema !== "object") return "";
  const values = [];
  if (schema.const !== undefined) values.push(`${locale === "de" ? "fester Wert" : "fixed value"}: ${code(JSON.stringify(schema.const))}`);
  if (schema.enum) values.push(`${locale === "de" ? "Werte" : "values"}: ${schema.enum.map(value => code(JSON.stringify(value))).join(", ")}`);
  if (schema.items?.enum) values.push(`${locale === "de" ? "Listenelemente" : "list items"}: ${schema.items.enum.map(value => code(JSON.stringify(value))).join(", ")}`);
  for (const [key, label] of [["minLength", locale === "de" ? "Mindestlänge" : "minimum length"], ["maxLength", locale === "de" ? "Höchstlänge" : "maximum length"], ["minItems", locale === "de" ? "Mindesteinträge" : "minimum items"], ["maxItems", locale === "de" ? "Höchsteinträge" : "maximum items"], ["minimum", locale === "de" ? "Minimum" : "minimum"], ["maximum", locale === "de" ? "Maximum" : "maximum"], ["exclusiveMinimum", locale === "de" ? "größer als" : "greater than"]]) {
    if (schema[key] !== undefined) values.push(`${label}: ${code(schema[key])}`);
  }
  if (schema.pattern) values.push(`${locale === "de" ? "Muster" : "pattern"}: ${code(schema.pattern)}`);
  if (schema.uniqueItems) values.push(locale === "de" ? "Einträge müssen eindeutig sein." : "Items must be unique.");
  if (schema.description) {
    const translations = {
      "Optional license identifier (for example, MIT). Use null when no license is declared.": [
        "Optionale Lizenzkennung, zum Beispiel MIT. Verwende null, wenn keine Lizenz angegeben wird.",
        schema.description,
      ],
      "Optional mod icon filename in the package root or path under assets/. Use null when the mod has no icon.": [
        "Optionaler Dateiname des Mod-Icons im Paketordner oder ein Pfad unter assets/. Verwende null, wenn der Mod kein Icon hat.",
        schema.description,
      ],
      "Mods that must be available before this mod starts. Each id is another mod's manifest id. Version is a SemVer version requirement. If optional is true, the dependency does not block startup when missing, disabled, or outside the version range.": [
        "Mods, die vor dem Start dieses Mods verfügbar sein müssen. id muss der Mod-ID aus der anderen mod.json entsprechen. version ist ein SemVer-Bereich. Bei optional: true blockiert eine fehlende, deaktivierte oder unpassende Abhängigkeit den Start nicht.",
        schema.description,
      ],
      "Exact id from the dependency mod's mod.json.": [
        "Exakte ID aus der mod.json des benötigten Mods.",
        schema.description,
      ],
      "SemVer version requirement, for example ^1.2.0 or >=1.2.0, <2.0.0.": [
        "SemVer-Versionsbereich, zum Beispiel ^1.2.0 oder >=1.2.0, <2.0.0.",
        schema.description,
      ],
      "When true, this mod may start if the dependency is unavailable. Defaults to false.": [
        "Wenn true, darf dieser Mod auch starten, wenn die Abhängigkeit fehlt. Standard ist false.",
        schema.description,
      ],
      "Processes this mod runs in. Legacy EML packages without targets default to client and server. ShroudForge packages default to client.": [
        "Legt fest, in welchen Prozessen der Mod läuft. EML-Pakete ohne targets laufen standardmäßig auf Client und Server. ShroudForge-Pakete laufen standardmäßig auf dem Client.",
        schema.description,
      ],
      "Launcher that originally identified this mod. Omit for ShroudForge mods. Use EML to preserve EML provenance.": [
        "Herkunft des Mods. Für ShroudForge-Mods weglassen. EML eintragen, um die Herkunft eines migrierten EML-Mods zu erhalten.",
        schema.description,
      ],
    };
    values.push(translations[schema.description]?.[locale === "de" ? 0 : 1] || schema.description);
  }
  return values.length ? values.map(value => `- ${value}`).join("\n") : (locale === "de" ? "Keine zusätzlichen Werte oder Grenzen im Schema angegeben." : "No additional values or limits are declared by the schema.");
}

function sampleForSchema(schema) {
  if (!schema || typeof schema !== "object") return null;
  if (schema.const !== undefined) return schema.const;
  if (schema.enum?.length) return schema.enum[0];
  const types = Array.isArray(schema.type) ? schema.type.filter(type => type !== "null") : [schema.type];
  const type = types[0];
  if (type === "array") return [sampleForSchema(schema.items)];
  if (type === "object") {
    const sample = {};
    for (const key of schema.required || []) sample[key] = sampleForSchema(schema.properties?.[key]);
    return sample;
  }
  if (type === "boolean") return true;
  if (type === "integer" || type === "number") return schema.minimum ?? 1;
  return "example";
}

function exampleForField(pathLabel, property, value) {
  if (pathLabel.startsWith("dependencies[].")) {
    const dependency = { id: "helper.mod", version: "^1.0.0" };
    dependency[property] = value;
    return { dependencies: [dependency] };
  }
  if (pathLabel.startsWith("settings.<key>.")) {
    const setting = { value: true };
    if (property === "control") setting.control = value;
    if (property === "control" && value === "slider") setting.value = 1;
    if (["min", "max", "step"].includes(property)) {
      setting.value = 1;
      setting.control = "number";
    }
    if (["minLength", "maxLength"].includes(property)) {
      setting.value = "hello";
      setting.control = "text";
    }
    if (property === "options") {
      setting.value = "easy";
      setting.control = "select";
    }
    if (["select", "radio", "segmented"].includes(value)) setting.value = "easy";
    if (value === "multiselect") setting.value = ["easy"];
    if (["select", "radio", "segmented", "multiselect"].includes(value) || property === "options") {
      setting.options = { easy: "Easy", hard: "Hard" };
    }
    setting[property] = value;
    return { settings: { example: setting } };
  }
  if (pathLabel.startsWith("groups[].actions[].")) {
    const action = { id: "reset", label: "Reset" };
    action[property] = value;
    return { groups: [{ label: "Example", actions: [action] }] };
  }
  if (pathLabel.startsWith("groups[].")) {
    const group = { label: "Example", settings: [] };
    group[property] = value;
    return { groups: [group] };
  }
  return { [property]: value };
}

function schemaField({ schemaName, schema, property, pathLabel, file, section, group, groupOrder, fieldOrder, purposeName = property, example, requiredOverride }) {
  const localized = fieldPurpose[purposeName] || fieldPurpose[property] || [
    `Dieses Feld konfiguriert ${code(pathLabel)}. Der genaue technische Vertrag steht im Schema.`,
    `This field configures ${code(pathLabel)}. The exact technical contract is defined by the schema.`,
  ];
  const required = requiredOverride ?? (schemaName === "mod.json" ? manifestSchema.required?.includes(property) : extensionSchema.required?.includes(property));
  const type = Array.isArray(schema?.type) ? schema.type.join(" | ") : schema?.type || (schema?.enum ? "string" : "object");
  const typeTextDe = `${code(type)}${required ? " · erforderlich" : " · optional"}`;
  const typeTextEn = `${code(type)}${required ? " · required" : " · optional"}`;
  const exampleText = example ?? sampleForSchema(schema);
  const exampleJson = JSON.stringify(exampleForField(pathLabel, property, exampleText), null, 2);
  const baseId = `field-${slug(pathLabel)}`;
  const id = pages.some(page => page.id === baseId) ? `${baseId}-${slug(schemaName)}` : baseId;
  const folder = `${section}/${slug(pathLabel)}`;
  const titleDe = `Feld ${code(pathLabel)}`;
  const titleEn = `The ${code(pathLabel)} field`;
  const summaryDe = localized[0];
  const summaryEn = localized[1];
  addPage({
    id, folder, navGroup: group, navOrder: groupOrder, order: fieldOrder,
    titleDe, titleEn, summaryDe, summaryEn,
    bodyDe: referenceText("de", "reference.bodyDe.61", articleIntro(titleDe, summaryDe), typeTextDe, constraints(schema, "de"), exampleJson, schemaName, file),
    bodyEn: referenceText("en", "reference.bodyEn.61", articleIntro(titleEn, summaryEn), typeTextEn, constraints(schema, "en"), exampleJson, schemaName, file),
  });
}

const modFieldOrder = ["id", "name", "version", "description", "authors", "license", "icon", "dependencies", "capabilities", "$schema"];
for (const [property, schema] of Object.entries(manifestSchema.properties)) {
  schemaField({ schemaName: "mod.json", schema, property, pathLabel: property, file: "manifest.schema.json", section: "mod-json/fields", group: { de: referenceText("de", "reference.group.de.62"), en: referenceText("en", "reference.group.en.62") }, groupOrder: 10, fieldOrder: modFieldOrder.indexOf(property) < 0 ? 99 : modFieldOrder.indexOf(property), example: property === "id" ? "author.example-mod" : property === "name" ? "Example Mod" : property === "version" ? "1.0.0" : property === "authors" ? ["Your Name"] : property === "capabilities" ? ["runtime"] : undefined });
}
const depSchema = manifestSchema.properties.dependencies.items.properties;
for (const property of ["id", "version", "optional"]) schemaField({ schemaName: "mod.json", schema: depSchema[property], property, purposeName: property === "id" ? "dependencyId" : property === "version" ? "dependencyVersion" : property, pathLabel: `dependencies[].${property}`, file: "manifest.schema.json", section: "mod-json/dependencies", group: { de: referenceText("de", "reference.group.de.63"), en: referenceText("en", "reference.group.en.63") }, groupOrder: 11, fieldOrder: ["id", "version", "optional"].indexOf(property), requiredOverride: property === "id" || property === "version", example: property === "id" ? "author.shared-library" : property === "version" ? "^1.2.0" : true });

for (const [property, schema] of Object.entries(extensionSchema.properties)) {
  if (property === "settings" || property === "groups") {
    const purpose = property === "settings"
      ? ["Dynamische Schlüssel und Werte, die der Modloader als Mod-Einstellungen darstellt und Lua lesen kann.", "Dynamic keys and values that Modloader presents as mod settings and Lua can read."]
      : ["Optionale Gruppen für Einstellungsfelder und Lua-Aktionen.", "Optional groups for setting fields and Lua actions."];
    addPage({
      id: `extension-${property}`, folder: `extended-mod-json/${property}`, navGroup: { de: referenceText("de", "reference.navGroup.de.64"), en: referenceText("en", "reference.navGroup.en.64") }, navOrder: 20, order: property === "settings" ? 10 : 11,
      titleDe: referenceText("de", "reference.titleDe.65", code(property)), titleEn: referenceText("en", "reference.titleEn.65", code(property)), summaryDe: purpose[0], summaryEn: purpose[1],
      bodyDe: articleIntro("Feld " + code(property), purpose[0]) + "\n\n" + (property === "settings" ? "Jeder Schlüssel ist mod-eigen und muss dem Schlüssel entsprechen, den Lua an `shroudforge.settings.get(key, fallback)` übergibt. Siehe [Einstellungen und Steuerelemente](#doc-setting-controls), [settings.<key>](#doc-extension-setting-key) und [alle Metadatenfelder](#doc-field-setting-key-value)." : "Eine Gruppe hat `label` und kann `description`, `settings` sowie `actions` enthalten. Nicht gruppierte Einstellungen erscheinen in der Standardgruppe. Siehe [Gruppen und Aktionsknöpfe](#doc-setting-groups-actions)."),
      bodyEn: articleIntro("The " + code(property) + " field", purpose[1]) + "\n\n" + (property === "settings" ? "Every key is specific to the mod and must match the key passed to `shroudforge.settings.get(key, fallback)` in Lua. See [Settings and controls](#doc-setting-controls), [settings.<key>](#doc-extension-setting-key), and [all metadata fields](#doc-field-setting-key-value)." : "A group has `label` and may include `description`, `settings`, and `actions`. Ungrouped settings appear in the default group. See [Groups and action buttons](#doc-setting-groups-actions)."),
    });
  }
  if (property === "settings" || property === "groups") continue;
  schemaField({ schemaName: "extended.mod.json", schema, property, pathLabel: property, file: "extended.mod.schema.json", section: "extended-mod-json/fields", group: { de: referenceText("de", "reference.group.de.66"), en: referenceText("en", "reference.group.en.66") }, groupOrder: 20, fieldOrder: Object.keys(extensionSchema.properties).indexOf(property), example: property === "targets" ? ["client"] : property === "launcher" ? "EML" : property === "schemaVersion" ? 1 : property === "enabled" ? false : undefined });
}
const settingSchema = extensionSchema.$defs.setting.properties;
for (const property of Object.keys(settingSchema)) schemaField({ schemaName: "extended.mod.json", schema: settingSchema[property], property, pathLabel: `settings.<key>.${property}`, file: "extended.mod.schema.json", section: "extended-mod-json/setting-fields", group: { de: referenceText("de", "reference.group.de.67"), en: referenceText("en", "reference.group.en.67") }, groupOrder: 30, fieldOrder: Object.keys(settingSchema).indexOf(property), requiredOverride: property === "value", example: property === "value" ? true : property === "control" ? "slider" : property === "options" ? { easy: "Easy", hard: "Hard" } : property === "step" ? 0.1 : property === "minLength" ? 3 : property === "maxLength" ? 5 : undefined });
for (const property of ["label", "description", "settings", "actions"]) schemaField({ schemaName: "extended.mod.json", schema: extensionSchema.$defs.group.properties[property], property, pathLabel: `groups[].${property}`, file: "extended.mod.schema.json", section: "extended-mod-json/group-fields", group: { de: referenceText("de", "reference.group.de.68"), en: referenceText("en", "reference.group.en.68") }, groupOrder: 31, fieldOrder: ["label", "description", "settings", "actions"].indexOf(property), purposeName: property === "settings" ? "groupSettings" : property, requiredOverride: property === "label", example: property === "label" ? "Gameplay" : property === "settings" ? ["enabledFeature"] : property === "actions" ? [{ id: "reset", label: "Reset" }] : undefined });
const actionSchema = extensionSchema.$defs.action.properties;
for (const property of Object.keys(actionSchema)) schemaField({ schemaName: "extended.mod.json", schema: actionSchema[property], property, pathLabel: `groups[].actions[].${property}`, file: "extended-mod.schema.json", section: "extended-mod-json/action-fields", group: { de: referenceText("de", "reference.group.de.69"), en: referenceText("en", "reference.group.en.69") }, groupOrder: 31, fieldOrder: Object.keys(actionSchema).indexOf(property), purposeName: property === "id" ? "actionId" : property, requiredOverride: property === "id" || property === "label", example: property === "id" ? "resetFeature" : property === "label" ? "Reset" : property === "style" ? "secondary" : undefined });
addPage({
  id: "extension-setting-key", folder: "extended-mod-json/settings/key", navGroup: { de: referenceText("de", "extension-setting-key.navGroup.de.70"), en: referenceText("en", "extension-setting-key.navGroup.en.70") }, navOrder: 30, order: 1,
  titleDe: referenceText("de", "extension-setting-key.titleDe.71"), titleEn: referenceText("en", "extension-setting-key.titleEn.71"),
  summaryDe: referenceText("de", "extension-setting-key.summaryDe.72"),
  summaryEn: referenceText("en", "extension-setting-key.summaryEn.72"),
  bodyDe: referenceText("de", "extension-setting-key.bodyDe.73"),
  bodyEn: referenceText("en", "extension-setting-key.bodyEn.73"),
});
addPage({
  id: "extension-link-id", folder: "extended-mod-json/links/id", navGroup: { de: referenceText("de", "extension-link-id.navGroup.de.74"), en: referenceText("en", "extension-link-id.navGroup.en.74") }, navOrder: 31, order: 10,
  titleDe: referenceText("de", "extension-link-id.titleDe.75"), titleEn: referenceText("en", "extension-link-id.titleEn.75"),
  summaryDe: referenceText("de", "extension-link-id.summaryDe.76"),
  summaryEn: referenceText("en", "extension-link-id.summaryEn.76"),
  bodyDe: referenceText("de", "extension-link-id.bodyDe.77"),
  bodyEn: referenceText("en", "extension-link-id.bodyEn.77"),
});

const linkIds = ["source", "source-github", "source-gitlab", "source-codeberg", "issues", "wiki", "website", "store", "support-bmac", "support-patreon", "support-paypal", "support-github", "support-ko-fi", "support-open-collective", "support-other"];
for (const [index, id] of linkIds.entries()) addPage({
  id: `link-${slug(id)}`, folder: `extended-mod-json/links/${slug(id)}`, navGroup: { de: referenceText("de", "reference.navGroup.de.78"), en: referenceText("en", "reference.navGroup.en.78") }, navOrder: 33, order: index,
  titleDe: referenceText("de", "reference.titleDe.79", code(id)), titleEn: referenceText("en", "reference.titleEn.79", code(id)),
  summaryDe: referenceText("de", "reference.summaryDe.80", id), summaryEn: referenceText("en", "reference.summaryEn.80", id),
  bodyDe: referenceText("de", "reference.bodyDe.81", code(id), code("extended.mod.json → links." + id), id),
  bodyEn: referenceText("en", "reference.bodyEn.81", code(id), code("extended.mod.json → links." + id), id),
});

const capabilities = [
  ["patch", "Game file preparation", "Vorbereitung unterstützter Spieldateien vor dem Start", "Prepare supported game files before launch"],
  ["export", "File export", "Exportfunktionen für Mod-Ausgaben", "Export functions for mod output"],
  ["runtime", "Lua runtime", "Laufzeit-Callbacks und ShroudForge-Laufzeitfunktionen", "Runtime callbacks and ShroudForge runtime functions"],
  ["runtime-register-dll", "EML DLL registration", "Registrierung einer DLL aus dem Mod-Paket im Runtime-Schritt", "Register a package-local DLL during the runtime phase"],
];
for (const [value, , de, en] of capabilities) addPage({
  id: `capability-${value}`, folder: `mod-json/capabilities/${value}`, navGroup: { de: referenceText("de", "reference.navGroup.de.82"), en: referenceText("en", "reference.navGroup.en.82") }, navOrder: 12, order: capabilities.findIndex(row => row[0] === value),
  titleDe: referenceText("de", "reference.titleDe.83", code(value)), titleEn: referenceText("en", "reference.titleEn.83", code(value)), summaryDe: de, summaryEn: en,
  bodyDe: referenceText("de", "reference.bodyDe.84", value, de, value),
  bodyEn: referenceText("en", "reference.bodyEn.84", value, en, value),
});

const controls = [
  ["toggle", "Schalter", "Toggle", "Zeigt einen Ein/Aus-Schalter für einen booleschen Wert.", "Displays an on/off switch for a boolean value.", true],
  ["checkbox", "Kontrollkästchen", "Checkbox", "Zeigt einen booleschen Wert als Kontrollkästchen.", "Displays a boolean value as a checkbox.", false],
  ["text", "Einzeiliges Textfeld", "Single-line text field", "Für kurze Texte und Namen.", "For short text and names.", "text"],
  ["textarea", "Mehrzeiliges Textfeld", "Multi-line text field", "Für längere Texte, die über mehrere Zeilen bearbeitet werden.", "For longer text edited across multiple lines.", "Longer text"],
  ["number", "Zahlenfeld", "Number field", "Für numerische Werte. min, max und step können den Eingabebereich begrenzen.", "For numeric values. min, max, and step can constrain input.", 1],
  ["slider", "Regler", "Slider", "Für Zahlen, die Spieler innerhalb eines Bereichs einstellen.", "For numbers players adjust within a range.", 1],
  ["select", "Auswahlliste", "Select list", "Für genau eine Auswahl aus options.", "For exactly one choice from options.", "easy"],
  ["radio", "Optionsfelder", "Radio buttons", "Zeigt alle Werte aus options, von denen genau einer aktiv ist.", "Shows every option with exactly one active choice.", "easy"],
  ["segmented", "Segmentierte Auswahl", "Segmented choices", "Zeigt Auswahlwerte als eine Reihe von Schaltflächen.", "Shows choices as a row of buttons.", "easy"],
  ["multiselect", "Mehrfachauswahl", "Multi-select", "Erlaubt mehrere Werte. value ist eine Liste und options muss die möglichen Elemente enthalten.", "Allows several values. value is a list and options defines the available items.", ["easy"]],
  ["keybind", "Tastenkürzel", "Key binding", "Für eine Tastenkombination, die der Modloader als Schlüsselwert speichert.", "For a keyboard shortcut stored by Modloader as a key value.", "F7"],
  ["color", "Farbauswahl", "Color picker", "Für einen Farbwert, der im Modloader als Farbfeld bearbeitet wird.", "For a color value edited through a color input in Modloader.", "#d0ae6d"],
];
for (const [id, titleDe, titleEn, purposeDe, purposeEn, value] of controls) addPage({
  id: `control-${id}`, folder: `extended-mod-json/controls/${id}`, navGroup: { de: referenceText("de", "reference.navGroup.de.85"), en: referenceText("en", "reference.navGroup.en.85") }, navOrder: 32, order: controls.findIndex(row => row[0] === id),
  titleDe: referenceText("de", "reference.titleDe.86", code(id)), titleEn: referenceText("en", "reference.titleEn.86", code(id)), summaryDe: purposeDe, summaryEn: purposeEn,
  bodyDe: referenceText("de", "reference.bodyDe.87", id, purposeDe, JSON.stringify(value), titleDe, id, ["select", "radio", "segmented", "multiselect"].includes(id) ? ", \"options\": {\"easy\": \"Einfach\", \"hard\": \"Schwer\"}" : "", ["select", "radio", "segmented", "multiselect"].includes(id) ? "Die Optionsschlüssel sind gespeicherte Werte. Bei \x60multiselect\x60 muss \x60value\x60 eine Liste sein." : "Der Wert in \x60value\x60 sollte zum Steuerelement passen."),
  bodyEn: referenceText("en", "reference.bodyEn.87", id, purposeEn, JSON.stringify(value), titleEn, id, ["select", "radio", "segmented", "multiselect"].includes(id) ? ", \"options\": {\"easy\": \"Easy\", \"hard\": \"Hard\"}" : "", ["select", "radio", "segmented", "multiselect"].includes(id) ? "Option keys are the stored values. For \x60multiselect\x60, \x60value\x60 must be a list." : "The value in \x60value\x60 should match the selected control."),
});

addPage({
  id: "targets", folder: "extended-mod-json/targets", navGroup: { de: referenceText("de", "targets.navGroup.de.88"), en: referenceText("en", "targets.navGroup.en.88") }, navOrder: 20, order: 1,
  titleDe: referenceText("de", "targets.titleDe.89"), titleEn: referenceText("en", "targets.titleEn.89"),
  summaryDe: referenceText("de", "targets.summaryDe.90"),
  summaryEn: referenceText("en", "targets.summaryEn.90"),
  bodyDe: referenceText("de", "targets.bodyDe.91"),
  bodyEn: referenceText("en", "targets.bodyEn.91"),
});

// User-facing Modloader preferences are separate from settings packaged by a mod.
const general = loaderSchema.properties.general.properties;
const modules = loaderSchema.properties.modules.properties;
const preferencePurposes = {
  locale: ["Sprache der Modloader-Oberfläche, als gültiger Sprach-Tag.", "Language of the Modloader interface, as a valid language tag."],
  compactMode: ["Verdichtet die Modloader-Oberfläche.", "Uses a denser Modloader layout."],
  reducedMotion: ["Reduziert Bewegungs- und Übergangseffekte.", "Reduces motion and transition effects."],
  usernameMode: ["Legt fest, ob der Spielername automatisch erkannt oder manuell festgelegt wird.", "Chooses automatic player-name detection or a manual name."],
  messageNameFallback: ["Name, den Nachrichten verwenden, wenn die automatische Erkennung keinen eindeutigen Spielernamen findet.", "Name used by notices when automatic detection cannot find one player name."],
  minimumLevel: ["Niedrigste Log-Stufe, die in ShroudForge-Dateien gespeichert wird.", "Lowest log level written to ShroudForge log files."],
  enabled: ["Schaltet dieses eingebaute Loader-Modul ein oder aus.", "Enables or disables this built-in loader module."],
  serverSteamId: ["Fallback-SteamID64 eines Dedicated Servers, wenn der Client sie nicht automatisch aus dem Live-Kontext erkennt.", "Fallback SteamID64 for a Dedicated Server when the client cannot detect it from the live world context."],
  allowedClientSteamIds: ["Optionale Liste erlaubter Client-SteamID64-Werte für Server-P2P-Anfragen.", "Optional list of client SteamID64 values allowed to send server P2P requests."],
  startPage: ["Startseite, die die Modloader-Oberfläche nach dem Öffnen zeigt.", "Page shown when the Modloader interface opens."],
  toggleKey: ["Tastencode zum Öffnen und Schließen dieses Fensters.", "Key code used to open and close this window."],
  refreshMilliseconds: ["Intervall, in dem das UI den aktuellen Zustand erneut abfragt.", "Interval used by the UI to refresh its current state."],
  defaultSource: ["Logquelle, die in der Debug Console zuerst ausgewählt wird.", "Log source selected first in the Debug Console."],
  levelFilter: ["Anzeigefilter für Log-Stufen in der Debug Console.", "Display filter for log levels in the Debug Console."],
  autoScroll: ["Scrollt bei neuen Logzeilen automatisch zum Ende.", "Automatically scrolls to the newest log lines."],
  tailBytes: ["Maximale Dateimenge, die die Debug Console beim Lesen vom Log-Ende berücksichtigt.", "Maximum amount read from the end of a log file by the Debug Console."],
  continuous: ["Lässt die Laufzeitdiagnose über einzelne Schnappschüsse hinaus regelmäßig weiterlaufen.", "Keeps runtime diagnostics sampling beyond a one-time snapshot."],
  intervalMilliseconds: ["Abstand zwischen Diagnose-Schnappschüssen.", "Delay between diagnostic snapshots."],
  maximumDurationSeconds: ["Höchstdauer einer Diagnosesitzung.", "Maximum duration of a diagnostics capture."],
  slowCallbackMilliseconds: ["Schwelle, ab der ein Callback als langsam markiert wird.", "Threshold at which a callback is marked slow."],
  onlyChanges: ["Veröffentlicht wiederholte Diagnosen nur, wenn sich der Zustand ändert.", "Reports repeated diagnostics only when the state changes."],
  areas: ["Diagnosebereiche, die ein Schnappschuss aufnehmen darf.", "Diagnostic areas included in a snapshot."],
  checkMinutes: ["Wartezeit zwischen Update-Prüfungen.", "Delay between update checks."],
  channel: ["Release-Kanal, der auf Updates geprüft wird.", "Release channel used for update checks."],
  includePrereleases: ["Bezieht Vorabversionen in Update-Prüfungen ein.", "Includes pre-release versions in update checks."],
};
const pathPurposes = {
  mods: ["Ordner, in dem ShroudForge installierte Mod-Pakete sucht.", "Folder where ShroudForge searches for installed mod packages."],
  state: ["Ordner für den persistenten Loader-Zustand, darunter Aktivierungs- und Laufzeitstatus.", "Folder for persistent loader state, including enablement and runtime status."],
  logs: ["Ordner für ShroudForge- und Mod-Logdateien. Client und Dedicated Server schreiben getrennte Dateien.", "Folder for ShroudForge and mod log files. The client and Dedicated Server write separate files."],
  cache: ["Ordner für wiederverwendbare Zwischendaten.", "Folder for reusable cached data."],
  exports: ["Zielordner für Dateien, die Mods exportieren.", "Destination folder for files exported by mods."],
  updates: ["Ordner für heruntergeladene oder vorbereitete Updates.", "Folder for downloaded or staged updates."],
  ui: ["Ordner für persistente Daten und das WebView-Profil der Modloader-Oberfläche.", "Folder for persistent data and the Modloader interface WebView profile."],
  runtime: ["Ordner für Lua- und native Laufzeitdaten sowie erkannte Spielprofile.", "Folder for Lua and native runtime data and detected game profiles."],
};
const preferencePathPurposes = {
  "exports.enabled": ["Schaltet die Exportfunktionen für Mod-Ausgaben ein oder aus.", "Enables or disables mod export functions."],
  "logging.minimumLevel": ["Niedrigste Stufe, die ShroudForge in Logdateien speichert. Die Debug Console hat zusätzlich einen eigenen Anzeigefilter.", "Lowest level ShroudForge writes to log files. The Debug Console also has its own display filter."],
  "modules.debugConsole.toggleKey": ["Taste zum Öffnen oder Schließen der Debug Console. Der Standardwert 121 entspricht F10.", "Key used to open or close the Debug Console. The default value 121 is F10."],
  "modules.debugConsole.defaultSource": ["Logquelle, die beim Öffnen der Debug Console ausgewählt ist.", "Log source selected when the Debug Console opens."],
  "modules.debugConsole.levelFilter": ["Filtert sichtbare Zeilen in der Debug Console. Das ändert nicht, welche Zeilen in der Logdatei gespeichert werden.", "Filters visible rows in the Debug Console. It does not change which rows are written to the log file."],
  "modules.modloaderUi.toggleKey": ["Taste zum Öffnen oder Schließen des Modloader-Fensters. Der Standardwert 120 entspricht F9.", "Key used to open or close the Modloader window. The default value 120 is F9."],
  "modules.worldEditor.toggleKey": ["Taste zum Öffnen oder Schließen des World-Editor-Fensters. Der Standardwert 113 entspricht F2.", "Key used to open or close the World Editor window. The default value 113 is F2."],
  "modules.network.serverSteamId": ["Optionale SteamID64 als Fallback, wenn der Client die Dedicated-Server-ID nicht aus dem aktuellen Weltkontext erkennt.", "Optional SteamID64 fallback when the client cannot detect the Dedicated Server ID from the current world context."],
  "modules.network.allowedClientSteamIds": ["Optionale SteamID64-Liste für Clients, die Server-P2P-Anfragen senden dürfen. Ohne Einträge gelten authentifizierte Spieler.", "Optional SteamID64 list for clients allowed to send server P2P requests. Authenticated players are accepted when the list is empty."],
};

addPage({
  id: "modloader-settings-guide", folder: "modloader-settings", navGroup: { de: referenceText("de", "modloader-settings-guide.navGroup.de.92"), en: referenceText("en", "modloader-settings-guide.navGroup.en.92") }, navOrder: 60, order: 0,
  titleDe: referenceText("de", "modloader-settings-guide.titleDe.93"), titleEn: referenceText("en", "modloader-settings-guide.titleEn.93"),
  summaryDe: referenceText("de", "modloader-settings-guide.summaryDe.94"),
  summaryEn: referenceText("en", "modloader-settings-guide.summaryEn.94"),
  bodyDe: referenceText("de", "modloader-settings-guide.bodyDe.95"),
  bodyEn: referenceText("en", "modloader-settings-guide.bodyEn.95"),
});

function getAt(value, keys) {
  return keys.reduce((current, key) => current && typeof current === "object" ? current[key] : undefined, value);
}
function schemaLeaves(node, prefix = [], output = []) {
  if (!node || typeof node !== "object" || !node.properties) return output;
  for (const [key, schema] of Object.entries(node.properties)) {
    const next = [...prefix, key];
    if (schema.properties) schemaLeaves(schema, next, output);
    else output.push({ path: next, schema });
  }
  return output;
}
const preferenceRoots = [
  ...schemaLeaves({ properties: general }, ["general"]),
  ...schemaLeaves(loaderSchema.properties.logging, ["logging"]),
  ...schemaLeaves(loaderSchema.properties.exports, ["exports"]),
  ...schemaLeaves(loaderSchema.properties.paths, ["paths"]),
  ...schemaLeaves({ properties: { modules } }, ["modules"]),
];
const excludedPreference = new Set(["modules.runtimeDiagnostics.requestId", "modules.debugConsole.window.position", "modules.modloaderUi.window.position"]);
for (const [index, entry] of preferenceRoots.entries()) {
  const pathText = entry.path.join(".");
  if (excludedPreference.has(pathText) || pathText.startsWith("modules.debugConsole.window.position.") || pathText.startsWith("modules.modloaderUi.window.position.")) continue;
  const key = entry.path.at(-1);
  const purpose = entry.path[0] === "paths"
    ? pathPurposes[key]
    : preferencePathPurposes[pathText] || preferencePurposes[key] || [`Konfiguriert ${code(pathText)}.`, `Configures ${code(pathText)}.`];
  let groupKey = entry.path[0];
  if (groupKey === "modules") groupKey = entry.path[1] === "updates" ? "updates" : entry.path[1];
  let groupNames = {
    general: { de: referenceText("de", "reference.general.de.96"), en: referenceText("en", "reference.general.en.96") },
    logging: { de: referenceText("de", "reference.logging.de.97"), en: referenceText("en", "reference.logging.en.97") },
    exports: { de: referenceText("de", "reference.exports.de.98"), en: referenceText("en", "reference.exports.en.98") },
    debugConsole: { de: referenceText("de", "reference.debugConsole.de.99"), en: referenceText("en", "reference.debugConsole.en.99") },
    worldEditor: { de: referenceText("de", "reference.worldEditor.de.100"), en: referenceText("en", "reference.worldEditor.en.100") },
    network: { de: referenceText("de", "reference.network.de.101"), en: referenceText("en", "reference.network.en.101") },
    modloaderUi: { de: referenceText("de", "reference.modloaderUi.de.102"), en: referenceText("en", "reference.modloaderUi.en.102") },
    runtimeDiagnostics: { de: referenceText("de", "reference.runtimeDiagnostics.de.103"), en: referenceText("en", "reference.runtimeDiagnostics.en.103") },
    updates: { de: referenceText("de", "reference.updates.de.104"), en: referenceText("en", "reference.updates.en.104") },
    paths: { de: referenceText("de", "reference.paths.de.105"), en: referenceText("en", "reference.paths.en.105") },
  }[groupKey] || { de: referenceText("de", "reference.de.106"), en: referenceText("en", "reference.en.106") };
  const navOrder = ["general", "logging", "exports", "debugConsole", "modloaderUi", "worldEditor", "network", "runtimeDiagnostics", "updates", "paths"].indexOf(groupKey);
  const defaultValue = getAt(loaderDefaults, entry.path);
  const displayDefault = defaultValue === undefined ? (entry.schema.default === undefined ? (entry.schema.const ?? "nicht festgelegt") : entry.schema.default) : defaultValue;
  const defaultTextDe = entry.path[0] === "paths" && defaultValue === null
    ? "Automatisch aus dem Spielordner ermittelt"
    : `\x60${JSON.stringify(displayDefault)}\x60`;
  const defaultTextEn = entry.path[0] === "paths" && defaultValue === null
    ? "Resolved automatically from the game folder"
    : `\x60${JSON.stringify(displayDefault)}\x60`;
  const exampleValue = entry.path[0] === "paths" ? `./custom-${key}`
    : pathText === "modules.network.serverSteamId" ? "76561198000000000"
      : pathText === "modules.network.allowedClientSteamIds" ? "76561198000000000,76561198000000001"
        : defaultValue;
  const exampleObject = entry.path.reduceRight((child, pathPart) => ({ [pathPart]: child }), exampleValue);
  const exampleJson = JSON.stringify(exampleObject, null, 2);
  const networkHelpDe = pathText === "modules.network.serverSteamId"
    ? "Die ID muss aus 16 bis 20 Dezimalziffern bestehen. Leer lassen, wenn kein manueller Fallback gebraucht wird."
    : pathText === "modules.network.allowedClientSteamIds"
      ? "Trenne mehrere IDs durch Kommas oder Semikolons. Jede ID muss aus 16 bis 20 Dezimalziffern bestehen. Ein leerer Wert erlaubt authentifizierte Spieler."
      : "";
  const networkHelpEn = pathText === "modules.network.serverSteamId"
    ? "Use 16 to 20 decimal digits. Leave it empty when no manual fallback is needed."
    : pathText === "modules.network.allowedClientSteamIds"
      ? "Separate multiple IDs with commas or semicolons. Each ID must contain 16 to 20 decimal digits. An empty value allows authenticated players."
      : "";
  const type = Array.isArray(entry.schema.type) ? entry.schema.type.join(" | ") : entry.schema.type || (entry.schema.enum ? "string" : "array");
  const allowedDe = constraints(entry.schema, "de");
  const allowedEn = constraints(entry.schema, "en");
  const pageId = `preference-${slug(pathText)}`;
  const pageFolder = `modloader-settings/${slug(pathText)}`;
  const titleDe = `Einstellung ${code(pathText)}`;
  const titleEn = `Setting ${code(pathText)}`;
  addPage({
    id: pageId, folder: pageFolder, navGroup: groupNames, navOrder: 60 + Math.max(navOrder, 0), order: index + 1,
    titleDe, titleEn, summaryDe: purpose[0], summaryEn: purpose[1],
    bodyDe: referenceText("de", "reference.bodyDe.107", articleIntro(titleDe, purpose[0]), purpose[0], defaultTextDe, entry.path[0] === "paths" ? "Ein nicht gesetzter Wert wird beim Start relativ zum Spielordner aufgelöst. Ein relativer eigener Pfad bezieht sich ebenfalls auf den Spielordner und wird danach als absoluter Pfad gespeichert. Ein absoluter Pfad bleibt erhalten." : "", networkHelpDe, type, allowedDe, exampleJson, entry.path.join("/")),
    bodyEn: referenceText("en", "reference.bodyEn.107", articleIntro(titleEn, purpose[1]), purpose[1], defaultTextEn, entry.path[0] === "paths" ? "An unset value is resolved relative to the game folder at startup. A custom relative path also starts from the game folder and is then saved as an absolute path. Absolute paths are preserved." : "", networkHelpEn, type, allowedEn, exampleJson, entry.path.join("/")),
  });
}

const builtinSettingTitlesDe = {
  "sf-auto-stamina-refill": { autoRefill: "Ausdauer automatisch auffüllen" },
  "sf-production-time": { seconds: "Basis-Produktionszeit in Sekunden" },
  "world-editor": {
    panelWidth: "Breite des Blueprint-Fensters",
    panelHeight: "Höhe des Blueprint-Fensters",
    panelPositionX: "Fensterposition X, minus eins zentriert",
    panelPositionY: "Fensterposition Y ab oberem Rand",
    maximumCopyableProps: "Maximale Props pro Blueprint",
    sourceX: "Quellzelle X als manueller Ersatzwert",
    sourceY: "Quellzelle Y als manueller Ersatzwert",
    sourceZ: "Quellzelle Z als manueller Ersatzwert",
    sizeX: "Auswahlgröße X als manueller Ersatzwert",
    sizeY: "Auswahlgröße Y als manueller Ersatzwert",
    sizeZ: "Auswahlgröße Z als manueller Ersatzwert",
    blueprintName: "Dauerhafter Blueprint-Name",
    blueprintNewName: "Neuer Name beim Umbenennen oder Duplizieren",
    pasteVoxelMode: "Voxel-Modus beim Einfügen",
    targetPropMode: "Props am Einfügeziel",
    targetX: "Zielzelle X",
    targetY: "Zielzelle Y",
    targetZ: "Zielzelle Z",
    targetWorldX: "Weltposition X für Props-only-Blueprints",
    targetWorldY: "Weltposition Y für Props-only-Blueprints",
    targetWorldZ: "Weltposition Z für Props-only-Blueprints",
    rotationQuarterTurns: "Drehung vor dem Einfügen",
    rotationAxis: "Obere Blueprint-Achse",
    entityHandle: "Interner Entity-Handle",
    templateUuidHigh: "Obere Hälfte der Template-UUID in Hexadezimal",
    templateUuidLow: "Untere Hälfte der Template-UUID in Hexadezimal",
    trackingId: "Tracking-ID",
    feedbackId: "Material-Feedback-ID nur für Platzierung",
    entityX: "Entity-Position X",
    entityY: "Entity-Position Y",
    entityZ: "Entity-Position Z",
    boundsMinX: "Untere Begrenzung X",
    boundsMinY: "Untere Begrenzung Y",
    boundsMinZ: "Untere Begrenzung Z",
    boundsMaxX: "Obere Begrenzung X",
    boundsMaxY: "Obere Begrenzung Y",
    boundsMaxZ: "Obere Begrenzung Z",
  },
};
const builtinSettingPurposes = {
  "sf-auto-stamina-refill": {
    autoRefill: ["Schaltet die wiederholte Auffüllung der Ausdauer bis zum Maximum ein oder aus.", "Turns repeated stamina refill to maximum on or off."],
  },
  "sf-production-time": {
    seconds: ["Legt die Basisdauer in Sekunden für alle zeitgesteuerten Produktionsrezepte fest. Stelle denselben Wert auf Client und Host oder Server ein, bevor das Spiel startet.", "Sets the base duration in seconds for timed production recipes. Set the same value on the client and host or server before starting the game."],
  },
  "world-editor": {
    panelWidth: ["Breite des World-Editor-Fensters in logischen Pixeln. Änderungen gelten während des laufenden Spiels.", "Width of the World Editor window in logical pixels. Changes apply while the game is running."],
    panelHeight: ["Höhe des World-Editor-Fensters in logischen Pixeln. Änderungen gelten während des laufenden Spiels.", "Height of the World Editor window in logical pixels. Changes apply while the game is running."],
    panelPositionX: ["Horizontale Position relativ zum Spielfenster. Der Wert −1 zentriert das Fenster, nichtnegative Werte legen den linken Abstand fest.", "Horizontal position relative to the game window. A value of −1 centers the window, nonnegative values set its left offset."],
    panelPositionY: ["Vertikaler Abstand des World-Editor-Fensters vom oberen Rand des Spielfensters.", "Vertical offset of the World Editor window from the top of the game window."],
    maximumCopyableProps: ["Maximale Prop-Anzahl pro Blueprint. Aufnahme, Speichern, Laden und Einfügen lehnen größere Blueprints ab, statt Props still zu entfernen.", "Maximum props per blueprint. Capture, save, load, and paste reject larger blueprints instead of silently dropping props."],
    sourceX: ["Manueller X-Koordinatenwert der Quellzelle für eine Aufnahme ohne vollständig markierte Cursor-Ecken. Normalerweise die Auswahl mit F5 markieren.", "Manual X coordinate of the source cell when capturing without both cursor corners marked. Normally mark the selection with F5."],
    sourceY: ["Manueller Y-Koordinatenwert der Quellzelle für eine Aufnahme ohne vollständig markierte Cursor-Ecken. Normalerweise die Auswahl mit F5 markieren.", "Manual Y coordinate of the source cell when capturing without both cursor corners marked. Normally mark the selection with F5."],
    sourceZ: ["Manueller Z-Koordinatenwert der Quellzelle für eine Aufnahme ohne vollständig markierte Cursor-Ecken. Normalerweise die Auswahl mit F5 markieren.", "Manual Z coordinate of the source cell when capturing without both cursor corners marked. Normally mark the selection with F5."],
    sizeX: ["Manuelle Ausdehnung der Aufnahme entlang X in Voxel-Zellen. Wird nur verwendet, wenn keine vollständige Auswahl über Cursor-Ecken vorliegt.", "Manual capture extent along X in voxel cells. Used only when there is no complete selection marked by cursor corners."],
    sizeY: ["Manuelle Ausdehnung der Aufnahme entlang Y in Voxel-Zellen. Wird nur verwendet, wenn keine vollständige Auswahl über Cursor-Ecken vorliegt.", "Manual capture extent along Y in voxel cells. Used only when there is no complete selection marked by cursor corners."],
    sizeZ: ["Manuelle Ausdehnung der Aufnahme entlang Z in Voxel-Zellen. Wird nur verwendet, wenn keine vollständige Auswahl über Cursor-Ecken vorliegt.", "Manual capture extent along Z in voxel cells. Used only when there is no complete selection marked by cursor corners."],
    blueprintName: ["Standardname für das Speichern eines Blueprints und den ersten Namen in den Mod-Aktionen zum Umbenennen oder Duplizieren.", "Default name for saving a blueprint and the first name used by rename or duplicate actions."],
    blueprintNewName: ["Zielname für die Mod-Aktionen zum Umbenennen oder Duplizieren eines Blueprints.", "Destination name for the mod actions that rename or duplicate a blueprint."],
    pasteVoxelMode: ["Bestimmt, wie Voxel beim Einfügen zusammengeführt werden. `replace` ersetzt die Zielzellen, `add` fügt nur belegte Quellzellen hinzu.", "Controls how voxels are combined during paste. `replace` replaces target cells, while `add` adds only occupied source cells."],
    targetPropMode: ["Bestimmt, ob Props am Ziel erhalten bleiben oder sicher überprüfte, sich überschneidende Props ersetzt werden.", "Controls whether props at the target are kept or safely verified intersecting props are replaced."],
    targetX: ["Manuelle X-Zielkoordinate in Voxel-Zellen für ein Blueprint mit Voxel-Daten, wenn weder Cursor noch markiertes Ziel verwendet werden.", "Manual X target coordinate in voxel cells for a blueprint with voxels when neither the cursor nor a marked target is used."],
    targetY: ["Manuelle Y-Zielkoordinate in Voxel-Zellen für ein Blueprint mit Voxel-Daten, wenn weder Cursor noch markiertes Ziel verwendet werden.", "Manual Y target coordinate in voxel cells for a blueprint with voxels when neither the cursor nor a marked target is used."],
    targetZ: ["Manuelle Z-Zielkoordinate in Voxel-Zellen für ein Blueprint mit Voxel-Daten, wenn weder Cursor noch markiertes Ziel verwendet werden.", "Manual Z target coordinate in voxel cells for a blueprint with voxels when neither the cursor nor a marked target is used."],
    targetWorldX: ["Weltkoordinate X als manuelles Einfügeziel für Props-only-Blueprints, wenn weder Cursor noch markiertes Ziel verwendet werden.", "World coordinate X as the manual paste target for props-only blueprints when neither the cursor nor a marked target is used."],
    targetWorldY: ["Weltkoordinate Y als manuelles Einfügeziel für Props-only-Blueprints, wenn weder Cursor noch markiertes Ziel verwendet werden.", "World coordinate Y as the manual paste target for props-only blueprints when neither the cursor nor a marked target is used."],
    targetWorldZ: ["Weltkoordinate Z als manuelles Einfügeziel für Props-only-Blueprints, wenn weder Cursor noch markiertes Ziel verwendet werden.", "World coordinate Z as the manual paste target for props-only blueprints when neither the cursor nor a marked target is used."],
    rotationQuarterTurns: ["Anfangsdrehung des aktiven Blueprints in Vierteldrehungen. 0, 1, 2 und 3 entsprechen 0°, 90°, 180° und 270°. F3 erhöht den Wert im Spiel.", "Initial rotation of the active blueprint in quarter turns. 0, 1, 2, and 3 mean 0°, 90°, 180°, and 270°. F3 advances it in game."],
    rotationAxis: ["Obere Achse neuer Aufnahmen. Sie wird im Blueprint gespeichert, bereits gespeicherte Blueprints behalten ihre eigene Achse.", "Up axis for new captures. It is stored in the blueprint, and existing blueprints keep their own axis."],
    entityHandle: ["Laufzeit-Handle eines Props aus der Aktion List props. Es gilt nur für den aktuellen Prozess und ist für die erweiterten nativen Aktionen gedacht.", "Runtime handle for a prop returned by the List props action. It is valid only in the current process and is intended for advanced native actions."],
    templateUuidHigh: ["Obere hexadezimale Hälfte der nativen Template-UUID für die erweiterte Aktion Queue prop spawn.", "Upper hexadecimal half of the native template UUID for the advanced Queue prop spawn action."],
    templateUuidLow: ["Untere hexadezimale Hälfte der nativen Template-UUID für die erweiterte Aktion Queue prop spawn.", "Lower hexadecimal half of the native template UUID for the advanced Queue prop spawn action."],
    trackingId: ["Nichtnuller Tracking-Wert aus der ItemInfo-Platzierungsdefinition. Die erweiterten nativen Spawn- und Platzierungsaktionen benötigen ihn.", "Nonzero tracking value from the ItemInfo placement definition. The advanced native spawn and placement actions require it."],
    feedbackId: ["Material-Feedback-ID aus der ItemInfo-Platzierungsdefinition. Die erweiterte Aktion Place at coordinates benötigt sie, Destroy at coordinates nicht.", "Material feedback ID from the ItemInfo placement definition. The advanced Place at coordinates action requires it, Destroy at coordinates does not."],
    entityX: ["Weltkoordinate X für die erweiterten nativen Props-Aktionen zum Erstellen, Platzieren oder Entfernen.", "World coordinate X for the advanced native prop spawn, placement, or removal actions."],
    entityY: ["Weltkoordinate Y für die erweiterten nativen Props-Aktionen zum Erstellen, Platzieren oder Entfernen.", "World coordinate Y for the advanced native prop spawn, placement, or removal actions."],
    entityZ: ["Weltkoordinate Z für die erweiterten nativen Props-Aktionen zum Erstellen, Platzieren oder Entfernen.", "World coordinate Z for the advanced native prop spawn, placement, or removal actions."],
    boundsMinX: ["X-Minimum der Begrenzung, die an die erweiterten nativen Props-Platzierungs- und Entfernungsaktionen übergeben wird.", "X minimum of the bounds passed to the advanced native prop placement and removal actions."],
    boundsMinY: ["Y-Minimum der Begrenzung, die an die erweiterten nativen Props-Platzierungs- und Entfernungsaktionen übergeben wird.", "Y minimum of the bounds passed to the advanced native prop placement and removal actions."],
    boundsMinZ: ["Z-Minimum der Begrenzung, die an die erweiterten nativen Props-Platzierungs- und Entfernungsaktionen übergeben wird.", "Z minimum of the bounds passed to the advanced native prop placement and removal actions."],
    boundsMaxX: ["X-Maximum der Begrenzung, die an die erweiterten nativen Props-Platzierungs- und Entfernungsaktionen übergeben wird.", "X maximum of the bounds passed to the advanced native prop placement and removal actions."],
    boundsMaxY: ["Y-Maximum der Begrenzung, die an die erweiterten nativen Props-Platzierungs- und Entfernungsaktionen übergeben wird.", "Y maximum of the bounds passed to the advanced native prop placement and removal actions."],
    boundsMaxZ: ["Z-Maximum der Begrenzung, die an die erweiterten nativen Props-Platzierungs- und Entfernungsaktionen übergeben wird.", "Z maximum of the bounds passed to the advanced native prop placement and removal actions."],
  },
};
const builtinGermanText = new Map([
  ["Repeatedly refills stamina to maximum.", "Füllt die Ausdauer wiederholt bis zum Maximum auf."],
  ["Apply the same value to client and host/server before starting the game. World speed settings multiply this base duration. Instant recipes and inventory quantities are preserved.", "Lege vor dem Spielstart denselben Wert auf Client und Host oder Server fest. Weltgeschwindigkeitseinstellungen multiplizieren diese Basisdauer. Sofortrezepte und Inventarmengen bleiben unverändert."],
  ["Copy, rotate, save, and paste parts of the world.", "Weltbereiche kopieren, drehen, speichern und einfügen."],
  ["Keeps the original stack when splitting items. Also affects other inventory operations.", "Behält beim Teilen von Gegenständen den ursprünglichen Stapel. Das wirkt sich auch auf weitere Inventaraktionen aus."],
  ["Prevents items from being consumed when used.", "Verhindert, dass Gegenstände bei der Verwendung verbraucht werden."],
  ["Prevents health loss from falling.", "Verhindert Lebenspunkteverlust durch Stürze."],
  ["Removes resource consumption in supported building, crafting, and item-use actions.", "Entfernt den Ressourcenverbrauch bei unterstützten Bau-, Herstellungs- und Verwendungsaktionen."],
  ["Sets timed production recipes to a chosen base duration. World speed settings still apply.", "Legt eine Basisdauer für zeitgesteuerte Produktionsrezepte fest. Weltgeschwindigkeitseinstellungen gelten weiterhin."],
  ["Lets you keep flying without a time limit.", "Ermöglicht unbegrenztes Fliegen."],
  ["Changes recipe unlock requirements to the first Flame Altar hint.", "Ändert die Freischaltbedingungen von Rezepten auf den ersten Hinweis am Flammenaltar."],
  ["Spawn and placement use native game operations.", "Erstellen und Platzieren verwenden native Spieloperationen."],
  ["Position is relative to the game window in logical pixels. X = −1 centers the panel. Y is measured down from the top. Changes apply while the game is running.", "Die Position bezieht sich auf das Spielfenster und wird in logischen Pixeln gemessen. X = −1 zentriert das Fenster. Y wird vom oberen Rand nach unten gemessen. Änderungen gelten während des laufenden Spiels."],
  ["Capture voxel-and-prop or props-only blueprints, preview, paste, and undo through native world APIs.", "Voxel- und Prop-Blueprints oder reine Prop-Blueprints aufnehmen, prüfen, einfügen und über native Welt-APIs rückgängig machen."],
  ["Maximum props per blueprint", "Maximale Anzahl an Props pro Blueprint"],
  ["Replace target voxels", "Voxel am Ziel ersetzen"],
  ["Add occupied source voxels", "Belegte Voxel aus der Quelle hinzufügen"],
  ["Keep existing props", "Vorhandene Props behalten"],
  ["Replace intersecting props", "Props am Ziel ersetzen, wenn sie sich überschneiden"],
  ["X axis", "X-Achse"],
  ["Y axis", "Y-Achse"],
  ["Z axis", "Z-Achse"],
]);
const builtinGroupTitlesDe = new Map([
  ["In-game panel", "Fenster im Spiel"],
  ["World Editor limits", "Grenzen des World Editors"],
  ["World Editor", "World Editor"],
  ["Native entity operations", "Native Entity-Operationen"],
  ["Stamina behavior", "Ausdauerverhalten"],
  ["Production time", "Produktionszeit"],
]);

// Each shipped mod setting gets its own page based on the checked-in package definition.
const modsRoot = path.join(root, "mods");
const modDirectories = fs.readdirSync(modsRoot, { withFileTypes: true }).filter(entry => entry.isDirectory()).sort((a, b) => a.name.localeCompare(b.name));
let builtinOrder = 0;
for (const directory of modDirectories) {
  const extensionPath = path.join(modsRoot, directory.name, "extended.mod.json");
  if (!fs.existsSync(extensionPath)) continue;
  const extension = readJson(extensionPath);
  if (!extension.settings || typeof extension.settings !== "object") continue;
  const modManifest = readJson(path.join(modsRoot, directory.name, "mod.json"));
  const modTitle = modManifest.name || titleFromKey(directory.name.replace(/^sf-/, "").replaceAll("-", " "));
  const modDescription = modManifest.description || "";
  const modDescriptionDe = builtinGermanText.get(modDescription) || modDescription;
  for (const [key, raw] of Object.entries(extension.settings)) {
    const metadata = raw && typeof raw === "object" && !Array.isArray(raw) ? raw : { value: raw };
    const settingTitle = metadata.label || titleFromKey(key);
    const settingTitleDe = builtinSettingTitlesDe[directory.name]?.[key] || settingTitle;
    const group = extension.groups?.find(entry => entry.settings?.includes(key));
    const groupDescriptionDe = group?.description ? builtinGermanText.get(group.description) || group.description : "";
    const groupPurposeDe = groupDescriptionDe ? ` ${groupDescriptionDe}` : group ? ` Sie gehört zur Gruppe ${code(builtinGroupTitlesDe.get(group.label) || group.label)}.` : "";
    const groupPurposeEn = group?.description ? ` ${group.description}` : group ? ` It belongs to the ${code(group.label)} group.` : "";
    const specificPurpose = builtinSettingPurposes[directory.name]?.[key];
    const purposeDe = metadata.description || specificPurpose?.[0] || `Legt den Wert für ${code(settingTitleDe)} im Mod ${code(modTitle)} fest.${groupPurposeDe}`;
    const purposeEn = metadata.description || specificPurpose?.[1] || `Sets the value for ${code(settingTitle)} in the ${code(modTitle)} mod.${groupPurposeEn}`;
    const id = `builtin-${slug(directory.name)}-${slug(key)}`;
    const folder = `built-in-mods/${directory.name}/${slug(key)}`;
    const groupIndex = group ? extension.groups.indexOf(group) : 0;
    const groupLabelDe = group ? builtinGermanText.get(group.label) || builtinGroupTitlesDe.get(group.label) || group.label : "Einstellungen";
    const navGroup = { de: referenceText("de", "reference.de.108", modTitle, groupLabelDe), en: referenceText("en", "reference.en.108", modTitle, group?.label || "Settings") };
    const value = Object.hasOwn(metadata, "value") ? metadata.value : null;
    const valueType = value === null ? "null" : Array.isArray(value) ? "array" : typeof value;
    const acceptedDe = metadata.options
      ? `Erlaubte Werte: ${Object.entries(metadata.options).map(([option, label]) => `${code(option)} für ${builtinGermanText.get(label) || label}`).join(", ")}.`
      : typeof value === "boolean"
        ? "Erlaubte Werte: `true` oder `false`."
        : typeof value === "number" && (metadata.min !== undefined || metadata.max !== undefined)
          ? `Zahlenbereich: ${metadata.min ?? "kein Minimum"} bis ${metadata.max ?? "kein Maximum"}.`
          : typeof value === "number"
            ? "Zahlenwert. Das Mod-Paket legt keine weitere Grenze fest."
            : typeof value === "string" && (metadata.minLength !== undefined || metadata.maxLength !== undefined)
              ? `Textlänge: ${metadata.minLength ?? 0} bis ${metadata.maxLength ?? "unbegrenzt"} Zeichen.`
              : typeof value === "string"
                ? "Textwert. Das Mod-Paket legt keine weitere Längenbegrenzung fest."
                : "Der Werttyp und die erlaubten Inhalte richten sich nach diesem gespeicherten Standardwert.";
    const acceptedEn = metadata.options
      ? `Allowed values: ${Object.entries(metadata.options).map(([option, label]) => `${code(option)} for ${label}`).join(", ")}.`
      : typeof value === "boolean"
        ? "Allowed values: `true` or `false`."
        : typeof value === "number" && (metadata.min !== undefined || metadata.max !== undefined)
          ? `Number range: ${metadata.min ?? "no minimum"} to ${metadata.max ?? "no maximum"}.`
          : typeof value === "number"
            ? "Numeric value. The mod package declares no additional limit."
            : typeof value === "string" && (metadata.minLength !== undefined || metadata.maxLength !== undefined)
              ? `Text length: ${metadata.minLength ?? 0} to ${metadata.maxLength ?? "unlimited"} characters.`
              : typeof value === "string"
                ? "Text value. The mod package declares no additional length limit."
                : "The value type and accepted content follow the saved default value.";
    const readmePath = path.join(modsRoot, directory.name, "README.md");
    const readmeLinkDe = fs.existsSync(readmePath)
      ? `\n\n[Mod-Anleitung](https://github.com/bonsaibauer/shroudforge/blob/HEAD/mods/${directory.name}/README.md)`
      : "";
    const readmeLinkEn = fs.existsSync(readmePath)
      ? `\n\n[Mod guide](https://github.com/bonsaibauer/shroudforge/blob/HEAD/mods/${directory.name}/README.md)`
      : "";
    const detailsDe = [
      `Mod: ${modTitle} (${code(directory.name)}).`,
      `Schlüssel für ${code(`shroudforge.settings.get("${key}", fallback)`)}: ${code(key)}.`,
      `Datentyp: ${code(valueType)}.`,
      `Startwert: ${code(JSON.stringify(value))}.`,
      acceptedDe,
      metadata.control ? `Steuerelement: ${code(metadata.control)}.` : "Das Steuerelement wird aus dem Werttyp abgeleitet.",
      metadata.min !== undefined ? `Minimum: ${code(metadata.min)}.` : "",
      metadata.max !== undefined ? `Maximum: ${code(metadata.max)}.` : "",
      metadata.step !== undefined ? `Schritt: ${code(metadata.step)}.` : "",
      metadata.options ? `Optionen: ${Object.entries(metadata.options).map(([option, label]) => `${code(option)} (${builtinGermanText.get(label) || label})`).join(", ")}.` : "",
    ].filter(Boolean).map(item => `- ${item}`).join("\n");
    const detailsEn = [
      `Mod: ${modTitle} (${code(directory.name)}).`,
      `Key read with ${code(`shroudforge.settings.get("${key}", fallback)`)}: ${code(key)}.`,
      `Type: ${code(valueType)}.`,
      `Starting value: ${code(JSON.stringify(value))}.`,
      acceptedEn,
      metadata.control ? `Control: ${code(metadata.control)}.` : "The control is inferred from the value type.",
      metadata.min !== undefined ? `Minimum: ${code(metadata.min)}.` : "",
      metadata.max !== undefined ? `Maximum: ${code(metadata.max)}.` : "",
      metadata.step !== undefined ? `Step: ${code(metadata.step)}.` : "",
      metadata.options ? `Options: ${Object.entries(metadata.options).map(([option, label]) => `${code(option)} (${label})`).join(", ")}.` : "",
    ].filter(Boolean).map(item => `- ${item}`).join("\n");
    addPage({
      id, folder, navGroup, navOrder: 80 + modDirectories.findIndex(item => item.name === directory.name) * 10 + Math.max(groupIndex, 0), order: builtinOrder++,
      titleDe: referenceText("de", "reference.titleDe.109", settingTitleDe, key), titleEn: referenceText("en", "reference.titleEn.109", settingTitle, key),
      summaryDe: purposeDe, summaryEn: purposeEn,
      bodyDe: referenceText("de", "reference.bodyDe.110", settingTitleDe, purposeDe, modDescriptionDe ? `${modDescriptionDe}\n\n` : "", detailsDe, directory.name, key, directory.name, readmeLinkDe),
      bodyEn: referenceText("en", "reference.bodyEn.110", settingTitle, purposeEn, modDescription ? `${modDescription}\n\n` : "", detailsEn, directory.name, key, directory.name, readmeLinkEn),
    });
  }
}

const modIndexRowsDe = [];
const modIndexRowsEn = [];
for (const [modIndex, directory] of modDirectories.entries()) {
  const manifestPath = path.join(modsRoot, directory.name, "mod.json");
  if (!fs.existsSync(manifestPath)) continue;
  const manifest = readJson(manifestPath);
  const title = manifest.name || directory.name;
  const description = manifest.description || "";
  const descriptionDe = builtinGermanText.get(description) || description;
  const extensionPath = path.join(modsRoot, directory.name, "extended.mod.json");
  const extension = fs.existsSync(extensionPath) ? readJson(extensionPath) : {};
  const settingEntries = Object.entries(extension.settings || {});
  const modGuideId = `builtin-mod-${slug(directory.name)}`;
  const modGuideFolder = `built-in-mods/${directory.name}`;
  const settingLink = (key, raw, locale) => {
    const metadata = raw && typeof raw === "object" && !Array.isArray(raw) ? raw : {};
    const label = locale === "de"
      ? builtinSettingTitlesDe[directory.name]?.[key] || metadata.label || titleFromKey(key)
      : metadata.label || titleFromKey(key);
    return `[${label}](#doc-builtin-${slug(directory.name)}-${slug(key)})`;
  };
  const groupedKeys = new Set();
  const settingSectionsDe = [];
  const settingSectionsEn = [];
  for (const group of extension.groups || []) {
    const entries = (group.settings || []).filter(key => Object.hasOwn(extension.settings || {}, key));
    if (!entries.length) continue;
    entries.forEach(key => groupedKeys.add(key));
    const groupLabelDe = builtinGermanText.get(group.label) || builtinGroupTitlesDe.get(group.label) || group.label;
    settingSectionsDe.push(`## ${groupLabelDe}\n\n${(group.description ? `${builtinGermanText.get(group.description) || group.description}\n\n` : "")}${entries.map(key => `- ${settingLink(key, extension.settings[key], "de")}`).join("\n")}`);
    settingSectionsEn.push(`## ${group.label}\n\n${group.description ? `${group.description}\n\n` : ""}${entries.map(key => `- ${settingLink(key, extension.settings[key], "en")}`).join("\n")}`);
  }
  const ungrouped = settingEntries.map(([key]) => key).filter(key => !groupedKeys.has(key));
  if (ungrouped.length) {
    settingSectionsDe.push(`## Weitere Einstellungen\n\n${ungrouped.map(key => `- ${settingLink(key, extension.settings[key], "de")}`).join("\n")}`);
    settingSectionsEn.push(`## Other settings\n\n${ungrouped.map(key => `- ${settingLink(key, extension.settings[key], "en")}`).join("\n")}`);
  }
  const targetValues = Array.isArray(extension.targets) ? extension.targets : extension.launcher === "EML" ? ["client", "server"] : ["client"];
  const targetTextDe = targetValues.map(value => value === "client" ? "Client" : "Dedicated Server").join(" und ");
  const targetTextEn = targetValues.map(value => value === "client" ? "client" : "Dedicated Server").join(" and ");
  addPage({
    id: modGuideId, folder: modGuideFolder,
    navGroup: { de: referenceText("de", "reference.navGroup.de.111", title), en: referenceText("en", "reference.navGroup.en.111", title) },
    navOrder: 80 + modIndex * 10, order: 0,
    titleDe: title, titleEn: title,
    summaryDe: descriptionDe || `Übersicht des Mods ${title} und seiner Einstellungen.`,
    summaryEn: description || `Overview of ${title} and its settings.`,
    bodyDe: referenceText("de", "reference.bodyDe.112", title, descriptionDe, manifest.id, targetTextDe, manifest.capabilities?.includes("runtime") ? "ja" : "nein", settingEntries.length, settingSectionsDe.join("\n\n") || "Dieser Mod deklariert keine spielerseitigen Einstellungen.", directory.name),
    bodyEn: referenceText("en", "reference.bodyEn.112", title, description, manifest.id, targetTextEn, manifest.capabilities?.includes("runtime") ? "yes" : "no", settingEntries.length, settingSectionsEn.join("\n\n") || "This mod does not declare player-facing settings.", directory.name),
  });
  const settingCountDe = settingEntries.length
    ? `${settingEntries.length} ${settingEntries.length === 1 ? "Einstellungsseite" : "Einstellungsseiten"}`
    : "keine spielerseitigen Einstellungen";
  const settingCountEn = settingEntries.length
    ? `${settingEntries.length} ${settingEntries.length === 1 ? "setting page" : "setting pages"}`
    : "no player settings";
  modIndexRowsDe.push(`| [${title}](#doc-${modGuideId}) | ${descriptionDe} | ${settingCountDe} |`);
  modIndexRowsEn.push(`| [${title}](#doc-${modGuideId}) | ${description} | ${settingCountEn} |`);
}
addPage({
  id: "builtin-mods-guide", folder: "built-in-mods", navGroup: { de: referenceText("de", "builtin-mods-guide.navGroup.de.113"), en: referenceText("en", "builtin-mods-guide.navGroup.en.113") }, navOrder: 79, order: 0,
  titleDe: referenceText("de", "builtin-mods-guide.titleDe.114"), titleEn: referenceText("en", "builtin-mods-guide.titleEn.114"),
  summaryDe: referenceText("de", "builtin-mods-guide.summaryDe.115"),
  summaryEn: referenceText("en", "builtin-mods-guide.summaryEn.115"),
  bodyDe: referenceText("de", "builtin-mods-guide.bodyDe.116", modIndexRowsDe.join("\n")),
  bodyEn: referenceText("en", "builtin-mods-guide.bodyEn.116", modIndexRowsEn.join("\n")),
});

console.log(`Created ${pages.length} Markdown page folders from the checked-in schemas and settings.`);

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}
