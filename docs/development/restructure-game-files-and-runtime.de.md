# Umsetzung der Struktur für Spieldateien und Spielzugriff

Stand: 17. Oktober 2026. Der freigegebene Gesamtplan ist umgesetzt. Die drei Besitzer liegen unter `modules/game-files/`, `modules/game-runtime/` und `modules/mod-runtime/`. Das Parser-Submodul bleibt direkt unter `kfc-parser/`. Die aktuelle Codeübersicht ist [code-ownership.en.md](code-ownership.en.md).

Dieses Dokument bewahrt den Ausgangsbefund und den freigegebenen [Einzeldateiplan](game-integration-file-plan.csv). Das [Leseinventar](game-integration-read-inventory.csv) beschreibt ausdrücklich den Anfangsstand vor der Umsetzung. Der ältere [Plan für das Mod-Scripting](restructure-mod-scripting.de.md) bleibt als vorhandener Planungsstand erhalten. Parallel erfolgte Änderungen am übrigen Repository, insbesondere der Umzug der Einstiegspunkte nach `modules/entrypoints/`, wurden beibehalten.

## Umgesetzter Umfang

Alle 392 unterschiedlichen Zielpfade aus den 313 ursprünglichen Dateizuordnungen sind vorhanden. Mit neun beim Zusammenführen belegter gemeinsamer Verträge entstandenen Zusatzdateien enthalten die drei Bereiche jetzt 401 eigene Dateien. Die 1.164 Fremddateien des unveränderten JSON-Submoduls zählen nicht dazu. Der folgende Baum zeigt den tatsächlichen Stand einschließlich dieser Zusatzdateien.

Originalbackups besitzen getrennte Aktionen für Lesen, Erfassen, Wiederherstellen und Status. Die Transaktion bleibt zusammen. Die Windows-Bridge gliedert ihre bestehenden Aktionen in zehn Fachbereiche. Native Hookinstallation, Spielthread-Warteschlange, Enginebeobachtung, Profilauswahl, ECS-Zugriffe und Ressourcenaktionen haben eigene Besitzer. Die Modausführung gliedert sich in Sitzungen, Startup, laufendes Spiel, Lua-Anbindung und Reflection.

Die bisherigen Rust-Crate-Namen und öffentlichen Exporte bleiben erhalten. Gleiches gilt für Lua-API-Namen, ABI-Version 12, DLL-Namen, installierte Profilpfade und Datenformate. Reexports und Modulpfade erhalten bestehende Aufrufer, ohne Weiterleitungsfunktionen einzuführen. Loader, Updater und UI verwenden weiterhin dieselben kanonischen Fachaktionen.

Eine bereits im Audit benannte Korrektur betrifft die erzeugte Lua-Definitionsdatei. Build und Website entdecken jetzt ausschließlich Lua-Dateien unter `definitions/`. Die beiden ausführbaren Overridehelfer bleiben über ihre bisherigen `include_str!`-Aufrufer eingebunden und gelangen nicht mehr zusätzlich in das Definitionsbundle. Die 15 gepflegten Definitionsquellen und die 1.015 daraus dokumentierten API-Symbole bleiben inhaltlich erhalten. Diese Eingrenzung der Build-Eingaben ist eine gezielte Korrektur und kein bloßer Dateiumzug.

Wiederholte Aktivierungsbereinigung, optionale DLL-Symbolauflösung, JSON-Abfragen der Bridge sowie lokale Buffer-, IO-, Logging-, Bildkoordinaten- und Ergebnisprojektionen sind beim jeweiligen Besitzer zusammengeführt. Hooks und Patches teilen jetzt ihre vorhandene Signaturprüfung, Nahallokation und relative Sprungberechnung. Ihre unterschiedlichen Abbruch- und Fehlerverträge bleiben erhalten. Die Spielthread-Warteschlange verwendet `deque::pop_front` statt des Verschiebens aller Folgeelemente. Eine Laufzeit- oder Speicherverbesserung wurde nicht gemessen.

### Zusätzliche Dateien gegenüber dem freigegebenen Plan

| Vollständiger Zielpfad | Handlung, Besitzer und tatsächliche Aufrufer | Grund |
| --- | --- | --- |
| `modules/game-runtime/bridge/runtime_records.rs` | Gemeinsame serialisierbare ECS-, Prop- und Grid-Datensätze der Bridge. `windows/native_abi.rs` und `unsupported_platform.rs` verwenden sie | Die bislang doppelt definierten plattformunabhängigen Datensätze haben einen Besitzer. Native Zeiger bleiben im Windows-Teil |
| `modules/game-runtime/native/engine-hooks/allocate_near_code.cpp` | Native Hook-/Patch-Nahallokation. `install_engine_hooks.cpp` und `set_runtime_patch.cpp` verwenden sie | Führt die tatsächlich gleichen Allokationsschritte zusammen |
| `modules/game-runtime/native/engine-hooks/allocate_near_code.h` | Deklaration der gemeinsamen Nahallokation für dieselben beiden Aufrufer | Gemeinsamer C++-Vertrag benötigt einen direkt inkludierten Header |
| `modules/game-runtime/native/engine-hooks/encode_relative_jump.h` | Prüfung und Berechnung relativer Sprünge für Hook- und Patchbesitzer | Entfernt die belegte doppelte rel32-Berechnung ohne neue Zustandsverwaltung |
| `modules/game-runtime/native/engine-hooks/scan_executable_signature.cpp` | Signaturen lesen und ausführbare Bereiche durchsuchen. Hook- und Patchbesitzer rufen den gemeinsamen Ablauf mit ihren bisherigen Grenzen auf | Beide besitzen denselben Scanner. Hooks stoppen weiterhin beim zweiten Treffer, Patches zählen vollständig |
| `modules/game-runtime/native/engine-hooks/scan_executable_signature.h` | Scannervertrag für Hook- und Patchbesitzer | Die beiden Übersetzungseinheiten benötigen dieselbe Deklaration |
| `modules/game-runtime/native/game-thread/game_thread_state.h` | Privater Warteschlangen- und Shutdown-Vertrag für `run_game_thread_operations.cpp` und `install_engine_hooks.cpp` | Trennt gemeinsam benötigten Zustand von der öffentlichen Spielthread-Aktion |
| `modules/game-runtime/native/profiles/identify_game_image.h` | Deklaration der EXE-Identifizierung für `identify_game_image.cpp` und `select_game_profile.cpp` | Die bereits geplante Aufteilung benötigt ihren C++-Vertrag |
| `modules/game-runtime/native/profiles/read_game_profile.h` | Deklaration des Profilformats für `read_game_profile.cpp` und `select_game_profile.cpp` | Die bereits geplante Aufteilung benötigt ihren C++-Vertrag |

## Zuordnung der bestehenden Bereiche

| Ausgangspunkt | Umgesetzter Besitzer | Tatsächliche Aufgabe |
| --- | --- | --- |
| `game-assets/` | `modules/game-files/` | Originaldateien sichern, ihren Zustand anzeigen, einen KFC-Dateisatz transaktional veröffentlichen und Wiederherstellung anfordern |
| `kfc-runtime/` | `modules/game-runtime/` | Den nativen Zugriff auf das laufende Spiel, seine ABI, Rust-Anbindung und geprüfte Spielprofile besitzen |
| Anfangs `execution/`, inzwischen `modules/mod-runtime/` | `modules/mod-runtime/` mit klarer interner Gliederung | Lua-Mods vorbereiten und ausführen, ihre Sitzungen, Callbacks, Konfiguration und paketlokalen DLLs verwalten |

`mod-runtime` ist der inzwischen vorhandene Name des Modausführungsmoduls. Die fachliche interne Gliederung aus dem älteren `mod-scripting`-Vorschlag wird darunter weiterverwendet. `game-files` beschreibt den Dateiablauf genauer als `game-assets`, denn Parser und Assetformatimplementierung liegen im direkten Submodul `kfc-parser/`. `game-runtime` umfasst tatsächlich mehr als KFC-Ressourcen, darunter ECS, Weltkontext, Baueingaben, Voxel und Spielthread-Aufträge.

`mod-management` wäre für Modsuche, Installation, Aktivierung, Einstellungen und Updates verständlich. Diese Handlungen haben bereits ihre Besitzer in `modloader` und `updater`. Der hier untersuchte Bereich besitzt dagegen die Ausführung und Lebensdauer laufender Mods. Dafür ist `mod-runtime` präziser und umfasst auch Mod-DLLs. `mod-moderation` bezeichnet eine inhaltliche Kontrolle und beschreibt keine der hier vorhandenen Aufgaben.

Alle drei Bereiche sind bestehende Features mit verschiedenen Besitzern. Ihre Einordnung unter `modules/` folgt dem vorhandenen Modulmodell. Sie werden weder zu einem allgemeinen Ausführungssystem zusammengelegt noch als neue Wrapper über dem alten Code angelegt. Bestehende öffentliche Rust-Crate-Namen, Lua-API-Namen, DLL-Namen und Exportnamen bleiben bei einem Strukturwechsel erhalten.

Das Muster aus `dev/` und `diagnostics-jobs/` ist die klare Zuordnung nach Handlung. Entwicklungswerkzeuge bleiben unter `dev/`. Die auswählbaren Diagnosejobs bleiben bei `runtime-diagnostics`. Die installierten JSON-Jobdefinitionen unter `runtime/diagnostics/jobs/` sind Nutzerdaten und kein Ziel für Runtime-Quellcode.

## Belegter Ausgangszustand

| Befund | Beleg | Konsequenz |
| --- | --- | --- |
| Ein Originalbackup-Modul besitzt mehrere unabhängige Handlungen | `game-assets/originals.rs` prüft Originale, erfasst sie, stellt sie wieder her und baut zusätzlich die UI-Gesamtübersicht auf | Status, Erfassung und Wiederherstellung fachlich trennen. Identität, Manifest und gemeinsame private Pfadfunktionen beim Originalbackup-Besitzer halten |
| Die Datei-Transaktion bildet bereits eine geschlossene Verantwortung | `game-assets/transaction.rs` verbindet Journal, Recovery, Staging, Commit und Abort | Zusammen behalten und genauer `game_file_transaction.rs` nennen |
| Windows-Bridge-Aktionen liegen weitgehend flach | `kfc-runtime/bridge/windows/mod.rs` verbindet ECS, Ressourcen, Services, Welt, Eingaben, Patches und Provider | Bestehende Aktionen in die belegten Fachbereiche legen. Die native Wirkung bleibt in C++ |
| Zwei Symbolauflösungswege haben unterschiedliche Verträge | `bridge/windows/load_native_provider.rs` prüft die ABI. `bridge/player_input.rs` löst Eingabesymbole separat auf | Die Abweichung sichtbar machen. Eine gemeinsame ABI-Prüfung für Eingaben wäre eine gezielte Verhaltensänderung und gehört nicht stillschweigend in einen Move |
| Der native Dispatcher besitzt drei Aufgaben | `native/execution/dispatcher.cpp` installiert Hooks, verwaltet Spielthread-Aufträge und beobachtet den EntityManager | Die Warteschlange nach `game-thread/`, Hookinstallation zu ihrem Besitzer und Beobachtung zum Enginekontext ziehen |
| Private Header ziehen fachfremde Abhängigkeiten ein | `native/host/provider_state.h` deklariert eine Atomic-Variable, inkludiert dafür aber viele Runtime-Bereiche. `session_internal.h` mischt Cursor-, Voxel- und Gebäudezustand | Direkte Includes beim jeweiligen Verbraucher und zuständige private Verträge verwenden |
| Eine Quelldatei besteht nur aus zwei ABI-Weiterleitungen | `native/patches/patch_exports.cpp` delegiert beide Aufrufe an `PatchRuntime` | In die kanonische Patchimplementierung integrieren und den separaten CMake-Quelleneintrag entfernen |
| Quellpfade sind zugleich Build- und Integrationspfade | Cargo, Root-CMake, Runtime-CMake, Bootstrap-Projekt, Profilerstellung, Paketierung und Website referenzieren diese Bereiche | Den Umzug als geschlossene Pfadänderung durchführen. Keine zweite Registrierungs- oder Profilliste ergänzen |

## Freigegebener Dateiplan und tatsächlicher Dateibaum

Die [gesamte Dateizuordnung](game-integration-file-plan.csv) enthält genau eine Zeile für jede der 313 eigenen Dateien des Anfangsinventars. Sie nennt ursprünglichen und zuletzt beobachteten Pfad, vollständigen repositoryrelativen Zielpfad, Handlung, Besitzer, Verantwortung, tatsächliche Aufrufer, Begründung, mögliche Einsparung, Verhaltensgrenzen und gegebenenfalls zusätzliche Zielaktionen.

Die 173 ursprünglichen Execution-Zuordnungen wurden anhand vollständiger frischer Lektüre aus der vorhandenen [Mod-Scripting-Dateizuordnung](mod-scripting-file-plan.csv) übernommen und auf den inzwischen vorhandenen Modulnamen `mod-runtime` angepasst. Die ältere Datei bleibt als vorhandener Arbeitsstand unberührt. Für diesen Gesamtvorschlag ist die neue 313-Zeilen-Dateizuordnung maßgeblich. Sie führt keinen zweiten Produktmechanismus ein.

Ein technischer Einstieg wie `lib.rs`, `mod.rs`, `Cargo.toml` oder `CMakeLists.txt` behält seinen Namen. Ein bereits präziser Aktionsname bleibt ebenfalls erhalten. Zusätzliche Rust-Modulstarts deklarieren und exportieren ihre echten Aktionen. Es entstehen keine bloßen Weiterleitungsfunktionen.

<!-- BEGIN GENERATED TARGET TREE -->
```text
modules/
  game-files/
    originals/
      capture_original_game_files.rs
      mod.rs
      read_original_backups.rs
      restore_original_game_files.rs
    ui/
      show-game-file-backups.tsx
    asset-preparation.schema.json
    Cargo.toml
    documents.rs
    game_file_transaction.rs
    lib.rs
    parser-status.schema.json
    read_game_file_status.rs
    request_game_file_restore.rs
  game-runtime/
    bridge/
      windows/
        building/
          apply_entity_placement.rs
          destroy_entity.rs
          finish_building.rs
          format_operation_error.rs
          mod.rs
          read_cursor.rs
          spawn_entity.rs
        ecs/
          check_component_write_access.rs
          check_ecs_write_access.rs
          configure_ecs.rs
          describe_component.rs
          mod.rs
          query_entities.rs
          query_entities_in_bounds.rs
          read_component.rs
          read_component_registration.rs
          read_component_registry.rs
          read_components.rs
          read_ecs_readiness.rs
          read_entity_identity.rs
          read_entity_template_uuid.rs
          resolve_component_type.rs
          resolve_entity.rs
          update_component.rs
        functions/
          mod.rs
          read_function_catalog.rs
          read_function_code.rs
        named_services/
          mod.rs
          read_service.rs
          update_service.rs
        patches/
          check_runtime_patch.rs
          mod.rs
          set_runtime_patch_enabled.rs
        props/
          mod.rs
          query_props.rs
          query_props_in_bounds.rs
          read_entity_transform.rs
          read_prop_query_readiness.rs
          register_prop_recipes.rs
          set_entity_scale.rs
        provider/
          load_native_provider.rs
          mod.rs
          read_native_catalog.rs
          read_provider_status.rs
        resources/
          check_resource_exists.rs
          check_resource_write_access.rs
          find_resources_by_type.rs
          mod.rs
          read_resource.rs
          read_resource_scalars.rs
          read_resource_status.rs
          resolve_item_resource.rs
          update_resource_scalar.rs
        voxels/
          mod.rs
          read_grid_spec.rs
          read_voxels.rs
          write_voxels.rs
        world_context/
          check_entity_context.rs
          check_world_context.rs
          check_world_operation.rs
          mod.rs
          read_world_context_kind.rs
          read_world_session.rs
        mod.rs
        native_abi.rs
      building_input.rs
      Cargo.toml
      lib.rs
      README.md
      runtime_records.rs
      unsupported_platform.rs
    compatibility/
      Cargo.toml
      lib.rs
      README.md
    native/
      abi/
        runtime_api.h
      building/
        building_operations.h
        building_request_state.h
        finish_building.cpp
        place_building_prop.cpp
        register_prop_recipes.cpp
        remove_building_prop.cpp
        run_building_requests.cpp
        spawn_building_prop.cpp
      code-inventory/
        export_function_inspection.cpp
        inspect_game_functions.h
      ecs/
        registry/
          component_discovery_state.h
          find_component_registry.h
          read_component_registry.h
          refresh_component_registry.cpp
          sample_component_layouts.cpp
        bind_world_entities.cpp
        building_prop_query_state.h
        component_access.h
        component_operation.h
        describe_ecs_runtime.cpp
        ecs_session_state.h
        entity_session.h
        entity_storage.h
        query_building_props.cpp
        query_entities.cpp
        query_entities_in_bounds.cpp
        read_building_prop.cpp
        read_building_prop.h
        read_entity_components.cpp
        read_entity_membership.cpp
        resolve_component_storage.cpp
        resolve_entity_handle.cpp
        set_building_prop_scale.cpp
        snapshot_entity_ids.h
        write_entity_components.cpp
      engine-context/
        configure_engine_layout.cpp
        engine_context_state.h
        engine_owners.h
        hold_resource_lifetime.cpp
        hold_resource_lifetime.h
        lock_scene_access.cpp
        lock_scene_access.h
        observe_entity_manager.cpp
        observe_entity_manager.h
        resolve_engine_owners.cpp
        track_world_context.cpp
        world_context.h
        world_context_state.h
      engine-hooks/
        allocate_near_code.cpp
        allocate_near_code.h
        encode_relative_jump.h
        install_engine_hooks.cpp
        install_engine_hooks.h
        scan_executable_signature.cpp
        scan_executable_signature.h
        write_executable_bytes.cpp
        write_executable_bytes.h
      game-thread/
        game_thread_state.h
        run_game_thread_operations.cpp
        run_game_thread_operations.h
      host/
        describe_runtime.cpp
        manage_runtime_lifecycle.cpp
        runtime_lifecycle_state.h
      json/
        [Git-Submodul]
      loaded-resources/
        compare_exchange_resource_scalar.cpp
        find_loaded_resources_by_type.cpp
        initialize_resource_access.cpp
        loaded_resource_access.h
        loaded_resource_state.h
        parse_resource_layout.cpp
        read_loaded_resource.cpp
        read_resource_scalars.cpp
        resolve_item_resource.cpp
        resolve_loaded_resource.cpp
      named-services/
        named_service_contract.h
        read_and_update_named_service.cpp
        resolve_named_service.cpp
      patches/
        set_runtime_patch.cpp
        set_runtime_patch.h
      player-input/
        apply_building_input.cpp
        apply_building_input.h
        sample_world_cursor.cpp
        sample_world_cursor.h
        world_cursor_state.h
      profiles/
        game_profile.h
        identify_game_image.cpp
        identify_game_image.h
        read_game_profile.cpp
        read_game_profile.h
        select_game_profile.cpp
      voxels/
        read_and_write_voxels.cpp
        read_voxel_grid.cpp
        voxel_operations.h
        voxel_request.h
      CMakeLists.txt
    profiles/
      enshrouded/
        client/
          1076226.json
          25966634.json
          25988036.json
        server/
          1024233.json
      README.md
    tests/
      building_input_test.cpp
      building_prop_query_test.cpp
      ecs_access_test.cpp
      loaded_resource_access_test.cpp
      named_service_access_test.cpp
      world_context_session_test.cpp
    .gitignore
  mod-runtime/
    ingame/
      activate_runtime_mod.rs
      lifecycle_tests.rs
      load_native_plugins.rs
      load_runtime_callbacks.rs
      mod.rs
      plan_runtime_mods.rs
      process_runtime_reload_request.rs
      publish_runtime_status.rs
      refresh_runtime_configuration.rs
      reload_runtime_packages.rs
      restore_runtime_packages.rs
      start_ingame_runtime.rs
      stop_ingame_runtime.rs
      unload_runtime_mod.rs
      unload_runtime_packages.rs
      update_ingame_runtime.rs
      update_mod_callbacks.rs
    lua/
      bindings/
        buffer/
          byte_order.rs
          errors.rs
          manage_buffer.rs
          mod.rs
          read_buffer.rs
          write_buffer.rs
        game/
          assets/
            access_file_content.rs
            access_file_resources.rs
            content.rs
            mod.rs
            resource.rs
          content_id_casts.rs
          guid.rs
          mod.rs
          resolve_type.rs
          types.rs
        image/
          image.rs
          image_format.rs
          mod.rs
          pixel_format.rs
        io/
          manage_exports.rs
          mod.rs
          path_names.rs
          read_mod_files.rs
        extend_table_operations.rs
        hash_bytes.rs
        integer_operations.rs
        mod.rs
        register_functions.rs
        write_mod_log.rs
      definitions/
        export_lua_definitions.rs
        generate_game_type_definitions.rs
        mod.rs
      eml/
        v1/
          definitions/
            assets/
              asset_manager_api.lua
              content_type.lua
              resource_type.lua
            buffer_api.lua
            game_api.lua
            globals.lua
            guid_api.lua
            hasher_api.lua
            image_api.lua
            integer_api.lua
            io_api.lua
            loader_api.lua
            type_registry_api.lua
          create_loader_api.rs
          mod.rs
        mod.rs
      ingame/
        ecs/
          attributes/
            check_attribute_component.rs
            encode_attribute_scalar.rs
            evaluate_attributes.rs
            expect_root_values.rs
            format_attribute_root.rs
            mod.rs
            read_attribute.rs
            read_attribute_catalog.rs
            read_attribute_descriptor.rs
            read_attribute_storage_range.rs
            read_attributes.rs
            read_entity_attribute_storage.rs
            recalculate_bound_component.rs
            recalculate_component_update.rs
            register_attribute_api.rs
            resolve_attribute_storage.rs
            tests.rs
            update_attribute.rs
            validate_attribute_bits.rs
            write_attribute_storage.rs
          access_component_fields.rs
          access_template_component.rs
          mod.rs
          query_entities.rs
          read_component.rs
          read_component_catalog.rs
          read_component_registry.rs
          resolve_entity.rs
          write_component.rs
        functions/
          bind_profile_modifier.rs
          function_catalog_tests.rs
          mod.rs
          read_function_catalog.rs
        overrides/
          claim_runtime_storage.rs
          create_override_targets.lua
          mod.rs
          track_runtime_overrides.lua
          track_runtime_overrides.rs
        player_input/
          mod.rs
          read_building_cursor.rs
          submit_building_input.rs
        resources/
          index_loaded_resources.rs
          index_scalar_tests.rs
          mod.rs
          read_loaded_resources.rs
          write_loaded_field.rs
        world/
          access_prop_transform.rs
          change_building_entities.rs
          mod.rs
          query_props.rs
          read_world_context.rs
          read_write_voxels.rs
          register_prop_recipes.rs
        access_named_service.rs
        check_ingame_access.rs
        mod.rs
        prove_buffer_transform.rs
        set_profile_patch_enabled.rs
      resource_values/
        convert_resource_values.rs
        mapped_values.rs
        mod.rs
        owned_values.rs
        validate_resource_values.rs
        value_path.rs
      shroudforge/
        v1/
          definitions/
            mod_api.lua
            runtime_api.lua
          create_mod_api.rs
          dispatch_mod_ui_actions.rs
          read_runtime_operations.rs
          runtime-operations.json
          runtime-operations.schema.json
          set_mod_settings.rs
      values/
        collect_inline_leaves.rs
        create_default_value.rs
        decode_owned_value.rs
        encode_owned_value.rs
        expect_inline_fields.rs
        inline_field.rs
        lua_value_result.rs
        mod.rs
        normalize_field.rs
        parse_field_path.rs
        register_value_api.rs
        reject_external_overlap.rs
        resolve_cached_inline_field.rs
        resolve_inline_field.rs
        resolve_loaded_field.rs
        resolve_owned_type.rs
        resolve_resource_field.rs
        tests.rs
        validate_owned_type.rs
        validate_runtime_scalar.rs
      arguments.rs
      cast_values.rs
      convert_json.rs
      create_sandbox.rs
      errors.rs
      format_type_name.rs
      mod.rs
      mod_runner.rs
      parse_require_path.rs
      readonly_collections.rs
      require_mod.rs
    reflection/
      build_attribute_catalog.rs
      evaluate_attribute_program.rs
      load_type_registry.rs
      mod.rs
      resolve_system_dependencies.rs
      traverse_reflected_types.rs
    session/
      asset_resources.rs
      configure_component_contract.rs
      create_mod_session.rs
      loaded_resource_indexes.rs
      loaded_resource_indexes_tests.rs
      loaded_resources.rs
      mod.rs
      native_plugins.rs
      read_native_plugin_declaration.rs
      record_runtime_state.rs
    startup/
      cache/
        mod.rs
        store_file_state.rs
        track_input_files.rs
      commit_asset_changes.rs
      manage_asset_baseline.rs
      mod.rs
      open_game_assets.rs
      run_startup_mods.rs
    tests/
      compile_bundled_mods.rs
      override_recovery.lua
      override_recovery.rs
      survival_lifecycle.lua
      survival_lifecycle.rs
      unlimited_flight_lifecycle.lua
      unlimited_flight_lifecycle.rs
    build.rs
    Cargo.toml
    documents.rs
    lib.rs
    logging.rs
    README.md
    type_aliases.rs
```
<!-- END GENERATED TARGET TREE -->

Die 392 geplanten Ziele sind vollständig umgesetzt. Bei einer Zusammenführung zeigen mehrere Originaldateien auf dieselbe kanonische Implementierung. Die CSV benennt den Grund. Die neun zusätzlichen Verträge und Implementierungen sind oben einzeln begründet und im tatsächlichen Dateibaum enthalten. Das JSON-Submodul bleibt als bestehende Abhängigkeit zusammen und wird nicht in neue Kopien aufgeteilt.

## Die Aufteilung von originals.rs

| Ziel | Bestehende Handlung | Aufrufer und Grund |
| --- | --- | --- |
| `modules/game-files/originals/mod.rs` | Originalbackup-Identität, Manifestzugriff, Hash- und Pfadfunktionen sowie Moduldeklarationen | Status, Erfassung und Restore verwenden dieselbe Build- und Dateipaaridentität. Die privaten Grundlagen bleiben hier kanonisch |
| `modules/game-files/originals/read_original_backups.rs` | `originals_status` | Loader, UI und Restore prüfen, ob ein passendes Originalbackup verfügbar ist. Die leichte Statusabfrage bleibt von vollständigem Hashen getrennt |
| `modules/game-files/originals/capture_original_game_files.rs` | `ensure_originals`, `capture_verified_originals` und der vorhandene Erfassungsablauf | Automatischer Loaderstart und ausdrücklich bestätigte UI-Erfassung haben unterschiedliche Zulassungsbedingungen, nutzen aber dieselbe Erfassung |
| `modules/game-files/originals/restore_original_game_files.rs` | `restore_originals` und sein Rollback | Der Updater-Worker führt nach Spielende den bestehenden Restore aus. EXE-Hash, Dateihashes, Sicherheitskopie und Rücknahme bleiben zusammen |
| `modules/game-files/read_game_file_status.rs` | `management_status` | Die UI benötigt zusätzlich Arbeitsbaseline, offene Transaktion, Updatebackups und Restore-Status. Diese Gesamtprojektion gehört nicht in die Prüfung unveränderlicher Originale |

Der Originalbackup-Pfad ist nicht die `.bak`-Arbeitsbaseline aus dem Startup-Ablauf. Beide haben andere Zulassungsbedingungen und Lebensdauern. Der Vorschlag vereinigt sie nicht ohne einen eigenen Gleichheitsnachweis.

## Ursprünglich belegte Möglichkeiten für weniger Doppelungen

Diese Tabelle bewahrt die Befunde mit ihren ursprünglichen Quellpfaden. Die tatsächlich umgesetzten Zusammenführungen sind im Umsetzungsumfang am Dokumentanfang beschrieben.

| Stelle | Vorgeschlagene Vereinfachung | Grenze |
| --- | --- | --- |
| `native/patches/patch_exports.cpp` | Die beiden externen ABI-Einstiege beim bestehenden Patchbesitzer implementieren | Exportnamen, ABI und Sonderbehandlung von `runtime.gameplay.patch` erhalten |
| `native/host/provider_state.h` | Nur den tatsächlich geteilten Providerzustand deklarieren. Konkrete Includes an die verwendende Übersetzungseinheit legen | Keine neue Zustandskopie und keine veränderte Initialisierungsreihenfolge |
| `native/engine-context/session_internal.h` | Die doppelte Deklaration von `matching_prop_handles` entfernen und bestehende private Fachverträge verwenden | Der gemeinsame Sitzungszustand bleibt einmalig. Keine Änderung von Sperren, atomaren Feldern oder Lebensdauer |
| `native/execution/dispatcher.cpp` und `native/patches/patch_runtime.cpp` | Die ähnlichen Signatur-, rel32- und Nahallokationsabläufe beim konkreten Hook-/Patchbesitzer vergleichen und gegebenenfalls zusammenführen | Unterschiedliche Instruction Guards, Trampoline, Seitenschutz und Fehlerpfade vorher erhalten. Kein pauschales Speicherzugriffsmodul |
| `bridge/windows/load_native_provider.rs` | Die wiederholte optionale Symbolauflösung lokal mit einer zum vorhandenen Pflichtsymbol-Makro passenden Variante ausdrücken | Pflichtsymbole, optionale Erweiterungen, ABI-Prüfung, Wiederholung bei noch fehlender DLL und gespeicherte Fehler unterscheiden |
| `execution/lua/bindings/io.rs` | Paketgebundenes Lesen für `read` und `read_to_string` lokal gemeinsam nutzen | Die vorhandene Mod-Dateisystemgrenze und unterschiedliche Byte-/Textfehler bleiben erhalten |
| `execution/lua/bindings/image/value.rs` | Dieselbe Koordinatenprüfung für `set_pixel` und `set_pixel_packed` lokal einmal implementieren | Pixelformat, Indexberechnung und Fehlervertrag bleiben erhalten |
| ECS- und Prop-Ergebnisse in der Lua-Anbindung | Wiederholten Aufbau identischer Ergebnistabellen beim Besitzer bündeln | Abfragearten, Grenzen, nil-/Fehlerergebnisse und native Freigaben bleiben getrennt |
| Modaktivierung und Operationskatalog | Die im vorhandenen Scripting-Plan belegten Wiederholungen kanonisch umsetzen | Cleanup, Messungen, Fehlerdarstellung, Validierung und unveränderlicher Katalogvertrag erhalten |

Die vorhandenen Einzelaktionen für Attribute, Inlinewerte und Native-CAS sind kein Anlass für ein allgemeines Jobsystem. Ähnlich aussehende Speicherleser mit unterschiedlichen RPM-, SEH-, Sitzungs- oder Gate-Verträgen werden nicht allein zur Zeileneinsparung vereinigt. Laufzeit-, Speicher- oder Prozentverbesserungen wurden nicht gemessen.

## Nachgeführte Aufrufer, Build und Paketierung

Die folgende Tabelle hält die Integrationsänderungen fest. Die Einstiegspunkte liegen inzwischen unter `modules/entrypoints/`. Ihre Manifeste und nativen Includes berücksichtigen diesen parallel erfolgten Umzug.

| Verbraucher | Umgesetzte Anpassung |
| --- | --- |
| Root-`Cargo.toml` | Die vier bestehenden Mitglieder auf `modules/game-files`, `modules/game-runtime/bridge`, `modules/game-runtime/compatibility` und `modules/mod-runtime` richten. Workspace-Abhängigkeit `shroudforge-game-assets` auf `modules/game-files` setzen |
| `modules/entrypoints/loader/Cargo.toml` | Direkte Pfade sind `../../game-files`, `../../game-runtime/compatibility` und `../../mod-runtime`. Der Parser liegt relativ unter `../../../kfc-parser/crates/kfc`. Bestehende Crate-Namen bleiben erhalten |
| `modules/modloader/Cargo.toml` | Direkte Spieldateiabhängigkeit auf `../game-files` richten. Über Workspace geerbte UI-/Updater-Abhängigkeiten folgen dem Rootmanifest |
| `modules/mod-runtime/Cargo.toml` | Bridge über `../game-runtime/bridge`, Compatibility über `../game-runtime/compatibility`, Spieldateien über `../game-files`, Parser über `../../kfc-parser/crates/kfc` und Geschwistermodule über `../` erreichen |
| Bridge-/Compatibility-Manifeste | Parser über `../../../kfc-parser/crates/kfc` erreichen. Bridge-Diagnostikpfad auf `../../runtime-diagnostics` richten |
| Root-`CMakeLists.txt` | `add_subdirectory(modules/game-runtime/native)` verwenden. Der Binärtargetname bleibt `kfc-runtime` |
| Native-`CMakeLists.txt` | Alle umbenannten eigenen Quellen und Includes aktualisieren. `../profiles/*.json` bleibt gültig. Entwicklungspfad wird `../../../dev/profiles`, Transportquelle `../../transport/native/steam_network.cpp` |
| `.gitmodules` und Git-Submodulverwaltung | Den vorhandenen nlohmann-JSON-Gitlink nach `modules/game-runtime/native/json` bewegen. Submodulidentität, Commit und Inhalt erhalten. Den Unterordner nicht mit gewöhnlichen Dateikopien nachbauen |
| `modules/entrypoints/windows-bootstrap/runtime_bridge.cpp` und `ShroudForge.Bootstrap.vcxproj` | ABI-Header über `../../game-runtime/native/abi/runtime_api.h` und JSON über `../../game-runtime/native/json` erreichen. Die tatsächlich geladene Datei bleibt `shroudforge/kfc-runtime.dll` |
| `dev/profiles/cli/read_pe_image.h` und `dev/profiles/checks/verify_component_registry.cpp` | Die belegten direkten Runtime-Includes und umbenannten Header gemeinsam nachführen |
| `modules/transport/native/steam_network.cpp` | Den ABI-Include auf `abi/runtime_api.h` richten. Transport bleibt Besitzer seiner Netzwerklogik |
| `dev/profiles/verify/audit_live_component_registry.ps1`, Profilwerkzeuge und aktive Profilguides | Quellprofilpfade aktualisieren. Geprüfte Profile bleiben beim Runtime-Modul, Drafts, lokale Captures und Entwicklungswerkzeuge bei `dev/profiles/` |
| `dev/build/package_release.ps1` | Quellprofile aus `modules/game-runtime/profiles` lesen. ZIP-Pfade, `kfc-runtime.dll`, `shroudforge/runtime/profiles/` und Managed-Paths erhalten |
| `dev/build/build_kfc_runtime.ps1` | Bestehenden Root-CMake-Aufruf und `build/native-runtime/bin/kfc-runtime.dll` erhalten. Der neue Quellort verlangt keinen neuen Buildtreiber |
| `modules/loader-config/ui/show-loader-settings.tsx` | Den Import auf `../../game-files/ui/show-game-file-backups` richten |
| `modules/modloader/ui/tsconfig.json` | Spieldatei-UI über `../../game-files/ui/*.tsx` einbeziehen. Eigene Imports der verschobenen UI zu Transport, Übersetzungen, Card und Snapshot korrigieren |
| Rust-Moduldeklarationen, Includes und Testfixtures | Interne Pfade und zusätzliche Fachaktionen gemeinsam aktualisieren. Öffentliche `originals`, `transaction` und vorhandene Funktionsnamen können durch Reexports erhalten bleiben |
| Website-Generator und Quelllinks | Lua unter `modules/mod-runtime/lua` entdecken, Definitionsdateien aus `definitions/` einbeziehen und ausführbare Lua-Hilfsskripte bei ihrem Besitzer lassen. Dieselbe Entdeckungsgrenze gilt im Definitionsbundle. Symbolquelle und deterministische Reihenfolge bleiben erhalten |
| Aktive READMEs und `code-ownership.en.md` | Erst nach dem tatsächlichen Umzug neue Besitz- und Aktionspfade dokumentieren. Historische Evidenz nicht pauschal umschreiben |

### Korrektur zum vorhandenen Scripting-Plan

Die fünf Beispieltargets wurden außerhalb dieses Auftrags bereits auf die beiden vorhandenen Quellen unter `dev/profiles/inspect/` korrigiert. Im inzwischen verschobenen `modules/mod-runtime/Cargo.toml` ist auch die zusätzliche relative Ebene bereits als `../../dev/profiles/inspect/` berücksichtigt. Der ältere Hinweis im Scripting-Vorschlag auf fehlende Beispielquellen ist damit überholt. Diese fremden Änderungen wurden erhalten.

Die fachliche interne Execution-Zuordnung folgt dem vorhandenen Einzelplan unter dem aktuellen Präfix `modules/mod-runtime/`. Seine Aussagen, dass Assettransaktionen und Profile weiterhin unter den bisherigen Rootordnern liegen, sind durch die tatsächlichen Besitzer `modules/game-files` und `modules/game-runtime` ersetzt. Das Profilfixture zeigt von `modules/mod-runtime/reflection/` aus auf `../../../modules/game-runtime/profiles/enshrouded/client/25966634.json`. Der Profilinhalt ist unverändert.

## Verhalten, das erhalten bleiben muss

Die Originaldateien bleiben pro Spielbuild unveränderlich. Die UI führt weiterhin ihre leichte Metadatenprüfung aus. Vollständige Hashprüfung geschieht im Erfassungs- und Wiederherstellungsablauf. Originalbackups, Startup-Baseline und Transaktionsjournal bleiben verschiedene Datenbestände.

Bei der Asset-Transaktion bleiben Dateinamenzulassung, Journalformat, Synchronisierung, Indexveröffentlichung als letzter Schritt, Recovery und Fehlerreihenfolge bestehen. Der Updater wartet weiterhin auf das Spielende, bevor er Originale wiederherstellt. Die bestehenden Speicherresolver bestimmen unverändert konfigurierbare Pfade.

Native Zugriffe behalten Profilzulassung, EXE-Identität, ABI-Version, optionale Erweiterungen, aktuelle Welt- und Szenenprüfung, Entity-Generation und Schreibfreigaben. Vollständige Snapshots, Masken, CAS, Readback und die Unterscheidung zwischen Busy, Timeout und ungewissem Schreibausgang bleiben erhalten. Hookinstallation und Spielthread-Verbrauch werden beim Aufteilen nicht nebenbei umgeordnet.

Lua behält EML v1 und ShroudForge v1, Capability-Prüfungen, Sandbox, getrennte Modumgebungen, Callbackintervalle, Require-Caches, Override-Claims, wiederholbares Cleanup und Reload-Reihenfolge. Die ausführbaren Lua-Hilfsskripte im bisherigen Definitionsbundle wurden durch die oben beschriebene Eingrenzung der Build-Eingaben gesondert korrigiert. Weitere im Audit benannte Unterschiede zwischen API-Deklaration und Implementierung wurden nicht zu stillschweigenden API-Änderungen.

## Freigegebene Umsetzungsfolge

1. Den dann aktuellen Arbeitsbaum prüfen und vorhandene Änderungen erneut abgleichen. Die bestehenden Crates und das native Modul mit ihren Submodulgrenzen verschieben. Alle oben genannten Verbraucher in derselben Änderung nachführen.
2. Namen und direkte Modulzuordnungen korrigieren. Build, Website-Eingaben, Profile, Includes und Paketpfade anhand der vorhandenen Einstiegspunkte prüfen.
3. Die konkret begründeten Fachaufteilungen einzeln umsetzen. Erst Originalbackups und Dispatcher, dann die im Scripting-Plan benannten Sitzungs- und Lua-Grenzen bearbeiten. Jeder Ablauf bleibt einmalig.
4. Nachgewiesene Doppelungen beim zuständigen Besitzer entfernen. Abweichende Fehler- und Sicherheitsverträge vor jeder Zusammenführung vergleichen.
5. Die passenden vorhandenen Prüfungen ausführen. Für eine Implementierung sind Cargo-/UI-Build, native Runtime und Bootstrap, vorhandene betroffene Tests, Website und bei Paketpfadänderungen Release-Paketierung relevant. Eine Leistungsbehauptung verlangt eine reale Messung.

## Leseumfang und Prüfung der ursprünglichen Planungsphase

Die folgenden Zähler und Fingerprints beziehen sich auf die vollständige Untersuchung vor der Umsetzung. Sie belegen die Herkunft der Dateizuordnung. Die Implementierungsprüfungen werden getrennt dokumentiert.

<!-- BEGIN GENERATED AUDIT RESULTS -->
Die Inventur wurde vor dem Lesen für den vereinbarten Bereich erstellt und mit dem Dateisystem abgeglichen. Die geltende Root-AGENTS.md und der Coding-Skill wurden gelesen. In den betroffenen Verzeichnissen existierten keine weiteren AGENTS.md. Die Hauptverantwortung und die fachliche Zusammenführung liegen beim Hauptagenten.

| Bereich des Anfangsinventars | Eigene Dateien | Vollständig gelesen |
| --- | ---: | ---: |
| game-assets | 9 | 9 |
| kfc-runtime ohne JSON-Submodul | 131 | 131 |
| execution, inzwischen modules/mod-runtime | 173 | 173 |
| **Gesamt** | **313** | **313** |

**295 von 295 eigenen Quelldateien wurden vollständig inhaltlich gelesen, also 100 %.** Die weiteren 18 eigenen Dateien sind Manifeste, Schemas, Profile, READMEs, CMake und Ignore-Konfiguration. Auch sie wurden vollständig gelesen. Die Arbeit wurde auf abgegrenzte Teilprüfungen verteilt. Ein Hashabgleich wurde nicht als Ersatz für Inhaltslektüre gezählt.

Das [Leseinventar](game-integration-read-inventory.csv) erfasst alle 1.477 Dateien des Anfangsumfangs einzeln. Die 1.164 Dateien des nlohmann-JSON-Submoduls sind als Fremdcode ausdrücklich ausgeschlossen und nicht als vollständig gelesen gezählt. Das direkte kfc-parser-Submodul, übrige Module, lokale Captures, Installationen und erzeugte Buildausgaben gehören nicht zum vollständigen Inhaltsaudit.

Die 173 Execution-Dateien wurden während des Auftrags außerhalb dieser Prüfung nach modules/mod-runtime verschoben. Sechs Dateien änderten zusätzlich ihren Inhalt: Cargo.toml, README.md, reflection/functions.rs sowie tests/bundled_lua.rs, tests/flight.rs und tests/survival.rs. Alle sechs wurden danach erneut vollständig gelesen. Das Inventar trennt Lesefingerprint, beobachteten aktuellen Pfad und aktuellen Fingerprint. Es behauptet keinen unveränderten Arbeitsbaum.

| Ausgeführte Prüfung | Ergebnis |
| --- | --- |
| Anfangsinventar gegen Einzeldateiplan | 313 eindeutige Quellen, keine fehlende oder zusätzliche eigene Datei |
| Leseabdeckung | 313 eigene Dateien vollständig gelesen, davon 295 Quelldateien |
| Aktuelle Pfade und SHA-256 | Alle 313 Quellen wiedergefunden. Externe Umzüge und Inhaltsänderungen einzeln ausgewiesen |
| Zielzuordnung | Alle Ziele liegen bei einem der drei benannten Besitzer. Beabsichtigte Zusammenführungen werden unten genannt |
| CSV-Rücklesen und Vollständigkeit | Beide UTF-8-Dateien geparst und gegen Inventar und Pflichtfelder abgeglichen |
| Zielbaum | Aus denselben Primärzielen und zusätzlichen Fachaktionen wie die CSV erzeugt |
| Markdown-Verweise | Die verlinkten lokalen Planungsdateien sind vorhanden |
| Änderungen der ursprünglichen Planungsphase | Ausschließlich dieses Dokument und zwei neue CSV-Dateien geschrieben. Die Implementierung folgte erst nach Freigabe |

Beabsichtigte gemeinsame Primärziele:

- `modules/game-runtime/native/patches/set_runtime_patch.cpp` aus `kfc-runtime/native/patches/patch_exports.cpp`, `kfc-runtime/native/patches/patch_runtime.cpp`.
- `modules/mod-runtime/lua/bindings/game/resolve_type.rs` aus `execution/lua/bindings/registry.rs`, `execution/lua/bindings/type_lookup.rs`.

Zusätzlich vollständig gelesene Integrationsdateien umfassen Root-Cargo, Root-CMake, .gitmodules, .gitignore, die aktuelle Codeübersicht, den vorhandenen Scripting-Vorschlag, dev/build/build_kfc_runtime.ps1, dev/build/build_modloader.ps1, dev/build/package_release.ps1, dev/tests/run_workspace_tests.ps1, die Bootstrap-Bridge und ihr vcxproj, modules/modloader/ui/tsconfig.json, die beiden Hostaktionen für Erfassung und Restore, die Loaderaktionen apply_asset_mods und capture_original_game_files, die Updateraktionen restore_gamefiles und capture_original_game_files, die Dokumentleser und -schreiber samt Pfadauflösung sowie README, Katalog und Erfassung der Diagnosejobs. Die übrigen externen Verbraucher in der Integrationstabelle wurden gezielt nach Aufrufen, Pfaden und Registrierungen durchsucht. Suchtreffer gelten nicht als vollständige Lektüre dieser externen Dateien.

In der ursprünglichen Planungsphase wurden keine Tests, Produktbuilds, Website-Builds, Paketierung oder Live-Spielprüfungen gestartet. Diese Aussage gilt ausschließlich für die Planungsphase. Die nach Freigabe ausgeführten Implementierungsprüfungen stehen im anschließenden Abschnitt.

<!-- END GENERATED AUDIT RESULTS -->

Die drei Planungsdateien gehören zur Entwicklungsdokumentation. Dieses Dokument erklärt Umsetzung und Zuordnung, die CSV besitzt den freigegebenen Einzeldateiplan, das Leseinventar weist untersuchte und ausgeschlossene Dateien des Anfangsstands aus. Sie enthalten keine zweite Produktregistrierung. Persönliche Einstellungen, Installationen und lokale Captures wurden bei der Umsetzung nicht verändert.

## Prüfung der Umsetzung

<!-- BEGIN IMPLEMENTATION CHECKS -->
| Ausgeführte Prüfung | Ergebnis |
| --- | --- |
| Einzeldateiplan gegen tatsächliche Dateien | Alle 392 unterschiedlichen Planziele aus 313 Quellen vorhanden. Tatsächlicher Baum mit 401 eigenen Dateien geprüft, davon 13 in `game-files`, 181 in `game-runtime` und 207 in `mod-runtime`. Neun Zusatzdateien oben begründet |
| Aktive Pfade und Registrierungen | Lokale Pfade aller 17 Workspace-Manifeste, Rust-Einbindungen, relative C++-Includes und Bootstrap-Projektpfade vorhanden. Keine veralteten realen Quellverweise auf die alten Rootordner gefunden. Jede der 52 eigenen Produktions-CPPs steht genau einmal im DLL-Ziel |
| Rust-Formatierung | `rustfmt --edition 2024 --check` für alle 262 Rust-Dateien der drei Bereiche erfolgreich |
| Gezielte Cargo-Prüfungen | `shroudforge-game-assets`, `shroudforge-runtime-bridge` und `shroudforge-compatibility` geprüft. `cargo check -p shroudforge-api --all-targets` einschließlich der bestehenden Beispieltargets erfolgreich |
| `dev/tests/run_workspace_tests.ps1` | Startup-Gate für Erfolg und Vorbereitungsfehler erfolgreich. `cargo test --release --workspace` erfolgreich mit 101 bestandenen Tests, keinem Fehler und sechs bestehenden ignorierten Tests |
| Native Release-Targets | DLL, sechs bestehende Testprogramme, `kfc-runtime-dev` und `kfc-runtime-verify-component-registry` erfolgreich gebaut |
| Native Testprogramme | Ressourcen 761, Prop-Abfragen 35, ECS 184, Services 47, Baueingaben 30 und Weltkontext 53 Checks bestanden. Zusammen 1.110 Checks |
| Reguläre Produktaktionen | `build_modloader_frontend.ps1 -UseInstalledDependencies`, `build_modloader.ps1 -UseInstalledDependencies -SkipUiBuild`, `build_kfc_runtime.ps1`, `build_windows_bootstrap.ps1` und `build_windows_proxies.ps1` erfolgreich |
| Öffentliche native Schnittstelle | Alle 74 bisherigen Exportnamen und sämtliche ABI-Deklarationen erhalten. Alle 74 Symbole direkt an der Produktions-DLL aufgelöst. `KfcRuntimeAbi()` liefert 12. Der vorhandene CLI-Aufruf `inspect-provider` bestätigt zusätzlich ABI 12 und seine 31 geprüften Kernexporte |
| Website und Lua-Symbolkatalog | `npm run site:check` erfolgreich. 156 zweisprachige Dokumentationsrouten, zwölf lokalisierte Ansichten und 1.015 Lua-Symbole validiert. Vorher-/Nachhervergleich aller Symbolinhalte außer ihren verschobenen Quellpfaden ohne hinzugefügte, entfernte oder geänderte Einträge |
| Datenverträge | Vier Spielprofile und beide Spieldatei-Schemas bytegleich zum Anfangsinventar. Alle 15 Lua-Definitionsquellen textgleich zum gesicherten Anfangsstand. Zeilenendennormalisierung wird dabei nicht als Inhaltsänderung gezählt |
| Submodulgrenzen | Parser weiterhin direkt unter `kfc-parser/` auf `742b1cb796238ebdec143842545a5a8f5fc79459`. JSON unter `modules/game-runtime/native/json/` unverändert auf `55f93686c01528224f448c19128836e7df245f72` |
| Release-Paketierung | `package_release.ps1 -BuildNumber structure-check` erfolgreich. ZIP mit 31 Einträgen geprüft. Alle sieben Binärdateien entsprechen den aktuellen Buildausgaben, alle vier Profile ihren Quellen. Konfiguration, unveränderte Installationspfade, Managed-Paths und SHA-256-Datei geprüft. Kein Release veröffentlicht |
| Diff- und Quellreview | `git diff --check` für die betroffenen Bereiche erfolgreich. Fachliche Aufteilungen und zentrale Funktionskörper unabhängig gegen gesicherte Originale geprüft, darunter Aktivierung/Cleanup, Hooks, Profile, Komponenten-Schreibzugriffe und Ressourcen |

Die Prüfung hat zwei konkrete Fehler vor dem Abschluss sichtbar gemacht. Der alte Prop-Test scheitert auch im ausführbaren Vorher-Build bereits an seinem ersten Check, weil der vorhandene Tabellenleser Nullslots entfernt. Die Fixture verwendet jetzt belegte private Platzhalter für den Budgetscan und prüft im Sparse-Fall den tatsächlich vorhandenen Eintrag. Alle 35 Checks bleiben erhalten. Produktlogik wurde dafür nicht geändert. Beim Zusammenführen der Patch-ABI-Einstiege fehlte zunächst deren Exportdeklaration. Direkter ABI-Include und explizite Exportmakros sind ergänzt. Der erneute Produktionsbuild und die vollständige DLL-Exportprüfung bestehen.

Der erste Workspace-Lauf fand keine erzeugte UI-HTML. Der vollständig geprüfte Quell- und Ausgabepfad war korrekt. Nach erneutem Frontend-Build besteht der vollständige Testlauf. Die genannten Ergebnisse beziehen sich auf die abschließenden erfolgreichen Läufe.

Die Builds melden weiterhin Parser-/Abhängigkeitswarnungen, bestehende Beispiel-Target-Aliase und native Alignmentwarnungen. Diese Meldungen wurden nicht durch fachfremde Änderungen unterdrückt.

Nicht ausgeführt wurden Live-Spielprüfungen und die sechs standardmäßig ignorierten Tests mit expliziten Installations-, Registry-, Capture- oder Engine-Vektoreingaben. Die DLL-Prüfung ruft nur `KfcRuntimeAbi()` auf, keine Spielinitialisierung. Eine vergleichende Laufzeit- oder Speichermessung ist offen. Weitergehende Änderungen an Reload-Besitz, Content-Lebensdauer und API-Verträgen wurden bewusst nicht in das Refactoring aufgenommen.
<!-- END IMPLEMENTATION CHECKS -->
