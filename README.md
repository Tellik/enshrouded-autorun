# ShroudForge

[![Made With Love](https://img.shields.io/badge/Made%20with%20%E2%9D%A4%EF%B8%8F-by%20bonsaibauer-green)](https://github.com/bonsaibauer)
[![Repository](https://img.shields.io/badge/Repository-shroudforge-blue?style=flat&logo=github)](https://github.com/bonsaibauer/shroudforge)
[![License](https://img.shields.io/badge/License-MIT-blue)](https://github.com/bonsaibauer/shroudforge/blob/main/LICENSE)
[![Visitors](https://visitor-badge.laobi.icu/badge?page_id=bonsaibauer.shroudforge)](https://github.com/bonsaibauer/shroudforge)
[![Downloads](https://img.shields.io/github/downloads/bonsaibauer/shroudforge/total)](https://github.com/bonsaibauer/shroudforge/releases)
[![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?style=flat&logo=windows&logoColor=white)](https://github.com/bonsaibauer/shroudforge)


[![Report Bug](https://img.shields.io/badge/Report-Bug-critical?style=flat&logo=github)](https://github.com/bonsaibauer/shroudforge/issues/new?template=bug_report.yml)
[![Request Feature](https://img.shields.io/badge/Request-Feature-green?style=flat&logo=github)](https://github.com/bonsaibauer/shroudforge/issues/new?template=feature_request.yml)
[![Compatibility Problem](https://img.shields.io/badge/Report-Compatibility_Problem-important?style=flat&logo=github)](https://github.com/bonsaibauer/shroudforge/issues/new?template=version_mismatch.yml)

![GitHub Stars](https://img.shields.io/github/stars/bonsaibauer/shroudforge?style=social)
![GitHub Forks](https://img.shields.io/github/forks/bonsaibauer/shroudforge?style=social)

## Welcome!

ShroudForge is the loader that finds, checks, and starts mods for **Enshrouded**. A mod is a separate add-on that changes or adds something in the game. You can use mods made by other players, or create your own. Read [how ShroudForge works](https://bonsaibauer.github.io/shroudforge/en/#workings) for a clear explanation of how the loader and mods fit together.

New to mods? Start with the illustrated guide. It explains everything in simple steps, in English and German:

- [English: install ShroudForge, add mods, and make your first mod](https://bonsaibauer.github.io/shroudforge/en/)
- [Deutsch: ShroudForge einrichten, Mods hinzufügen und den ersten Mod bauen](https://bonsaibauer.github.io/shroudforge/de/)

The guide includes separate instructions for your PC and game server, pictures of the Modloader, a first-mod lesson, and an interactive preview for mod settings.

> [!IMPORTANT]
> 🚧 **ModLoader 1.0 – Work in Progress!**
>
> I'm currently working on restoring all ModLoader features for version 1.0!
>
> ⚠️ **Automatic updates are currently broken!** Please download and install new versions manually:
>
> 📥 **[Download Latest Release](https://github.com/bonsaibauer/shroudforge/releases/latest)**
>
> ✅ **Light Version is now available and working!**
>
> 🎮 **SF Survival Mod – 8 Features Available:**
> - ✅ No Fall Damage
> - ✅ Fast Health Regen
> - ✅ Fast Mana Regen
> - ✅ Infinite Stamina
> - ✅ Infinite Underwater Breath
> - ✅ Infinite Weapon Durability
> - ✅ Infinite Frost Timer
> - ✅ Infinite Mana
>
> 🪂 **Unlimited Flight Mod – 5 Settings Available:**
> - ✅ Unlimited Flying
> - ✅ Turn Speed
> - ✅ Move Speed
> - ✅ Lift Speed
> - ✅ Descend Speed
>
> 🟢 **All features tested and working in Singleplayer!**
>
> 🌐 Multiplayer support and additional features will be added gradually!
>
> ❤️ **Thanks for your patience! More updates are coming soon!**

<p align="center">
  <a href="https://github.com/bonsaibauer/shroudforge/releases/latest"><img src="assets\shroudforge-banner-4-release-1.0.svg" width="100%" alt="ShroudForge 10 Support"></a>
</p>

<p align="center">
  <a href="https://github.com/bonsaibauer/shroudforge/releases/latest"><img src="assets\shroudforge-banner-2-github_v1.svg" width="100%" alt="ShroudForge Download Link"></a>
</p>

<p align="center">
  <a href="https://www.youtube.com/watch?v=V57iZcn-hfo"><img src="assets\shroudforge-banner-1-ohne-strich_v1.svg" width="100%" alt="ShroudForge on YouTube: watch the trailer"></a>
</p>

![ShroudForge Modloader in Enshrouded](assets/modloader-ui-2.png)

## Quickstart: use ShroudForge

1. Close Enshrouded.
2. Download the Latest release:
   
   [![Latest release](https://img.shields.io/github/v/release/bonsaibauer/shroudforge?label=Latest%20Release)](https://github.com/bonsaibauer/shroudforge/releases/latest) [![Downloads](https://img.shields.io/github/downloads/bonsaibauer/shroudforge/total)](https://github.com/bonsaibauer/shroudforge/releases)
   
3. Right-click the downloaded ZIP and choose **Extract All**.
4. Open the Enshrouded folder that contains **enshrouded.exe**.
5. Copy the files from the ZIP into that folder, beside **enshrouded.exe**.
6. Start Enshrouded and press **F9** to open the Modloader.

A common Steam folder is:

~~~text
C:\Program Files (x86)\Steam\steamapps\common\Enshrouded
~~~

The Modloader is the ShroudForge window inside the game. It shows your mods and lets you switch them on or off. Press **F10** to read messages if something does not work.

### Add a mod

Follow the mod creator's instructions. Unpack the mod download, then copy the mod folder into **mods**, beside **enshrouded.exe**. Keep any folder structure and extra files the creator includes.

Open the game, press **F9**, find the mod, and switch it on. Some mods need a game restart. Mods that change game files must prepare those changes before the game starts.

Read the full [PC quickstart](https://bonsaibauer.github.io/shroudforge/en/#play) for screenshots and help with each step.

## Multiplayer server quickstart

> [!NOTE]
> The extra environment setting below is only for the Proton Docker image. It is not needed for the Windows game or the Enshrouded Dedicated Server installed through Steam.

Install ShroudForge and the mod on your PC and on the server.

1. Stop the server and back up your world.
2. Download the [latest ShroudForge release](https://github.com/bonsaibauer/shroudforge/releases/latest).
3. Right-click the ZIP file and choose **Extract All**.
4. Open the server folder containing **enshrouded_server.exe**.
5. Copy the ShroudForge files from the ZIP into that folder, beside **enshrouded_server.exe**.
6. Unpack the mod you want to use, then copy its folder into the server's **mods** folder.
7. If you use the Proton Docker image [`mornedhels/enshrouded-server`](https://github.com/mornedhels/enshrouded-server), add the environment setting below. It lets Proton load ShroudForge's DLLs.

   For an existing `docker run` setup, add this option to its command:

   ```bash
   -e 'WINEDLLOVERRIDES=dbghelp=native,builtin;winmm=native,builtin' \
   ```

   For Docker Compose, add the setting under `environment`, as shown in this complete example:

   ```yaml
   services:
     enshrouded:
       image: mornedhels/enshrouded-server:dev-proton
       restart: unless-stopped
       ports:
         - "15637:15637/udp"
       volumes:
         - ./gamefiles:/opt/enshrouded
       environment:
         - "WINEDLLOVERRIDES=dbghelp=native,builtin;winmm=native,builtin"
   ```

   Keep your existing image, ports, volume, and other settings. After changing the environment, recreate the Docker container. The normal server port is **15637/udp**.

8. Start the server again.

See the [English server guide](https://bonsaibauer.github.io/shroudforge/en/#server) or the [German server guide](https://bonsaibauer.github.io/shroudforge/de/#server) for the full Docker steps, including how to find and inspect an existing container before recreating it.

## Make your first mod

Start with the [mod template](docs/modding/starter-mod/README.md). It gives you the files for a small mod and explains what to change.

The website's [first-mod lesson](https://bonsaibauer.github.io/shroudforge/en/#first) walks through a complete example. When you are ready, try the [mod settings guide](https://bonsaibauer.github.io/shroudforge/en/#manifests): edit an example on the left and see the Modloader-style preview on the right. It also shows how `extended.mod.json` exposes a setting and how Lua reads it with `shroudforge.settings.get`.

To look up functions or Enshrouded game information, open the [searchable API reference](https://bonsaibauer.github.io/shroudforge/en/#api).

## Mods included in the release

The release build validates each mod's manifest, ID, Lua entry point, and Lua syntax. The table below lists the ShroudForge mods included in the release and the targets declared by each mod.

**Target support** means the package can be installed for that process. In multiplayer, the game process that handles an action determines where its effect must run. A checkmark does not by itself confirm that every gameplay effect has been verified end to end on a live server.

| Icon | Mod | Description | Version | Client install | Dedicated server install | Singleplayer | Multiplayer |
| --- | --- | --- | ---: |:---:|:---:|:---:|:---:|
| <img src="mods/sf-survival/icon.svg" width="28" alt="SF Survival"> | **SF Survival** | Eight survival settings for health, mana, stamina, breath, body heat, durability, and fall damage. | 1.0.0 | ✅ | ✅ | ✅ | ✅* |
| <img src="mods/sf-unlimited-flight/icon.svg" width="28" alt="SF Unlimited Flight"> | **SF Unlimited Flight** | Controls original glider stamina and movement values, including upward pitch. | 1.2.0 | ✅ | ✅ | ✅ | ✅* |

\* Multiplayer support depends on installing and enabling the mod in the process that handles the relevant game action. For asset changes such as recipe data or production time, prepare both client and server before starting them. A client-side change alone does not prove that the server accepted or persisted the result.

The source tree currently contains the two release mods listed above. The earlier reference and diagnostic package directories have been removed. See the [architecture guide](docs/sf/Architecture.md) for current source locations and concrete build actions.

## Find or build something in the repository

Find the action you want to change, then open the file with that name. For example, **modules/modloader/save_mod_settings.rs** contains the mod-setting save operation. Its UI lives in the same module. See the [action and file guide](docs/sf/Architecture.md) and the [module owners](modules/README.md).

Run these commands from the repository root:

Initialize the Git submodules before building. For a direct Rust-only build, first run `./dev/prepare_parser.ps1` once to apply the recorded parser integration. The complete `build.ps1` sequence already performs this preparation.

| What you want to do | Command |
| --- | --- |
| Build the Modloader and its UI | `./dev/build_modloader.ps1` |
| Build the native runtime | `./dev/build_runtime.ps1` |
| Build and validate the website | `./dev/build_website.ps1` |
| Package existing build outputs | `./dev/package_release.ps1` |
| Build the complete release without running tests | `./build.ps1 -SkipTests` |

Executable inspection, captures and profile preparation belong to [dev/profiles/](dev/profiles/README.md). The [starter mod](docs/modding/starter-mod/README.md) and [small Lua examples](docs/modding/examples/) belong to the documentation. Build outputs are generated under **build/** and **target/**. Installed loader state is separate from the repository source.

## Development roadmap

This table describes the current state of ShroudForge itself. **Proven** means the workflow has been verified for the stated scope. **Beta** means it is available but still has known reliability or coverage limits. **Open** means implementation or validation remains.

| Area | Current state | Status |
| --- | --- | --- |
| **Release build and mod checks** | Release packaging checks each mod's manifest, ID, Lua entry point, and Lua syntax. | ✅ Proven |
| **Build profiles** | Profiles are maintained per Enshrouded build and target. The current client profile is for build **1076226**. Supporting another build requires reviewing its profile and the operations it enables. | ✅ Proven |
| **Asset changes before launch** | Supported asset edits are prepared before the game starts. Coverage depends on the resource and game build. Client and server may need matching settings. | ✅ Proven |
| **Runtime hooks and gameplay mods** | The included gameplay mods use profile-backed hooks, patches, or asset changes. Their multiplayer effect depends on which process handles the action and has not been verified end to end for every mod. | 🚧 Beta |
| **KFC runtime and ECS** | Native runtime and ECS operations are available, but live world-context detection and some read/write paths still have reliability limits. | 🚧 Beta |
| **World Editor: capture and placement** | Blueprint capture and local placement are available. Dedicated Server placement over Steam P2P has been observed. Saving and persistence still need separate live-session checks. | 🚧 Beta |
| **World Editor: prop undo** | Undo can fail when the native runtime cannot remove a placed prop. Preserve the undo journal and report the operation as incomplete. Do not describe prop undo as proven. | 🕓 Open |
| **World Editor: automatic world targeting** | The client selects the local or Dedicated Server route from the detected world context. Startup, reconnect, and world-switch cases need continued live validation. | 🚧 Beta |
| **EML mod support** | ShroudForge supports documented EML-style package and API flows. Compatibility depends on the APIs and native behavior used by each mod. | 🚧 Beta |
| **Mod discovery and updater** | The Modloader can find and install catalog mods, queue mod updates, and stage ShroudForge updates with backups. Broader recovery scenarios still need validation. | 🚧 Beta |
| **Client-server loader and mod compatibility** | Steam P2P `PING`/`PONG` checks connectivity only. ShroudForge validates mod targets, dependency version ranges, and ShroudForge API requirements locally, but does not compare loader/API versions or installed mod IDs and versions when a client joins a server. Add a compatibility handshake that reports missing or mismatched mods and defines whether each mismatch warns or blocks the session; validate it with client and Dedicated Server builds. | 🕓 Open |
| **More game builds and server profiles** | Add and validate profiles for further Enshrouded builds and targets. | 🕓 Open |
| **Stable ECS-backed mod API** | Make world-context detection and ECS access dependable, then document supported operations based on live verification. | 🕓 Open |

## License and support

ShroudForge is shared under the [MIT License](https://github.com/bonsaibauer/shroudforge/blob/main/LICENSE).

# Buy Me A Coffee

If this project has helped you in any way, do buy me a coffee so I can continue to build more of such projects in the future and share them with the community!

<a href="https://buymeacoffee.com/bonsaibauer" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/default-orange.png" alt="Buy Me A Coffee" height="41" width="174"></a>
