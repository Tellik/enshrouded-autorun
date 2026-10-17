# Rust-Aufrufe an die native Runtime

Diese Bibliothek verbindet die Lua-Ausführung mit der kanonischen C-ABI unter `../native/abi/runtime.h`. `execution` verwendet die Bibliothek als normale Cargo-Abhängigkeit.

Unter `windows/` liegt jede tatsächliche Aufrufhandlung in ihrer eigenen Datei:

| Handlung | Datei |
| --- | --- |
| Provider laden und ABI prüfen | `windows/load_native_provider.rs` |
| C-Datentypen und Funktionssignaturen | `windows/native_abi.rs` |
| ECS-Registrierungen lesen | `windows/read_component_registry.rs` |
| Entities suchen | `windows/query_entities.rs` |
| Komponente lesen | `windows/read_component.rs` |
| Komponenten als Stapel lesen | `windows/read_components.rs` |
| Komponente geschützt aktualisieren | `windows/update_component.rs` |
| Ressource lesen | `windows/read_resource.rs` |
| Ressourcenskalar geschützt aktualisieren | `windows/update_resource_scalar.rs` |
| Dienst lesen oder aktualisieren | `windows/read_service.rs`, `windows/update_service.rs` |
| Originalen Baueingabeauftrag übergeben | `player_input.rs` |

Die übrigen Aktionsdateien folgen demselben Namensschema. `windows/mod.rs` ordnet die bisherigen Rust-Funktionsnamen den Implementierungen zu. Die vorhandenen Plattformantworten für andere Betriebssysteme liegen in `unsupported_platform.rs`.

Die gemeinsame Messung der Runtimeoperationen gehört zum Diagnosemodul. Bridge und Lua-Ausführung verwenden dieselbe Implementierung unter `modules/runtime-diagnostics/measure_runtime_operations.rs`.
