---
{
  "id": "preference-paths-logs",
  "locale": "en",
  "title": "Setting `paths.logs`",
  "summary": "Folder for ShroudForge and mod log files. The client and Dedicated Server write separate files.",
  "navigation": {
    "path": [
      "Storage paths"
    ]
  }
}
---

# Setting `paths.logs`

Folder for ShroudForge and mod log files. The client and Dedicated Server write separate files.

## What it changes

Folder for ShroudForge and mod log files. The client and Dedicated Server write separate files.

## Default

Resolved automatically from the game folder

An unset value is resolved relative to the game folder at startup. A custom relative path also starts from the game folder and is then saved as an absolute path. Absolute paths are preserved.



## Type and allowed values

Type: `string | null`.

- minimum length: `1`

## Example

This is a partial config snippet. Keep the other entries in your file.

```json
{
  "paths": {
    "logs": "./custom-logs"
  }
}
```

## Configuration location

`shroudforge/config/modloader-config.json`, JSON path `/paths/logs`. This configures the loader and is not copied into a mod package.

Source: `modules/loader-config/loader.schema.json` and `loader.default.json`.
