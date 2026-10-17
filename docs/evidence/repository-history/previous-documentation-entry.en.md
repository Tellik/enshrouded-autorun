# ShroudForge developer documentation

Welcome! This folder is for people who want to understand or contribute to the ShroudForge project. You do not need these pages to install or play with mods.

For player instructions, start with the [English website guide](https://bonsaibauer.github.io/shroudforge/en/) or the [German website guide](https://bonsaibauer.github.io/shroudforge/de/). They include pictures, a PC and server quickstart, and a step-by-step first-mod lesson.

## Choose a guide

| If you want to… | Read… |
| --- | --- |
| Find an action and its implementation | [Architecture](Architecture.md), [module owners](../../modules/README.md) |
| Review the current action-based implementation and checks | [Implementation report](action-refactor-implementation.md) |
| Review the earlier cleanup phase | [Reduction report](reduction-report.md), [its path changes](reduction-paths.csv) |
| See how the repository and mod loading fit together | [Architecture](Architecture.md) |
| Review the earlier structural migration | [Historical implementation report](refactor-implementation.md), [migration](refactor-migration.csv) |
| See the concrete refactor plan and target file paths | [Refactor master plan](refactor-masterplan.md) |
| Read the actual role, callers and risks of all 287 own code files | [Complete source audit](refactor-source-audit.md) |
| Inspect each file's proposed owner, target and actual review coverage | [Refactor file inventory](refactor-file-inventory.csv) |
| Check the exact audit scope and separate dependency inventory | [Coverage and limits](refactor-audit-coverage.md), [dependencies](refactor-dependency-inventory.csv) |
| Understand mod files and settings | [Mod packages](mod-packages.md) |
| Follow shared log levels and message rules | [Logging rules](logging.md) |
| Review client, local-host, and dedicated-server behavior | [Multiplayer execution audit](mod-multiplayer.md) |
| Work on the native runtime or its game profiles | [Native runtime](runtime.md) |
| Review runtime preparation, shared search scheduling, performance and safety | [Runtime performance implementation, live measurements and plan](runtime-performance-concept.md) |
| Change how Enshrouded game data is read | [Game data parser](parser.md) |
| Edit notices or link buttons shown in the Modloader | [Modloader content](modloader-content.md) |
| Preview Modloader typography at different window widths | [Modloader typography preview](modloader-typography-preview.html) |

The [root README](../../README.md) is the quick introduction and player starting point. The [mod template](../modding/starter-mod/README.md) is a practical place to begin creating a mod.

## Keep information easy to find

Each page covers one job and links to the code that owns it. The current schemas and source files define what ShroudForge does. Use the guide to find and understand them.

## Build one thing

Use `dev/build_modloader.ps1`, `dev/build_runtime.ps1` or `dev/build_website.ps1` for those concrete build actions. `dev/package_release.ps1` packages existing outputs. Profile investigation actions are documented in [dev/profiles/README.md](../../dev/profiles/README.md). Each file contains the implementation of the named action.
