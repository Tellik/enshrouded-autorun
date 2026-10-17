---
{
  "id": "preference-paths-ui",
  "locale": "en",
  "title": "Setting `paths.ui`",
  "summary": "Folder for persistent data and the Modloader interface WebView profile.",
  "navigation": {
    "path": [
      "Modloader settings",
      "Storage paths"
    ]
  }
}
---

# Setting `paths.ui`

Folder for persistent data and the Modloader interface WebView profile.

## What it changes

Folder for persistent data and the Modloader interface WebView profile.

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
    "ui": "./custom-ui"
  }
}
```

## Configuration location

`shroudforge/config/modloader-config.json`, JSON path `/paths/ui`. This configures the loader and is not copied into a mod package.

Source: `modules/loader-config/loader.schema.json` and `loader.default.json`.
