# Included mods

Each direct child directory is one mod package. The repository includes `sf-survival/`, `sf-unlimited-flight/`, `sf-crafting-essentials/`, `sf-unlock-blueprints/`, and `world-editor/`. The release build validates their package files against the Modloader schemas.

The [World Editor package](world-editor/README.md) preserves its reference settings and blueprint workflow. Its Enshrouded 1.0 migration still requires verified voxel and entity creation/removal adapters before full paste and undo are available.

See the [mod package guide](../docs/modding/create-mod-package.en.md) for the manifest format and package layout. The authoritative schemas are under `modules/modloader/manifest/`.
