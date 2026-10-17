---
{
  "id": "preference-modules-modules",
  "locale": "de",
  "title": "Moduleinstellungen",
  "summary": "Die Loader-Konfiguration hat einzelne Abschnitte für Module. Einen Eintrag `modules.modules` gibt es nicht.",
  "navigation": {
    "path": [
      "Modloader-Einstellungen"
    ]
  }
}
---

# Moduleinstellungen

In `shroudforge/config/modloader-config.json` ist `/modules` ein Objekt mit einzelnen Abschnitten für Module. Eine Einstellung `modules.modules` oder eine gemeinsame Modulliste gibt es nicht.

## Abschnitte

Das Schema kennt `debugConsole`, `worldEditor`, `network`, `modloaderUi`, `runtimeDiagnostics` und `updates`. Die Standardkonfiguration belegt `debugConsole`, `worldEditor`, `modloaderUi`, `runtimeDiagnostics` und `updates` mit Ausgangswerten.

Ein Ausschnitt aus den Standardwerten:

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

Jeder Abschnitt gehört zu dem Modul, dessen Namen er trägt. Diese Loader-Einstellungen sind von den Einstellungen eines einzelnen Mods in dessen `extended.mod.json` getrennt.

## Quelle

`modules/loader-config/loader.schema.json` beschreibt zulässige Felder und Werte. `modules/loader-config/loader.default.json` enthält die Standardwerte.
