# Spielbuilds untersuchen und Runtimeprofile erstellen

Hier liegt die Implementierung der wiederverwendbaren Entwicklungsaktionen. Jede Aktionsdatei nennt ihre Handlung. Die Dateien unter `checks/` enthalten konkrete technische Nachweise. Sie werden ausdrücklich gestartet.

| Aufgabe | Implementierung |
| --- | --- |
| EXE-Metadaten lesen | `inspect_executable.cpp` |
| Spielbuild auswählen und seine Belege aufnehmen | `select_executable_build.cpp` |
| Funktionskandidaten finden | `discover_function_candidates.cpp` |
| Funktionssignaturen suchen | `scan_function_signatures.cpp` |
| Funktionskatalog aus einem Profil exportieren | `export_profile_functions.cpp` |
| Prozessmodule erfassen | `capture_process_modules.cpp` |
| Alte ECS-Textausgabe übernehmen | `import_legacy_ecs_capture.cpp` |
| Provider-ABI prüfen | `inspect_provider_abi.cpp` |
| Profilentwurf erstellen | `create_runtime_profile.cpp` |
| Profil prüfen | `validate_runtime_profile.cpp` |
| Geprüftes Profil freigeben | `approve_runtime_profile.cpp` |
| Runtime und optionale Prozessdaten erfassen | `capture_runtime.py` |
| ECS-Registrierungen erfassen oder exportieren | `capture_ecs_registry.py` |
| Typ anhand von Namen oder Hash suchen | `lookup_runtime_type.py` |
| Zwei erfasste Spielbuilds vergleichen | `compare_runtime_builds.py` |
| Signaturen bis zu Originalfunktionen und Aufrufern verfolgen | `trace_function_anchors.py` |
| KFC-Ressourcen exportieren | `export_kfc_resources.rs` |
| Systemabhängigkeiten auflösen | `inspect_system_dependencies.rs` |
| Bereits beobachtete Registrierung eindeutig exportieren | `export_observed_registry.py` |
| Fabriken und verknüpfte Inventare erfassen | `capture_factory_state.py` |
| Rezeptressourcen vergleichen | `compare_recipe_resources.py` |

`profile_commands.cpp` ordnet die bestehenden Konsolenbefehle ihren Implementierungen zu. Die gemeinsame PE-Verarbeitung liegt in `pe_image.cpp` und `pe_image.py`. Der Signaturmatcher liegt in `function_signatures.cpp`. Die Python-Dateien für Reflection, Prozessspeicher, Komponentenregistrierungen und Funktionsindizes besitzen jeweils diese konkrete Aufgabe.

## Zuerst ausdrücklich bauen

Die folgenden Befehle werden im Repository ausgeführt:

```powershell
cmake -S . -B build/native-runtime -A x64
cmake --build build/native-runtime --config Release --target kfc-runtime-dev
cmake --build build/native-runtime --config Release --target kfc-runtime-verify-component-registry
cargo build -p shroudforge-api --example export_kfc_resources --example inspect_system_dependencies --offline
```

Die native Konsole liegt danach unter `build/native-runtime/tools/kfc-runtime-dev.exe`. Die Rust-Programme liegen unter `target/debug/examples/`. Ein gesetztes `CARGO_TARGET_DIR` bestimmt deren tatsächlichen Ausgabeort.

## Eine konkrete Handlung ausführen

```powershell
$exe = 'D:/Games/Enshrouded/enshrouded.exe'
$capture = 'dev/profiles/captures/my-build'
build/native-runtime/tools/kfc-runtime-dev.exe inspect $exe --out "$capture/image-report.json"
python dev/profiles/capture_runtime.py $exe --functions --out $capture
python dev/profiles/lookup_runtime_type.py $capture 'keen::ecs::CurrentTransform'
cargo run -p shroudforge-api --example export_kfc_resources --offline -- $exe "$capture/resources.json"
```

Der KFC-Exporter verwendet dieselbe Implementierung für die Attributauswahl, ausdrücklich gewählte `--type`-Einträge und die bisherige Baustoffauswahl mit `--building-inputs`. Die früheren Cargo-Beispielnamen bleiben verfügbar und kompilieren dieselbe Datei. Die ItemInfo-Feldprojektion der Baustoffauswahl bleibt erhalten.

Für die komplette vorhandene Client-/Servererfassung gibt es `capture_runtime.ps1`. Diese Aktion führt die einzelnen Erfassungsschritte in sichtbarer Reihenfolge aus. Benötigte Programme werden vorher gebaut oder mit `-NativeVerifier` und `-ResourceExporter` übergeben. Die Aktion baut keine Programme nebenbei.

`-VerifyBufferAdapters` startet den konkreten Differentialnachweis. Der bereits vorhandene Rust-Korpustest wird ausschließlich mit `-RunCallbackCorpusTests` oder direkt über `checks/run_callback_corpus_tests.ps1` ausgewählt. Eine normale Erfassung startet keine Tests.

## Quellen, Ergebnisse und Freigaben

| Ort | Inhalt |
| --- | --- |
| `catalogs/` | Gepflegte Funktionssignaturen und kleine erfasste Buildindizes. |
| `schemas/` | Verträge für Profile und Entwicklungskataloge. |
| `captures/` | Ignorierte lokale Eingaben, Originaldateien, Aufnahmen und Entwürfe. |
| `checks/` | Ausdrücklich ausführbare technische Nachweise. |
| `tests/` | Bereits vorhandene Entwicklungswerkzeugtests. |
| `../../kfc-runtime/profiles/` | Ausgelieferte Runtimeprofile. |
| `../../docs/runtime-investigations/` | Untersuchungsberichte und geschlossene historische Quellbelege. |

`select-build` sucht standardmäßig die Kataloge dieses Profilbereichs. `--catalog-directory` und `--out-dir` wählen ausdrücklich andere vorhandene Eingaben beziehungsweise Ausgaben. Der Aufruf hängt nicht mehr vom zufälligen Arbeitsverzeichnis ab.

Die [ausführliche Anleitung](../../docs/sf/runtime-discovery.md) erklärt Erfassungsformate, Bildidentität, Auflösung und bekannte Grenzen. Freigabeschritte und exakte SHA-256-Zuordnung bleiben bestehen. Ein Funktionsfund ist keine Freigabe für einen Engineaufruf.
