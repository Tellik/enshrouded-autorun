# Spielbuilds untersuchen und Runtimeprofile erstellen

Native Entwicklungsaktionen liegen unter `dev/profiles/`. Die Python-Implementierungen liegen gemeinsam unter `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/`. Diagnosejobs verwenden sie über denselben Jobablauf wie Lua und Modaktionen. Jede Aktionsdatei nennt ihre Handlung. Technische Nachweise werden ausdrücklich gestartet.

| Aufgabe | Implementierung |
| --- | --- |
| EXE-Metadaten lesen | `dev/profiles/cli/inspect_executable.cpp` |
| Spielbuild auswählen und seine Belege aufnehmen | `dev/profiles/cli/select_game_build.cpp` |
| Funktionskandidaten finden | `dev/profiles/cli/discover_function_candidates.cpp` |
| Funktionssignaturen suchen | `dev/profiles/cli/scan_function_signatures.cpp` |
| Funktionskatalog aus einem Profil exportieren | `dev/profiles/cli/export_profile_function_catalog.cpp` |
| Prozessmodule erfassen | `dev/profiles/cli/capture_process_modules.cpp` |
| Alte ECS-Textausgabe übernehmen | `dev/profiles/cli/import_legacy_ecs_capture.cpp` |
| Provider-ABI prüfen | `dev/profiles/cli/inspect_provider_abi.cpp` |
| Profilentwurf erstellen | `dev/profiles/cli/create_runtime_profile_draft.cpp` |
| Profil prüfen | `dev/profiles/cli/validate_runtime_profile.cpp` |
| Geprüftes Profil freigeben | `dev/profiles/cli/approve_runtime_profile.cpp` |
| Runtime und optionale Prozessdaten erfassen | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/capture/discover_runtime_build.py` |
| ECS-Registrierungen erfassen oder exportieren | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/capture/capture_ecs_registry.py` |
| Typ anhand von Namen oder Hash suchen | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/analyze/find_runtime_type.py` |
| Zwei erfasste Spielbuilds vergleichen | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/compare/compare_runtime_builds.py` |
| Signaturen bis zu Originalfunktionen und Aufrufern verfolgen | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/analyze/trace_function_anchors.py` |
| KFC-Ressourcen exportieren | `dev/profiles/inspect/export_kfc_resources.rs` |
| ECS-Systemabhängigkeiten aus einer Live-Erfassung auflösen | Runtime Diagnostics: `inspect.system-dependencies` nach `capture.runtime` |
| Bereits beobachtete Registrierung eindeutig exportieren | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/capture/export_observed_registry.py` |
| Fabriken und verknüpfte Inventare erfassen | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/capture/capture_factory_state.py` |
| Rezeptressourcen vergleichen | `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/compare/compare_recipe_resources.py` |

`dev/profiles/cli/kfc_runtime_dev.cpp` ordnet die bestehenden Konsolenbefehle ihren Implementierungen zu. Die gemeinsame PE-Verarbeitung liegt in `dev/profiles/cli/read_pe_image.cpp` und `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/inspect/read_pe_image.py`. Der Signaturmatcher liegt in `dev/profiles/cli/match_function_signatures.cpp`. Die Python-Unterordner behalten ihre bisherigen Zuständigkeiten.

## Zuerst ausdrücklich bauen

Die folgenden Befehle werden im Repository ausgeführt:

```powershell
cmake -S . -B build/native-runtime -A x64
cmake --build build/native-runtime --config Release --target kfc-runtime-dev
cmake --build build/native-runtime --config Release --target kfc-runtime-verify-component-registry
cargo build -p shroudforge-api --example export_kfc_resources --offline
```

Die native Konsole liegt danach unter `build/native-runtime/tools/kfc-runtime-dev.exe`. Die Rust-Programme liegen unter `target/debug/examples/`. Ein gesetztes `CARGO_TARGET_DIR` bestimmt deren tatsächlichen Ausgabeort.

## Eine konkrete Handlung ausführen

```powershell
$exe = 'D:/Games/Enshrouded/enshrouded.exe'
$capture = 'dev/profiles/captures/my-build'
$diagnosticPythonSources = 'modules/runtime-diagnostics/diagnostics-jobs/game_inspection'
build/native-runtime/tools/kfc-runtime-dev.exe inspect $exe --out "$capture/image-report.json"
python "$diagnosticPythonSources/capture/discover_runtime_build.py" $exe --functions --out $capture
python "$diagnosticPythonSources/analyze/find_runtime_type.py" $capture 'keen::ecs::CurrentTransform'
cargo run -p shroudforge-api --example export_kfc_resources --offline -- $exe "$capture/resources.json"
```

Der KFC-Exporter verwendet dieselbe Implementierung für die Attributauswahl, ausdrücklich gewählte `--type`-Einträge und die bisherige Baustoffauswahl mit `--building-inputs`. Die früheren Cargo-Beispielnamen bleiben verfügbar und kompilieren dieselbe Datei. Die ItemInfo-Feldprojektion der Baustoffauswahl bleibt erhalten.

Für die komplette vorhandene Client-/Servererfassung gibt es `dev/profiles/capture/capture_runtime.ps1`. Diese Aktion führt die einzelnen Erfassungsschritte in sichtbarer Reihenfolge aus. Benötigte Programme werden vorher gebaut oder mit `-NativeVerifier` und `-ResourceExporter` übergeben. Die Aktion baut keine Programme nebenbei.

Die Python-Abhängigkeiten stehen unter `modules/runtime-diagnostics/diagnostics-jobs/game_inspection/requirements.txt`. Die [Diagnosejobs](../../modules/runtime-diagnostics/diagnostics-jobs/README.md) bieten dieselben Programme über das vorhandene Jobformat und die vorhandene Steuerung an.

Diagnosejobs zeigen ihren Ablauf im Debug Log und schreiben Berichte sowie Wiederherstellungsdateien in den konfigurierten Exportordner unter `diagnostics/<job-id>/`. Der Job `capture.factory-state` liest die benötigten `reflection.json` und `components.json` aus dem Export von `capture.runtime`. Die oben gezeigten direkten Entwicklungsaufrufe behalten ihre ausdrücklich gewählten Capturepfade.

`-VerifyBufferAdapters` startet den konkreten Differentialnachweis. Der bereits vorhandene Rust-Korpustest wird ausschließlich mit `-RunCallbackCorpusTests` oder direkt über `dev/profiles/checks/run_callback_corpus_tests.ps1` ausgewählt. Eine normale Erfassung startet keine Tests.

## Quellen, Ergebnisse und Freigaben

| Ort | Inhalt |
| --- | --- |
| `catalogs/` | Gepflegte Funktionssignaturen und kleine erfasste Buildindizes. |
| `schemas/` | Verträge für Profile und Entwicklungskataloge. |
| `captures/` | Ignorierte lokale Eingaben, Originaldateien, Aufnahmen und Entwürfe. |
| `checks/` | Ausdrücklich ausführbare technische Nachweise. |
| `modules/runtime-diagnostics/tests/python/` | Entwicklungswerkzeugtests für die übernommenen Python-Implementierungen. |
| `../../modules/game-runtime/profiles/` | Ausgelieferte Runtimeprofile. |
| `../evidence/runtime/` | Untersuchungsberichte und geschlossene historische Quellbelege. |

`select-build` sucht standardmäßig die Kataloge dieses Profilbereichs. `--catalog-directory` und `--out-dir` wählen ausdrücklich andere vorhandene Eingaben beziehungsweise Ausgaben. Der Aufruf hängt nicht mehr vom zufälligen Arbeitsverzeichnis ab.

Die [frühere Runtime-Discovery-Anleitung](../evidence/runtime/previous-runtime-discovery.en.md) dokumentiert den damaligen Stand. Diese Seite beschreibt die heute vorhandenen Entwicklungsaktionen. Freigabeschritte und exakte SHA-256-Zuordnung bleiben bestehen. Ein Funktionsfund ist keine Freigabe für einen Engineaufruf.
