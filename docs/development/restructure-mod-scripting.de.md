# Strukturvorschlag für das bisherige execution-Modul

Stand: 17. Oktober 2026. Grundlage ist der vorhandene, bereits umfangreich geänderte Arbeitsbaum. **Dieses Dokument beschreibt einen Vorschlag. Es wurden keine Implementierungen verschoben oder umbenannt.** Die aktuelle Zuständigkeitsübersicht bleibt [code-ownership.en.md](code-ownership.en.md).

## Empfehlung

`execution/` soll zu **`modules/mod-scripting/`** werden. Dieser Name beschreibt den vorhandenen Funktionsumfang: getrennte Lua-Umgebungen für Mods, Startup-Mods, Ingame-Callbacks, Livekonfiguration und die Anbindung paketlokaler Mod-DLLs. Die bestehende Cargo-Crate heißt weiterhin `shroudforge-api`. Ihre öffentlichen Rust-Schnittstellen und Lua-Namen bleiben erhalten.

Die wesentliche Verbesserung liegt in der Zuordnung des Codes. Heute stimmen physischer Ordner, Rust-Modul und tatsächlicher Besitzer mehrfach nicht überein. Der neue Aufbau trennt Sitzungszustand, Startup-Aktionen, Ingame-Lebenszyklus und Lua-Anbindung anhand der tatsächlich gelesenen Implementierungen.

`startup/` und `ingame/` benennen die zwei vorhandenen Phasen. `session/` besitzt den gemeinsamen Zustand. `lua/ingame/` stellt die Lua-Anbindung an das laufende Spiel bereit. Die bereits brauchbaren Einzelaktionen unter `attributes/` und `values/` bleiben zusammen. Es entsteht kein Job-Framework für Mods.

## Vollständig untersuchter Umfang

| Bereich | Dateien | Vollständig gelesen | Anteil |
| --- | ---: | ---: | ---: |
| Rust-Quellen | 149 | 149 | 100 % |
| Lua-Quellen einschließlich Deklarationen und Szenarien | 20 | 20 | 100 % |
| Cargo-Manifest und JSON-Verträge | 3 | 3 | 100 % |
| README | 1 | 1 | 100 % |
| **Gesamt** | **173** | **173** | **100 %** |

Die 169 Quelldateien wurden vollständig gelesen. Insgesamt umfasst der Bereich 31.289 Zeilen. Der Dateibaum wurde zusätzlich mit einer rekursiven Dateisysteminventur einschließlich versteckter Dateien abgeglichen. Im Umfang lagen keine ausgelassenen, generierten oder vendorten Dateien. Die 15 Lua-Definitionsdateien sind hier gepflegte Vertragsquellen. Das erzeugte Definitionsbundle in Cargo-OUT_DIR liegt außerhalb dieses Quellumfangs. Im Ausgangscode existieren 45 explizite Rust-`#[path]`-Attribute. Ihre Zahl allein ist kein Qualitätsmaß, die querliegenden Zuständigkeiten sind der belegte Mangel.

Die Dateien wurden im Hauptauftrag und in drei abgegrenzten Teilprüfungen vollständig inhaltlich gelesen. Die Gesamtverantwortung und die Zusammenführung der Vorschläge liegen beim Hauptagenten. Die [Dateizuordnung](mod-scripting-file-plan.csv) enthält für jede Quelldatei den Lesestatus, Zeilenzahl und SHA-256 des gelesenen Arbeitsstands. Das ist ein vollständiger Audit von `execution/`, kein vollständiger Audit des übrigen Repositorys.

Gelesen wurden die Repository-`AGENTS.md`, der ausdrücklich angeforderte Coding-Skill und die aktuelle Zuständigkeitsübersicht. Unter `execution/`, `docs/`, `dev/` und `modules/` wurden keine zusätzlichen `AGENTS.md` gefunden. Übergeordnete Regeln unter `A:\AGENTS.md` und `A:\Github\AGENTS.md` waren nicht vorhanden.

## Belegter Ist-Zustand

| Befund | Beleg | Folgerung für den Vorschlag |
| --- | --- | --- |
| Physischer Pfad und Rust-Besitzer fallen auseinander | `lua/bindings/mod.rs` bindet Sitzungszustand, Live-API, Ownership und Versionsimplementierungen über `#[path]` ein. `lua/assets/files.rs` lädt Live-Ressourcen und deren Index als Untermodul | Direkte Moduldeklarationen beim tatsächlichen Besitzer verwenden. Interne Aufrufer anpassen, statt alte Pfade durch Weiterleitungsdateien zu konservieren |
| Der ganze Bereich ist mehr als eine Laufzeitschleife | `preparation/run.rs`, `runner/`, `context.rs`, `native_mods.rs`, `lua/` und `runtime/` erfüllen verschiedene Handlungen | Ein Produktmodul `mod-scripting` mit benannten fachlichen Bereichen |
| `context.rs` übernimmt auch Initialisierung | `AppState::new` prüft Exportpfade, lädt das Typregister, öffnet KFC-Dateien und startet gegebenenfalls die Asset-Transaktion | Initialisierung nach `session/create_mod_session.rs`, Zustandsbesitz nach `session/mod.rs` |
| `preparation/game_files.rs` vereint vier Aufgaben | KFC öffnen, Arbeitsbaseline sichern und wiederherstellen, Typregistercache laden, Lua-Definitionen ausgeben | Nach diesen Aufgaben aufteilen. Parsercode und Datei-Transaktion behalten ihre bestehenden Module |
| Das Ressourcenwertmodell wird tatsächlich geteilt | `bindings/game/assets/value/` wird durch Dateiressourcen, Live-/ECS-Ansichten und Buffer-Konvertierungen verwendet | Ein kanonisches Modell unter `lua/resource_values/` erhalten |
| Owned-Werte benötigen teilweise keinen Live-Zugriff | `lua/runtime/values/` enthält `new`, `decode`, `encode` und Feldauflösung | Diese Aktionen nach `lua/values/`, getrennt von der Ingame-Anbindung |
| Die Indexdatei mischt Besitz und Lua-Schnittstelle | `lua/runtime/loaded_resources/index.rs` enthält Indexzustand und Fortschritt sowie Lua-Argumente, Codec und Bindung | In einem eigenen Folgeschritt Zustand nach `session/`, API nach `lua/ingame/resources/` trennen |
| Ein Name verdeckt eine andere Handlung | `lua/bindings/buffer_transform.rs` beweist und interpretiert einen eingeschränkten nativen Buffer-Transformationsablauf für die Funktionsbindung | Nach `lua/ingame/prove_buffer_transform.rs`, ohne daraus allgemeine native Aufrufberechtigung abzuleiten |
| Ein Test ist intern eingebunden | `tests/code_inventory.rs` verwendet private `super::*`-Implementierung und wird per `#[path]` geladen. Die übrigen Rust-Integrationstests sind explizit in Cargo registriert | Diesen Unit-Test neben den Besitzer legen und die beiden Testarten erhalten |

## Zielstruktur und vollständige Dateizuordnung

Die folgende Struktur enthält die Ziele aller 173 vorhandenen Dateien und die zusätzlich begründeten Zielaktionen. Wenn zwei Quellen gemeinsam eine kanonische Zielimplementierung bilden sollen, führt die CSV beide Quellen mit ihrer jeweiligen Verantwortung auf. Ein gemeinsames Ziel bedeutet keine zweite Implementierung.

Die primäre Zuordnung ist der Strukturplan. Fachliche Aufteilungen stehen zusätzlich im folgenden Abschnitt mit ihren vollständigen Zielpfaden. Diese Trennung erlaubt einen prüfbaren Ortswechsel vor einem weitergehenden Refactoring.

```text
modules/mod-scripting/
  build.rs
  Cargo.toml
  documents.rs
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
  lib.rs
  logging.rs
  lua/
    arguments.rs
    bindings/
      buffer/
        byte_order.rs
        errors.rs
        manage_buffer.rs
        mod.rs
        read_buffer.rs
        write_buffer.rs
      extend_table_operations.rs
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
      hash_bytes.rs
      image/
        image_format.rs
        image.rs
        mod.rs
        pixel_format.rs
      integer_operations.rs
      io/
        manage_exports.rs
        mod.rs
        path_names.rs
        read_mod_files.rs
      mod.rs
      register_functions.rs
      write_mod_log.rs
    cast_values.rs
    convert_json.rs
    create_sandbox.rs
    definitions/
      export_lua_definitions.rs
      generate_game_type_definitions.rs
      mod.rs
    eml/
      mod.rs
      v1/
        create_loader_api.rs
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
        mod.rs
    errors.rs
    format_type_name.rs
    ingame/
      access_named_service.rs
      check_ingame_access.rs
      ecs/
        access_component_fields.rs
        access_template_component.rs
        attributes/
          check_attribute_component.rs
          encode_attribute_scalar.rs
          evaluate_attributes.rs
          expect_root_values.rs
          format_attribute_root.rs
          mod.rs
          read_attribute_catalog.rs
          read_attribute_descriptor.rs
          read_attribute_storage_range.rs
          read_attribute.rs
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
        mod.rs
        query_entities.rs
        read_component_catalog.rs
        read_component_registry.rs
        read_component.rs
        resolve_entity.rs
        write_component.rs
      functions/
        bind_profile_modifier.rs
        function_catalog_tests.rs
        mod.rs
        read_function_catalog.rs
      mod.rs
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
      prove_buffer_transform.rs
      resources/
        index_loaded_resources.rs
        index_scalar_tests.rs
        mod.rs
        read_loaded_resources.rs
        write_loaded_field.rs
      set_profile_patch_enabled.rs
      world/
        access_prop_transform.rs
        change_building_entities.rs
        mod.rs
        query_props.rs
        read_world_context.rs
        read_write_voxels.rs
        register_prop_recipes.rs
    mod_runner.rs
    mod.rs
    parse_require_path.rs
    readonly_collections.rs
    require_mod.rs
    resource_values/
      convert_resource_values.rs
      mapped_values.rs
      mod.rs
      owned_values.rs
      validate_resource_values.rs
      value_path.rs
    shroudforge/
      v1/
        create_mod_api.rs
        definitions/
          mod_api.lua
          runtime_api.lua
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
  README.md
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
    loaded_resource_indexes_tests.rs
    loaded_resource_indexes.rs
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
  type_aliases.rs
```

Die [CSV für jede einzelne Datei](mod-scripting-file-plan.csv) enthält `Altpfad`, `Zielpfad`, `Handlung`, `Modul`, `Verantwortung`, `Aufrufer`, `Begründung`, `Hinweise`, `Zwischenpfad`, `ZusätzlicheZiele`, `Lesestatus`, `Zeilen` und `SHA256`. Alle Pfade sind vollständig relativ zu `A:\Github\shroudforge`. In der CSV gibt es für jede vorhandene Datei genau eine Zeile.

Rust- und Cargo-Einstiege behalten ihre vorgeschriebenen beziehungsweise etablierten Namen `lib.rs`, `mod.rs`, `build.rs` und `Cargo.toml`. Ein guter bestehender Aktionsname wird nicht allein deshalb geändert, weil seine Datei umzieht. Versionsordner bleiben unmittelbar unter `lua/eml/v1/` und `lua/shroudforge/v1/`.

## Gezielte Aufteilungen und Zusammenführungen

Alle zusätzlichen Ziele gehören zu `modules/mod-scripting/`. Die primäre CSV-Zielspalte beschreibt die Zielstruktur nach den empfohlenen Aufteilungen. `Zwischenpfad` beschreibt den ungeteilten Ortswechsel davor, `ZusätzlicheZiele` die zugehörigen Fachaktionen. Bei der kurzen Entity-Auflösung werden die verwandten Funktionen bewusst gemeinsam behalten. Die optionale Aufteilung des großen Runtime-Definitionsvertrags in weitere Domänendateien wird zurückgestellt, bis dort inhaltlich gearbeitet wird.

| Quelle | Vollständiger zusätzlicher Zielpfad | Handlung und Aufrufer | Grund |
| --- | --- | --- | --- |
| execution/runtime/load_runtime_callbacks.rs, execution/runtime/refresh_runtime_configuration.rs, execution/runtime/restore_runtime_packages.rs | `modules/mod-scripting/ingame/activate_runtime_mod.rs` | Aktivscope setzen, on_load ausführen und gescheiterte Aktivierung zurückrollen. Aufrufer: Erststart, Liveaktivierung und Wiederherstellung | Belegte Wiederholung erst nach Vergleich von Messungen und Cleanup-Grenzen kanonisieren. Aufrufer behalten Fehlerdarstellung und Abhängigkeitsentscheidung. |
| execution/runtime/update_runtime_mods.rs | `modules/mod-scripting/ingame/update_mod_callbacks.rs` | Fällige Modcallbacks und ausstehendes Cleanup fortsetzen. Aufrufer: update_ingame_runtime | Die bestehende Callbackschleife ist von Diagnostik-, Transport-, Reload- und Statuskoordination abgrenzbar. |
| execution/lua/assets/files.rs | `modules/mod-scripting/lua/bindings/game/assets/access_file_content.rs` | Dateiinhalte anhand ContentHash suchen, auflisten und anlegen. Aufrufer: game/assets/mod.rs | Andere Identität und Datenschnittstelle als Resource, bisher drei eigene Funktionen im Sammelmodul. |
| execution/lua/assets/files.rs | `modules/mod-scripting/lua/bindings/game/assets/access_file_resources.rs` | Dateiressourcen suchen, auflisten, typisieren und anlegen. Aufrufer: game/assets/mod.rs | Heute besitzt assets/files.rs diese echte Handlung zusammen mit Registrierung und Content-Aktionen. Auslagern an dieselbe API, keine zweite Fachimplementierung. |
| execution/lua/bindings/io.rs | `modules/mod-scripting/lua/bindings/io/manage_exports.rs` | Exportdateien unter dem konfigurierten Exportordner lesen, erzeugen, kopieren, umbenennen und löschen. Aufrufer: bindings/io/mod.rs | Exportfeature, Pfadprüfung und Anlagegrenzen gehören zusammen. Vorhandene Pfadprüfungen nicht ungeprüft gleichsetzen. |
| execution/lua/bindings/io.rs | `modules/mod-scripting/lua/bindings/io/path_names.rs` | Dateinamen, Endung, Elternpfad und Join ohne Dateizugriff bestimmen. Aufrufer: bindings/io/mod.rs | Reine Zeichen-/Pfadoperationen benötigen weder Mod-Sitzung noch Exportzugriff und liegen heute zwischen diesen fachfremden Handlungen. |
| execution/lua/bindings/io.rs | `modules/mod-scripting/lua/bindings/io/read_mod_files.rs` | Dateien und Verzeichnisse ausschließlich über Mod::fs lesen. Aufrufer: bindings/io/mod.rs | Bestehende sechs paketgebundene Handler erhalten den Mod-Eigentümer. Kein Archivsystem und kein Dateisystem-Nachbau erforderlich. |
| execution/lua/bindings/registry.rs | `modules/mod-scripting/lua/convert_json.rs` | JSON-Berichte in abgelöste Lua-Tabellen projizieren. Aufrufer: lua/ingame/ecs/registry.rs, lua/ingame/ecs/attributes/*, lua/ingame/functions/read_function_catalog.rs | registry.rs ist bereits kanonische Implementierung, gehört aber nicht zu Typidentität. Beim Herauslösen denselben Converter behalten. |
| execution/lua/mod.rs | `modules/mod-scripting/lua/create_sandbox.rs` | LuaVM mit erlaubten Bibliotheken und Basisumgebung erzeugen. Aufrufer: LuaModRunner::new und create_mod_environment | Vorhandene echte Sandbox aus dem Modulstart herausziehen. Keine neue VM-Abstraktion. |
| execution/preparation/game_files.rs, execution/preparation/run.rs | `modules/mod-scripting/lua/definitions/export_lua_definitions.rs` | Basis- und Typdefinitionen in den bestehenden Cache ausgeben. Aufrufer: Pregame-Sitzungsinitialisierung und öffentliche export_lua_definitions-Funktion | Generator existiert bereits, die Ausgabe liegt heute in zwei Startup-Dateien. Diese Aktion an den vorhandenen Definitionsbesitzer ziehen. |
| execution/lua/runtime/ecs/components.rs | `modules/mod-scripting/lua/ingame/ecs/access_component_fields.rs` | Felder einer Komponente über den gemeinsamen Inline-Codec lesen und ändern. Aufrufer: Komponenten-Read und -Write | Vorhandenen gemeinsamen Feldweg erhalten. Die typisierten Schutzprüfungen nicht pro Aufrufer kopieren. |
| execution/lua/runtime/mod.rs | `modules/mod-scripting/lua/ingame/ecs/mod.rs` | Bestehende ECS-Tabelle aus Komponenten-, Entity- und Attributaktionen zusammensetzen. Aufrufer: lua/ingame/mod.rs | Vorhandenen Zusammenbau zum tatsächlichen ECS-Besitzer verschieben. Rust-Modulgrenze verlangt einen Einstieg. |
| execution/lua/runtime/ecs/registry.rs | `modules/mod-scripting/lua/ingame/ecs/read_component_catalog.rs` | Reflektierten Komponentenkatalog in Lua projizieren. Aufrufer: runtime.ecs-Katalogabfrage | Reflected-only und aktuelle native Registrierung bleiben unterschiedliche Ergebnisse. |
| execution/lua/runtime/ecs/components.rs | `modules/mod-scripting/lua/ingame/ecs/write_component.rs` | Komponente mit Änderungsmaske, Claims und vollständigem Snapshotvergleich schreiben. Aufrufer: Öffentliche Komponentenwriter | Die Datei besitzt den bestehenden Schreibablauf einschließlich collect_changed_ranges. Read- und Write-Fehler nicht vereinheitlichen. |
| execution/lua/runtime/mod.rs, execution/lua/runtime/code_inventory.rs | `modules/mod-scripting/lua/ingame/functions/mod.rs` | Deklarierte Runtime-Operationen mit geprüften Wrappern binden. Aufrufer: Lua-runtime-Registrierung Funktionskatalog und belegte Adapter in Lua verfügbar machen. Aufrufer: Lua-runtime-Registrierung | Checked Wrapper und Kataloginterpretation bleiben einmalig beim Funktionsbereich. Registry-/Refresh-Zustand und Lua-Handler haben verschiedene Aufrufer, verwenden denselben Catalog. |
| execution/lua/bindings/mod.rs, execution/runtime/overrides.rs, execution/runtime/ownership.rs | `modules/mod-scripting/lua/ingame/overrides/mod.rs` | Gemeinsame Override-Tabelle mit Trackern und Schreibclaims erzeugen. Aufrufer: lua/bindings/mod.rs und Ingame-API | Existierende Zusammenbauarbeit beim Override-Eigentümer bündeln. Lua- und Rust-Implementierungen bleiben kanonisch. |
| execution/lua/runtime/mod.rs | `modules/mod-scripting/lua/ingame/player_input/mod.rs` | Bau-Eingabefunktionen und Cursorzugriff aus ihren vorhandenen Aktionen zusammensetzen. Aufrufer: Welt-API-Registrierung | Echte Registrierungsarbeit aus dem bisherigen großen Einstieg, kein allgemeines Eingabesystem. |
| execution/lua/runtime/loaded_resources/index.rs | `modules/mod-scripting/lua/ingame/resources/index_loaded_resources.rs` | Lua-Indexanfragen, ScalarField und Fortschrittsergebnisse anbinden. Aufrufer: game.assets-Indexfunktionen | Session besitzt den Indexkern. Lua-spezifische Argumente und Codec bleiben in der Anbindung. |
| execution/lua/runtime/loaded_resources/index_tests.rs | `modules/mod-scripting/lua/ingame/resources/index_scalar_tests.rs` | Vorhandene ScalarField-Codec-Prüfungen beim Lua-Index behalten. Aufrufer: Bestehender Unit-Testbaum | Dies sind verschobene Testinhalte aus index_tests.rs. Keine neuen Tests oder automatische Ausführung. |
| execution/lua/runtime/mod.rs, execution/lua/assets/files.rs | `modules/mod-scripting/lua/ingame/resources/mod.rs` | Live-Aktionen in die vorhandene game.assets-Tabelle eintragen. Aufrufer: lua/bindings/game/assets/mod.rs | Die öffentliche Tabelle bleibt gemeinsam. Nur ihr Live-Registrierungsanteil wechselt zum tatsächlichen Besitzer. |
| execution/lua/runtime/loaded_resources.rs | `modules/mod-scripting/lua/ingame/resources/write_loaded_field.rs` | Ein Feld geladener Ressourcen mit erwarteten Bytes und Sitzungsfreigabe schreiben. Aufrufer: game.assets-Livewriter | Eigener überprüfter Schreibablauf mit bestätigtem Readback. Dateiressourcen weiterhin über Startup-Transaktion schreiben. |
| execution/lua/runtime/ecs/props.rs | `modules/mod-scripting/lua/ingame/world/access_prop_transform.rs` | Prop-Transform lesen und schreiben. Aufrufer: runtime.world.entity.get_transform und runtime.world.entity.set_scale | Eine gemeinsame lokale Prop-Projektion für alle Ausgabewege erhalten. Koordinatenvertrag nicht ändern. |
| execution/lua/runtime/mod.rs | `modules/mod-scripting/lua/ingame/world/mod.rs` | Vorhandene Lua-Welt-, Prop- und Eingabeaktionen registrieren. Aufrufer: lua/ingame/mod.rs | Die Welt-Registrierung gehört zu ihrem vorhandenen Fachbereich. Keine zweite Funktionsliste. |
| execution/lua/runtime/ecs/props.rs | `modules/mod-scripting/lua/ingame/world/register_prop_recipes.rs` | Prop-Rezepte beim bestehenden Provider registrieren. Aufrufer: runtime.world.entity.register_prop_recipes | Rezeptregistrierung unterscheidet sich von räumlicher Abfrage. Der Provider bleibt Eigentümer der nativen Wirkung. |
| execution/lua/shroudforge/v1/shroudforge.rs | `modules/mod-scripting/lua/shroudforge/v1/dispatch_mod_ui_actions.rs` | Gespeicherte UI-Aktionen an registrierte Modcallbacks zustellen. Aufrufer: Ingame-Tick | Dateianfrage, Callbackzustellung und Limit 32 gemeinsam erhalten. Keine UI-Fachlogik in den Loader verlagern. |
| execution/lib.rs, execution/lua/runtime/mod.rs, execution/lua/runtime/policy.rs | `modules/mod-scripting/lua/shroudforge/v1/read_runtime_operations.rs` | Eingebetteten Operationskatalog kanonisch lesen und seine unveränderliche Darstellung bereitstellen. Aufrufer: Öffentliche runtime_operations-Funktion, Funktionsregistrierung und get_operations | Drei vorhandene Leser parsen denselben Vertrag. Gemeinsame Darstellung und gegebenenfalls einmaliges Parsing erst unter Erhalt von Validierung und Fehlersemantik einführen. Kein neues Speicherformat. |
| execution/lua/shroudforge/v1/shroudforge.rs | `modules/mod-scripting/lua/shroudforge/v1/set_mod_settings.rs` | Vorhandene Mod-Settings-Tabelle live aktualisieren. Aufrufer: Ingame-Konfigurationsabgleich | Die Handlung ist bereits eine eigene Funktion. Keine zusätzliche Einstellungsablage. |
| execution/preparation/game_files.rs | `modules/mod-scripting/reflection/load_type_registry.rs` | Typregister anhand EXE-Identität und KFC-Version laden beziehungsweise extrahieren. Aufrufer: Sitzungsinitialisierung und Definitionsexport | Vorhandener kanonischer Registercache ist kein Teil des KFC-Leser-/Schreiberadapters. Identitätsformat und Fehlerpfade erhalten. |
| execution/lua/runtime/mod.rs | `modules/mod-scripting/session/configure_component_contract.rs` | Vorhandene ECS-Komponentenverträge und Profile der Sitzung konfigurieren. Aufrufer: Lua-API-Initialisierung und vorhandener Configure-Retry | Die Initialisierung des Bridge-Vertrags ist keine allgemeine Lua-Tabellenregistrierung. |
| execution/context.rs | `modules/mod-scripting/session/create_mod_session.rs` | Konfiguration prüfen, Register und Dateihandles öffnen, Sitzungszustand erzeugen. Aufrufer: Startup-Lauf und IngameRuntime::start_with_schema | AppState::new besitzt diese Initialisierung heute selbst. Vorhandene Speicherresolver und game-assets-Transaktion weiterverwenden. |
| execution/native_mods.rs | `modules/mod-scripting/session/read_native_plugin_declaration.rs` | Vorhandene Plugin-Sektion aus native-plugin.ini lesen. Aufrufer: Ingame-DLL-Scan und Sitzungs-DLL-Manager | Deklaration und native Handle-Lebensdauer sind verschiedene Handlungen. Tolerierte Werte, Sektionen und Enabled-Verhalten erhalten. |
| execution/context.rs | `modules/mod-scripting/session/record_runtime_state.rs` | Aktivstatus, Effekte und unsichere Writes der Sitzung verwalten. Aufrufer: Lifecycle, Lua-Writer und Statuspublikation | Zustandsänderungen besitzen andere Aufrufer als Dateieröffnung. Bestehende Felder bleiben kanonisch in der Sitzung. |
| execution/preparation/run.rs | `modules/mod-scripting/startup/commit_asset_changes.rs` | Geänderte typisierte Ressourcen schreiben, Writer finalisieren und Transaktion committen. Aufrufer: run_startup_mods nach erfolgreichem Lua-Lauf | Bestehender echter Commitblock ist von Planung und Cacheabschluss verschieden. Keine zweite Transaktion oder zusätzliche Commitgrenze. |
| execution/preparation/game_files.rs, execution/preparation/run.rs | `modules/mod-scripting/startup/manage_asset_baseline.rs` | Arbeitsbaseline prüfen, sichern und paarweise wiederherstellen. Aufrufer: Sitzungsinitialisierung und öffentliche restore-/backup_companion_file-Aufrufe | Bestehende .bak-Semantik ist von den verwalteten Originalbackups verschieden. game-assets/originals.rs ohne Gleichheitsbeweis nicht ersetzen. |
| execution/lib.rs, execution/preparation/run.rs | `modules/mod-scripting/startup/mod.rs` | Startup-Aktionen und Cache als Rust-Modul deklarieren. Aufrufer: lib.rs | Technisch notwendiger Moduleinstieg ohne neue Wrapperfunktion oder neue Crate. |

Die zwei Registrierungsanteile im Funktionsbereich werden in demselben `lua/ingame/functions/mod.rs` zusammengeführt. Rust-Modulstarts besitzen tatsächlichen Zusammenbau oder die technisch notwendige Deklaration ihrer vorhandenen Aktionen. Sie führen keine zusätzlichen Weiterleitungsfunktionen ein. Gezielte `#[path]`-Attribute können für versionierte Einstiege sowie die stabilen öffentlichen Pfade `reflection::functions` und `reflection::types` sinnvoll bleiben.

Diese Ziele sind konkrete Handlungsvorschläge innerhalb des zuständigen Moduls. Sie sind keine Aufforderung, jede große Datei nach Zeilenzahl zu zerlegen. Bestehende `attributes/`-Aktionen, validierende Schreibpfade, unterschiedliche Selektorverträge und das gemeinsame Ressourcenwertmodell werden erhalten.

## Weniger Code und gezieltere Arbeit

| Änderung | Belegter Ansatz | Grenze |
| --- | --- | --- |
| Prop-Ergebnistabellen einmal aufbauen | In `lua/runtime/ecs/props.rs` wiederholen `query_props`, `query_props_in_bounds` und `get_transform` den Tabellenaufbau | Gemeinsame lokale Umwandlung verwenden. Abfrage, Begrenzung, Fehler und native Freigaben bleiben pro Handlung |
| Typauswahl an einer Stelle pflegen | `bindings/type_lookup.rs` und `bindings/registry.rs` lösen beide Typen auf | Zulässige Selektoren, mehrdeutige Treffer, nil-Ergebnisse und Fehler unterscheiden sich. Diese Verträge beim gemeinsamen Resolver ausdrücklich erhalten |
| Bytepuffer-Prüfungen einmal implementieren | Skalare Reads und Writes wiederholen Zustands-, Bereichs- und Reservierungsprüfungen, die `read_bytes` und `write_bytes` bereits ausführen. `copy` wiederholt einen Byte-Schreibpfad | Vorhandene Bytewege verwenden. Cursorfortschritt, Endian, f16, u64-Bitmuster, Closed-/ReadOnly-Fehler und überlappendes Kopieren erhalten |
| Kleine Lua-Adapter lokal vereinheitlichen | `integer.rs` besitzt einen eigenen Registrierungsadapter. `log.rs` formatiert Variadic-Werte zweimal. `game/types.rs` hat zwei gleiche Handler. Buffer-Registrierung wiederholt Zahlenadapter | Bestehenden Registrar, lokale Formatierer und passende Makros verwenden. Unterschiedliche Signaturen und Fehlerverträge bleiben ausdrücklich getrennt |
| Operationskatalog kanonisch lesen | `lib.rs`, `lua/runtime/mod.rs` und `lua/runtime/policy.rs` parsen denselben eingebetteten Katalog | Gemeinsame unveränderliche Darstellung prüfen. Schemafehler nicht verschlucken und kein zweites Config-Dokument einführen |
| Aktivierung nicht mehrfach implementieren | `load_runtime_callbacks.rs`, `refresh_runtime_configuration.rs` und `restore_runtime_packages.rs` setzen Aktivstatus, rufen `on_load` auf und rollen Fehler zurück | Gemeinsame Aktivierungsaktion erst nach Abgleich aller Mess-, Fehler- und Cleanup-Pfade. Die Aufrufer behalten ihre jeweilige Fehlerdarstellung |
| Zustandsübernahme beim Reload an einem Ort bündeln | `reload_runtime_packages.rs` übernimmt heute viele Runtime-Felder einzeln durch `mem::swap` | Eine zusammenhängende Übernahme des ersetzbaren Zustands prüfen. Diagnose-Session, Request-Deduplizierung, Drop-Status und alte DLL-Lebensdauer nicht versehentlich verändern |
| Vorhandene Caches erhalten | Attributkatalog, begrenzter Inline-Feldcache und zeitbudgetierter Ressourcenindex existieren bereits | Keine weiteren Live-Snapshots oder Schreibberechtigungen cachen. Ein Layoutcache ersetzt keine aktuelle Sitzung, Claim-Prüfung oder CAS-Vorbedingung |
| Große Inhaltskopien gezielt untersuchen | `assets/session.rs::get_content` klont gespeicherte Inhaltsbytes und `create_content` hält eine Kopie vor. Der Code markiert diese Stellen selbst als Optimierungsansatz | Zuerst Besitz, Buffer-Userdata und Leserlebensdauer prüfen. Eine Umstellung auf geteilte Bytes ist ein eigener Verhaltenseingriff, kein einfacher Move |
| Wiederholte Lua-Deklarationen pflegen | `integer.lua` enthält viele parallele Deklarationen und Dokumentationsblöcke | Ohne nachgewiesenen Generator und vollständigen Vertragsvergleich keine Quelle durch neue Generierung ersetzen |

Eine Verringerung von Laufzeit, Speicher oder Quellzeilen wurde nicht gemessen. Der Vorschlag benennt konkrete Doppelungen und unnötige Verflechtung. Eine belastbare Leistungsangabe setzt eine spätere Umsetzung und Messung mit den vorhandenen Diagnoseaktionen voraus.

## Getrennte funktionale Befunde

### Definitionsbundle enthält ausführbare Lua-Skripte

`execution/build.rs` läuft rekursiv über alle `.lua` unter `lua/`, sortiert sie und verkettet ihren Inhalt. Damit werden auch `override_targets.lua` und `overrides.lua` eingebunden. Die erste Datei endet mit einem Top-Level-`return`, während die folgende Datei weiteren Code enthält. Daraus folgt aus der gelesenen Verkettung ein Syntaxrisiko für die erzeugte Basisdefinition. Ein Lua-Parserlauf wurde nicht ausgeführt.

`docs/website/tools/build-website-output.mjs` durchsucht ebenfalls alle `.lua` unter diesem Stamm. Dort wird aus dem Quelltext ein Symbolkatalog extrahiert, kein Lua-Chunk ausgeführt. Beide Verbraucher sollen gezielt nur die bestehenden `definitions/`-Bäume einlesen und dieselbe deterministische Reihenfolge behalten. Das ist eine gesonderte funktionale Korrektur. Ein Verschieben der Hilfsskripte innerhalb `lua/` behebt die Rekursion allein nicht.

### Cargo-Beispiele zeigen noch auf alte Entwicklungsorte

`execution/Cargo.toml` registriert fünf Beispieltargets über zwei Quellpfade direkt unter `dev/profiles/`. Beide dort referenzierten Dateien fehlen im gelesenen Arbeitsstand. Die vorhandenen Quellen stehen unter `dev/profiles/inspect/`. Beim geplanten Modulwechsel lauten die entsprechenden relativen Pfade `../../dev/profiles/inspect/export_kfc_resources.rs` und `../../dev/profiles/inspect/inspect_system_dependencies.rs`. Die fünf Targetnamen bleiben erhalten.

Das ist durch Pfadprüfung belegt. Ein Cargo-Build wurde hierfür nicht gestartet. Die bereits laufende Umstrukturierung von `dev/` bleibt als fremder Arbeitsstand erhalten.

### Definitionsvollständigkeit gesondert prüfen

Die tatsächliche Builtin-Registrierung und die Aufzählung in `eml/v1/definitions/global.lua` stimmen beim `loader`-Eintrag nicht vollständig überein. ShroudForge-Wurzel- und Logging-Deklarationen sind außerdem auf `runtime.lua` und `shroudforge.lua` verteilt. Das ist Anlass für einen getrennten Vertragsabgleich, keine Erlaubnis, öffentliche Lua-Symbole beim Umzug neu zu gestalten.

## Aufrufer, Build, Paketierung und Dokumentation

| Verbraucher | Konkrete notwendige Anpassung bei Umsetzung |
| --- | --- |
| `Cargo.toml` | Workspace-Mitglied `execution` durch `modules/mod-scripting` ersetzen |
| `entrypoints/loader/Cargo.toml` | Dependency auf `../../modules/mod-scripting` richten. Crate-Name bleibt `shroudforge-api` |
| Neues `modules/mod-scripting/Cargo.toml` | Lokale Pfade nach dem Wechsel um eine Ebene korrigieren. Geschwistermodule über `../`, `kfc-runtime`, `kfc-parser`, `game-assets` und `dev` über `../../`. Explizite Testtargets und `autotests = false` erhalten |
| `entrypoints/loader/embedded.rs` | Öffentliche `IngameRuntime`-Aufrufe und C-ABI bleiben unverändert. Keine Lifecycle-Logik in diesen Aufrufer kopieren |
| `entrypoints/loader/asset_preparation/` | Öffentliche Startup-, Restore- und Exportaufrufe erhalten. Startup-Lease, Fingerprint und äußere Fehlerbehandlung bleiben beim bestehenden Besitzer |
| Rust-Moduldeklarationen und `include_str!` | Relative Quellen und Lua-Helfer mitziehen. Gewöhnliche `mod`-Deklarationen am Besitzer einsetzen. Private Testanbindung mitziehen. Das Profilfixture aus `reflection/functions.rs` muss nach `../../../kfc-runtime/profiles/enshrouded/client/25966634.json` zeigen |
| `entrypoints/loader/tests.rs` | Verweis auf die Runtime-Definitionsdatei aktualisieren |
| `dev/profiles/inspect/inspect_system_dependencies.rs` | Öffentliche Pfade `shroudforge_api::reflection::functions` und `shroudforge_api::reflection::types` erhalten. Eingebetteten Quellpfad in Berichtmetadaten aktualisieren |
| `docs/website/tools/build-website-output.mjs` | Lua-Stamm auf `modules/mod-scripting/lua` setzen. Die exakte `addIntegerHelp`-Bedingung von `eml/v1/definitions/integer.lua` auf `eml/v1/definitions/integer_api.lua` ändern. Separat die Definitionsfilterung korrigieren und Symbolkatalog sowie Reihenfolge vergleichen |
| `docs/website/reference-search.js` | GitHub-Quelllinks auf `modules/mod-scripting/lua/` setzen. Relative `source`-Felder weiterhin aus den tatsächlichen Dateien erzeugen |
| `docs/development/code-ownership.en.md` | Erst nach dem tatsächlichen Umzug Repositorykarte und Aktionspfade aktualisieren |
| `execution/README.md`, `entrypoints/loader/README.md`, weitere aktive Dokumentationslinks | Nach dem Umzug relative Links und Eigentümer korrigieren. Historische Evidenz nicht als aktuelle Routenliste behandeln |
| `dev/build/build_modloader.ps1` und `dev/tests/run_workspace_tests.ps1` | Bestehende Workspace-Einstiege verwenden. Keine zusätzliche Build- oder Teststeuerung einführen |
| `dev/build/{dev_build,release_build}.ps1`, `.github/workflows/` | Bestehende Buildketten beibehalten. Sie erreichen die Crate über den Workspace |
| `dev/build/package_release.ps1` | Bestehende Binärdateien, Modordner und Konfigurationspaketierung erhalten. Der Quellordnername erzeugt keinen neuen Installationspfad |

Die Operationenliste wird aus dem Binary über `include_str!` gelesen und gegen das eingebettete Schema validiert. `documents::API` nennt zwar einen Config-Speicherort, `validate_document` führt hier aber keine Dateileseoperation durch. Der Vorschlag führt keine zweite API-Konfigurationsdatei ein.

Reload-Anfragen schreibt `modules/updater/control/request_runtime_mod_action.rs`. Der Ingame-Tick verarbeitet sie und schreibt das bestehende Ergebnis mit derselben Request-ID. Der Updater wartet auf diese ID. Laufzeitstatus landet über `modules/runtime-diagnostics/documents.rs::MOD_STATUS` im bestehenden Statusspeicher und wird unter anderem von `modules/modloader/ui/host/read_configuration_status.rs` gelesen. Diese Datenwege und deren Formate bleiben erhalten.

## Zu erhaltende Verhaltensgrenzen

- Modsuche, Manifestprüfung und Dependency-Reihenfolge gehören weiterhin zu `modules/modloader/`. Das Scripting-Modul verwendet deren Ergebnis.
- Installierte Mods bleiben entpackte Ordner. `find_installed_mods.rs` erzeugt dafür ein Disk-Dateisystem. Der vorhandene DLL-Code enthält zusätzlich einen allgemeinen Dateisystem-Fallback. Dieser Zweig ist kein Beleg für Archivbetrieb im normalen Spielablauf und wird bei einem Move nicht ungeprüft gelöscht.
- Asset-Transaktionen bleiben unter `game-assets/transaction.rs`, Parserimplementierungen im direkten `kfc-parser/`-Submodul. Das Scripting-Modul besitzt nur die zugehörige Lua- und Sitzungsanbindung.
- EML v1, ShroudForge v1, Capability-Prüfungen, erlaubte Exportpfade, Client-/Serverfilter und Sandbox-Bibliotheken behalten ihre Verträge.
- Modmodule, Ladezyklen, require-Caches und die Behandlung exakter Manifest-IDs bleiben erhalten. Jeder Mod behält seine eigene Umgebung.
- Callbackintervalle bleiben zwischen 8 und 1000 ms mit 50 ms als Standard. Verpasste Perioden werden übersprungen, nicht in einem Schub nachgeholt.
- Konfiguration und Status werden im bestehenden Ingame-Intervall bearbeitet. Diagnoseauswahl, Sampler und Messlogik bleiben bei `runtime-diagnostics`, Transport bei `transport`.
- Cleanup bleibt wiederholbar. Ein fehlgeschlagener Unload behält seinen aktiven Scope, bis die erforderlichen Wiederherstellungen erfolgreich sind. Automatischer API-Rollback wird auch bei einem fehlerhaften Mod-Callback versucht.
- Originalwerte, aktuelle Snapshots, Session-IDs, Entity-Generationen, Claims, native Freigaben, CAS-Vorbedingungen und Unsicherheits-Schreibsperren behalten ihre getrennte Bedeutung.
- Ressourcenclaims können Weltwechsel überleben. Entity- und Serviceclaims verfallen erst beim Nachweis einer anderen, von null verschiedenen Sitzung. Ein vorübergehend fehlender Provider ist kein Freigabebeweis.
- DLL-Laden bleibt auf isolierten Workern, mit bestehender Stallwarnung und Freigabe am Sitzungsende. Ein Runtime-Reload wird nicht nebenbei auf inkrementelles Neuladen einzelner Mods umgestellt.
- Cache-Dateinamen, Formate, Buildkennung, Backupprüfung, Staging, Commit-/Abort-Grenzen und Paketpfade bleiben erhalten.

## Reihenfolge der Umsetzung

1. Den gelesenen Arbeitsstand erneut prüfen und neue Nutzeränderungen berücksichtigen. Den Ordner als bestehende Crate verschieben, Cargo-, Include-, Test-, Website- und Dokumentationspfade geschlossen aktualisieren.
2. Physische und Rust-Modulzuständigkeiten zusammenführen. Sitzungszustand, Startup, Lifecycle und Lua-Anbindung an ihre vorgesehenen Orte legen. Große fachliche Aufteilungen einzeln umsetzen und jeweils den gesamten relevanten Diff prüfen.
3. Nachgewiesene Doppelungen zusammenführen. Verschiedene Selektorverträge, Fehlerpfade, Messungen und Cleanup-Grenzen vorher vergleichen.
4. Definitionsfilterung und gegebenenfalls Vertragslücken als gesonderte funktionale Korrekturen umsetzen. Geänderte Symbolkataloge und Bundle-Eingaben vergleichen.
5. Vorhandene passende Prüfungen ausführen. Für die Implementierung gehören dazu Workspace-Build, vorhandene Rust-/Bootstrap-Prüfungen, Website-Prüfung und bei einer Paketänderung die vorhandene Release-Paketierung. Laufzeit- oder Leistungsbehauptungen erfordern zusätzlich eine passende reale Messung.

## Tatsächlich ausgeführte Prüfung dieses Vorschlags

| Prüfung | Ergebnis |
| --- | --- |
| Rekursive Inventur gegen Dateizuordnung | 173 Dateien, 173 eindeutige Altpfade, keine fehlenden oder zusätzlichen Quellen |
| Inhaltslesestatus | Alle Einträge vollständig gelesen, 169 von 169 Quelldateien |
| SHA-256 gegen gelesenen Anfangsstand und aktuelle Dateien | Alle 173 Quellen unverändert |
| Zielkollisionen | Eine beabsichtigte Zusammenführung: registry.rs und type_lookup.rs nach resolve_type.rs. Zwischenpfade erhalten zunächst beide Implementierungen |
| Zusätzliche Zielaktionen | 35 begründete Zielpfade mit Besitzer, Handlung und Aufrufern |
| Fremde Änderungen | Bereits vorhandenen Arbeitsstand erhalten, nur zwei neue Planungsdateien geschrieben |
| Markdown-/CSV-Konsistenz | Struktur und CSV aus derselben vollständigen Dateizuordnung erzeugt |

Es wurden keine Tests, Produktbuilds, Website-Builds, Paketierung oder Live-Spielprüfungen ausgeführt. Dieses Arbeitsergebnis verändert ausschließlich die beiden neuen Planungsdokumente. Eine spätere Implementierung ist damit weder kompiliert noch laufzeitgeprüft.

Beim abschließenden Vergleich des Gitstatus hatte sich der außerhalb dieses Audits liegende Submodulstatus von `kfc-parser` gegenüber der Anfangsaufnahme geändert. Dieser Auftrag hat dort keine Datei geändert, nichts gestagt und den neuen fremden Stand erhalten. Der bestätigte SHA-256-Abgleich bezieht sich auf die 173 auditierten Dateien unter `execution/`.

## Zusätzliche Lesebelege außerhalb von execution

**Zusätzlich vollständig gelesen: 41 Repositorydateien.**

- `AGENTS.md`
- `Cargo.toml`
- `dev/build/build_modloader.ps1`
- `dev/build/check_website.ps1`
- `dev/build/dev_build.ps1`
- `dev/build/package_release.ps1`
- `dev/build/release_build.ps1`
- `dev/profiles/inspect/inspect_system_dependencies.rs`
- `dev/tests/run_workspace_tests.ps1`
- `docs/development/code-ownership.en.md`
- `docs/website/reference-search.js`
- `docs/website/tools/build-website-output.mjs`
- `entrypoints/loader/asset_preparation/apply_asset_mods.rs`
- `entrypoints/loader/asset_preparation/prepare_startup_assets.rs`
- `entrypoints/loader/Cargo.toml`
- `entrypoints/loader/embedded.rs`
- `entrypoints/loader/README.md`
- `entrypoints/loader/tests.rs`
- `game-assets/Cargo.toml`
- `game-assets/lib.rs`
- `kfc-runtime/bridge/lib.rs`
- `kfc-runtime/bridge/player_input.rs`
- `kfc-runtime/bridge/windows/configure_ecs.rs`
- `kfc-runtime/bridge/windows/mod.rs`
- `kfc-runtime/bridge/windows/read_component_registration.rs`
- `kfc-runtime/bridge/windows/read_components.rs`
- `kfc-runtime/bridge/windows/read_resource_scalars.rs`
- `kfc-runtime/bridge/windows/read_resource.rs`
- `kfc-runtime/bridge/windows/read_service.rs`
- `kfc-runtime/bridge/windows/update_component.rs`
- `kfc-runtime/bridge/windows/update_resource_scalar.rs`
- `kfc-runtime/bridge/windows/update_service.rs`
- `modules/loader-config/validate_document.rs`
- `modules/modloader/find_installed_mods.rs`
- `modules/modloader/mod_package_filesystem.rs`
- `modules/README.md`
- `modules/runtime-diagnostics/diagnostics-jobs/README.md`
- `modules/runtime-diagnostics/documents.rs`
- `modules/runtime-diagnostics/README.md`
- `modules/updater/control/request_runtime_mod_action.rs`
- `package.json`

**Nur gezielt gelesen oder durchsucht:**

- `.github/workflows/build.yml`
- `dev/profiles/capture/capture_runtime.ps1`
- `dev/profiles/capture/export_runtime_bindings.py`
- `dev/profiles/checks/verify_component_copy_adapters.py`
- `dev/profiles/cli/discover_function_candidates.cpp`
- `dev/profiles/README.md`
- `docs/website/data/enshrouded-1076226---game38-branches-ea-update-08-2026-06-29t10-27-39-052394z/types.lua`
- `docs/website/tools/validate-website-output.mjs`
- `game-assets/originals.rs`
- `game-assets/transaction.rs`
- `mods/sf-survival/src/mod.lua`
- `mods/sf-unlimited-flight/src/mod.lua`
- `modules/loader-config/lib.rs`
- `modules/loader-config/read_document.rs`
- `modules/loader-config/resolve_document_path.rs`
- `modules/loader-config/write_document.rs`
- `modules/logging/lib.rs`
- `modules/modloader/lib.rs`
- `modules/modloader/plan_mod_load_order.rs`
- `modules/modloader/ui/host/mod.rs`
- `modules/modloader/ui/host/read_configuration_status.rs`
- `modules/modloader/ui/host/read_runtime_profiles.rs`
- `modules/runtime-diagnostics/diagnostics-jobs/mod.rs`
- `modules/updater`
- `modules/updater/lib.rs`

Repositoryweite Aufrufersuchen dienten der Pfad- und Integrationsprüfung. Ein Suchtreffer bedeutet keine vollständige Inhaltsprüfung der zugehörigen Datei. Der zusätzlich vollständig gelesene Coding-Skill liegt außerhalb des Repositorys unter `C:\Users\mail\.codex\skills\coding\SKILL.md`.

Die gesamte übrige Quelle ist außerhalb dieses Audits. Insbesondere wurden `kfc-parser`, native Providerimplementierungen, lokale Installationen, persönliche Einstellungen, Logs, Buildausgaben und historische Evidenz nicht vollständig auditiert und nicht verändert.
