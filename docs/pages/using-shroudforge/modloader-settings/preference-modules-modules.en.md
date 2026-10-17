---
{
  "id": "preference-modules-modules",
  "locale": "en",
  "title": "Module settings",
  "summary": "The loader configuration has separate sections for modules. There is no `modules.modules` entry.",
  "navigation": {
    "path": [
      "Modloader"
    ]
  }
}
---

# Module settings

In `shroudforge/config/modloader-config.json`, `/modules` is an object with separate sections for modules. There is no `modules.modules` setting or shared module list.

## Sections

The schema defines `debugConsole`, `worldEditor`, `network`, `modloaderUi`, `runtimeDiagnostics`, and `updates`. The default configuration provides initial values for `debugConsole`, `worldEditor`, `modloaderUi`, `runtimeDiagnostics`, and `updates`.

An excerpt of the default values:

```json
{
  "modules": {
    "debugConsole": { "enabled": true },
    "worldEditor": { "enabled": true },
    "modloaderUi": { "enabled": true },
    "runtimeDiagnostics": { "enabled": false },
    "updates": { "system": { "enabled": true, "checkMinutes": 60 } }
  }
}
```

Each section belongs to the module named by its key. These loader settings are separate from an individual mod's settings in its `extended.mod.json`.

## Source

`modules/loader-config/loader.schema.json` defines allowed fields and values. `modules/loader-config/loader.default.json` contains the default values.
