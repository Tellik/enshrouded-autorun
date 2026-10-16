# Bundled mod packages

Follow the shared [ShroudForge logging rules](../docs/sf/logging.md) when adding or changing mod log messages.

The release build checks each direct child folder for a `mod.json` file with a
mod ID and the Lua entry point at `src/mod.lua`. It also checks the Lua files
before packaging them. The repository can contain work folders without these
package files. Those folders are not copied into the release.

## One rule for each manifest

- `mod.json` is the package manifest. It contains the package identity, version,
  author, dependencies, and explicit `capabilities`. Optional `license` and
  `icon` fields accept a string or `null`. `null` means no value is declared.
  Omitting either field is also valid. An icon is a root filename or a path under
  `assets/`.
- `targets` in `extended.mod.json` lists the supported processes using `client`
  and/or `server`. Existing EML packages without this field default to the
  `client` and `server` entries so migration does not block them. The loader
  applies this generically to every package folder without mod-ID lists.
- `capabilities` lists only the permissions the Lua code uses: `patch` for EML
  asset writes, `export` for `io.export` and export reads, and `runtime` for
  ShroudForge in-game runtime APIs. EML mods that register a package DLL also
  use `runtime-register-dll`. API scanning derives execution phase. Process
  targets come only from `extended.mod.json`.
- `extended.mod.json` is optional ShroudForge state and metadata. It holds the
  enabled value, setting values, groups, actions, links, and changelog. The
  loader writes player changes into this same file. Package updates can reset
  those choices. There is no settings migration.
- An EML package without ShroudForge-specific state can omit
  `extended.mod.json`. The loader creates it when a ShroudForge value is first
  changed. EML `mod.json` stays in the EML format.

The SF gameplay packages use the same two-file format. EML data tools only need
`mod.json` until a ShroudForge setting or action is added.

## Capability assignment

| Packages | Capabilities | Reason |
| --- | --- | --- |
| `fishing-exporter`, `item-translator`, `kfc-parser-mimic` | `export` | Write generated reports through `io.export`. Asset reads do not need `patch`. |
| `sf-survival`, `sf-unlimited-flight` | `runtime` | Independent gameplay packages using the shared reflected ECS/resource API and common reversible-override bookkeeping. Both declare `dependencies: []`. |

Both current gameplay packages, `sf-survival` and `sf-unlimited-flight`, declare
`targets: ["client", "server"]`. Neither depends on the other.

Historical packages are kept under `reference-mods-0.9/`. Temporary runtime probes
are kept under `diagnostic-mods/`. Neither folder is scanned by the release build.

Process targets describe where a package can execute. They do **not** promise
multiplayer replication. See [the per-mod multiplayer audit](../docs/sf/mod-multiplayer.md)
for local hosting, joined clients, dedicated servers and simultaneous installation.
Runtime effect reports include `processTarget` and `scope: "this-process"`.
`write-confirmed` on a modifier means its code change was verified in that process.

Inline `shroudforge` data is not part of the current manifest contract and is
rejected. ShroudForge state belongs in the neighboring `extended.mod.json`.

## Package layout

```text
mod-name/
├── mod.json
├── extended.mod.json   # only when ShroudForge-specific metadata is needed
└── src/
    └── mod.lua
```

## API details

`runtime.patch.*` names approved native runtime operations. It belongs to the
`runtime` capability. It does not mean EML asset-writing `patch`. Asset writes
through `game.assets.write` need the `patch` capability. Keep Lua entrypoints
free of gameplay reads and writes at module initialization. Register actions
there, then do runtime work from lifecycle or action callbacks.

For a setting that Lua can read, declare its key, starting `value`, and optional display details in `extended.mod.json`. List the key in one group's `settings` array so the Modloader shows it. Read the current value in Lua with `shroudforge.settings.get("key", fallback)`. Use the same key and value type in both places. The [package guide](../docs/sf/mod-packages.md) has a complete JSON/Lua example, and [`templates/mod/`](../templates/mod/) is a working version.
Build-specific signatures and trampoline data belong in KFC compatibility
profiles. Lua code uses stable operation names.
