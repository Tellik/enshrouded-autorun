# Einzeldateianalyse des Repositorys

Dieser Bericht dokumentiert den damaligen Struktur- und Auditstand. Die anschließende Reduzierung mit aktuellen Dateinamen, Löschungen und Prüfungen steht im [Reduzierungsbericht](reduction-report.md). Prüfsummen und Pfadlisten dieser Strukturphase sind historische Momentaufnahmen.

Stand: 17. Oktober 2026. Basis: `81f9c5f`. Dieser Anhang enthält die vollständige Inhaltsanalyse aller **287 eigenen Codedateien mit 85.865 physischen Zeilen**. Leseanteil **287 / 287 = 100 %**. „Nachgewiesen“ bezeichnet statische Codebelege, keinen ausgeführten Spieltest. Tests und Builds wurden nicht ausgeführt.

Die Herkunftsgruppen dienen nur der Navigation. Die neue funktionale Zuordnung steht pro Datei und im [Gesamtinventar](refactor-file-inventory.csv). Architektur und Migration stehen im [Bericht A–J](refactor-masterplan.md). Externe und generierte Dateien werden im [Abdeckungsnachweis](refactor-audit-coverage.md) separat ausgewiesen.

## Einstieg, Build und Parser

18 Dateien, 3.589 Zeilen.

### [build.ps1](../../build.ps1)

Vollständig gelesen, 174 Zeilen. SHA-256: `7597f406c979fe4b08d47e04b2d2b1e7aefdaa27d11a22ed860e80277b77d0ae`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Baut UI, native Runtime, Proxy-DLLs, Rustworkspace und Bootstrap, prüft Mods und erstellt ReleaseZIP mit ManagedPaths und SHA256. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Baut UI, native Runtime, Proxy-DLLs, Rustworkspace und Bootstrap, prüft Mods und erstellt ReleaseZIP mit ManagedPaths und SHA256. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Assert-BuildPath, Copy-ShroudForgeMods, versionManifest` |
| 4. Aufrufer und Verbraucher | Root-Dokumentation und GitHub-Buildworkflow |
| 5. Verwendete Komponenten | npm, Cargo, VisualStudio/MSBuild, CMake, VERSION, mods |
| 6. Eingaben und Ausgaben | Eingaben: Buildnummer, Schalter und Quellen Ausgabe: ReleaseZIP, Hash und Binaries |
| 7. Seiteneffekte und Ressourcen | startet Buildwerkzeuge und Tests, kopiert/bereinigt Buildausgaben |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Eigenständige Lua-Tokenprüfung neben Rust-Vertragstests. Cargo-Test ignoriert konfigurierten Targetdir. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Build und Release. Technische Kopplung: npm, Cargo, VisualStudio/MSBuild, CMake, VERSION, mods. Aufruferbezug: Root-Dokumentation und GitHub-Buildworkflow |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Build und Releasepaketierung. Neue Einordnung: Build und Release. Besitzer: Build-Einstieg. |

**Belege:** Assert-BuildPath, Copy-ShroudForgeMods, versionManifest

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** beibehalten. Ein auffindbarer Root-Einstieg bleibt. Konkrete Buildschritte können später nach tools/build extrahiert werden, ohne einen zweiten Einstieg einzuführen.

**Vorgeschlagene Zielpfade:** `build.ps1`.

### [src/bootstrap/windows/bootstrap.cpp](../../entrypoints/windows-bootstrap/startup.cpp)

Vollständig gelesen, 731 Zeilen. SHA-256: `dc4c881c8dd66943312a633547160ed946d75ffda0609726b965105d986e298c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Steuert frühen Spielstart mit Gate, Updatercheck, Logsession, UI-Prozessen, Assetvorbereitung, nativer Runtime und Rust-Lifecycle-Tick sowie Shutdown. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Steuert frühen Spielstart mit Gate, Updatercheck, Logsession, UI-Prozessen, Assetvorbereitung, nativer Runtime und Rust-Lifecycle-Tick sowie Shutdown. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `36: std::string single_line(std::string_view value) {`<br>`47: std::string windows_error(DWORD code) {`<br>`68: struct LogGuard {`<br>`83: bool is_dedicated_server_process() {`<br>`100: void begin_log_session(const std::filesystem::path& root) {`<br>`139: void log(char level, const std::string& message) {`<br>`165: void log(const std::string& message) {`<br>`169: void log_shroudforge_banner(bool dedicated_server) {`<br>`183: void write_runtime_heartbeat(const std::filesystem::path& root) {`<br>`201: void startup_stage(const std::filesystem::path& root, const char* stage) {`<br>`222: std::string runtime_status_state(std::string_view status) {`<br>`237: void show_startup_error(const std::filesystem::path& root, const std::string& detail);`<br>`239: void start_recovery_update_window(const std::filesystem::path& root) {`<br>`265: bool run_startup_update_check(const std::filesystem::path& root) {`<br>`321: void startup_failed(const std::filesystem::path& root, const char* stage,`<br>`327: void show_startup_error(const std::filesystem::path& root, const std::string& detail) {`<br>`337: void clear_runtime_heartbeat(const std::filesystem::path& root) {`<br>`342: void start_debug_console(const std::filesystem::path& root) {`<br>`403: void stop_debug_console() {`<br>`424: void start_modloader_ui(const std::filesystem::path& root) {`<br>`459: void stop_modloader_ui() {`<br>`476: void start_world_editor_ui(const std::filesystem::path& root) {`<br>`510: void stop_world_editor_ui() {`<br>`526: void start_pending_update(const std::filesystem::path& root) {`<br>`558: DWORD WINAPI run(void*) {`<br>`692: bool is_supported_game_process() {`<br>`713: BOOL APIENTRY DllMain(HMODULE module, DWORD reason, LPVOID reserved) {` |
| 4. Aufrufer und Verbraucher | winmm DllMain im unterstützten Spielprozess |
| 5. Verwendete Komponenten | Win32 Prozesse/Events, startup_gate, RuntimeDLL-FFI, C++ Loggingconfig |
| 6. Eingaben und Ausgaben | Eingaben: DLL-Attach, Root und Stopereignis Ausgabe: Startfreigabe und Runtimebetrieb |
| 7. Seiteneffekte und Ressourcen | startet Prozesse, lädt DLLs, schreibt Logs/Heartbeat, ruft Lifecycle |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Frühstart-Gate, Kindprozesse und Runtime-Lifecycle sind konkret trennbare Aufgaben. DllMain-/Loaderlockgrenzen erhalten. Befunde: Startablauf, Logwriter, Fensterprozessmanagement, Heartbeat und spezifischer WorldEditorstart sind zusammen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Früher Prozessstart. Technische Kopplung: Win32 Prozesse/Events, startup_gate, RuntimeDLL-FFI, C++ Loggingconfig. Aufruferbezug: winmm DllMain im unterstützten Spielprozess |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | startup_gate_dll.cpp, startup_gate_exe.cpp und tests/run.ps1 prüfen Gate, nicht gesamten Bootstrap. Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Windows-Prozessstart und Runtimekoordination. Neue Einordnung: Früher Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** DllMain, run, run_startup_update_check, start_pending_update, start_debug_console, write_runtime_heartbeat

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** aufteilen. Frühstart-Gate, Kindprozesse und Runtime-Lifecycle sind konkret trennbare Aufgaben. DllMain-/Loaderlockgrenzen erhalten.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/startup.cpp`, `entrypoints/windows-bootstrap/processes.cpp`, `entrypoints/windows-bootstrap/lifecycle.cpp`, `entrypoints/windows-bootstrap/logging.cpp`.

### [src/bootstrap/windows/logging_config.h](../../entrypoints/windows-bootstrap/logging_config.h)

Vollständig gelesen, 56 Zeilen. SHA-256: `2f067e055740519e64df082cceb85842dc48335edca2ae75a4d9c367daf7cd3e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest Loaderkonfiguration für frühen C++-Start, cached 500ms und löst Pfade, Level und Modulaktivierung auf. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest Loaderkonfiguration für frühen C++-Start, cached 500ms und löst Pfade, Level und Modulaktivierung auf. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Read, Directory, Allows, ModuleEnabled` |
| 4. Aufrufer und Verbraucher | bootstrap.cpp über ShroudforgeConfig::* |
| 5. Verwendete Komponenten | nlohmann JSON, Dateisystem |
| 6. Eingaben und Ausgaben | Eingaben: Root, Pfadschlüssel oder Log-Level Ausgabe: Configsnapshot, Pfad und Bool |
| 7. Seiteneffekte und Ressourcen | liest Loaderconfig mit 500msCache |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Teilinterpretation derselben Config in C++, technisch vor Ruststart erforderlich. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: nlohmann JSON, Dateisystem. Aufruferbezug: bootstrap.cpp über ShroudforgeConfig::* |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Frühstartkonfigurationsadapter. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** Read, Directory, Allows, ModuleEnabled

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/logging_config.h`.

### [src/bootstrap/windows/runtime_bridge.cpp](../../entrypoints/windows-bootstrap/runtime_bridge.cpp)

Vollständig gelesen, 48 Zeilen. SHA-256: `db3cfe8f74abe9b12a44d0f4b2d5cbb787bcb45dd335c0f10f4b1c211494bfe3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt kfc-runtime.dll und prüft ABI sowie Pflichtsymbole, ruft deren Initialize/Tick/Status/Shutdown auf. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt kfc-runtime.dll und prüft ABI sowie Pflichtsymbole, ruft deren Initialize/Tick/Status/Shutdown auf. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14: std::string error{"provider-not-loaded"};`<br>`17: bool Initialize() {`<br>`42: void Tick() { if (tick) tick(); }`<br>`43: void Shutdown() { if (shutdown) shutdown(); }`<br>`44: std::string Status() {` |
| 4. Aufrufer und Verbraucher | bootstrap.cpp run |
| 5. Verwendete Komponenten | Windows loader, native/runtime.h |
| 6. Eingaben und Ausgaben | Eingaben: DLL-Pfad und Lifecycle-Aufruf Ausgabe: Initialisierungsresultat oder Status |
| 7. Seiteneffekte und Ressourcen | LoadLibrary, GetProcAddress, native Aufrufe |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Eigener enger C++-Zugang zur selben nativen Runtime, semantisch kein KFC-Dateiparser. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: Windows loader, native/runtime.h. Aufruferbezug: bootstrap.cpp run |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Native DLL-Anbindung des Bootstrap. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** EcsRuntime::Initialize, GetProcAddress, KfcRuntimeAbi

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/runtime_bridge.cpp`.

### [src/bootstrap/windows/runtime_bridge.h](../../entrypoints/windows-bootstrap/runtime_bridge.h)

Vollständig gelesen, 10 Zeilen. SHA-256: `d36d150353db058bbb6690cff23267c88631583046e4b1ae3c9aae32552d1e88`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert vier native Runtime-Lifecycle-Aufrufe für den Bootstrap. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert vier native Runtime-Lifecycle-Aufrufe für den Bootstrap. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `6: bool Initialize();`<br>`7: void Tick();`<br>`8: std::string Status();`<br>`9: void Shutdown();` |
| 4. Aufrufer und Verbraucher | bootstrap.cpp und runtime_bridge.cpp |
| 5. Verwendete Komponenten | std::string |
| 6. Eingaben und Ausgaben | Eingaben: Lifecycle-Aufruf Ausgabe: Funktionsdeklarationen |
| 7. Seiteneffekte und Ressourcen | keine |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: std::string. Aufruferbezug: bootstrap.cpp und runtime_bridge.cpp |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Interner Runtime-Bridge-Vertrag. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** EcsRuntime::Initialize/Tick/Status/Shutdown

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/runtime_bridge.h`.

### [src/bootstrap/windows/startup_gate.h](../../entrypoints/windows-bootstrap/startup_gate.h)

Vollständig gelesen, 68 Zeilen. SHA-256: `4c908af80a76a7a4d0a7a966e183b423d33d519f990e5a79ddc703cb7b8d42fe`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Parkt den EXE-Einstieg außerhalb des Loaderlocks bis Vorbereitung abgeschlossen ist, stellt originale14Instruktionsbytes wieder her und führt Einstieg aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Parkt den EXE-Einstieg außerhalb des Loaderlocks bis Vorbereitung abgeschlossen ist, stellt originale14Instruktionsbytes wieder her und führt Einstieg aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `StartupGate::Install, Run, Complete` |
| 4. Aufrufer und Verbraucher | bootstrap DllMain und StartupGate-TestDLL |
| 5. Verwendete Komponenten | Win32 Events, PE-Header, VirtualProtect, FlushInstructionCache |
| 6. Eingaben und Ausgaben | Eingaben: PE-Einstieg und Vorbereitungsabschluss Ausgabe: Startfreigabe oder Prozessabbruch |
| 7. Seiteneffekte und Ressourcen | ändert Einstiegscode, hält Events und wartet |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Verändert vorübergehend EXE-Einstieg im Prozess. Verschiebung darf DLL-Initialisierungsreihenfolge nicht ändern. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: Win32 Events, PE-Header, VirtualProtect, FlushInstructionCache. Aufruferbezug: bootstrap DllMain und StartupGate-TestDLL |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | startup_gate_dll.cpp, startup_gate_exe.cpp und tests/run.ps1 prüfen Gate, nicht gesamten Bootstrap. Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Frühe Windows-Startsynchronisierung. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** StartupGate::Install, Run, Complete

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/startup_gate.h`.

### [src/bootstrap/windows/tests/run.ps1](../../entrypoints/windows-bootstrap/tests/run.ps1)

Vollständig gelesen, 31 Zeilen. SHA-256: `cb4b0f8a0338887443656839121982d99de691d3a47ade4fc8d6ad0adce75e9b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Kompiliert StartupGate-TestDLL und TestEXE mit MSVC und prüft Erfolg sowie absichtlichen Vorbereitungsfehler mit Timeout. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Kompiliert StartupGate-TestDLL und TestEXE mit MSVC und prüft Erfolg sowie absichtlichen Vorbereitungsfehler mit Timeout. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `ProcessStartInfo, SF_GATE_TEST_FAIL` |
| 4. Aufrufer und Verbraucher | build.ps1 falls Tests aktiv |
| 5. Verwendete Komponenten | VisualStudio, cl.exe, TestDLL/TestEXE |
| 6. Eingaben und Ausgaben | Eingaben: MSVC-Umgebung und Testquellen Ausgabe: Testerfolg oder Fehler |
| 7. Seiteneffekte und Ressourcen | baut und startet Testbinaries |
| 8. Art der Verantwortung | Test |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Testskript, kein ausgelieferter Prozess. Nicht ausgeführt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: VisualStudio, cl.exe, TestDLL/TestEXE. Aufruferbezug: build.ps1 falls Tests aktiv |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Nativer Integrationsprüfungsstarter. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** ProcessStartInfo, SF_GATE_TEST_FAIL

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/tests/run.ps1`.

### [src/bootstrap/windows/tests/startup_gate_dll.cpp](../../entrypoints/windows-bootstrap/tests/startup_gate_dll.cpp)

Vollständig gelesen, 28 Zeilen. SHA-256: `ec0eed5fe6b5ba72f8ad1f0feab8ba3fd9fb2ebc5915f5da92a86ec3bc2d576e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Installiert Testgate beim DLL-Attach und beendet Vorbereitung nach echtem DLL-Laden außerhalb DllMain, exportiert Verifikationsfunktion. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Installiert Testgate beim DLL-Attach und beendet Vorbereitung nach echtem DLL-Laden außerhalb DllMain, exportiert Verifikationsfunktion. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `3: DWORD WINAPI Prepare(void*) {`<br>`21: BOOL WINAPI DllMain(HMODULE, DWORD reason, LPVOID) {` |
| 4. Aufrufer und Verbraucher | startup_gate_exe und run.ps1 |
| 5. Verwendete Komponenten | startup_gate.h, Win32 Threads/Events, version.dll |
| 6. Eingaben und Ausgaben | Eingaben: DLL-Attach und Fehler-Umgebungsvariable Ausgabe: GateVerified-Ergebnis |
| 7. Seiteneffekte und Ressourcen | installiert Gate und startet Vorbereitungsworker |
| 8. Art der Verantwortung | Test |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Testfixture verändert EXE-Einstieg. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: startup_gate.h, Win32 Threads/Events, version.dll. Aufruferbezug: startup_gate_exe und run.ps1 |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: StartupGate-Testfixture. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** Prepare, GateVerified, DllMain

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/tests/startup_gate_dll.cpp`.

### [src/bootstrap/windows/tests/startup_gate_exe.cpp](../../entrypoints/windows-bootstrap/tests/startup_gate_exe.cpp)

Vollständig gelesen, 7 Zeilen. SHA-256: `c743d7fb2a401f62a8a4ff6d34bb926fd5882148507e0a334e4436130670c8b0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft beim Programmeinstieg dass Gate abgeschlossen und Originalinstruktionen wiederhergestellt sind. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft beim Programmeinstieg dass Gate abgeschlossen und Originalinstruktionen wiederhergestellt sind. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main` |
| 4. Aufrufer und Verbraucher | tests/run.ps1 |
| 5. Verwendete Komponenten | GateVerified aus TestDLL |
| 6. Eingaben und Ausgaben | Eingaben: GateVerified-Ergebnis Ausgabe: Exitcode und Text |
| 7. Seiteneffekte und Ressourcen | stdout |
| 8. Art der Verantwortung | Test |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: GateVerified aus TestDLL. Aufruferbezug: tests/run.ps1 |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: StartupGate-Testprogramm. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** main

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/tests/startup_gate_exe.cpp`.

### [src/bootstrap/windows/winmm_proxy.cpp](../../entrypoints/windows-bootstrap/winmm_proxy.cpp)

Vollständig gelesen, 182 Zeilen. SHA-256: `73cd9993842e5da033ce62ee59d585d8093d3ef79e5fb8a907a8e74d41f9f4b3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert Linker-Weiterleitungen sämtlicher deklarierter winmm-Exporte an Windows-Systembibliothek. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert Linker-Weiterleitungen sämtlicher deklarierter winmm-Exporte an Windows-Systembibliothek. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `pragma comment(linker /export ...)` |
| 4. Aufrufer und Verbraucher | Bootstrap-vcxproj als Kompilationseinheit |
| 5. Verwendete Komponenten | MSVC-Linker, c:/windows/system32/winmm |
| 6. Eingaben und Ausgaben | Eingaben: MSVC-Kompilation Ausgabe: PE-Exportweiterleitungen |
| 7. Seiteneffekte und Ressourcen | Windows-Systemexporte in erstellter DLL |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Systempfade und Exportordinale gehören zur Windows-Integration. Keine Updatelogik. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Prozessstart. Technische Kopplung: MSVC-Linker, c:/windows/system32/winmm. Aufruferbezug: Bootstrap-vcxproj als Kompilationseinheit |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Windows-Lifecycle, Exportnamen und Ressourcenlebensdauer erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Windows DLL-Exportweiterleitung. Neue Einordnung: Prozessstart. Besitzer: Windows-Bootstrap. |

**Belege:** pragma comment(linker /export ...)

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/windows-bootstrap/winmm_proxy.cpp`.

### [src/loader/compatibility/src/lib.rs](../../kfc-runtime/compatibility/lib.rs)

Vollständig gelesen, 116 Zeilen. SHA-256: `fe9abfb6a0540a5b351b81215f420f6a85bb35583cfecd9eb5c0f11574cdd6b1`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verbindet GameSchema mit deklarierter Verfügbarkeit einzelner Runtimeoperationen und liefert begründete Unverfügbarkeit für unbekannte Namen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verbindet GameSchema mit deklarierter Verfügbarkeit einzelner Runtimeoperationen und liefert begründete Unverfügbarkeit für unbekannte Namen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10: pub enum Availability {`<br>`16: pub struct GameContract {`<br>`22: pub fn schema(&self) -> &GameSchema {`<br>`26: pub fn into_schema(self) -> GameSchema {`<br>`30: pub fn runtime(&self, operation: &str) -> Availability {`<br>`42: pub fn has_runtime(&self, operation: &str) -> bool {`<br>`48: pub struct Compatibility {`<br>`53: pub fn new(verified_operations: impl IntoIterator<Item = impl Into<String>>) -> Self {`<br>`62: pub fn unavailable(mut self, operation: impl Into<String>, reason: impl Into<String>) -> Self {`<br>`72: pub fn resolve(self, schema: GameSchema) -> GameContract {`<br>`88: fn unknown_runtime_operation_is_scoped_and_explained() {`<br>`104: fn missing_unrelated_type_does_not_disable_provider_operations() {` |
| 4. Aufrufer und Verbraucher | Workflow::pregame und API über Compatibility::resolve |
| 5. Verwendete Komponenten | shroudforge_parser::GameSchema, serde, BTreeMap |
| 6. Eingaben und Ausgaben | Eingaben: GameSchema und Operationsverfügbarkeit Ausgabe: GameContract und Availability |
| 7. Seiteneffekte und Ressourcen | keine |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Deklarierte Verfügbarkeit ist keine Prüfung momentaner Live-Bereitschaft. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Build-Kompatibilität. Technische Kopplung: shroudforge_parser::GameSchema, serde, BTreeMap. Aufruferbezug: Workflow::pregame und API über Compatibility::resolve |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: unknown_runtime_operation_is_scoped_and_explained, missing_unrelated_type_does_not_disable_provider_operations (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Spiel-API-Verfügbarkeitsmodell. Neue Einordnung: Build-Kompatibilität. Besitzer: Spiel-API-Verfügbarkeit. |

**Belege:** Availability, GameContract, Compatibility

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/compatibility/lib.rs`.

### [src/loader/workflow/lib.rs](../../entrypoints/loader/embedded.rs)

Vollständig gelesen, 261 Zeilen. SHA-256: `33bb67596dd381a07ced8a779c09b1687a41417276e69b07e95272824e7e578f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert Rust-C-ABI für Startvorbereitung/Create/Update/Destroy und kapselt ModLoader mit IngameRuntime samt Panicgrenze. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert Rust-C-ABI für Startvorbereitung/Create/Update/Destroy und kapselt ModLoader mit IngameRuntime samt Panicgrenze. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `13: pub enum LoaderError {`<br>`23: pub struct LoadReport {`<br>`27: pub struct ModLoader {`<br>`34: pub fn new(api: ShroudForgeApi, is_client: bool) -> Self {`<br>`42: pub fn load_directory(&mut self, game: impl AsRef<Path>) -> Result<LoadReport, LoaderError> {`<br>`66: pub fn update(&mut self, delta_seconds: f64) {`<br>`74: pub struct RuntimeHandle(ModLoader);`<br>`76: fn panic_detail(payload: &(dyn Any + Send)) -> String {`<br>`90: pub unsafe extern "C" fn shroudforge_prepare_startup(game: *const u16) -> bool {`<br>`91: fn path(value: *const u16) -> Option<PathBuf> {`<br>`169: pub unsafe extern "C" fn shroudforge_create(game: *const u16) -> *mut RuntimeHandle {`<br>`170: fn path(value: *const u16) -> Option<PathBuf> {`<br>`184: fn append_runtime_log(game: &Path, message: &str) {`<br>`187: fn create_runtime(game: PathBuf) -> Result<RuntimeHandle, String> {`<br>`241: pub unsafe extern "C" fn shroudforge_update(`<br>`255: pub unsafe extern "C" fn shroudforge_destroy(handle: *mut RuntimeHandle) {` |
| 4. Aufrufer und Verbraucher | Bootstrap GetProcAddress und Rust-Aufrufer von prepare |
| 5. Verwendete Komponenten | API::IngameRuntime, package::ModEnvironment, pregame, updater Headlesscontroller |
| 6. Eingaben und Ausgaben | Eingaben: öffentliche Crate-Aufrufe Ausgabe: Exporte und Ergebnisse der Einstiegsschnittstellen |
| 7. Seiteneffekte und Ressourcen | siehe konkrete actual_role |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Lässt Headless-Konfigurationscontroller aus Updater starten. UTF16-Pfadkonvertierung doppelt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebetteter Loaderstart. Technische Kopplung: API::IngameRuntime, package::ModEnvironment, pregame, updater Headlesscontroller. Aufruferbezug: Bootstrap GetProcAddress und Rust-Aufrufer von prepare |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Loader-Einstieg und Runtime-FFI. Neue Einordnung: Eingebetteter Loaderstart. Besitzer: Loader-Einstieg. |

**Belege:** ModLoader, shroudforge_prepare_startup, shroudforge_create, shroudforge_update, shroudforge_destroy

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** refaktorieren. C-ABI und Panicgrenze bleiben am Einstieg. Headless Runtimekontrolle aus dem Updaterbesitz lösen.

**Vorgeschlagene Zielpfade:** `entrypoints/loader/embedded.rs`.

### [src/loader/workflow/main.rs](../../entrypoints/loader/main.rs)

Vollständig gelesen, 351 Zeilen. SHA-256: `b66cf3f871e96befef8a879f1774337ca98a8cddf4d764fb804102e703262a94`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Routet EXE-Modi auf Fenster/Worker, bietet CLI für Typenexport, Vorbereitung, Modanlage, Restore, Launch und Inspect und öffnet Desktop bei Doppelklick. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Routet EXE-Modi auf Fenster/Worker, bietet CLI für Typenexport, Vorbereitung, Modanlage, Restore, Launch und Inspect und öffnet Desktop bei Doppelklick. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5: fn main() -> Result<(), Box<dyn std::error::Error>> {`<br>`202: fn create_mod(`<br>`298: fn launch_desktop() -> Result<(), Box<dyn std::error::Error>> {`<br>`335: fn game_executable(game: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {`<br>`345: fn detect_files(game: &PathBuf) -> GameFiles {` |
| 4. Aufrufer und Verbraucher | Explorer, Bootstrap und gestartete Worker |
| 5. Verwendete Komponenten | Alle Modulbibliotheken, Parser, API und package, Windows MessageBox |
| 6. Eingaben und Ausgaben | Eingaben: EXE-Argumente oder Doppelklick Ausgabe: CLI-Ergebnis oder gestarteter Modus |
| 7. Seiteneffekte und Ressourcen | Prozessstart, Modanlage, Schemaexport und Assetvorbereitung |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Prozessmodus-Routing und CLI-Orchestrierung bleiben Einstieg. Geschäftslogik wird nur aufgerufen. Befunde: Prozessrouting, Modvorlage und Windows-Fehlermeldungsdarstellung teilen Datei. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Anwendungskomposition. Technische Kopplung: Alle Modulbibliotheken, Parser, API und package, Windows MessageBox. Aufruferbezug: Explorer, Bootstrap und gestartete Worker |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Anwendungseinstieg und CLI. Neue Einordnung: Anwendungskomposition. Besitzer: Loader-Einstieg. |

**Belege:** main, create_mod, launch_desktop, game_executable

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** aufteilen. Prozessmodus-Routing und CLI-Orchestrierung bleiben Einstieg. Geschäftslogik wird nur aufgerufen.

**Vorgeschlagene Zielpfade:** `entrypoints/loader/main.rs`, `entrypoints/loader/modes.rs`, `entrypoints/loader/cli.rs`.

### [src/loader/workflow/pregame.rs](../../entrypoints/loader/prepare.rs)

Vollständig gelesen, 719 Zeilen. SHA-256: `3264cb8dde15c190ff12ecc21795fdd647c18b3ba7a58a40f458ecf78095f2e9`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Koordiniert Assetmods vor Spielstart, Plan/Fingerprint, Originalbackup, Restore, Parserstatus und API-Ausführung mit eigenem Startup-Lock und Fehlerrecovery. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Koordiniert Assetmods vor Spielstart, Plan/Fingerprint, Originalbackup, Restore, Parserstatus und API-Ausführung mit eigenem Startup-Lock und Fehlerrecovery. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `7: pub fn run(game_directory: impl AsRef<Path>) -> Result<(), LoaderError> {`<br>`30: pub(crate) fn run_startup(game_directory: impl AsRef<Path>) -> Result<(), LoaderError> {`<br>`84: fn ensure_original_gamefiles(game_directory: &Path, file_name: &str) {`<br>`95: pub(crate) fn recover_failed_startup(game_directory: &Path, detail: &str) -> bool {`<br>`150: fn export_pass_needed(game_directory: &Path, file_name: &str) -> Result<bool, LoaderError> {`<br>`183: fn already_applied(game_directory: &Path, file_name: &str) -> Result<bool, LoaderError> {`<br>`220: fn run_inner(game_directory: &Path, file_name: &str, phase: &str) -> Result<(), LoaderError> {`<br>`390: fn target_name(game_directory: &Path) -> Result<&'static str, LoaderError> {`<br>`403: fn process_target_name(game_directory: &Path) -> Result<&'static str, LoaderError> {`<br>`417: fn startup_asset_lease(path: &Path) -> Result<std::fs::File, LoaderError> {`<br>`437: fn startup_asset_lease(path: &Path) -> Result<std::fs::File, LoaderError> {`<br>`451: fn ensure_game_stopped(file_name: &str) -> Result<(), LoaderError> {`<br>`492: fn ensure_game_stopped(_file_name: &str) -> Result<(), LoaderError> {`<br>`496: fn format_report(report: &shroudforge_package::ModEnvironmentErrorReport) -> String {`<br>`514: fn blueprint_startup_matches_legacy_and_recovers_from_mod_failure() {`<br>`515: fn recipe_bytes(root: &Path, stem: &str) -> Vec<u8> {`<br>`550: fn drop(&mut self) {`<br>`644: fn runtime_exports_do_not_restore_prepared_assets_during_startup() {`<br>`655: fn drop(&mut self) {` |
| 4. Aufrufer und Verbraucher | prepare CLI und shroudforge_prepare_startup FFI |
| 5. Verwendete Komponenten | Parser, API, GameContract, package config/backups/prepared |
| 6. Eingaben und Ausgaben | Eingaben: Spielroot, Modplan und Phase Ausgabe: Assetrevision oder Recoveryfehler |
| 7. Seiteneffekte und Ressourcen | sichert/ersetzt Spieldateien und führt Lua-Vorbereitung aus |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Prüffunktionen laden Umgebung/Plan mehrfach. Eingebetteter ignorierter Integrationstest verweist auf mods/sf-unlock-blueprints, heute dort nicht vorhanden. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Startvorbereitung. Technische Kopplung: Parser, API, GameContract, package config/backups/prepared. Aufruferbezug: prepare CLI und shroudforge_prepare_startup FFI |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: blueprint_startup_matches_legacy_and_recovers_from_mod_failure, runtime_exports_do_not_restore_prepared_assets_during_startup (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Assetvorbereitungsablauf. Neue Einordnung: Startvorbereitung. Besitzer: Loader-Einstieg. |

**Belege:** run, run_startup, run_inner, recover_failed_startup, startup_asset_lease

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** refaktorieren. Orchestriert Paketplan, Originale, Parser und Lua-Ausführung. Der Parser besitzt keine Anwendungsschritte.

**Vorgeschlagene Zielpfade:** `entrypoints/loader/prepare.rs`.

### [src/loader/workflow/tests.rs](../../entrypoints/loader/tests.rs)

Vollständig gelesen, 102 Zeilen. SHA-256: `407953b010effd451307fc6efe9b76e2f986d174c4df8f9aec3bfc93d1a01992`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft Lua-Syntax aktiver Modpakete, deklarierte API-Fähigkeiten und Abwesenheit öffentlicher Maschinenpatch-Primitiven. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft Lua-Syntax aktiver Modpakete, deklarierte API-Fähigkeiten und Abwesenheit öffentlicher Maschinenpatch-Primitiven. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5: fn every_mod_has_valid_lua_and_declares_its_used_capabilities() {`<br>`78: fn runtime_api_definition_has_no_machine_patch_surface() {`<br>`90: fn assert_no_machine_patch_primitives(label: &str, source: &str) {` |
| 4. Aufrufer und Verbraucher | Cargo-Testharness über workflow/lib.rs |
| 5. Verwendete Komponenten | mlua, aktive mods, read_manifest_path |
| 6. Eingaben und Ausgaben | Eingaben: aktive Modquellen und API-Definitionen Ausgabe: Syntax-/Capability-/Vertragsassertionen |
| 7. Seiteneffekte und Ressourcen | Repositorylesen und Lua-Parser |
| 8. Art der Verantwortung | Test |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Textmustersuche ergänzt Syntaxprüfung, beweist keine vollständige Verhaltenserkennung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Anwendungskomposition. Technische Kopplung: mlua, aktive mods, read_manifest_path. Aufruferbezug: Cargo-Testharness über workflow/lib.rs |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: every_mod_has_valid_lua_and_declares_its_used_capabilities, runtime_api_definition_has_no_machine_patch_surface (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modpaket- und API-Vertragstests. Neue Einordnung: Anwendungskomposition. Besitzer: Loader-Einstieg. |

**Belege:** every_mod_has_valid_lua_and_declares_its_used_capabilities, runtime_api_definition_has_no_machine_patch_surface

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `entrypoints/loader/tests.rs`.

### [src/parser/src/lib.rs](../../kfc-parser/crates/kfc-base/src/container/resource_lookup.rs)

Vollständig gelesen, 271 Zeilen. SHA-256: `a7164bb641967ad58bc58fd2cf91cf9f96482586c637fe5624191565371dbe5b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt KFC-formatbezogene ID-/Typzuordnung und Parseradapter bereit, extrahiert Reflection/Container in ein serialisierbares GameSchema. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt KFC-formatbezogene ID-/Typzuordnung und Parseradapter bereit, extrahiert Reflection/Container in ein serialisierbares GameSchema. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `13: pub fn resource_id(`<br>`24: pub fn resource_id_for_type(`<br>`35: pub fn type_for_resource<'a>(`<br>`42: pub fn same_type(left: &ResourceId, right: &ResourceId) -> bool {`<br>`46: pub fn resources_by_type(`<br>`59: pub fn resource_type_indices(file: &KFCFile, registry: &TypeRegistry) -> Vec<TypeIndex> {`<br>`76: pub struct GameFiles {`<br>`82: pub fn client(game: impl AsRef<Path>) -> Self {`<br>`89: pub fn server(game: impl AsRef<Path>) -> Self {`<br>`98: pub struct FieldDefinition {`<br>`105: pub struct EnumValueDefinition {`<br>`111: pub struct TypeDefinition {`<br>`127: pub struct ResourceReference {`<br>`133: pub struct GameSchema {`<br>`141: pub enum ParserError {`<br>`150: pub trait GameParser: Send + Sync {`<br>`151: fn id(&self) -> &'static str;`<br>`152: fn parse(&self, files: &GameFiles) -> Result<GameSchema, ParserError>;`<br>`156: pub struct KfcParser;`<br>`159: fn id(&self) -> &'static str {`<br>`163: fn parse(&self, files: &GameFiles) -> Result<GameSchema, ParserError> {`<br>`178: pub fn schema_from_registry(`<br>`262: fn require_file(path: &Path, executable: bool) -> Result<(), ParserError> {` |
| 4. Aufrufer und Verbraucher | Workflow types/inspect/prepare, API und Compatibility |
| 5. Verwendete Komponenten | kfc container/reflection, serde, thiserror |
| 6. Eingaben und Ausgaben | Eingaben: GameFiles oder geladene Reflection und KFCFile Ausgabe: GameSchema, ResourceIds oder ParserError |
| 7. Seiteneffekte und Ressourcen | liest EXE und KFC-Dateien |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Formathelfer, öffentliche Schemaprojektion und Parserzugang direkt im Parserprojekt ergänzen. Client-/Serverdateiauswahl bleibt Anwendung. Zwei aktuelle Parserrevisionen vor direkter Pfadeinbindung bewusst vereinheitlichen. Befunde: Re-exportiert zusätzlich Transaktion und Snapshotgenerator. Kein Live-Speicherzugriff. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Parserformat, Schema und Spieldateiauswahl. Technische Kopplung: kfc container/reflection, serde, thiserror. Aufruferbezug: Workflow types/inspect/prepare, API und Compatibility |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: KFC-Dateiparser und Metadatenmodell. Neue Einordnung: Parserformat, Schema und Spieldateiauswahl. Besitzer: Parser-Submodul und Loader-Einstieg. |

**Belege:** GameParser, KfcParser::parse, schema_from_registry, kfc_format

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** aufteilen. Formathelfer, öffentliche Schemaprojektion und Parserzugang direkt im Parserprojekt ergänzen. Client-/Serverdateiauswahl bleibt Anwendung. Zwei aktuelle Parserrevisionen vor direkter Pfadeinbindung bewusst vereinheitlichen.

**Vorgeschlagene Zielpfade:** `kfc-parser/crates/kfc-base/src/container/resource_lookup.rs`, `kfc-parser/crates/kfc/src/schema.rs`, `kfc-parser/crates/kfc/src/lib.rs`, `entrypoints/loader/game_files.rs`.

### [src/parser/src/snapshot.rs](../../kfc-parser/crates/kfc/src/snapshot.rs)

Vollständig gelesen, 223 Zeilen. SHA-256: `e3b8208d503bbf3f125e3356e07a7cfba9e1d9499ad2eb79dd6bb03d7e70cde5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert ein GameSchema als versionierten Profilsnapshot mit types.json, resources.json und Lua-Typdeklarationen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert ein GameSchema als versionierten Profilsnapshot mit types.json, resources.json und Lua-Typdeklarationen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `12: pub enum SnapshotError {`<br>`22: pub struct SnapshotResult {`<br>`29: struct Profile<'a> {`<br>`40: struct PublicType<'a> {`<br>`55: pub fn export_api_snapshot(`<br>`108: fn write_json<T: Serialize>(`<br>`118: fn write_file(path: PathBuf, bytes: &[u8]) -> Result<(), SnapshotError> {`<br>`122: fn profile_id(version: &str) -> String {`<br>`136: fn generate_lua(schema: &GameSchema) -> String {`<br>`153: fn append_struct(output: &mut String, definition: &TypeDefinition, schema: &GameSchema) {`<br>`176: fn append_enum(output: &mut String, definition: &TypeDefinition) {`<br>`184: fn lua_type(schema: &GameSchema, name: &str) -> String {`<br>`221: fn lua_name(value: &str) -> String {` |
| 4. Aufrufer und Verbraucher | CLI-Unterbefehl types |
| 5. Verwendete Komponenten | GameSchema, serde_json, Dateisystem |
| 6. Eingaben und Ausgaben | Eingaben: GameSchema und Exportroot Ausgabe: SnapshotResult und JSON/Lua-Typdateien |
| 7. Seiteneffekte und Ressourcen | schreibt Exportordner und Dateien |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Lua-Typabbildung ist ein eigener Export neben API-Definitionen, Gleichheit muss am Verbraucher geprüft werden. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Parser-Snapshotexport. Technische Kopplung: GameSchema, serde_json, Dateisystem. Aufruferbezug: CLI-Unterbefehl types |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Parserdatenexport und Typdeklarationen. Neue Einordnung: Parser-Snapshotexport. Besitzer: KFC-Parser-Submodul. |

**Belege:** export_api_snapshot, generate_lua, lua_type

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Bestehenden JSON-/Lua-Snapshotvertrag direkt im Parserprojekt implementieren. Kein zusätzlicher ShroudForge-Parseradapter.

**Vorgeschlagene Zielpfade:** `kfc-parser/crates/kfc/src/snapshot.rs`.

### [src/parser/src/transaction.rs](../../game-assets/transaction.rs)

Vollständig gelesen, 211 Zeilen. SHA-256: `77026ab8457002e46354ca18d6bea8fd9347cde44813aae59d57b366904fbd51`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Veröffentlicht zusammenhängende KFC-, Ressource- und DAT-Dateien aus Staging mit Rollbackjournal und Wiederanlauf nach unterbrochener Publikation. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Veröffentlicht zusammenhängende KFC-, Ressource- und DAT-Dateien aus Staging mit Rollbackjournal und Wiederanlauf nach unterbrochener Publikation. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `12: fn invalid() -> io::Error {`<br>`16: fn sync(path: &Path) -> io::Result<()> {`<br>`20: pub fn directory(root: &Path, stem: &str) -> PathBuf {`<br>`24: fn valid_name(stem: &str, name: &str) -> bool {`<br>`35: fn records(stage: &Path, stem: &str) -> io::Result<Vec<(String, bool)>> {`<br>`44: fn clean(stage: &Path) -> io::Result<()> {`<br>`58: fn replace(source: &Path, target: &Path) -> io::Result<()> {`<br>`65: pub fn recover(root: &Path, stem: &str) -> io::Result<()> {`<br>`84: pub fn begin(root: &Path, stem: &str) -> io::Result<PathBuf> {`<br>`91: pub fn abort(root: &Path, stem: &str) -> io::Result<()> {`<br>`95: fn prepare(root: &Path, stem: &str) -> io::Result<Vec<(String, bool)>> {`<br>`132: pub fn commit(root: &Path, stem: &str) -> io::Result<()> {`<br>`154: fn fixture() -> PathBuf {`<br>`162: fn remove_fixture(root: &Path) {`<br>`173: fn interrupted_publish_restores_previous_set() {`<br>`197: fn commit_publishes_complete_set() {` |
| 4. Aufrufer und Verbraucher | Workflow-Recovery und API-Assetcommit |
| 5. Verwendete Komponenten | Dateisystem, serde_json, pending.json und rollback-Dateien |
| 6. Eingaben und Ausgaben | Eingaben: Root, stem und Staging-Dateimenge Ausgabe: Stagepfad oder Commit-/Recoveryergebnis |
| 7. Seiteneffekte und Ressourcen | Dateiersatz, Journal, fsync und Rollback |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Verantwortung ist persistente Assetänderung, nicht das Lesen des KFC-Formats. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Spielasset-Veröffentlichung. Technische Kopplung: Dateisystem, serde_json, pending.json und rollback-Dateien. Aufruferbezug: Workflow-Recovery und API-Assetcommit |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: interrupted_publish_restores_previous_set, commit_publishes_complete_set (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Transaktionale Assetveröffentlichung. Neue Einordnung: Spielasset-Veröffentlichung. Besitzer: Spielasset-Dateien. |

**Belege:** begin, commit, recover, prepare, valid_name

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Journal, Commit und Recovery betreffen persistente Spielasset-Sätze, nicht das Parserformat.

**Vorgeschlagene Zielpfade:** `game-assets/transaction.rs`.

## Paket-, Konfigurations- und Logcode

19 Dateien, 4.353 Zeilen.

### [src/loader/package/src/alias.rs](../../packages/alias.rs)

Vollständig gelesen, 4 Zeilen. SHA-256: `c837ef2aad75c6b9b3db8bb254adff970cacfbaa066d9791971c891b32e3db1d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Vereinheitlicht Paketpfade als UTF-8-Pfadtypen von camino. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Vereinheitlicht Paketpfade als UTF-8-Pfadtypen von camino. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Path, PathBuf` |
| 4. Aufrufer und Verbraucher | registry, env, error über crate::alias |
| 5. Verwendete Komponenten | camino |
| 6. Eingaben und Ausgaben | Eingaben: camino-Pfadtypen Ausgabe: Typaliase Path/PathBuf |
| 7. Seiteneffekte und Ressourcen | keine |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: camino. Aufruferbezug: registry, env, error über crate::alias |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Interne Pfadtypen. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** Path, PathBuf

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/alias.rs`.

### [src/loader/package/src/backups.rs](../../game-assets/originals.rs)

Vollständig gelesen, 397 Zeilen. SHA-256: `0cfea69a22237a73db87b9f4bffd794048672ad2a3094fd33d74209204b87fab`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erfasst buildgebundene unveränderliche KFC-Originalpaare mit Prüfsummen, zeigt Backupbestand und stellt verifizierte Originale mit Rücksicherung wieder her. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erfasst buildgebundene unveränderliche KFC-Originalpaare mit Prüfsummen, zeigt Backupbestand und stellt verifizierte Originale mit Rücksicherung wieder her. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `11: fn stem(target: &str) -> Result<&'static str, String> {`<br>`19: fn target_name(stem: &str) -> &'static str {`<br>`27: fn pair(stem: &str) -> [String; 2] {`<br>`31: fn game_executable(root: &Path, stem: &str) -> PathBuf {`<br>`39: fn originals_target_dir(root: &Path, stem: &str) -> PathBuf {`<br>`45: fn originals_dir(root: &Path, stem: &str) -> PathBuf {`<br>`64: fn sha256(path: &Path) -> Result<String, String> {`<br>`78: fn manifest(root: &Path, stem: &str) -> Result<Value, String> {`<br>`90: pub fn originals_status(root: &Path, target: &str) -> Value {`<br>`153: pub fn management_status(root: &Path, target: &str) -> Value {`<br>`188: pub fn ensure_originals(root: &Path, target: &str) -> Result<Value, String> {`<br>`200: pub fn capture_verified_originals(root: &Path, target: &str) -> Result<Value, String> {`<br>`209: fn capture_originals_inner(root: &Path, stem: &str, user_verified: bool) -> Result<(), String> {`<br>`304: pub fn restore_originals(root: &Path, target: &str) -> Result<Value, String> {`<br>`384: fn rollback_files(root: &Path, safety: &Path, replaced: &[String]) {` |
| 4. Aufrufer und Verbraucher | Workflow ensure_originals, UI Backups und Updater Restoreauftrag |
| 5. Verwendete Komponenten | SHA256, EXE-Metadaten, backups/originals, config applied |
| 6. Eingaben und Ausgaben | Eingaben: Root, Ziel und bestätigte Originale Ausgabe: Backupstatus oder Restoreergebnis |
| 7. Seiteneffekte und Ressourcen | Hashen, Kopieren, Ersetzen, Sicherheitskopien |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Statusinventar zählt auch Updatebackups. Restore und Capture teilen Datei, beide gehören zu Spielassetverwaltung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Originale Spielassets. Technische Kopplung: SHA256, EXE-Metadaten, backups/originals, config applied. Aufruferbezug: Workflow ensure_originals, UI Backups und Updater Restoreauftrag |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Spielasset-Originalsicherung und Restore. Neue Einordnung: Originale Spielassets. Besitzer: Spielasset-Dateien. |

**Belege:** ensure_originals, capture_verified_originals, restore_originals, management_status

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Originalsicherung und Restore der Spieldaten haben einen anderen Inhalt und Lebenszyklus als Release-Backups.

**Vorgeschlagene Zielpfade:** `game-assets/originals.rs`.

### [src/loader/package/src/compatibility.rs](../../packages/conflicts.rs)

Vollständig gelesen, 47 Zeilen. SHA-256: `23887e46149d6d6e0df96cb43db32d85a963020e9120c58af38d9559d5109ce5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt Konfliktregeln und blockiert gemeinsam aktivierte Mods anhand ihrer IDs und SemVer-Bereiche. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt Konfliktregeln und blockiert gemeinsam aktivierte Mods anhand ihrer IDs und SemVer-Bereiche. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5: pub fn conflicts(env: &ModEnvironment) -> Result<HashMap<String, String>, String> {` |
| 4. Aufrufer und Verbraucher | ModEnvironment::plan_report_inner |
| 5. Verwendete Komponenten | ModEnvironment, rules.json, jsonschema, semver |
| 6. Eingaben und Ausgaben | Eingaben: Modumgebung und Konfliktregeln Ausgabe: blockierte Mod-IDs und Gründe |
| 7. Seiteneffekte und Ressourcen | liest Konfliktregeln |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Nicht mit dem separaten Spielschema-Compatibility-Crate verwechseln. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modkonflikte. Technische Kopplung: ModEnvironment, rules.json, jsonschema, semver. Aufruferbezug: ModEnvironment::plan_report_inner |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modkonfliktprüfung. Neue Einordnung: Modkonflikte. Besitzer: Modpakete. |

**Belege:** conflicts

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/conflicts.rs`.

### [src/loader/package/src/config.rs](../../support/storage/json.rs)

Vollständig gelesen, 449 Zeilen. SHA-256: `4ba29621e2956e6bb18eb78f34dcf794881bd8b06d48bc158d7edd7b64dbd7c4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Validiert Dokumente, schreibt JSON atomar, verwaltet gemeinsame Statusabschnitte, Loaderdefaults/Pfade, Sperren, Fensterwünsche, Fingerprints und Settingsvalidierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Validiert Dokumente, schreibt JSON atomar, verwaltet gemeinsame Statusabschnitte, Loaderdefaults/Pfade, Sperren, Fensterwünsche, Fingerprints und Settingsvalidierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `16: pub fn validate_document(_root: &Path, name: &str, value: &Value) -> Result<(), String> {`<br>`48: pub fn write_json(path: &Path, value: &Value) -> Result<(), String> {`<br>`72: pub fn write_document(root: &Path, name: &str, value: &Value) -> Result<(), String> {`<br>`83: fn state_section(name: &str) -> Option<&'static str> {`<br>`99: fn read_state_file(root: &Path) -> Result<Value, String> {`<br>`120: pub fn read_document(root: &Path, name: &str) -> Result<Value, String> {`<br>`142: pub fn update_state_section(`<br>`165: pub fn window_state(root: &Path) -> Value {`<br>`172: pub fn request_window_visibility(root: &Path, module: &str, visible: bool) -> Result<(), String> {`<br>`201: pub fn request_world_editor_module_settings(root: &Path) -> Result<(), String> {`<br>`237: pub fn publish_window_visibility(root: &Path, module: &str, visible: bool) -> Result<(), String> {`<br>`269: pub fn document_path(root: &Path, name: &str) -> std::path::PathBuf {`<br>`290: pub fn read_loader(root: &Path) -> Result<Value, String> {`<br>`302: fn merge(target: &mut Value, source: Value) {`<br>`324: pub fn initialize_loader_config(root: &Path) -> Result<(), String> {`<br>`356: pub fn update_loader(`<br>`369: pub fn installation_lock(root: &Path) -> Result<fs::File, String> {`<br>`382: pub fn revision(bytes: &[u8]) -> String {`<br>`390: pub fn manifest_revision(package: &Path) -> Result<String, String> {`<br>`415: pub fn validate_settings(definitions: &[SettingDefinition], values: &Value) -> Result<(), String> {` |
| 4. Aufrufer und Verbraucher | Loader, API, UI und Updater über config::* |
| 5. Verwendete Komponenten | jsonschema, tempfile, fs2, viele eingebettete Fremdbereich-Schemas |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentnamen, Root und JSON-Änderungen Ausgabe: validiertes JSON, Revision, Dateisperre oder Fehler |
| 7. Seiteneffekte und Ressourcen | Config-/Statuslesen und atomisches Schreiben |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Atomisches I/O gemeinsam nutzen. Loaderdefaults, Paketwerte und Fensteraufträge behalten eigene Zuständigkeiten. Befunde: Allgemeines I/O, Produktstatus und konkrete Fensteraktionen sind vermischt. Schema wird pro Validierung kompiliert. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Konfiguration, Persistenz und Steuerung. Technische Kopplung: jsonschema, tempfile, fs2, viele eingebettete Fremdbereich-Schemas. Aufruferbezug: Loader, API, UI und Updater über config::* |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Konfiguration, Zustand und Dateispeicherung. Neue Einordnung: Konfiguration, Persistenz und Steuerung. Besitzer: Loader-Support und konkrete Besitzer. |

**Belege:** validate_document, write_json, update_state_section, request_world_editor_module_settings, update_loader, manifest_revision

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** aufteilen. Atomisches I/O gemeinsam nutzen. Loaderdefaults, Paketwerte und Fensteraufträge behalten eigene Zuständigkeiten.

**Vorgeschlagene Zielpfade:** `support/storage/json.rs`, `support/storage/state.rs`, `support/config/loader.rs`, `support/control/windows.rs`, `packages/settings/validate.rs`.

### [src/loader/package/src/env.rs](../../packages/plan.rs)

Vollständig gelesen, 422 Zeilen. SHA-256: `53f5aa0defe5bf26231df49f8f65ee1404b958e5e2f13b2170fd2ea3b1749441`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt ModRegistry und Pfade, erfasst Deaktivierung und berechnet geordnete ausführbare bzw. Runtime-Kandidatenpläne mit Abhängigkeits- und Zielprüfung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt ModRegistry und Pfade, erfasst Deaktivierung und berechnet geordnete ausführbare bzw. Runtime-Kandidatenpläne mit Abhängigkeits- und Zielprüfung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10: struct ModEnvironmentInner {`<br>`21: pub struct ModPlanIssue {`<br>`28: fn new(mod_id: &str, detail: String) -> Self {`<br>`52: fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {`<br>`58: pub struct ModEnvironment {`<br>`63: pub fn load(game_dir: impl AsRef<Path>) -> Result<Self, ModEnvironmentErrorReport> {`<br>`96: pub fn game_dir(&self) -> &Path {`<br>`100: pub fn cache_dir(&self) -> &Path {`<br>`104: pub fn mods_dir(&self) -> &Path {`<br>`108: pub fn mod_registry(&self) -> &ModRegistry {`<br>`112: pub fn is_mod_enabled(&self, id: &str) -> bool {`<br>`116: pub fn enabled_mods(&self) -> impl Iterator<Item = &crate::Mod> {`<br>`124: pub fn plan(&self, is_server: bool, api_version: &str) -> Vec<&crate::Mod> {`<br>`128: pub fn plan_report(`<br>`143: pub fn plan_report_detailed(`<br>`153: pub fn runtime_plan_report(`<br>`168: pub fn runtime_plan_report_detailed(`<br>`211: fn plan_report_inner(`<br>`230: fn visit<'a>(`<br>`347: fn disabled_mods_remain_installed_but_are_not_enabled() {`<br>`389: fn client_only_root_mod_is_skipped_without_a_server_plan_error() {` |
| 4. Aufrufer und Verbraucher | Workflow prepare/create/inspect und API Runtime |
| 5. Verwendete Komponenten | Registry, Konfliktregeln, semver, ModTarget |
| 6. Eingaben und Ausgaben | Eingaben: Spielroot, Paketregistry, Prozessziel und API-Version Ausgabe: Modumgebung, Plan und Planungskonflikte |
| 7. Seiteneffekte und Ressourcen | liest Modordner und Dateien |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Fehlercodes werden aus Fehlertext abgeleitet. Unterschiedliche Pläne nutzen bereits eine gemeinsame Graphfunktion. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modplan und Abhängigkeiten. Technische Kopplung: Registry, Konfliktregeln, semver, ModTarget. Aufruferbezug: Workflow prepare/create/inspect und API Runtime |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: disabled_mods_remain_installed_but_are_not_enabled, client_only_root_mod_is_skipped_without_a_server_plan_error (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Paketumgebung und Modplanung. Neue Einordnung: Modplan und Abhängigkeiten. Besitzer: Modpakete. |

**Belege:** ModEnvironment::load, plan_report_inner, runtime_plan_report_detailed

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** refaktorieren. Gemeinsamen Graphalgorithmus erhalten. Fehlercodes aus typisierten Gründen statt Textmustern ableiten.

**Vorgeschlagene Zielpfade:** `packages/plan.rs`.

### [src/loader/package/src/error.rs](../../packages/error.rs)

Vollständig gelesen, 82 Zeilen. SHA-256: `796b00ea524fd7f31f81236eec247abd96048e74fe20cc8f34ab038baae264c4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert Fehler für Umgebung, einzelne Modpakete, doppelte IDs, JSON und Dateizugriffe. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert Fehler für Umgebung, einzelne Modpakete, doppelte IDs, JSON und Dateizugriffe. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10: pub struct ModEnvironmentErrorReport {`<br>`39: pub struct ModErrorReport {`<br>`61: pub enum ModError {`<br>`78: pub struct IoError {` |
| 4. Aufrufer und Verbraucher | ModRegistry::load und ModEnvironment::load |
| 5. Verwendete Komponenten | ModRegistry, thiserror, UTF-8-Pfade |
| 6. Eingaben und Ausgaben | Eingaben: IO-/JSON-Fehler und Teilregistry Ausgabe: typisierte Fehlerberichte |
| 7. Seiteneffekte und Ressourcen | keine |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: ModRegistry, thiserror, UTF-8-Pfade. Aufruferbezug: ModRegistry::load und ModEnvironment::load |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Paketfehler. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** ModEnvironmentErrorReport, ModErrorReport, ModError, IoError

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/error.rs`.

### [src/loader/package/src/lib.rs](../../packages/lib.rs)

Vollständig gelesen, 19 Zeilen. SHA-256: `75c5907344bc7596d2ac9b98ebeb12ecfaa491501b45229f394bf45f76b9c0a4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert und exportiert Paket-, Konfigurations-, Status-, Update-, Logging- und Fensterhilfen des Crates. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert und exportiert Paket-, Konfigurations-, Status-, Update-, Logging- und Fensterhilfen des Crates. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `pub mod config/backups/logging/news/paths/prepared/update_check/status/window_icon` |
| 4. Aufrufer und Verbraucher | shroudforge-package in Loader, API und Modulen |
| 5. Verwendete Komponenten | Unterdateien des package-Crates |
| 6. Eingaben und Ausgaben | Eingaben: öffentliche Crate-Aufrufe Ausgabe: Exporte und Ergebnisse der Einstiegsschnittstellen |
| 7. Seiteneffekte und Ressourcen | siehe konkrete actual_role |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Öffentlicher Einstieg bündelt mehrere fachliche Bereiche. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: Unterdateien des package-Crates. Aufruferbezug: shroudforge-package in Loader, API und Modulen |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Crate-Einstieg. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** pub mod config/backups/logging/news/paths/prepared/update_check/status/window_icon

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/lib.rs`.

### [src/loader/package/src/log.rs](../../support/log_macros.rs)

Vollständig gelesen, 30 Zeilen. SHA-256: `e65a52b11354b22e8d9e4363faadf1ba85f44e00b7027c86657cae16d0ebf0a8`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert tracing-Makros mit package-Target, keine zweite Logausgabe. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert tracing-Makros mit package-Target, keine zweite Logausgabe. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `info!, warning!, error!, debug!` |
| 4. Aufrufer und Verbraucher | Registry und Paketdateizugriff |
| 5. Verwendete Komponenten | tracing |
| 6. Eingaben und Ausgaben | Eingaben: tracing-Makroargumente Ausgabe: package-Logevent |
| 7. Seiteneffekte und Ressourcen | tracing-Aufruf |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Protokollierung. Technische Kopplung: tracing. Aufruferbezug: Registry und Paketdateizugriff |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Logging-Aufrufhilfen. Neue Einordnung: Protokollierung. Besitzer: Loader-Support. |

**Belege:** info!, warning!, error!, debug!

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `support/log_macros.rs`.

### [src/loader/package/src/logging.rs](../../support/logging.rs)

Vollständig gelesen, 321 Zeilen. SHA-256: `677444fe7b13d0f59c18173ca2ad0382840ec98bf03f34a9f5e5b9598a9253c0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Formatiert tracing und direkte Logzeilen, filtert Level, serialisiert Ausgabe und rotiert bestehende Prozesslogs beim Start. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Formatiert tracing und direkte Logzeilen, filtert Level, serialisiert Ausgabe und rotiert bestehende Prozesslogs beim Start. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `23: struct LogWriter(Arc<PathBuf>);`<br>`25: struct Output {`<br>`31: fn commit(&mut self) -> io::Result<()> {`<br>`48: fn drop(&mut self) {`<br>`56: fn make_writer(&'a self) -> Self::Writer {`<br>`65: fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {`<br>`71: fn flush(&mut self) -> io::Result<()> {`<br>`77: struct LineFormat;`<br>`82: pub fn timestamp() -> String {`<br>`94: fn single_line(value: &str) -> String {`<br>`99: pub fn format_line(level: char, source: &str, message: &str) -> String {`<br>`109: struct Fields {`<br>`116: fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {`<br>`125: fn record_str(&mut self, field: &tracing::field::Field, value: &str) {`<br>`139: fn format_event(`<br>`169: pub fn initialize(root: impl AsRef<Path>, archive_existing: bool) -> io::Result<()> {`<br>`226: pub fn append(`<br>`240: fn append_line(root: &Path, level: char, source: &str, message: &str) -> io::Result<()> {`<br>`256: fn with_log_lock<T>(_root: &Path, operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {`<br>`284: fn with_log_lock<T>(_root: &Path, operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {`<br>`293: fn configured_level(root: &Path) -> LevelFilter {`<br>`312: pub fn allows(root: &Path, level: char) -> bool {` |
| 4. Aufrufer und Verbraucher | Workflow, API und Module initialisieren oder append |
| 5. Verwendete Komponenten | tracing-subscriber, Loaderconfig, Pfade, Windows-Mutex |
| 6. Eingaben und Ausgaben | Eingaben: Level, Quelle und Nachricht Ausgabe: Logzeile und Schreibergebnis |
| 7. Seiteneffekte und Ressourcen | Configlesen, Logrotation, Datei/stdout und Mutex |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Direktes append lädt Konfiguration pro Aufruf. Windows-Mutexname ist installationsübergreifend. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Protokollierung. Technische Kopplung: tracing-subscriber, Loaderconfig, Pfade, Windows-Mutex. Aufruferbezug: Workflow, API und Module initialisieren oder append |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Loggingpipeline. Neue Einordnung: Protokollierung. Besitzer: Loader-Support. |

**Belege:** initialize, append, format_line, configured_level, with_log_lock

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `support/logging.rs`.

### [src/loader/package/src/news.rs](../../modules/news/service.rs)

Vollständig gelesen, 174 Zeilen. SHA-256: `07ad3502b4d9254c737e35ef796db545cf94b44485619b8a364305c5053ad641`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Validiert Benachrichtigungen, speichert Events und Gelesen-Zeitpunkte im gemeinsamen Zustand und liefert eingebettete Produktnachrichten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Validiert Benachrichtigungen, speichert Events und Gelesen-Zeitpunkte im gemeinsamen Zustand und liefert eingebettete Produktnachrichten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `7: pub fn directory(root: &Path) -> PathBuf {`<br>`10: pub fn events_directory(root: &Path) -> PathBuf {`<br>`14: pub fn validate_event(root: &Path, value: &serde_json::Value) -> Result<(), String> {`<br>`30: pub fn write_event(root: &Path, filename: &str, value: &serde_json::Value) -> Result<(), String> {`<br>`49: fn now() -> u64 {`<br>`56: fn read_state(root: &Path) -> Result<serde_json::Value, String> {`<br>`68: fn validate_state(root: &Path, value: &serde_json::Value) -> Result<(), String> {`<br>`72: pub fn read_ids(root: &Path) -> Result<Vec<String>, String> {`<br>`96: pub fn mark_read(root: &Path, ids: &[String]) -> Result<(), String> {`<br>`118: pub fn mark_unread(root: &Path, ids: &[String]) -> Result<(), String> {`<br>`136: pub fn read(root: &Path) -> Result<serde_json::Value, String> {`<br>`148: fn recurring_news_becomes_unread_and_can_be_acknowledged_again() {`<br>`164: fn invalid_state_is_reported_and_preserved() {` |
| 4. Aufrufer und Verbraucher | Modloader UI und Statusaggregation |
| 5. Verwendete Komponenten | config::update_state_section, News-Schemas, embedded news.json |
| 6. Eingaben und Ausgaben | Eingaben: Nachrichten-IDs und Events Ausgabe: Nachrichten und Gelesen-Zustand |
| 7. Seiteneffekte und Ressourcen | liest Nachrichten und schreibt Statusabschnitte |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Zusätzliche Event-Schemavalidierung kann externe Schemadatei lesen. Kein Modmanifest. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: config::update_state_section, News-Schemas, embedded news.json. Aufruferbezug: Modloader UI und Statusaggregation |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: recurring_news_becomes_unread_and_can_be_acknowledged_again, invalid_state_is_reported_and_preserved (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Nachrichten und Benachrichtigungszustand. Neue Einordnung: Eingebaute Module. Besitzer: Nachrichten. |

**Belege:** write_event, read_ids, mark_read, mark_unread, read

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/news/service.rs`.

### [src/loader/package/src/paths.rs](../../support/paths.rs)

Vollständig gelesen, 135 Zeilen. SHA-256: `f696ae289a8602a17c5c6d9d4d8cdbfb220227ec1259d4c78b7a393fcefbff62`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst konfigurierbare Arbeitsverzeichnisse und feste Programm-/Konfigurationspfade auf und bestimmt Client-/Serverlogpfad. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst konfigurierbare Arbeitsverzeichnisse und feste Programm-/Konfigurationspfade auf und bestimmt Client-/Serverlogpfad. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `4: fn configured_dir(root: &Path, key: &str) -> Option<PathBuf> {`<br>`15: pub fn data_dir(root: &Path) -> PathBuf {`<br>`18: pub fn backups_dir(root: &Path) -> PathBuf {`<br>`21: pub fn default_directory(root: &Path, key: &str) -> Option<PathBuf> {`<br>`35: pub fn mods_dir(root: &Path) -> PathBuf {`<br>`42: pub fn cache_dir(root: &Path) -> PathBuf {`<br>`47: pub fn export_dir(root: &Path) -> PathBuf {`<br>`50: pub fn loader_executable(root: &Path) -> PathBuf {`<br>`53: pub fn updater_executable(root: &Path) -> PathBuf {`<br>`56: pub fn runtime_library(root: &Path) -> PathBuf {`<br>`59: pub fn native_runtime_library(root: &Path) -> PathBuf {`<br>`62: pub fn runtime_profiles_dir(root: &Path) -> PathBuf {`<br>`65: pub fn startup_asset_lock(root: &Path) -> PathBuf {`<br>`68: pub fn runtime_dir(root: &Path) -> PathBuf {`<br>`73: pub fn config_dir(root: &Path) -> PathBuf {`<br>`76: pub fn loader_config(root: &Path) -> PathBuf {`<br>`79: pub fn version_file(root: &Path) -> PathBuf {`<br>`82: pub fn current_log(root: &Path) -> PathBuf {`<br>`90: fn is_dedicated_server(root: &Path) -> bool {`<br>`97: pub fn logs_dir(root: &Path) -> PathBuf {`<br>`102: pub fn updates_dir(root: &Path) -> PathBuf {`<br>`107: pub fn ui_data_dir(root: &Path) -> PathBuf {`<br>`112: pub fn state_dir(root: &Path) -> PathBuf {`<br>`117: pub fn state_file(root: &Path) -> PathBuf {`<br>`120: pub fn webview_profile(root: &Path) -> PathBuf {`<br>`129: fn mod_packages_are_a_sibling_of_loader_data() {` |
| 4. Aufrufer und Verbraucher | praktisch alle Loaderbereiche über paths::* |
| 5. Verwendete Komponenten | config::read_loader, Dateisystem, aktueller Prozess |
| 6. Eingaben und Ausgaben | Eingaben: Spielroot und Pfadschlüssel Ausgabe: konfigurierte oder feste Pfade |
| 7. Seiteneffekte und Ressourcen | Config-/Prozessidentitätslesen |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Jede konfigurierte Pfadauflösung liest Loaderconfig erneut. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Installationskontext. Technische Kopplung: config::read_loader, Dateisystem, aktueller Prozess. Aufruferbezug: praktisch alle Loaderbereiche über paths::* |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: mod_packages_are_a_sibling_of_loader_data (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Installationspfade. Neue Einordnung: Installationskontext. Besitzer: Loader-Support. |

**Belege:** configured_dir, default_directory, current_log

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** refaktorieren. Pfadkontext aus einer gelesenen Konfigurationsrevision bilden. Laufende Konfigurationsänderung weiterhin explizit aktualisieren.

**Vorgeschlagene Zielpfade:** `support/paths.rs`.

### [src/loader/package/src/prepared.rs](../../game-assets/fingerprint.rs)

Vollständig gelesen, 110 Zeilen. SHA-256: `4f7c1f2af614ee74ceacd514d00962de6e1ad800ff99694102c2a33433f932fa`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bildet Assetvorbereitungsfingerprint aus Spielbinärmetadaten und geplanter Modkonfiguration samt Paketdateien und vergleicht angewandten Status. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bildet Assetvorbereitungsfingerprint aus Spielbinärmetadaten und geplanter Modkonfiguration samt Paketdateien und vergleicht angewandten Status. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5: pub fn fingerprint(env: &ModEnvironment, server: bool, api: &str) -> Result<String, String> {`<br>`66: pub fn matches(root: &Path, fingerprint: &str) -> bool {`<br>`73: fn file_digest(path: &Path) -> Result<Vec<u8>, String> {` |
| 4. Aufrufer und Verbraucher | Workflow pregame, API Assetanwendung, Statusaggregation |
| 5. Verwendete Komponenten | ModEnvironment::plan, SHA256, applied-Status, walkdir |
| 6. Eingaben und Ausgaben | Eingaben: Modumgebung, Zielrolle und API-Version Ausgabe: Assetfingerprint und Vergleich |
| 7. Seiteneffekte und Ressourcen | liest Paketdateien und Status, hält Digestcache |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Enthält bereits begrenzten Dateidigestcache. Dateizweig ist nicht Beleg für tatsächlich laufende ZIP-Mods. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Vorbereitungsnachweis. Technische Kopplung: ModEnvironment::plan, SHA256, applied-Status, walkdir. Aufruferbezug: Workflow pregame, API Assetanwendung, Statusaggregation |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Assetvorbereitungszustand. Neue Einordnung: Vorbereitungsnachweis. Besitzer: Spielasset-Dateien. |

**Belege:** fingerprint, matches, file_digest

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `game-assets/fingerprint.rs`.

### [src/loader/package/src/registry/fs.rs](../../packages/manifest/fs.rs)

Vollständig gelesen, 454 Zeilen. SHA-256: `f331a2c66a7dd0dbe4e400191f0875cde01fad511e233a147bd18488798a5388`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Kapselt Dateien eines Pakets über Disk-, ZIP- und Empty-Varianten, mit Pfadgrenzen und einem Archivbaum. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Kapselt Dateien eines Pakets über Disk-, ZIP- und Empty-Varianten, mit Pfadgrenzen und einem Archivbaum. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `15: pub trait FileRead: Read {}`<br>`20: pub enum FileSystem {`<br>`27: pub fn new_disk<P: AsRef<Path>>(root: P) -> Result<Self> {`<br>`31: pub fn new_zip<P: AsRef<Path>>(path: P) -> Result<Self> {`<br>`35: pub fn root(&self) -> &Path {`<br>`43: pub fn files(&self) -> Vec<Result<PathBuf>> {`<br>`59: pub fn read_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Result<PathBuf>>> {`<br>`76: pub fn read_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Box<dyn FileRead + '_>> {`<br>`99: pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {`<br>`116: pub fn is_file<P: AsRef<Path>>(&mut self, path: P) -> bool {`<br>`134: pub fn is_directory<P: AsRef<Path>>(&mut self, path: P) -> bool {`<br>`152: pub fn absolute_path<P: AsRef<Path>>(&self, path: P) -> Result<Option<PathBuf>> {`<br>`171: pub struct DiskFileSystem {`<br>`176: fn new(root: &Path) -> Result<Self> {`<br>`182: fn root(&self) -> &Path {`<br>`186: fn files(&self) -> Vec<Result<PathBuf>> {`<br>`200: fn read_directory(&self, path: &Path) -> Result<Vec<Result<PathBuf>>> {`<br>`239: fn read_file(&mut self, path: &Path) -> Result<Box<dyn FileRead>> {`<br>`249: fn exists(&self, path: &Path) -> bool {`<br>`255: fn is_file(&self, path: &Path) -> bool {`<br>`261: fn is_directory(&self, path: &Path) -> bool {`<br>`269: pub struct ZipFileSystem {`<br>`276: fn new(path: &Path) -> Result<Self> {`<br>`304: fn root(&self) -> &Path {`<br>`308: fn files(&self) -> Vec<Result<PathBuf>> {`<br>`315: fn read_directory(&self, path: &Path) -> Result<Vec<Result<PathBuf>>> {`<br>`328: fn read_file(&mut self, path: &Path) -> Result<Box<dyn FileRead + '_>> {`<br>`339: fn exists(&self, path: &Path) -> bool {`<br>`343: fn is_file(&mut self, path: &Path) -> bool {`<br>`347: fn is_directory(&mut self, path: &Path) -> bool {`<br>`353: struct TreeNode {`<br>`360: fn new() -> Self {`<br>`368: fn insert_path(`<br>`410: fn get(&self, path: impl AsRef<Path>) -> Option<&Self> {`<br>`421: fn get_internal(&self, components: &[&str]) -> Option<&Self> {`<br>`435: fn iter(&self) -> impl Iterator<Item = &String> {`<br>`440: fn sanitize_path(root: &Path, path: &Path) -> Result<PathBuf> {` |
| 4. Aufrufer und Verbraucher | Registry, Manifestreader und API über Mod::fs |
| 5. Verwendete Komponenten | walkdir, zip, camino |
| 6. Eingaben und Ausgaben | Eingaben: Paketroot und relativer Pfad Ausgabe: Dateileser, Listen und Pfadstatus |
| 7. Seiteneffekte und Ressourcen | Datei-/Verzeichnislesen und optional Archivlesen |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: ZIP-Variante existiert im Code. Daraus folgt kein von Nutzern verwendeter Archiv-Modbetrieb. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: walkdir, zip, camino. Aufruferbezug: Registry, Manifestreader und API über Mod::fs |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Paketdateizugriff. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** FileSystem, DiskFileSystem, ZipFileSystem, TreeNode, sanitize_path

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/manifest/fs.rs`.

### [src/loader/package/src/registry/manifest.rs](../../packages/manifest/manifest.rs)

Vollständig gelesen, 197 Zeilen. SHA-256: `e5792d25baded8cfdfe7d39d6274715e89603b9d9e9aa44e88fa2df19066da53`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert das interne zusammengeführte Modmodell einschließlich Fähigkeiten, Prozesszielen, Abhängigkeiten, Controls, Gruppen und Werten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert das interne zusammengeführte Modmodell einschließlich Fähigkeiten, Prozesszielen, Abhängigkeiten, Controls, Gruppen und Werten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `6: pub struct ModManifest {`<br>`45: pub fn requires_pregame(&self) -> bool {`<br>`61: fn empty_settings() -> serde_json::Value {`<br>`67: pub enum ModTarget {`<br>`76: pub fn supports_process(self, is_server: bool) -> bool {`<br>`86: pub struct ModLinks(pub std::collections::BTreeMap<String, String>);`<br>`90: pub struct SettingGroup {`<br>`101: pub struct ModAction {`<br>`110: pub struct SettingDefinition {`<br>`132: fn default_apply_at() -> String {`<br>`138: pub enum SettingValueType {`<br>`148: pub enum SettingControl {`<br>`165: pub struct SettingOption {`<br>`171: pub struct Dependency {`<br>`179: pub enum Capability {`<br>`189: pub fn requires_pregame(self) -> bool {`<br>`194: pub fn requires_runtime(self) -> bool {` |
| 4. Aufrufer und Verbraucher | manifest_reader, Planung, UI und API |
| 5. Verwendete Komponenten | serde, semver |
| 6. Eingaben und Ausgaben | Eingaben: deserialisierte Moddaten Ausgabe: internes Mod-/Settingsmodell |
| 7. Seiteneffekte und Ressourcen | keine |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Internes Modell ist ausdrücklich vom EML-Dateiformat zu unterscheiden. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: serde, semver. Aufruferbezug: manifest_reader, Planung, UI und API |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Paket- und Settingsmodell. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** ModManifest, requires_pregame, Capability, ModTarget, SettingDefinition

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/manifest/manifest.rs`.

### [src/loader/package/src/registry/manifest_reader.rs](../../packages/manifest/read.rs)

Vollständig gelesen, 546 Zeilen. SHA-256: `1403ba5bb37f86678c9ff27de43da15e1ce0e0568f71308d2447151976fc1bcd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest EML-mod.json mit optionaler Extension, bildet Werte und Metadaten auf das interne Modell ab, bestimmt Ziele und erkennt Ausführungsphase aus Lua-Text. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest EML-mod.json mit optionaler Extension, bildet Werte und Metadaten auf das interne Modell ab, bestimmt Ziele und erkennt Ausführungsphase aus Lua-Text. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10: fn read_package_json(fs: &mut FileSystem, name: &str) -> Result<Value, String> {`<br>`16: pub fn read_manifest(root: &Path, fs: &mut FileSystem) -> Result<ModManifest, String> {`<br>`33: fn collect_lua_sources(`<br>`60: pub fn infer_api_contract(manifest: &mut ModManifest, source: &str) {`<br>`98: fn strip_lua_comments(source: &str) -> String {`<br>`161: pub fn parse_manifest(root: &Path, value: Value) -> Result<ModManifest, String> {`<br>`166: pub fn parse_manifest_with_extension(`<br>`268: fn parse_setting_definition(`<br>`397: pub fn read_manifest_path(root: &Path, package: &Path) -> Result<ModManifest, String> {`<br>`408: pub fn valid_id(id: &str) -> bool {`<br>`423: fn eml_manifest_remains_valid_without_shroudforge_extension() {`<br>`443: fn extension_settings_and_groups_are_read_from_the_neighbor_manifest() {`<br>`480: fn extension_targets_apply_to_all_packages_without_changing_mod_json() {`<br>`497: fn legacy_eml_without_extension_defaults_to_client_and_server() {`<br>`518: fn mod_targets_match_only_the_declared_processes() {`<br>`528: fn shipped_mod_packages_have_valid_eml_and_extension_manifests() {` |
| 4. Aufrufer und Verbraucher | Registry und config-Reexport für UI/API |
| 5. Verwendete Komponenten | FileSystem, Config-Schemavalidierung, SettingDefinition |
| 6. Eingaben und Ausgaben | Eingaben: EML-mod.json und optionale Extension Ausgabe: normalisiertes ModManifest |
| 7. Seiteneffekte und Ressourcen | liest Paketdateien und rekursiv Lua-Quellen |
| 8. Art der Verantwortung | Fachlogik/Integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Dateizugriff, EML-Normalisierung, Settingsprojektion und Lua-Textheuristik sind unterschiedliche Änderungsgründe. Befunde: Dateilesen, Normalisierung, Settingsaufbereitung und Lua-Textheuristik teilen eine Datei. Fehlende Extension ist erlaubter EML-Fall. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: FileSystem, Config-Schemavalidierung, SettingDefinition. Aufruferbezug: Registry und config-Reexport für UI/API |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: eml_manifest_remains_valid_without_shroudforge_extension, extension_settings_and_groups_are_read_from_the_neighbor_manifest, extension_targets_apply_to_all_packages_without_changing_mod_json, legacy_eml_without_extension_defaults_to_client_and_server, mod_targets_match_only_the_declared_processes, shipped_mod_packages_have_valid_eml_and_extension_manifests (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Manifestnormalisierung und EML-Kompatibilität. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** read_manifest, parse_manifest_with_extension, infer_api_contract, parse_setting_definition

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** aufteilen. Dateizugriff, EML-Normalisierung, Settingsprojektion und Lua-Textheuristik sind unterschiedliche Änderungsgründe.

**Vorgeschlagene Zielpfade:** `packages/manifest/read.rs`, `packages/manifest/normalize.rs`, `packages/settings/read.rs`, `packages/api_inference.rs`.

### [src/loader/package/src/registry/mod.rs](../../packages/manifest/mod.rs)

Vollständig gelesen, 310 Zeilen. SHA-256: `7f822c9accb91a41abed490f933413351281ab4ae5757d8ce4ef9b6565f50cc1`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Entdeckt ausschließlich Modordner, verwaltet geladene Pakete, verwirft doppelte IDs und prüft Settings, Gruppen, Aktionen und Links. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Entdeckt ausschließlich Modordner, verwaltet geladene Pakete, verwirft doppelte IDs und prüft Settings, Gruppen, Aktionen und Links. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `20: struct ModInner {`<br>`26: pub struct Mod {`<br>`31: pub fn info(&self) -> &ModManifest {`<br>`35: pub fn fs(&self) -> MutexGuard<'_, FileSystem> {`<br>`41: pub struct ModRegistry {`<br>`113: fn load_mod(`<br>`191: pub fn validate_manifest(manifest: &ModManifest) -> Result<(), String> {`<br>`192: fn identifier(value: &str) -> bool {`<br>`307: fn deref(&self) -> &Self::Target {` |
| 4. Aufrufer und Verbraucher | ModEnvironment::load, manifest_reader::parse_manifest_with_extension |
| 5. Verwendete Komponenten | FileSystem, config::read_manifest, UI-links/order.json |
| 6. Eingaben und Ausgaben | Eingaben: Modordner oder Modmanifest Ausgabe: Registry und Vertragsfehler |
| 7. Seiteneffekte und Ressourcen | liest Verzeichnisse, Dateien und eingebettete UI-Linkliste |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Validierung liest eine UI-eigene Linkliste. Registry und umfangreiche Vertragsprüfung liegen zusammen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete und Sollwerte. Technische Kopplung: FileSystem, config::read_manifest, UI-links/order.json. Aufruferbezug: ModEnvironment::load, manifest_reader::parse_manifest_with_extension |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Paketerkennung und Vertragsprüfung. Neue Einordnung: Modpakete und Sollwerte. Besitzer: Modpakete. |

**Belege:** ModRegistry::load/load_mod, validate_manifest

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `packages/manifest/mod.rs`.

### [src/loader/package/src/status.rs](../../execution/runtime/status.rs)

Vollständig gelesen, 506 Zeilen. SHA-256: `4f0b3a088bba759fccfe48eb2116315d6d5c9fc41c00b576231b8ee6137da895`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Aggregiert Konfigurations-, Modplan-, Asset-, Parser-, API- und Runtimezustände für Darstellung anhand frischer Prozessheartbeats und Effekte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Aggregiert Konfigurations-, Modplan-, Asset-, Parser-, API- und Runtimezustände für Darstellung anhand frischer Prozessheartbeats und Effekte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `4: pub fn configuration(root: &Path, server: bool, api: &str) -> Value {`<br>`394: fn failed_asset_startup_is_not_reported_as_another_restart_request() {`<br>`428: fn native_plugin_lifecycle_effects_pass_runtime_status_validation() {`<br>`454: fn restart_required_runtime_effect_passes_runtime_status_validation() {`<br>`478: fn installed_update_state_uses_the_updates_section_schema() {` |
| 4. Aufrufer und Verbraucher | Modloader UI über status::configuration |
| 5. Verwendete Komponenten | Config, News, ModEnvironment, prepared fingerprint, heartbeat.json |
| 6. Eingaben und Ausgaben | Eingaben: Root, Prozessziel und API-Version Ausgabe: JSON-Prüfstatus und Modzustände |
| 7. Seiteneffekte und Ressourcen | liest Heartbeat/Status/Pakete und berechnet Fingerprints |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Tatsächlichen Runtimezustand und Assetvorbereitung liefern. UI-Projektion aggregiert, besitzt aber keine Sollwerte. Befunde: Darstellung berechnet Modplan/Fingerprint und enthält große fachliche Zustandsentscheidung. Scan berücksichtigt ZIP obwohl Runtime-Registry nur Ordner entdeckt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Laufzeit- und Vorbereitungsstatus. Technische Kopplung: Config, News, ModEnvironment, prepared fingerprint, heartbeat.json. Aufruferbezug: Modloader UI über status::configuration |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Eingebettet: failed_asset_startup_is_not_reported_as_another_restart_request, native_plugin_lifecycle_effects_pass_runtime_status_validation, restart_required_runtime_effect_passes_runtime_status_validation, installed_update_state_uses_the_updates_section_schema (nicht ausgeführt) |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Statusaggregation. Neue Einordnung: Laufzeit- und Vorbereitungsstatus. Besitzer: Loader-Status. |

**Belege:** configuration, runtime_fresh, mod_states, checks

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** aufteilen. Tatsächlichen Runtimezustand und Assetvorbereitung liefern. UI-Projektion aggregiert, besitzt aber keine Sollwerte.

**Vorgeschlagene Zielpfade:** `execution/runtime/status.rs`, `modules/modloader/ui/host/snapshot.rs`.

### [src/loader/package/src/update_check.rs](../../modules/updater/system-updates/check_lock.rs)

Vollständig gelesen, 140 Zeilen. SHA-256: `e850633ec66e9c9835997f118fdb923250a6d3648de819b4d7bbbf124e7b17fa`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Koordiniert parallele Systemupdatechecks mit installationsbezogenem Windows-Mutex und fünfminütigem automatischem Cooldown. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Koordiniert parallele Systemupdatechecks mit installationsbezogenem Windows-Mutex und fünfminütigem automatischem Cooldown. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `11: pub enum SystemUpdateCheckStart {`<br>`20: pub fn begin_system_update_check(`<br>`51: pub fn unix_timestamp() -> u64 {`<br>`59: pub struct SystemUpdateCheckLock {`<br>`65: fn try_acquire(root: &Path) -> Result<Option<(Self, bool)>, String> {`<br>`115: fn drop(&mut self) {`<br>`125: pub struct SystemUpdateCheckLock {`<br>`131: fn try_acquire(_: &Path) -> Result<Option<(Self, bool)>, String> {` |
| 4. Aufrufer und Verbraucher | UI und Headless-Updater |
| 5. Verwendete Komponenten | Loaderconfig lastCheckAt, SHA256, Windows-Threading |
| 6. Eingaben und Ausgaben | Eingaben: Root und manuelle/automatische Abfrage Ausgabe: Lock, Cooldown oder AlreadyRunning |
| 7. Seiteneffekte und Ressourcen | liest Config und hält prozessübergreifenden Mutex |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Bereits gemeinsamer Check-Lock, keine Releaseabfrage in dieser Datei. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Updatekoordination. Technische Kopplung: Loaderconfig lastCheckAt, SHA256, Windows-Threading. Aufruferbezug: UI und Headless-Updater |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Updatecheck-Koordination. Neue Einordnung: Updatekoordination. Besitzer: Updates und Installation. |

**Belege:** begin_system_update_check, SystemUpdateCheckLock

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/updater/system-updates/check_lock.rs`.

### [src/loader/package/src/window_icon.rs](../../modules/ui-resources/window_icon.rs)

Vollständig gelesen, 10 Zeilen. SHA-256: `20c37676569845d661cebdc8fd35e330e4eef1e460ff359c136ab9ad280ed6b1`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dekodiert eingebettetes Hammer-PNG und skaliert es für Fenstericons auf256x256RGBA. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dekodiert eingebettetes Hammer-PNG und skaliert es für Fenstericons auf256x256RGBA. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `1: pub fn rgba() -> Result<(Vec<u8>, u32, u32), image::ImageError> {` |
| 4. Aufrufer und Verbraucher | Module über package::window_icon::rgba |
| 5. Verwendete Komponenten | image, assets/shroudedit-hammer.png |
| 6. Eingaben und Ausgaben | Eingaben: eingebettetes PNG Ausgabe: RGBA-Pixel und Größe |
| 7. Seiteneffekte und Ressourcen | keine externe Ausgabe |
| 8. Art der Verantwortung | Infrastruktur/Vertrag |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: UI-Ressource liegt im Paketcrate. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Darstellung. Technische Kopplung: image, assets/shroudedit-hammer.png. Aufruferbezug: Module über package::window_icon::rgba |
| 11. Duplikation | Keine zweite identische Implementierung nachgewiesen. Belegte Überschneidungen separat im Gesamtaudit. |
| 12. Nutzungsstatus | Statisch referenziert, siehe callers. Kein Live-Lauf in dieser Analyse. |
| 13. Risiko bei Änderung oder Verschiebung | Import-, include-, Cargo-/Build- und Fixturepfade der nachgewiesenen Aufrufer ändern. Datenformat und beobachtbares Verhalten der dokumentierten Funktion erhalten. |
| 14. Vorhandener Testschutz | Kein direkter isolierter Test in der Datei nachgewiesen. Externe Abdeckung nicht abschließend belegt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Fensterressource. Neue Einordnung: Darstellung. Besitzer: Gemeinsame UI-Ressourcen. |

**Belege:** rgba

**Evidenzgrenze:** Dateiinhalt nachgewiesen, Systemrolle statisch abgeleitet, kein Live-Nachweis.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/ui-resources/window_icon.rs`.

## Lua-API, Wertcodecs und Modausführung

106 Dateien, 33.831 Zeilen.

### [src/loader/api/build.rs](../../execution/build.rs)

Vollständig gelesen, 79 Zeilen. SHA-256: `0ff6df78eb9e795eaeef1d3aee0714291183342d62456453bda260cd83809f6c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Sammelt alle Lua-Dateien unter src rekursiv, verbindet sie zur eingebetteten definitions.lua und setzt zeitbasierten BUILD_ID für Cacheinvalidierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Sammelt alle Lua-Dateien unter src rekursiv, verbindet sie zur eingebetteten definitions.lua und setzt zeitbasierten BUILD_ID für Cacheinvalidierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main Zeile7`<br>`generate_definitions_file Zeile20`<br>`collect_lua_files Zeile69` |
| 4. Aufrufer und Verbraucher | Cargo-Buildskript laut Cargo.toml |
| 5. Verwendete Komponenten | std::fs · OUT_DIR · src/**/*.lua |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Quelldateien unter src und Buildzeit Ausgabe: OUT_DIR/definitions.lua und BUILD_ID |
| 7. Seiteneffekte und Ressourcen | Liest Quellen, schreibt Buildartefakt und Cargoanweisungen |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: collect_lua_files nimmt jede .lua-Datei auf, damit auch ausführbare shroudforge/v1/lua-Implementierungen in die Definitionsausgabe |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Vertragsbuild. Technische Kopplung: std::fs · OUT_DIR · src/**/*.lua. Aufruferbezug: Cargo-Buildskript laut Cargo.toml |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Lua-Auswahl, include_str und Cacheinvalidierung sind an Buildausgabe gebunden. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Build und Lua-Definitionsausgabe. Neue Einordnung: Lua-Vertragsbuild. Besitzer: Modausführung. |

**Belege:** main Zeile7 · generate_definitions_file Zeile20 · collect_lua_files Zeile69

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** refaktorieren. Definitionsquellen explizit auswählen. Ausführbare Lua-Overridehelper nicht versehentlich in die reine Definitionsausgabe einsammeln.

**Vorgeschlagene Zielpfade:** `execution/build.rs`.

### [src/loader/api/examples/inspect_attribute_resources.rs](../../tools/inspection/inspect_attribute_resources.rs)

Vollständig gelesen, 102 Zeilen. SHA-256: `ae714107663790b8d9e5e9928bf7b5ec4f2e701e0d8acac6b63b83ad71517af0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Extrahiert Typen frisch aus einer Spiel-EXE und exportiert ausgewählte Attribut-/Balancingressourcen sowie Attributkomponententypen aus KFC oder Originalbackup als JSON. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Extrahiert Typen frisch aus einer Spiel-EXE und exportiert ausgewählte Attribut-/Balancingressourcen sowie Attributkomponententypen aus KFC oder Originalbackup als JSON. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main Zeile14`<br>`default selected_types Zeilen39–48`<br>`Ressourcenexport Zeilen70–95` |
| 4. Aufrufer und Verbraucher | Cargo-Example inspect_attribute_resources, main |
| 5. Verwendete Komponenten | KFCReader/File · TypeRegistry · kfc_format · serde_json |
| 6. Eingaben und Ausgaben | Eingaben: EXE, OUTPUT.json, --backup, optionale --type-Auswahl Ausgabe: JSON mit Ressourcenwerten, Typen und Versionsangaben |
| 7. Seiteneffekte und Ressourcen | liest EXE/KFC und schreibt gewünschten Exportpfad |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: KFCReader/File · TypeRegistry · kfc_format · serde_json. Aufruferbezug: Cargo-Example inspect_attribute_resources, main |
| 11. Duplikation | EXE-/KFC-Auswahl, Backupoption und JSON-Dateiausgabe parallel zu inspect_building_inputs.rs |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Cargo-Examplename und dokumentierter Aufrufpfad erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Attributressourcen-Inspektionswerkzeug. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Spielinspektion. |

**Belege:** main Zeile14 · default selected_types Zeilen39–48 · Ressourcenexport Zeilen70–95

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Werkzeuge verwenden öffentliche Parser-/Reflectionfunktionen. Cargo-example Targets explizit nachziehen.

**Vorgeschlagene Zielpfade:** `tools/inspection/inspect_attribute_resources.rs`.

### [src/loader/api/examples/inspect_building_inputs.rs](../../tools/inspection/inspect_building_inputs.rs)

Vollständig gelesen, 79 Zeilen. SHA-256: `d05cc624d55c64f3c7f8805866197efb225f437893613d2df699f867a3b14d35`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert ItemInfo- und Voxelbauregisterressourcen aus Spiel-KFC oder Originalbackup mit frisch extrahiertem Typregister für Bauanalyse. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert ItemInfo- und Voxelbauregisterressourcen aus Spiel-KFC oder Originalbackup mit frisch extrahiertem Typregister für Bauanalyse. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main Zeile14`<br>`vier Ressourcentypen Zeilen41–46`<br>`ItemInfo-Projektion Zeile58` |
| 4. Aufrufer und Verbraucher | Cargo-Example inspect_building_inputs, main |
| 5. Verwendete Komponenten | KFCReader/File · TypeRegistry · kfc_format · serde_json |
| 6. Eingaben und Ausgaben | Eingaben: EXE, OUTPUT.json und optional --backup Ausgabe: JSON mit Baueingaberessourcen und Version |
| 7. Seiteneffekte und Ressourcen | liest EXE/KFC und schreibt JSON-Export |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: KFCReader/File · TypeRegistry · kfc_format · serde_json. Aufruferbezug: Cargo-Example inspect_building_inputs, main |
| 11. Duplikation | EXE-/KFC-Auswahl und Exportgerüst parallel zu inspect_attribute_resources.rs, fachliche Projektion unterscheidet sich |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Cargo-Example und Skriptaufrufe nachziehen. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Bauressourcen-Inspektionswerkzeug. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Spielinspektion. |

**Belege:** main Zeile14 · vier Ressourcentypen Zeilen41–46 · ItemInfo-Projektion Zeile58

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Werkzeuge verwenden öffentliche Parser-/Reflectionfunktionen. Cargo-example Targets explizit nachziehen.

**Vorgeschlagene Zielpfade:** `tools/inspection/inspect_building_inputs.rs`.

### [src/loader/api/examples/inspect_runtime_functions.rs](../../tools/inspection/inspect_runtime_functions.rs)

Vollständig gelesen, 198 Zeilen. SHA-256: `63c70082b38738f1a20802e3eca17d6f23ed2aa3bf71ec9caa8d2a18bd6b087d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft aufgezeichnete Funktions-/Registerinventare gegen EXE-Identität und löst ihre Systemabhängigkeiten über denselben reinen Resolver wie die Runtime auf, mit explizitem Altformatadapter. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft aufgezeichnete Funktions-/Registerinventare gegen EXE-Identität und löst ihre Systemabhängigkeiten über denselben reinen Resolver wie die Runtime auf, mit explizitem Altformatadapter. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `path-Module Zeilen6–9`<br>`provider_capture_adapter Zeile30`<br>`Identitätsprüfung Zeilen89–125`<br>`Resolver Zeilen132–174` |
| 4. Aufrufer und Verbraucher | Cargo-Example inspect_runtime_functions, main |
| 5. Verwendete Komponenten | runtime_resolution/functions.rs und types.rs via path · TypeRegistry · SHA256 · PE-Header · Provideraufzeichnung |
| 6. Eingaben und Ausgaben | Eingaben: EXE, Funktions-JSON, Register-JSON, Ausgabepfad und Formatwahl Ausgabe: Abhängigkeitslayouts, Abdeckung und ausdrücklich dokumentierte Evidenzgrenzen als JSON |
| 7. Seiteneffekte und Ressourcen | Dateilesen und JSON-Ausgabe, keine Prozessmutation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: runtime_resolution/functions.rs und types.rs via path · TypeRegistry · SHA256 · PE-Header · Provideraufzeichnung. Aufruferbezug: Cargo-Example inspect_runtime_functions, main |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Relative path-Einbindung des produktiven Resolvers muss mitgezogen werden. |
| 14. Vorhandener Testschutz | Resolverinterne Tests gehören zur separat gelesenen functions.rs |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtimefunktions-Inspektionswerkzeug. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Spielinspektion. |

**Belege:** path-Module Zeilen6–9 · provider_capture_adapter Zeile30 · Identitätsprüfung Zeilen89–125 · Resolver Zeilen132–174

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Werkzeuge verwenden öffentliche Parser-/Reflectionfunktionen. Cargo-example Targets explizit nachziehen.

**Vorgeschlagene Zielpfade:** `tools/inspection/inspect_runtime_functions.rs`.

### [src/loader/api/src/alias.rs](../../execution/alias.rs)

Vollständig gelesen, 12 Zeilen. SHA-256: `22b6d97ab8f6ecb8bb960de5254ac1dcdce08a9e30c7e3bc0bcef92396943bc9`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet UTF-8-Pfade und KFC-Typ- beziehungsweise Ressourcenansichten an die im API verwendeten Rc-Träger. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet UTF-8-Pfade und KFC-Typ- beziehungsweise Ressourcenansichten an die im API verwendeten Rc-Träger. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Path/PathBuf Zeilen5–6`<br>`MappedValue/MappedStruct/MappedArray/MappedVariant Zeilen9–12` |
| 4. Aufrufer und Verbraucher | load.rs · lib.rs · env/game/assets/value/*.rs · env/app_state.rs |
| 5. Verwendete Komponenten | camino · kfc::reflection · kfc::resource::mapped · Rc |
| 6. Eingaben und Ausgaben | Eingaben: KFC-Typen und gewählte Besitzer Rc Ausgabe: Typaliasnamen für Pfade und gemappte Daten |
| 7. Seiteneffekte und Ressourcen | keine Laufzeitwirkung durch die Aliasdefinition |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: camino · kfc::reflection · kfc::resource::mapped · Rc. Aufruferbezug: load.rs · lib.rs · env/game/assets/value/*.rs · env/app_state.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Viele API-Imports und Rc-Besitzverträge hängen an diesen Aliasnamen. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gemeinsame Datenrepräsentationen. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** Path/PathBuf Zeilen5–6 · MappedValue/MappedStruct/MappedArray/MappedVariant Zeilen9–12

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/alias.rs`.

### [src/loader/api/src/cache/io.rs](../../execution/preparation/cache/io.rs)

Vollständig gelesen, 75 Zeilen. SHA-256: `854053e1085111e26037205371462938dafb3c1a630ac6a55a6f942f90294dba`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt und schreibt files.json als Dateiänderungscache, fehlende oder defekte Eingaben ergeben einen absichtlich ungültigen Cache. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt und schreibt files.json als Dateiänderungscache, fehlende oder defekte Eingaben ergeben einen absichtlich ungültigen Cache. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `FileStateCache::read Zeile6`<br>`FileStateCache::write Zeile41` |
| 4. Aufrufer und Verbraucher | lib.rs run_with_api Zeile204 und Cacheaktualisierung Zeilen288–300 |
| 5. Verwendete Komponenten | FileStateCache · serde_json · BufReader/BufWriter · cache_dir/files.json |
| 6. Eingaben und Ausgaben | Eingaben: Cacheordner und FileStateCache Ausgabe: Cachewert oder files.json |
| 7. Seiteneffekte und Ressourcen | Dateilesen, Verzeichniserstellung, JSON-Schreiben und Logging |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Assetänderungscache. Technische Kopplung: FileStateCache · serde_json · BufReader/BufWriter · cache_dir/files.json. Aufruferbezug: lib.rs run_with_api Zeile204 und Cacheaktualisierung Zeilen288–300 |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Lesefehler müssen weiterhin einen schmutzigen Cache ergeben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Vorbereitungs-Cachepersistenz. Neue Einordnung: Assetänderungscache. Besitzer: Modausführung. |

**Belege:** FileStateCache::read Zeile6 · FileStateCache::write Zeile41

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/preparation/cache/io.rs`.

### [src/loader/api/src/cache/mod.rs](../../execution/preparation/cache/mod.rs)

Vollständig gelesen, 111 Zeilen. SHA-256: `921fa01785322b0e75ffe89cfa328c73d1226d878ec7ea2b6f8c117db0b0bf52`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Speichert Buildkennung und Dateizeitstempel und bestimmt daraus, ob Vorbereitungsergebnisse neu erzeugt werden müssen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Speichert Buildkennung und Dateizeitstempel und bestimmt daraus, ob Vorbereitungsergebnisse neu erzeugt werden müssen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `FileStateCache Zeile11`<br>`track Zeile33`<br>`CacheDiff Zeile46`<br>`diff Zeile78` |
| 4. Aufrufer und Verbraucher | lib.rs run_with_api, CacheDiff und Cacheaktualisierung |
| 5. Verwendete Komponenten | BUILD_ID · HashMap<PathBuf,u64> · io · track |
| 6. Eingaben und Ausgaben | Eingaben: Dateipfade, Dateimetadaten und zwei Cachezustände Ausgabe: Zeitstempelkarte und CacheDiff |
| 7. Seiteneffekte und Ressourcen | Dateimetadatenlesen |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Assetänderungscache. Technische Kopplung: BUILD_ID · HashMap<PathBuf,u64> · io · track. Aufruferbezug: lib.rs run_with_api, CacheDiff und Cacheaktualisierung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. BUILD_ID und Gleichheit bestimmen notwendige neue Vorbereitung. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Vorbereitungs-Cachemodell. Neue Einordnung: Assetänderungscache. Besitzer: Modausführung. |

**Belege:** FileStateCache Zeile11 · track Zeile33 · CacheDiff Zeile46 · diff Zeile78

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/preparation/cache/mod.rs`.

### [src/loader/api/src/cache/track.rs](../../execution/preparation/cache/track.rs)

Vollständig gelesen, 100 Zeilen. SHA-256: `eddef7450b9c2df084403052577cadca6cb21150f1b92418e423ecf2fe0c4bda`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erfasst Konfiguration, Spieldateien mit dat/kfc/exe und rekursiv die entpackten Modordner für den Dateiänderungscache. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erfasst Konfiguration, Spieldateien mit dat/kfc/exe und rekursiv die entpackten Modordner für den Dateiänderungscache. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `track_mod_config Zeile8`<br>`track_game_files Zeile15`<br>`track_mod_files Zeile53`<br>`track_directory Zeile57` |
| 4. Aufrufer und Verbraucher | lib.rs Cacheaufbau |
| 5. Verwendete Komponenten | mod_loader::config::document_path · WalkDir · FileStateCache::track |
| 6. Eingaben und Ausgaben | Eingaben: Spiel-/Modordner und Konfigurationspfad Ausgabe: registrierte Pfade mit Zeitstempeln |
| 7. Seiteneffekte und Ressourcen | Verzeichnisdurchlauf mit Symlinkverfolgung und Logging |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Assetänderungscache. Technische Kopplung: mod_loader::config::document_path · WalkDir · FileStateCache::track. Aufruferbezug: lib.rs Cacheaufbau |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Umfang der beobachteten Dateien muss beim Umzug erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Vorbereitungs-Dateiinventar. Neue Einordnung: Assetänderungscache. Besitzer: Modausführung. |

**Belege:** track_mod_config Zeile8 · track_game_files Zeile15 · track_mod_files Zeile53 · track_directory Zeile57

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/preparation/cache/track.rs`.

### [src/loader/api/src/definition/generator.rs](../../execution/definition/generator.rs)

Vollständig gelesen, 185 Zeilen. SHA-256: `0493fb64a6d55f355c35638bd8945c2cbfed13f7be8e730d48cfa3384e6007a6`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Übersetzt reflektierte KFC-Enums und Structlayouts einschließlich Vererbung, Container und DS-Metadaten in Lua-Language-Server-Typdefinitionen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Übersetzt reflektierte KFC-Enums und Structlayouts einschließlich Vererbung, Container und DS-Metadaten in Lua-Language-Server-Typdefinitionen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `generate Zeile9`<br>`append_type_name Zeile74`<br>`append_qualified_name Zeile171` |
| 4. Aufrufer und Verbraucher | load.rs export_lua_definitions Zeile450 · env/runtime_values.rs installed_registries_resolve_all_indices_when_explicitly_requested Zeile1252 |
| 5. Verwendete Komponenten | kfc::reflection::TypeRegistry · PrimitiveType · TypeFlags |
| 6. Eingaben und Ausgaben | Eingaben: KFC-TypeRegistry Ausgabe: Lua-Typdefinitionstext |
| 7. Seiteneffekte und Ressourcen | keine Dateiausgabe in dieser Datei |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: kfc::reflection::TypeRegistry · PrimitiveType · TypeFlags. Aufruferbezug: load.rs export_lua_definitions Zeile450 · env/runtime_values.rs installed_registries_resolve_all_indices_when_explicitly_requested Zeile1252 |
| 11. Duplikation | Keine austauschbare Duplikation zu parser/src/snapshot.rs generate_lua. Dieser Generator nutzt TypeRegistry, die tatsächlichen API-Containeransichten und NativeDs-Typen. Der Snapshotgenerator nutzt die öffentliche GameSchema-Projektion und erzeugt andere Dokumentationsfelder/Containerformen. |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Namespace- und Containerabbildung sind Teil der Modwerkzeugoberfläche. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Reflexionsbasierte Lua-Definitionsausgabe. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** generate Zeile9 · append_type_name Zeile74 · append_qualified_name Zeile171

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/definition/generator.rs`.

### [src/loader/api/src/definition/mod.rs](../../execution/definition/mod.rs)

Vollständig gelesen, 3 Zeilen. SHA-256: `f91e873233d28e80f1de329db07a89c499461e2b1628b4f17033cb86c64e7419`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert den reflektierten Definitionsgenerator und bindet die vom Build erzeugte gemeinsame Lua-Definitionsdatei ein. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert den reflektierten Definitionsgenerator und bindet die vom Build erzeugte gemeinsame Lua-Definitionsdatei ein. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `pub mod generator Zeile1`<br>`DEFINITION_FILE Zeile3` |
| 4. Aufrufer und Verbraucher | load.rs export_lua_definitions Zeile468 |
| 5. Verwendete Komponenten | generator · OUT_DIR/definitions.lua |
| 6. Eingaben und Ausgaben | Eingaben: OUT_DIR/definitions.lua Ausgabe: eingebetteter Definitionstext und Generatorreexport |
| 7. Seiteneffekte und Ressourcen | include_str zur Compilezeit |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: generator · OUT_DIR/definitions.lua. Aufruferbezug: load.rs export_lua_definitions Zeile468 |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Verbindung zu build.rs und load.rs erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Definitionsmodul. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** pub mod generator Zeile1 · DEFINITION_FILE Zeile3

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/definition/mod.rs`.

### [src/loader/api/src/eml/mod.rs](../../execution/lua/eml/mod.rs)

Vollständig gelesen, 1 Zeilen. SHA-256: `24191681066d5cf6a3022ad59c1baa92b57e3ce7717bb78d6a682af9e19a38c4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet die EML-v1-Kompatibilitätsimplementierung intern ein. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet die EML-v1-Kompatibilitätsimplementierung intern ein. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `pub(crate) mod v1 Zeile1` |
| 4. Aufrufer und Verbraucher | lib.rs Moduldeklaration |
| 5. Verwendete Komponenten | eml::v1 |
| 6. Eingaben und Ausgaben | Eingaben: EML-v1-Untermodul Ausgabe: internes Versionsmodul |
| 7. Seiteneffekte und Ressourcen | keine eigene Laufzeitaktion |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: eml::v1. Aufruferbezug: lib.rs Moduldeklaration |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Modulpfade und Sichtbarkeit aktualisieren. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: EML-Kompatibilitätsmodul. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** pub(crate) mod v1 Zeile1

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/mod.rs`.

### [src/loader/api/src/eml/v1/definitions/assets/content.lua](../../execution/lua/eml/v1/definitions/assets/content.lua)

Vollständig gelesen, 16 Zeilen. SHA-256: `1329cbefc9d94de596dbdd3d09dac26d08db348357fe658d8ac2e6443e086616`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert Content-GUID, Größe, Hashwörter und binäres Lesen für Lua-Werkzeuge. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert Content-GUID, Größe, Hashwörter und binäres Lesen für Lua-Werkzeuge. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Content-Klasse Zeilen5–11`<br>`read_data Zeile16` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Guid · Buffer · env/game/assets/content.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Guid · Buffer · env/game/assets/content.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Inhaltsdaten. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** Content-Klasse Zeilen5–11 · read_data Zeile16

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/assets/content.lua`.

### [src/loader/api/src/eml/v1/definitions/assets/manager.lua](../../execution/lua/eml/v1/definitions/assets/manager.lua)

Vollständig gelesen, 195 Zeilen. SHA-256: `3295b83a10e9ce69fc8dd39cdef21c62bb3bec18778a0b2a460a01a372ed9efb`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beschreibt die Lua-Assetoberfläche für Dateidaten, neue Assets, geladene Ressourcensnapshots, gemeinsame Feldindizes und geprüfte skalare Laufzeitschreibvorgänge. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beschreibt die Lua-Assetoberfläche für Dateidaten, neue Assets, geladene Ressourcensnapshots, gemeinsame Feldindizes und geprüfte skalare Laufzeitschreibvorgänge. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `get_resource Zeile13`<br>`get_loaded_resource_by_field Zeile77`<br>`prepare_loaded_index Zeile90`<br>`write_loaded_field Zeile135`<br>`create_content Zeile195` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Resource · Content · Type · LoadedResourceIndexProgress · env/game/assets.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Dateibasierte Vorbereitung und Laufzeitressourcenzugriff teilen denselben AssetManager-Vertrag mit phasenabhängigen Rechten |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Resource · Content · Type · LoadedResourceIndexProgress · env/game/assets.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Assetzugriff. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** get_resource Zeile13 · get_loaded_resource_by_field Zeile77 · prepare_loaded_index Zeile90 · write_loaded_field Zeile135 · create_content Zeile195

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/assets/manager.lua`.

### [src/loader/api/src/eml/v1/definitions/assets/resource.lua](../../execution/lua/eml/v1/definitions/assets/resource.lua)

Vollständig gelesen, 16 Zeilen. SHA-256: `7f241409caaf0752f18aaec0ae63f56d5002252130af26c3a2c536227fb3e64f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert die dokumentierte Lua-Sicht auf Ressourcendaten samt Herkunft, Ladezustand und Weltbindung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert die dokumentierte Lua-Sicht auf Ressourcendaten samt Herkunft, Ladezustand und Weltbindung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Resource-Klasse Zeilen6–15`<br>`TODO original_data Zeile3` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Guid · Type · env/game/assets/resource.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: original_data ist hier dokumentiert und als TODO markiert. Die vollständig gelesene env/game/assets/resource.rs registriert dieses Feld nicht. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Guid · Type · env/game/assets/resource.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Ressourcen. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** Resource-Klasse Zeilen6–15 · TODO original_data Zeile3

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/assets/resource.lua`.

### [src/loader/api/src/eml/v1/definitions/blob/image.lua](../../execution/lua/eml/v1/definitions/blob/image.lua)

Vollständig gelesen, 111 Zeilen. SHA-256: `b9ea6add9b0210b1f1c82774b2db3b9f9a0cf8831f21656ba48b7bf104b6a21d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beschreibt Lua-Bilder, Pixelzugriff und Umwandlung zwischen Bilddateiformaten, Spieltexturen und Puffern. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beschreibt Lua-Bilder, Pixelzugriff und Umwandlung zwischen Bilddateiformaten, Spieltexturen und Puffern. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `ImageFormat Zeilen3–20`<br>`image.decode_texture Zeile53`<br>`Image:set_pixel_packed Zeile111` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | ImageFormat · Buffer · keen.PixelFormat · env/image.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: ImageFormat · Buffer · keen.PixelFormat · env/image.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Bilddaten. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** ImageFormat Zeilen3–20 · image.decode_texture Zeile53 · Image:set_pixel_packed Zeile111

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/blob/image.lua`.

### [src/loader/api/src/eml/v1/definitions/buffer.lua](../../execution/lua/eml/v1/definitions/buffer.lua)

Vollständig gelesen, 488 Zeilen. SHA-256: `fea889128245df0a82b320312c408ad9b444217df61796a329b450a7a76b699a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert binäre Lua-Puffer mit Lese-/Schreibposition, Kapazität, Byteordnung, primitiven Codecs und Ressourcenserialisierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert binäre Lua-Puffer mit Lese-/Schreibposition, Kapazität, Byteordnung, primitiven Codecs und Ressourcenserialisierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `buffer.create Zeile17`<br>`Buffer Zeile70`<br>`ByteOrder Zeile154`<br>`read_resource Zeile366`<br>`write_resource Zeile479` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Type · ByteOrder · env/buffer.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Type · ByteOrder · env/buffer.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Binärpuffer. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** buffer.create Zeile17 · Buffer Zeile70 · ByteOrder Zeile154 · read_resource Zeile366 · write_resource Zeile479

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/buffer.lua`.

### [src/loader/api/src/eml/v1/definitions/game.lua](../../execution/lua/eml/v1/definitions/game.lua)

Vollständig gelesen, 11 Zeilen. SHA-256: `77138faeb6ad6b88f26fa34d0aeac3ee8c3543862fe220e9eb9c28614ca0f3bd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert den game-Einstiegspunkt mit Spielversion, Assets, GUID-Helfern und Typregister. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert den game-Einstiegspunkt mit Spielversion, Assets, GUID-Helfern und Typregister. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Game-Felder Zeilen7–10`<br>`game Zeile11` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | AssetManager · GuidHelper · TypeRegistry · env/game.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: AssetManager · GuidHelper · TypeRegistry · env/game.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Spielzugriff. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** Game-Felder Zeilen7–10 · game Zeile11

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/game.lua`.

### [src/loader/api/src/eml/v1/definitions/global.lua](../../execution/lua/eml/v1/definitions/global.lua)

Vollständig gelesen, 97 Zeilen. SHA-256: `43ffcc28e2025ecb3e7a7f7a9cd4fed058f68ba75da06d92300b1072e7451712`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert Lua-Typaliasnamen für Zahlen, Container und GUIDs sowie typeof, require und die global verfügbaren Bibliotheken. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert Lua-Typaliasnamen für Zahlen, Container und GUIDs sowie typeof, require und die global verfügbaren Bibliotheken. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Zahlenaliasnamen Zeilen3–40`<br>`Guid Zeilen59–63`<br>`require Zeile85`<br>`builtin Zeilen88–97` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | integer · game · buffer · hasher · runtime · shroudforge · image |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: integer · game · buffer · hasher · runtime · shroudforge · image. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Grundvertrag. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** Zahlenaliasnamen Zeilen3–40 · Guid Zeilen59–63 · require Zeile85 · builtin Zeilen88–97

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/global.lua`.

### [src/loader/api/src/eml/v1/definitions/guid.lua](../../execution/lua/eml/v1/definitions/guid.lua)

Vollständig gelesen, 24 Zeilen. SHA-256: `aaaccb34667449496bfdba3c57665a90a1ab19c7b0be87121b088d07125fca0b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert Null-GUID und Umwandlung beziehungsweise FNV-Hash zwischen GUID und ContentHash. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert Null-GUID und Umwandlung beziehungsweise FNV-Hash zwischen GUID und ContentHash. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `GuidHelper Zeilen5–7`<br>`from_content_hash Zeile12`<br>`hash Zeile24` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Guid · keen.ContentHash · env/game/guid.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Guid · keen.ContentHash · env/game/guid.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Assetkennungen. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** GuidHelper Zeilen5–7 · from_content_hash Zeile12 · hash Zeile24

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/guid.lua`.

### [src/loader/api/src/eml/v1/definitions/hash.lua](../../execution/lua/eml/v1/definitions/hash.lua)

Vollständig gelesen, 24 Zeilen. SHA-256: `b348f92359a8a36cd5095c5dddf7af173a29d6c1f9af17756ab8ff74592baf7c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert FNV-1a32-, CRC32- und CRC64-Prüfsummen für Lua-Zeichenfolgen und Puffer. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert FNV-1a32-, CRC32- und CRC64-Prüfsummen für Lua-Zeichenfolgen und Puffer. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `hasher.fnv1a32 Zeile12`<br>`hasher.crc32 Zeile18`<br>`hasher.crc64 Zeile24` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Buffer · env/hasher.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Buffer · env/hasher.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Prüfsummen. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** hasher.fnv1a32 Zeile12 · hasher.crc32 Zeile18 · hasher.crc64 Zeile24

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/hash.lua`.

### [src/loader/api/src/eml/v1/definitions/integer.lua](../../execution/lua/eml/v1/definitions/integer.lua)

Vollständig gelesen, 2448 Zeilen. SHA-256: `7562cae6d614027b17a5fe4d692b1d51a29d27c03f9aaf47304be54509c05807`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert für acht feste Ganzzahltypen 8/16/32/64 Bit die Grenzen, Konvertierungen, Bitoperationen und checked/saturating/wrapping/overflowing-Arithmetik. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert für acht feste Ganzzahltypen 8/16/32/64 Bit die Grenzen, Konvertierungen, Bitoperationen und checked/saturating/wrapping/overflowing-Arithmetik. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `integer.u8 Zeile14`<br>`integer.u64 Zeile920`<br>`integer.i8 Zeile1222`<br>`integer.i64 bis Zeile2448` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | env/integer.rs · u8/u16/u32/u64/i8/i16/i32/i64 |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: env/integer.rs · u8/u16/u32/u64/i8/i16/i32/i64. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Ganzzahlarithmetik. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** integer.u8 Zeile14 · integer.u64 Zeile920 · integer.i8 Zeile1222 · integer.i64 bis Zeile2448

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/integer.lua`.

### [src/loader/api/src/eml/v1/definitions/io.lua](../../execution/lua/eml/v1/definitions/io.lua)

Vollständig gelesen, 236 Zeilen. SHA-256: `d573a724e634c70e21237c6148a1b921f6f4bf4139f438a2f475a7680c929358`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beschreibt lesenden Moddateizugriff, Pfadhelfer und rechtgebundene Dateioperationen im Exportordner. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beschreibt lesenden Moddateizugriff, Pfadhelfer und rechtgebundene Dateioperationen im Exportordner. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `io.read Zeile21`<br>`io.join Zeile180`<br>`io.export Zeile205`<br>`export_exists bis export_delete Zeilen207–236` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | Buffer · env/io.rs · export-Berechtigung |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: Buffer · env/io.rs · export-Berechtigung. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Moddateien und Export. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** io.read Zeile21 · io.join Zeile180 · io.export Zeile205 · export_exists bis export_delete Zeilen207–236

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/io.lua`.

### [src/loader/api/src/eml/v1/definitions/loader.lua](../../execution/lua/eml/v1/definitions/loader.lua)

Vollständig gelesen, 24 Zeilen. SHA-256: `a3237a001ab6258745b851835baefff4a9561ec727808d454d6b06d1e35d8a49`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert den EML-loader-Kompatibilitätsvertrag für Prozessart, Modrechte, Modabfrage und DLL-Registrierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert den EML-loader-Kompatibilitätsvertrag für Prozessart, Modrechte, Modabfrage und DLL-Registrierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `EmlLoaderFeatures Zeilen3–9`<br>`loader.has_mod Zeile21`<br>`EmlLoaderRuntime Zeilen23–24` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | eml/v1/loader.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: eml/v1/loader.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: EML-Lua-Vertrag. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** EmlLoaderFeatures Zeilen3–9 · loader.has_mod Zeile21 · EmlLoaderRuntime Zeilen23–24

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/loader.lua`.

### [src/loader/api/src/eml/v1/definitions/types.lua](../../execution/lua/eml/v1/definitions/types.lua)

Vollständig gelesen, 161 Zeilen. SHA-256: `ef8565505626cd8eb14f31f32b8290f621b02449c45be68c9b55f8194d52bbba`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beschreibt das reflektierte Typregister, exakte Typauflösung, Hashkollisionen, Vererbung, effektive Felder und native DS-Layouts für Lua. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beschreibt das reflektierte Typregister, exakte Typauflösung, Hashkollisionen, Vererbung, effektive Felder und native DS-Layouts für Lua. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `TypeRegistry.resolve Zeile23`<br>`Type-Klasse Zeilen69–93`<br>`ResolvedStructField Zeilen101–104`<br>`PrimitiveType Zeilen124–155` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | TypeRegistry · Type · PrimitiveType · TypeFlag · env/game/types.rs |
| 6. Eingaben und Ausgaben | Eingaben: Dokumentierte Lua-Typnamen und Methodensignaturen Ausgabe: Lua-Language-Server-Vertrag in build.rs definitions.lua |
| 7. Seiteneffekte und Ressourcen | Im belegten Buildpfad nur Textquelle für Definitionsausgabe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: TypeRegistry · Type · PrimitiveType · TypeFlag · env/game/types.rs. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rekursive Buildaufnahme und Übereinstimmung mit ausführender Rust-Bindung müssen erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Vertrag für Typreflexion. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** TypeRegistry.resolve Zeile23 · Type-Klasse Zeilen69–93 · ResolvedStructField Zeilen101–104 · PrimitiveType Zeilen124–155

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/definitions/types.lua`.

### [src/loader/api/src/eml/v1/loader.rs](../../execution/lua/eml/v1/loader.rs)

Vollständig gelesen, 48 Zeilen. SHA-256: `97955e08dba710135ae4f18bd620df47e8298a8a151b7989307f42d2b6c4fdb2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erstellt den alten loader-Namensraum mit Client-/Serverstatus, Modabfrage, erteilten Patch-/Exportrechten und paketlokaler DLL-Registrierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erstellt den alten loader-Namensraum mit Client-/Serverstatus, Modabfrage, erteilten Patch-/Exportrechten und paketlokaler DLL-Registrierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create_loader Zeile7`<br>`has_mod Zeile14`<br>`features Zeilen19–28`<br>`register_dll Zeile34` |
| 4. Aufrufer und Verbraucher | env/mod.rs create_env Zeile73 |
| 5. Verwendete Komponenten | AppState · env::loader::available · Mod · AppFeatures::RUNTIME_DLL · load_mod_native_dll |
| 6. Eingaben und Ausgaben | Eingaben: Lua-VM, Mod und AppState Ausgabe: EML-loader-Tabelle |
| 7. Seiteneffekte und Ressourcen | DLL-Laden bei ausdrücklich aufgerufenem register_dll über AppState |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: AppState · env::loader::available · Mod · AppFeatures::RUNTIME_DLL · load_mod_native_dll. Aufruferbezug: env/mod.rs create_env Zeile73 |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Historischer Namensraum und Fähigkeitsprüfung müssen für EML-Mods erhalten bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: EML-Lua-Laufzeitadapter. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** create_loader Zeile7 · has_mod Zeile14 · features Zeilen19–28 · register_dll Zeile34

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/loader.rs`.

### [src/loader/api/src/eml/v1/mod.rs](../../execution/lua/eml/v1/mod.rs)

Vollständig gelesen, 3 Zeilen. SHA-256: `c8f21c26ecbe90a01b597a542b6c2474efea6b3af8b1cc9c0e45421ad22b3151`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert die Erstellung des EML-v1-loader-Namensraums an die Lua-Umgebung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert die Erstellung des EML-v1-loader-Namensraums an die Lua-Umgebung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `mod loader Zeile1`<br>`pub(crate) use Zeile3` |
| 4. Aufrufer und Verbraucher | env/mod.rs create_env |
| 5. Verwendete Komponenten | loader::create_loader |
| 6. Eingaben und Ausgaben | Eingaben: create_loader-Implementierung Ausgabe: interner Funktionsreexport |
| 7. Seiteneffekte und Ressourcen | keine eigene Laufzeitaktion |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: EML-Lua-Kompatibilität. Technische Kopplung: loader::create_loader. Aufruferbezug: env/mod.rs create_env |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. EML-Kompatibilitätsaufrufer erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: EML-Kompatibilitätsmodul. Neue Einordnung: EML-Lua-Kompatibilität. Besitzer: Modausführung. |

**Belege:** mod loader Zeile1 · pub(crate) use Zeile3

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/eml/v1/mod.rs`.

### [src/loader/api/src/env/app_state.rs](../../execution/context.rs)

Vollständig gelesen, 1353 Zeilen. SHA-256: `523b5ed2bc51b88a6cb6f6eea45c55be9f97926e7c8f2d09a672430fa52c8d8f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Besitzt Ausführungskontext aus Modumgebung, Reflection/KFC-Lesern und Writertransaktion, Ressourcen-/Typcaches, geladenen Ressourcensnapshots mit Sessionprüfung, Schreibbesitz/-fehlern und isolierter nativer Mod-DLL-Verwaltung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Besitzt Ausführungskontext aus Modumgebung, Reflection/KFC-Lesern und Writertransaktion, Ressourcen-/Typcaches, geladenen Ressourcensnapshots mit Sessionprüfung, Schreibbesitz/-fehlern und isolierter nativer Mod-DLL-Verwaltung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `37: pub struct AppState {`<br>`67: pub struct AppConfig {`<br>`87: pub fn new(`<br>`276: pub fn env(&self) -> &ModEnvironment {`<br>`281: pub fn api(&self) -> &ShroudForgeApi {`<br>`287: pub fn skip_cache(&self) -> bool {`<br>`292: pub fn is_server(&self) -> bool {`<br>`297: pub fn is_client(&self) -> bool {`<br>`302: pub fn phase(&self) -> RuntimePhase {`<br>`306: pub(crate) fn set_runtime_mod_active(&self, id: &str, active: bool) {`<br>`335: fn native_dll_state(&self, id: &str) -> Option<&'static str> {`<br>`350: pub(crate) fn runtime_mod_is_active(&self, id: &str) -> bool {`<br>`354: pub(crate) fn runtime_ownership(&self) -> Rc<RefCell<super::runtime_ownership::OwnershipStore>> {`<br>`358: pub(crate) fn runtime_write_fault(&self) -> Option<String> {`<br>`369: pub(crate) fn record_uncertain_runtime_write(&self, session: u64, reason: &str) {`<br>`380: pub(crate) fn report_runtime_effect(&self, id: &str, state: &str, detail: &str) {`<br>`400: pub(crate) fn runtime_effects(&self) -> serde_json::Value {`<br>`411: pub fn export_dir(&self) -> &PathBuf {`<br>`416: pub fn has_feature(&self, feature: AppFeatures) -> bool {`<br>`420: pub(crate) fn load_mod_native_dll(`<br>`438: fn queue_mod_native_dll(&self, target_mod: &Mod, relative_path: &str) -> Result<(), String> {`<br>`584: pub(crate) fn poll_native_dll_loads(&self) {`<br>`637: pub(crate) fn load_native_dll_declaration(&self, target_mod: &Mod) -> Result<bool, String> {`<br>`712: fn unload_native_dll(&self, handle: isize) {`<br>`733: pub fn kfc_file(&self) -> &KFCFile {`<br>`738: pub fn type_registry(&self) -> &Rc<TypeRegistry> {`<br>`742: pub(crate) fn attribute_catalog(&self) -> Result<&super::runtime_attributes::Catalog, String> {`<br>`756: pub fn reader(&self) -> RefMut<'_, KFCCursor<KFCReader>> {`<br>`761: pub fn take_writer(&self) -> mlua::Result<KFCWriter<Rc<KFCFile>, Rc<TypeRegistry>>> {`<br>`768: pub fn commit_assets(&self) -> mlua::Result<()> {`<br>`781: pub fn get_cached_resources(&self) -> Vec<Rc<ResourceInfo>> {`<br>`785: pub fn get_resource_info(&self, guid: &ResourceId) -> Option<Rc<ResourceInfo>> {`<br>`812: pub fn get_loaded_resource_info(&self, resource_id: &ResourceId) -> Rc<ResourceInfo> {`<br>`819: pub(crate) fn loaded_resource_indexes(&self) -> RefMut<'_, super::game::LoadedResourceIndexes> {`<br>`823: pub(crate) fn advance_loaded_resource_indexes(&self) {`<br>`829: pub(crate) fn inline_field(`<br>`850: pub(crate) fn invalidate_loaded_resource_indexes(&self) {`<br>`856: pub(crate) fn compare_exchange_loaded_resource_scalar(`<br>`876: pub(crate) fn get_loaded_resource_info_for_session(`<br>`886: pub(crate) fn get_loaded_resource_info_from_snapshot(`<br>`895: pub fn add_resource(`<br>`939: pub fn create_resource(`<br>`972: pub fn get_content(&self, guid: &ContentHash) -> mlua::Result<Option<Vec<u8>>> {`<br>`982: pub fn create_content(&self, data: &[u8]) -> mlua::Result<ContentHash> {`<br>`1002: pub fn get_type(`<br>`1025: fn drop(&mut self) {`<br>`1050: struct NativeDll {`<br>`1058: struct PendingNativeDll {`<br>`1068: fn load_native_library(mod_id: &str, path: &std::path::Path) -> Result<isize, String> {`<br>`1101: fn unload_native_library(handle: isize) {`<br>`1110: fn load_native_library(mod_id: &str, _path: &std::path::Path) -> Result<isize, String> {`<br>`1117: fn unload_native_library(_handle: isize) {}`<br>`1119: pub struct ResourceInfo {`<br>`1132: enum ResourceSource {`<br>`1139: fn loaded(resource_id: ResourceId, creation_session: u64) -> Self {`<br>`1152: fn loaded_snapshot(resource_id: ResourceId, session: u64, snapshot: Vec<u8>) -> Self {`<br>`1158: pub(crate) fn creation_session(&self) -> Option<u64> {`<br>`1162: pub(crate) fn require_runtime_write_session(&self, current: u64) -> Result<u64, String> {`<br>`1175: pub(crate) fn stop_runtime_writes(&self, reason: &str) {`<br>`1179: pub(crate) fn loaded_snapshot_bytes(&self) -> Result<&Rc<[u8]>, String> {`<br>`1198: pub fn source_name(&self) -> &'static str {`<br>`1206: pub fn get_lua_value(&self, lua: &mlua::Lua) -> mlua::Result<LuaValue> {`<br>`1221: pub fn set_lua_value(&self, lua_value: LuaValue, lua: &mlua::Lua) -> mlua::Result<()> {`<br>`1247: pub fn apply(&self, lua: &mlua::Lua) -> mlua::Result<Option<Value>> {`<br>`1276: fn get_mapped_value(&self, lua: &mlua::Lua) -> mlua::Result<Option<&MappedValue>> {`<br>`1320: fn loaded_resource_write_scope_rejects_menu_and_foreign_world_snapshots() {`<br>`1342: fn verified_resource_keeps_its_exact_readback_and_stopped_source_cannot_repeat() {` |
| 4. Aufrufer und Verbraucher | LuaModRunner und env-Bindings über lua.app_data_ref::<AppState>, IngameRuntime |
| 5. Verwendete Komponenten | kfc, AppFeatures/RunArgs, ModEnvironment, load, runtime_values/attributes/ownership, Provider, mpsc und Win32 |
| 6. Eingaben und Ausgaben | Eingaben: ModEnvironment, optionale API, RunArgs, CacheDiff, ResourceIds und Lua-/Bytewerte Ausgabe: AppState, typisierte ResourceInfo/Typeobjekte, Native-DLL-Resultate und Assetcommit |
| 7. Seiteneffekte und Ressourcen | liest Reflection/KFC, staged Spieldateien, startet LoadLibrary-Worker, hält Channels, lädt/entlädt DLLs, arbeitet mit Tempfiles und Caches |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. KFC-Session, Live-Snapshots/CAS und Native-Mod-DLLs haben verschiedene Ressourcen und Freigaberegeln. Befunde: Dateiassets, Live-Ressourcen und DLL-Verwaltung besitzen unterschiedliche Lifetimes innerhalb eines Besitzers. Archivextraktion ist separater Pfad bei filesystem.absolute_path=None, keine EML-Extensionerstellung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Ausführungskontext und Ressourcenbesitz. Technische Kopplung: kfc, AppFeatures/RunArgs, ModEnvironment, load, runtime_values/attributes/ownership, Provider, mpsc und Win32. Aufruferbezug: LuaModRunner und env-Bindings über lua.app_data_ref::<AppState>, IngameRuntime |
| 11. Duplikation | Überschneidungen mit Workflow-Assetplanung und Providerzugängen, keine pauschale Gleichsetzung unterschiedlicher Lebenszeiten |
| 12. Nutzungsstatus | Statisch belegter Produktaufruf, keine Liveausführung im Audit |
| 13. Risiko bei Änderung oder Verschiebung | Sessionbindung, write-fault Sperren, exaktes Snapshotreadback, Ownership und DLL-/Writerlifetimes müssen beim Zuschnitt erhalten bleiben |
| 14. Vorhandener Testschutz | Eingebettet: loaded_resource_write_scope_rejects_menu_and_foreign_world_snapshots, verified_resource_keeps_its_exact_readback_and_stopped_source_cannot_repeat. Integrationstests unter api/tests, im Audit nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Ausführungskontext mit Asset-, Runtime- und DLL-Lebensdauer. Neue Einordnung: Ausführungskontext und Ressourcenbesitz. Besitzer: Modausführung. |

**Belege:** AppState::new/commit_assets/queue_mod_native_dll/poll_native_dll_loads, ResourceInfo, ResourceSource, PendingNativeDll, Drop

**Evidenzgrenze:** Dateiinhalt und Aufrufe nachgewiesen, Verhalten aus statischem Kontrollfluss abgeleitet

**Maßnahme:** aufteilen. KFC-Session, Live-Snapshots/CAS und Native-Mod-DLLs haben verschiedene Ressourcen und Freigaberegeln.

**Vorgeschlagene Zielpfade:** `execution/context.rs`, `execution/assets/session.rs`, `execution/loaded-resources/session.rs`, `execution/native_mods.rs`.

### [src/loader/api/src/env/buffer.rs](../../execution/lua/bindings/buffer.rs)

Vollständig gelesen, 487 Zeilen. SHA-256: `764b73453a5740f4b764f5c3ce33772ef23edc0f911c3b3b828374b100e29a45`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Implementiert Lua-Binärpuffer und ihre Methodenbindung einschließlich reflektiertem Lesen und Schreiben von KFC-Ressourcenwerten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Implementiert Lua-Binärpuffer und ihre Methodenbindung einschließlich reflektiertem Lesen und Schreiben von KFC-Ressourcenwerten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Buffer Zeile52`<br>`UserData Zeile105`<br>`lua_read_resource Zeile340`<br>`lua_write_resource Zeile451` |
| 4. Aufrufer und Verbraucher | env/mod.rs register · env/io.rs · env/hasher.rs · env/image.rs · env/game/assets/content.rs |
| 5. Verwendete Komponenten | buffer/error,misc,order,read,write · MappedValue · AppState · Wertkonverter und Validator |
| 6. Eingaben und Ausgaben | Eingaben: Kapazität/Stringbytes und Lua-Puffermethoden Ausgabe: Buffer, primitive Werte und reflektierte Ressourcenwerte |
| 7. Seiteneffekte und Ressourcen | mutiert eigenen Bytepuffer, Lua-Allokationen |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Generische Pufferverwaltung und KFC-Ressourcenserialisierung teilen dieselbe Bindungsdatei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: buffer/error,misc,order,read,write · MappedValue · AppState · Wertkonverter und Validator. Aufruferbezug: env/mod.rs register · env/io.rs · env/hasher.rs · env/image.rs · env/game/assets/content.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Pufferpositionen, Rechte und Ressourcenserialisierung sind gekoppelt. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Binärpuffer. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Buffer Zeile52 · UserData Zeile105 · lua_read_resource Zeile340 · lua_write_resource Zeile451

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer.rs`.

### [src/loader/api/src/env/buffer/error.rs](../../execution/lua/bindings/buffer/error.rs)

Vollständig gelesen, 26 Zeilen. SHA-256: `d175aa7e55482f2844d9b7700f4102e39343eef5ea516829481e739d0a3a13ea`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert Pufferfehler für Grenzen, Kapazität, Zugriffsmodus und geschlossenen Zustand und übersetzt sie in Lua-Fehler. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert Pufferfehler für Grenzen, Kapazität, Zugriffsmodus und geschlossenen Zustand und übersetzt sie in Lua-Fehler. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `BufferError Zeile6`<br>`From<BufferError> Zeile22` |
| 4. Aufrufer und Verbraucher | env/buffer.rs und buffer-Unterdateien |
| 5. Verwendete Komponenten | thiserror · mlua |
| 6. Eingaben und Ausgaben | Eingaben: Pufferfehlerzustand Ausgabe: BufferError beziehungsweise mlua::Error |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: thiserror · mlua. Aufruferbezug: env/buffer.rs und buffer-Unterdateien |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Fehlerkonvertierungen und Importpfade erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Binärpuffer-Fehlermodell. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** BufferError Zeile6 · From<BufferError> Zeile22

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer/error.rs`.

### [src/loader/api/src/env/buffer/misc.rs](../../execution/lua/bindings/buffer/misc.rs)

Vollständig gelesen, 244 Zeilen. SHA-256: `480457039c8161e7abe84aea4857925e56ae64e0430d95876cbd36ca6f813c4f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verwaltet Lese-/Schreibmodus, Positionen, Kapazität, Byteordnung, Kopieren und Bereichsprüfung des Binärpuffers. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verwaltet Lese-/Schreibmodus, Positionen, Kapazität, Byteordnung, Kopieren und Bereichsprüfung des Binärpuffers. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `set_readable Zeile9`<br>`reserve Zeile117`<br>`copy_within Zeile171`<br>`check_read_bounds Zeile220` |
| 4. Aufrufer und Verbraucher | env/buffer.rs · buffer/read.rs · buffer/write.rs |
| 5. Verwendete Komponenten | Buffer · BufferState · BufferError · ByteOrder |
| 6. Eingaben und Ausgaben | Eingaben: Positionen, Modus, Kopierbereiche und Kapazitätsbedarf Ausgabe: geprüfter Pufferzustand und Bytesicht |
| 7. Seiteneffekte und Ressourcen | eigene Vektormutation und Allokation |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: Buffer · BufferState · BufferError · ByteOrder. Aufruferbezug: env/buffer.rs · buffer/read.rs · buffer/write.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Grenz- und Modusprüfung muss identisch bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Binärpuffer-Zustandsverwaltung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** set_readable Zeile9 · reserve Zeile117 · copy_within Zeile171 · check_read_bounds Zeile220

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer/misc.rs`.

### [src/loader/api/src/env/buffer/order.rs](../../execution/lua/bindings/buffer/order.rs)

Vollständig gelesen, 42 Zeilen. SHA-256: `a73be6bf1f6b94d491dc9989101472156731949f80c79cc882ebf671a8188bbe`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Übersetzt zwischen Big-/Little-Endian-Pufferbyteordnung und den Lua-Strings big/little. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Übersetzt zwischen Big-/Little-Endian-Pufferbyteordnung und den Lua-Strings big/little. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `ByteOrder Zeile6`<br>`CastLua Zeile11`<br>`IntoLua Zeile34` |
| 4. Aufrufer und Verbraucher | env/buffer.rs lua_order · buffer/read.rs · buffer/write.rs |
| 5. Verwendete Komponenten | CastLua · IntoLua |
| 6. Eingaben und Ausgaben | Eingaben: Lua-String big/little oder ByteOrder Ausgabe: ByteOrder oder Lua-String |
| 7. Seiteneffekte und Ressourcen | Lua-Stringallokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: CastLua · IntoLua. Aufruferbezug: env/buffer.rs lua_order · buffer/read.rs · buffer/write.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Akzeptierte Byteordnungsnamen erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Binärpuffer-Byteordnung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** ByteOrder Zeile6 · CastLua Zeile11 · IntoLua Zeile34

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer/order.rs`.

### [src/loader/api/src/env/buffer/read.rs](../../execution/lua/bindings/buffer/read.rs)

Vollständig gelesen, 204 Zeilen. SHA-256: `8eff4de8bdc5c1b08456b05eef62dfde286716a8c5cfec1131a9fd209a0caca2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest bool, vorzeichenbehaftete und vorzeichenlose Ganzzahlen, Halb-/Einfach-/Doppelfloats und Bytes mit geprüften Grenzen und Byteordnung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest bool, vorzeichenbehaftete und vorzeichenlose Ganzzahlen, Halb-/Einfach-/Doppelfloats und Bytes mit geprüften Grenzen und Byteordnung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `read_bool Zeile6`<br>`read_u16 Zeile32`<br>`read_f16 Zeile137`<br>`read_bytes Zeile194` |
| 4. Aufrufer und Verbraucher | env/buffer.rs Lua-Lesemethoden |
| 5. Verwendete Komponenten | Buffer · ByteOrder · half::f16 |
| 6. Eingaben und Ausgaben | Eingaben: Pufferbytes, Kopfposition und Byteordnung Ausgabe: primitive Zahlen oder Bytesequenz |
| 7. Seiteneffekte und Ressourcen | verschiebt Pufferleseposition |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: Buffer · ByteOrder · half::f16. Aufruferbezug: env/buffer.rs Lua-Lesemethoden |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Endian- und Positionseffekte sind beobachtbar. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Binärpuffer-Lesecodecs. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** read_bool Zeile6 · read_u16 Zeile32 · read_f16 Zeile137 · read_bytes Zeile194

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer/read.rs`.

### [src/loader/api/src/env/buffer/write.rs](../../execution/lua/bindings/buffer/write.rs)

Vollständig gelesen, 211 Zeilen. SHA-256: `5b4cdb8a4b4c6546330517c2715d62fd242d1e4143b4b6cf8122cf63e1257fe0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Schreibt primitive Zahlen, bool und Bytes mit Byteordnung und automatischer geprüfter Kapazitätserweiterung an die Pufferendposition. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Schreibt primitive Zahlen, bool und Bytes mit Byteordnung und automatischer geprüfter Kapazitätserweiterung an die Pufferendposition. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `write_bool Zeile6`<br>`write_u16 Zeile39`<br>`write_f16 Zeile147`<br>`write_bytes Zeile200` |
| 4. Aufrufer und Verbraucher | env/buffer.rs Lua-Schreibmethoden |
| 5. Verwendete Komponenten | Buffer · ByteOrder · half::f16 |
| 6. Eingaben und Ausgaben | Eingaben: primitive Zahlen/Bytes, Endposition und Byteordnung Ausgabe: angehängte Bytes im Puffer |
| 7. Seiteneffekte und Ressourcen | Vektorallokation und verschobene Endposition |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: Buffer · ByteOrder · half::f16. Aufruferbezug: env/buffer.rs Lua-Schreibmethoden |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Endian- und Wachstumseffekte erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Binärpuffer-Schreibcodecs. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** write_bool Zeile6 · write_u16 Zeile39 · write_f16 Zeile147 · write_bytes Zeile200

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer/write.rs`.

### [src/loader/api/src/env/buffer_transform.rs](../../execution/lua/bindings/buffer_transform.rs)

Vollständig gelesen, 237 Zeilen. SHA-256: `033cac71d04d763b3d8361c8fc36ff9d4a964abf85a29178d322919c29d3d767`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft begrenzte x64-Kopierroutinen vollständig bis RET und führt ausschließlich den bewiesenen Kopierplan auf eigenen Bytepuffern aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft begrenzte x64-Kopierroutinen vollständig bis RET und führt ausschließlich den bewiesenen Kopierplan auf eigenen Bytepuffern aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Plan::apply Zeile32`<br>`prove Zeile51`<br>`unhex Zeile178`<br>`positive/negative Tests Zeilen193–237` |
| 4. Aufrufer und Verbraucher | env/runtime_functions.rs Puffertransformationsbeschreibung und Anwendung |
| 5. Verwendete Komponenten | serde::Serialize · Windows-x64-Registerkonvention |
| 6. Eingaben und Ausgaben | Eingaben: bis zu512 x64-Codebytes und eigene Quell-/Zielbytes Ausgabe: bewiesener Kopierplan oder transformierte eigene Bytes |
| 7. Seiteneffekte und Ressourcen | nur eigene Zielkopie, keine native Ausführung |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: serde::Serialize · Windows-x64-Registerkonvention. Aufruferbezug: env/runtime_functions.rs Puffertransformationsbeschreibung und Anwendung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Vollständigkeit des begrenzten Instruktionsbeweises darf nicht abgeschwächt werden. |
| 14. Vorhandener Testschutz | owned_copy_roundtrip_preserves_unwritten_bytes_and_rejects_short_buffers · rejects_unproved_control_flow_aliases_globals_widths_and_truncation |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Nachgewiesene Puffertransformation. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Plan::apply Zeile32 · prove Zeile51 · unhex Zeile178 · positive/negative Tests Zeilen193–237

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/buffer_transform.rs`.

### [src/loader/api/src/env/game.rs](../../execution/lua/bindings/game.rs)

Vollständig gelesen, 22 Zeilen. SHA-256: `728136178686b5111faeb8d69564b8bfaa1449be36d760543a5439d011660751`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erstellt game mit KFC-Spielversion, Assetzugriff, Typregister und GUID-Helfern und exportiert die gemeinsamen Spielwerttypen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erstellt game mit KFC-Spielversion, Assetzugriff, Typregister und GUID-Helfern und exportiert die gemeinsamen Spielwerttypen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile12`<br>`game-Felder Zeilen16–19`<br>`LoadedResourceIndexes Zeilen8–9` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | AppState · assets · types · guid |
| 6. Eingaben und Ausgaben | Eingaben: Lua-VM, Mod und AppState Ausgabe: game-Tabelle mit Version/assets/types/guid |
| 7. Seiteneffekte und Ressourcen | Lua-Tabellen- und Funktionsallokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: AppState · assets · types · guid. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. game und runtime verwenden gemeinsame Typ-/Assetansichten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Spielzugriffseinstieg. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile12 · game-Felder Zeilen16–19 · LoadedResourceIndexes Zeilen8–9

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game.rs`.

### [src/loader/api/src/env/game/assets.rs](../../execution/lua/assets/files.rs)

Vollständig gelesen, 485 Zeilen. SHA-256: `5788824b73c9b75f3175512fc463d72187f80725c7403a36e36b18846233230f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet Assetabfragen und Erzeugung aus KFC/Transaktionen sowie Live-ResourceService-Snapshots, Feldindizes, ItemRegistry-Auflösung und abgesicherte einzelne 32-Bit-Schreibvorgänge an Lua. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet Assetabfragen und Erzeugung aus KFC/Transaktionen sowie Live-ResourceService-Snapshots, Feldindizes, ItemRegistry-Auflösung und abgesicherte einzelne 32-Bit-Schreibvorgänge an Lua. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile22`<br>`lua_resolve_loaded_resource Zeile140`<br>`lua_get_loaded_resource Zeile186`<br>`lua_read_loaded_field Zeile258`<br>`lua_write_loaded_field Zeile287`<br>`lua_create_resource Zeile415` |
| 4. Aufrufer und Verbraucher | env/game.rs create |
| 5. Verwendete Komponenten | AppState · runtime_provider · runtime_values · runtime_ownership · loaded_index · kfc_format · Mod |
| 6. Eingaben und Ausgaben | Eingaben: Lua-GUID/Typ/Teil · Weltgebundene Resource-Snapshots · Feldpfad, neuer und erwarteter Skalar · Modrechte Ausgabe: Resource-/Content-Userdata · typisierte Feldwerte · Fehlergründe |
| 7. Seiteneffekte und Ressourcen | Assettransaktionen · gemeinsame Indexregistrierung · native CAS-Schreibvorgänge über Provider · Sperre nach ungewissem Schreibausgang |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Vorbereitete KFC-Dateien und tatsächlich geladener ResourceService-Speicher haben unterschiedliche Besitzer. Bestehendes game.assets bleibt der öffentliche Einstieg. Befunde: Dateibasierte Assetvorbereitung, Live-Ressourcenmutation und konkret g38_items::ItemRegistry gebundene Auflösung teilen dieselbe Datei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: AppState · runtime_provider · runtime_values · runtime_ownership · loaded_index · kfc_format · Mod. Aufruferbezug: env/game.rs create |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Phasenrechte, Sitzungsbindung und unabhängig verifizierte Schreibresultate müssen bei jeder Trennung unverändert bleiben. |
| 14. Vorhandener Testschutz | loaded_index_tests.rs für Indexteil · Direkte CAS-Schreibprüfung in dieser Datei nicht nachgewiesen. Lua-Flugfixtures simulieren den öffentlichen Ressourcenvertrag. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Assetzugriff und Laufzeitressourcen. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile22 · lua_resolve_loaded_resource Zeile140 · lua_get_loaded_resource Zeile186 · lua_read_loaded_field Zeile258 · lua_write_loaded_field Zeile287 · lua_create_resource Zeile415

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** aufteilen. Vorbereitete KFC-Dateien und tatsächlich geladener ResourceService-Speicher haben unterschiedliche Besitzer. Bestehendes game.assets bleibt der öffentliche Einstieg.

**Vorgeschlagene Zielpfade:** `execution/lua/assets/files.rs`, `execution/lua/runtime/loaded_resources.rs`.

### [src/loader/api/src/env/game/assets/content.rs](../../execution/lua/bindings/game/assets/content.rs)

Vollständig gelesen, 55 Zeilen. SHA-256: `88d881e2d328b4c70b740ff3152b77c0e83cc050f1a7c070be80910a433358e7`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt ContentHash-Daten als Lua-Userdata dar und liest Inhalte über AppState in unveränderliche Puffer mit Lua-GC-Anrechnung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt ContentHash-Daten als Lua-Userdata dar und liest Inhalte über AppState in unveränderliche Puffer mit Lua-GC-Anrechnung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Content Zeile9`<br>`get_data Zeile22`<br>`UserData Zeile39` |
| 4. Aufrufer und Verbraucher | env/game/assets.rs Contentabfragen und -erzeugung |
| 5. Verwendete Komponenten | ContentHash · AppState::get_content · Buffer |
| 6. Eingaben und Ausgaben | Eingaben: ContentHash und AppState-Inhaltszugriff Ausgabe: Content-Userdata und schreibgeschützter Buffer |
| 7. Seiteneffekte und Ressourcen | liest Inhaltsbytes und rechnet Speicher an Lua-GC an |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: ContentHash · AppState::get_content · Buffer. Aufruferbezug: env/game/assets.rs Contentabfragen und -erzeugung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Inhaltsbesitz und Schreibschutz erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Inhaltsansicht. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Content Zeile9 · get_data Zeile22 · UserData Zeile39

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/content.rs`.

### [src/loader/api/src/env/game/assets/loaded_index.rs](../../execution/lua/runtime/loaded_resources/index.rs)

Vollständig gelesen, 718 Zeilen. SHA-256: `f2e657fcc9370c80226a98c8f1dd08557e6ac2838ef056ab3973598c45b507e5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verwaltet modübergreifend begrenzte, weltgebundene Ressourcenfeldindizes und treibt deren skalare Suche in budgetierten Runden voran, bestätigt eindeutige Treffer durch frischen Vollsnapshot. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verwaltet modübergreifend begrenzte, weltgebundene Ressourcenfeldindizes und treibt deren skalare Suche in budgetierten Runden voran, bestätigt eindeutige Treffer durch frischen Vollsnapshot. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Indexes Zeile96`<br>`register Zeile166`<br>`lookup Zeile227`<br>`advance Zeile286`<br>`ScalarField Zeile411`<br>`prepare Zeile583`<br>`tick Zeile676` |
| 4. Aufrufer und Verbraucher | assets.rs prepare_loaded_index/get_loaded_resource_by_field · env/app_state.rs advance_loaded_resource_indexes Zeile823 via env/game.rs Reexport · AppState Invalidierung und Besitzerbereinigung |
| 5. Verwendete Komponenten | runtime_provider · runtime_values::InlineField · AppState · ResourceId · Modbesitzer · Instant |
| 6. Eingaben und Ausgaben | Eingaben: Typ, Inlinefeldpfad, Skalarwert, Modbesitzer und Weltsitzung · Provider-IDs/Skalarbatches/Vollsnapshots Ausgabe: bestätigter Resource-Snapshot · Fortschritt, Pending-/Fehlergründe und Diagnostik |
| 7. Seiteneffekte und Ressourcen | gemeinsame begrenzte Warteschlange · Providerlesevorgänge · sitzungs- und mutationsabhängige Cacheinvalidierung |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: runtime_provider · runtime_values::InlineField · AppState · ResourceId · Modbesitzer · Instant. Aufruferbezug: assets.rs prepare_loaded_index/get_loaded_resource_by_field · env/app_state.rs advance_loaded_resource_indexes Zeile823 via env/game.rs Reexport · AppState Invalidierung und Besitzerbereinigung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Ein gemeinsamer Runtimebesitzer muss Fortschritt, Limits, Eigentümer und selektive Invalidierung erhalten. |
| 14. Vorhandener Testschutz | loaded_index_tests.rs vollständig zugeordnete 12 Tests |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gemeinsame Live-Ressourcenindizes. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Indexes Zeile96 · register Zeile166 · lookup Zeile227 · advance Zeile286 · ScalarField Zeile411 · prepare Zeile583 · tick Zeile676

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/loaded_resources/index.rs`.

### [src/loader/api/src/env/game/assets/loaded_index_tests.rs](../../execution/lua/runtime/loaded_resources/index_tests.rs)

Vollständig gelesen, 572 Zeilen. SHA-256: `464b7d18d78cc449f405f37a2dbdc05c7db2476358fc5ce06b8ba151c01ea964`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft Ressourcenindexjobs mit synthetischem Backend auf selbstständigen Fortschritt, Eindeutigkeit, Weltwechsel, Besitzer, Limits, Invalidierung, Backoff und typisierte Skalarcodecs. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft Ressourcenindexjobs mit synthetischem Backend auf selbstständigen Fortschritt, Eindeutigkeit, Weltwechsel, Besitzer, Limits, Invalidierung, Backoff und typisierte Skalarcodecs. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Backend Zeile5`<br>`one_preparation_finishes_without_more_lua_lookups_or_an_active_action Zeile113`<br>`active_capacity_is_not_evicted_and_owner_cleanup_preserves_other_interests Zeile382`<br>`fixed_scalar_codec_rejects_dynamic_paths_structs_unaligned_and_nonfinite_values Zeile502` |
| 4. Aufrufer und Verbraucher | loaded_index.rs cfg(test) path-Modul |
| 5. Verwendete Komponenten | loaded_index.rs interne Typen · env::registry::tests::fixture · mlua |
| 6. Eingaben und Ausgaben | Eingaben: synthetische ResourceIds, Skalare, Zeitpunkte und Typregister Ausgabe: Assertions über Indexzustände und Lesezahlen |
| 7. Seiteneffekte und Ressourcen | nur synthetischer In-Memory-Testzustand |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: loaded_index.rs interne Typen · env::registry::tests::fixture · mlua. Aufruferbezug: loaded_index.rs cfg(test) path-Modul |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Tests greifen über super auf interne Indexzustände und müssen mit dem Modul umziehen. |
| 14. Vorhandener Testschutz | Diese Datei definiert die schützenden Tests, wurde nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Ressourcenindex-Verhaltenstests. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Backend Zeile5 · one_preparation_finishes_without_more_lua_lookups_or_an_active_action Zeile113 · active_capacity_is_not_evicted_and_owner_cleanup_preserves_other_interests Zeile382 · fixed_scalar_codec_rejects_dynamic_paths_structs_unaligned_and_nonfinite_values Zeile502

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/loaded_resources/index_tests.rs`.

### [src/loader/api/src/env/game/assets/resource.rs](../../execution/lua/bindings/game/assets/resource.rs)

Vollständig gelesen, 51 Zeilen. SHA-256: `6b1892b765e2d78b26c9aa72139a6bf409620b97641b6d213a773d7eb1966874`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet ResourceInfo als Lua-Resource mit Kennung, Typ, Herkunft, Sitzung und les-/schreibbarer Datenproperty an. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet ResourceInfo als Lua-Resource mit Kennung, Typ, Herkunft, Sitzung und les-/schreibbarer Datenproperty an. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Resource Zeile11`<br>`UserData Zeile25`<br>`data getter/setter Zeilen46–49` |
| 4. Aufrufer und Verbraucher | env/game/assets.rs Ressourcenabfragen und Erzeugung |
| 5. Verwendete Komponenten | Rc<ResourceInfo> · AppState · kfc_format::type_for_resource |
| 6. Eingaben und Ausgaben | Eingaben: Rc<ResourceInfo> und AppState-Typregister Ausgabe: Resource-Userdata mit Herkunft/Typ/Sitzung/Daten |
| 7. Seiteneffekte und Ressourcen | delegierte ResourceInfo-Datenmutation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Das zugehörige resource.lua dokumentiert original_data, die vollständig gelesene UserData-Feldregistrierung bietet dieses Feld nicht an. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: Rc<ResourceInfo> · AppState · kfc_format::type_for_resource. Aufruferbezug: env/game/assets.rs Ressourcenabfragen und Erzeugung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. ResourceInfo entscheidet über Datei-/Transaktions-/Livezustand und Schreibrechte. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Ressourcenansicht. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Resource Zeile11 · UserData Zeile25 · data getter/setter Zeilen46–49

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/resource.rs`.

### [src/loader/api/src/env/game/assets/value.rs](../../execution/lua/bindings/game/assets/value.rs)

Vollständig gelesen, 254 Zeilen. SHA-256: `2e686a95ce07409e9cfa742993cddfd3ac5c115f90a955af3aaba23cc32150b0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Fasst Validator und Konverter zusammen und verteilt Dirtyprüfung, Arraymutation, Quellbytes und Typabfragen auf eigene sowie gemappte Lua-Wertrepräsentationen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Fasst Validator und Konverter zusammen und verteilt Dirtyprüfung, Arraymutation, Quellbytes und Typabfragen auf eigene sowie gemappte Lua-Wertrepräsentationen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `convert_lua_to_value Zeile25`<br>`is_dirty_lua_value Zeile44`<br>`mapped_source_bytes Zeile73`<br>`push Zeile85`<br>`type_of Zeile199` |
| 4. Aufrufer und Verbraucher | env/table.rs · env/game/types.rs · env/buffer.rs · env/app_state.rs · env/runtime_values.rs |
| 5. Verwendete Komponenten | converter · mapped · simple · validator · TypeId · TypeHandle |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Werte/Userdata · reflektierter Typ · Arrayposition und Mutation Ausgabe: KFC-Value · geklonte Lua-Werte · Dirty-/Typinformationen |
| 7. Seiteneffekte und Ressourcen | mutiert eigene Lua-Arraywerte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: converter · mapped · simple · validator · TypeId · TypeHandle. Aufruferbezug: env/table.rs · env/game/types.rs · env/buffer.rs · env/app_state.rs · env/runtime_values.rs |
| 11. Duplikation | TypeId-Verteilung für sechs Repräsentationen wiederholt sich in is_dirty_lua_value, type_of, Converter und Validator |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Reexporte über env::value und Typidentität der Userdata müssen konsistent umgestellt werden. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Reflektierte Lua-Wertoperationen. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** convert_lua_to_value Zeile25 · is_dirty_lua_value Zeile44 · mapped_source_bytes Zeile73 · push Zeile85 · type_of Zeile199

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/value.rs`.

### [src/loader/api/src/env/game/assets/value/converter.rs](../../execution/lua/bindings/game/assets/value/converter.rs)

Vollständig gelesen, 560 Zeilen. SHA-256: `1721c3a6a1ad392f27d0bff68c980b4ed79f3d5702ebaee016fd2363a017673a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Wandelt typisierte normalisierte Lua-Werte in KFC-Werte um und gemappte KFC-Daten in Lua-Skalare beziehungsweise verzögert gelesene Userdata. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Wandelt typisierte normalisierte Lua-Werte in KFC-Werte um und gemappte KFC-Daten in Lua-Skalare beziehungsweise verzögert gelesene Userdata. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Converter::convert Zeile48`<br>`process Zeile65`<br>`try_convert_struct_ud Zeile376`<br>`value_to_lua Zeile486` |
| 4. Aufrufer und Verbraucher | value.rs · mapped.rs · simple.rs · env/buffer.rs · env/app_state.rs |
| 5. Verwendete Komponenten | MappedValue · TypeHandle · simple/mapped-Wertansichten · TreePath · kfc::resource::value::Value |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Wert und TypeHandle · MappedValue Ausgabe: KFC-Value oder Lua-Wert · Fehler mit Feldpfad |
| 7. Seiteneffekte und Ressourcen | Lua-Userdata-Allokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: DS-Array/String/Optional/Variant-Zweige sind mit todo! implementiert und dürfen nicht als unterstützte Serialisierung eingestuft werden |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: MappedValue · TypeHandle · simple/mapped-Wertansichten · TreePath · kfc::resource::value::Value. Aufruferbezug: value.rs · mapped.rs · simple.rs · env/buffer.rs · env/app_state.rs |
| 11. Duplikation | primitive Zahlen-/Enum-/GUID-Prüfungen entsprechen Validator::process_* mit anderem Ausgabetyp |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Validatornormalisierung, Bytekodierung und sämtliche sechs Userdataformen sind gekoppelt. |
| 14. Vorhandener Testschutz | loaded_index_tests.rs fixed_scalar_codec_rejects_dynamic_paths_structs_unaligned_and_nonfinite_values schützt Skalarpfad indirekt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: KFC-/Lua-Wertkonvertierung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Converter::convert Zeile48 · process Zeile65 · try_convert_struct_ud Zeile376 · value_to_lua Zeile486

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/value/converter.rs`.

### [src/loader/api/src/env/game/assets/value/mapped.rs](../../execution/lua/bindings/game/assets/value/mapped.rs)

Vollständig gelesen, 758 Zeilen. SHA-256: `fdbeb4cbad0ac2abd7a42e41ab3e89f9d1ae3d2c7de6f56d37490587dfb15bf7`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bietet verzögert dekodierte Struct-/Array-/Variantansichten über gehaltene KFC-Bytes mit separatem Änderungscache, rekursiver Dirtyprüfung und unveränderten Quellbytes. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bietet verzögert dekodierte Struct-/Array-/Variantansichten über gehaltene KFC-Bytes mit separatem Änderungscache, rekursiver Dirtyprüfung und unveränderten Quellbytes. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `MappedStructValue Zeile31`<br>`source_bytes Zeile55`<br>`get_field Zeile157`<br>`MappedArrayValue Zeile321`<br>`MappedVariantValue Zeile650` |
| 4. Aufrufer und Verbraucher | value/converter.rs · value/validator.rs · value.rs |
| 5. Verwendete Komponenten | MappedStruct/Array/Variant · TypeHandle · Validator · Converter · Rc/RefCell/Cell · ouroboros |
| 6. Eingaben und Ausgaben | Eingaben: gemappte KFC-Werte und Lua-Mutationen Ausgabe: verzögert gelesene Lua-Werte · KFC-Werte · Quellbytes und Dirtyzustand |
| 7. Seiteneffekte und Ressourcen | eigene Lazycaches und Änderungen, kein direkter Prozessspeicherschreibzugriff |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: MappedStruct/Array/Variant · TypeHandle · Validator · Converter · Rc/RefCell/Cell · ouroboros. Aufruferbezug: value/converter.rs · value/validator.rs · value.rs |
| 11. Duplikation | Array-/Struct-Iteratoren, tostring und Bereichsprüfungen parallel zu simple.rs bei anderer Besitz-/Lazylesesemantik |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Quellbyteerhalt und Dirtypropagation sind Teil der Konfliktprüfung von Laufzeitschreiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gemappte reflektierte Lua-Werte. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** MappedStructValue Zeile31 · source_bytes Zeile55 · get_field Zeile157 · MappedArrayValue Zeile321 · MappedVariantValue Zeile650

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/value/mapped.rs`.

### [src/loader/api/src/env/game/assets/value/simple.rs](../../execution/lua/bindings/game/assets/value/simple.rs)

Vollständig gelesen, 551 Zeilen. SHA-256: `bfc72a4e37ce5a24cab1aec29ca9850018814678b95a753cf7aae6e9bd11027f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Besitzt vollständig normalisierte Lua-Struct-, Array- und Variantwerte mit geprüfter Mutation, tiefer Kopie, Iteration und KFC-Konvertierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Besitzt vollständig normalisierte Lua-Struct-, Array- und Variantwerte mit geprüfter Mutation, tiefer Kopie, Iteration und KFC-Konvertierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `StructValue Zeile21`<br>`ArrayValue Zeile224`<br>`ArrayValue::insert Zeile319`<br>`VariantValue Zeile473` |
| 4. Aufrufer und Verbraucher | value/validator.rs · value/converter.rs · value.rs |
| 5. Verwendete Komponenten | TypeHandle · Validator · Converter · Rc/RefCell · ouroboros |
| 6. Eingaben und Ausgaben | Eingaben: Typ und normalisierte Feld-/Array-/Variantdaten · Lua-Mutationswerte Ausgabe: Userdata, Iteratorwerte, KFC-Value und Kopien |
| 7. Seiteneffekte und Ressourcen | mutiert eigene Rc/RefCell-Werte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: TypeHandle · Validator · Converter · Rc/RefCell · ouroboros. Aufruferbezug: value/validator.rs · value/converter.rs · value.rs |
| 11. Duplikation | Struct-/Array-Iteratoren, Bereichsprüfung und tostring entsprechen mapped.rs, Datenhaltung unterscheidet sich belegt |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Eigene und gemappte Repräsentation dürfen beim gemeinsamen Verhalten nicht versehentlich gleichgesetzt werden. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Eigene reflektierte Lua-Werte. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** StructValue Zeile21 · ArrayValue Zeile224 · ArrayValue::insert Zeile319 · VariantValue Zeile473

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/value/simple.rs`.

### [src/loader/api/src/env/game/assets/value/util.rs](../../execution/lua/bindings/game/assets/value/util.rs)

Vollständig gelesen, 54 Zeilen. SHA-256: `3a8021bba3e92b931dea17fc5fb997c791570071338d18346c10135c7589a0d3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Baut wiederverwendbare verschachtelte Fehlerpfade aus Feldnamen und Arrayindizes für Wertvalidierung und Konvertierung auf. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Baut wiederverwendbare verschachtelte Fehlerpfade aus Feldnamen und Arrayindizes für Wertvalidierung und Konvertierung auf. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `TreePath Zeile3`<br>`push Zeile17`<br>`push_index Zeile27`<br>`Display Zeile46` |
| 4. Aufrufer und Verbraucher | value/converter.rs · value/validator.rs |
| 5. Verwendete Komponenten | std::fmt::Display |
| 6. Eingaben und Ausgaben | Eingaben: Feldnamen und Indizes Ausgabe: durch Punkte getrennter Fehlerpfad |
| 7. Seiteneffekte und Ressourcen | interner wiederverwendeter Stack |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: std::fmt::Display. Aufruferbezug: value/converter.rs · value/validator.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Imports bei Konverter und Validator aktualisieren. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Wertkonvertierungs-Fehlerpfade. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** TreePath Zeile3 · push Zeile17 · push_index Zeile27 · Display Zeile46

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/value/util.rs`.

### [src/loader/api/src/env/game/assets/value/validator.rs](../../execution/lua/bindings/game/assets/value/validator.rs)

Vollständig gelesen, 735 Zeilen. SHA-256: `f6706fa36383aa4882d7f2b2894e882143b4771e9a8288f460e60ab2d22ef0f5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft Lua-Tabellen und Userdata gegen reflektierte Typen und erzeugt unabhängige normalisierte Werte einschließlich Vererbung, Arrays, Varianten und Assetreferenzen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft Lua-Tabellen und Userdata gegen reflektierte Typen und erzeugt unabhängige normalisierte Werte einschließlich Vererbung, Arrays, Varianten und Assetreferenzen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Validator::validate_and_clone Zeile90`<br>`process Zeile111`<br>`process_struct_fields Zeile299`<br>`process_blob_variant Zeile388`<br>`try_clone_lua_value Zeile671` |
| 4. Aufrufer und Verbraucher | value.rs · mapped.rs · simple.rs · env/buffer.rs · env/app_state.rs · env/runtime_values.rs |
| 5. Verwendete Komponenten | TypeHandle · simple/mapped-Werte · TreePath · Guid/ContentHash · Content/Resource |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Tabellen/Userdata/Skalare und reflektierter Zieltyp Ausgabe: normalisierte tiefe Lua-Kopie oder Fehler mit Pfad |
| 7. Seiteneffekte und Ressourcen | Lua-Userdata-Allokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: DS-Containerzweige sind todo! und damit keine implementierte Lua-Wertnormalisierung |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: TypeHandle · simple/mapped-Werte · TreePath · Guid/ContentHash · Content/Resource. Aufruferbezug: value.rs · mapped.rs · simple.rs · env/buffer.rs · env/app_state.rs · env/runtime_values.rs |
| 11. Duplikation | Zahlen-/Enum-/GUID-Prüflogik parallel zu converter.rs, anschließend anderer Ausgabetyp |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Änderung kann Typprüfung, Aliasfreiheit und alle Assetschreibpfade beeinflussen. |
| 14. Vorhandener Testschutz | loaded_index_tests.rs Skalarcodec-Prüfungen indirekt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Reflektierte Lua-Wertvalidierung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Validator::validate_and_clone Zeile90 · process Zeile111 · process_struct_fields Zeile299 · process_blob_variant Zeile388 · try_clone_lua_value Zeile671

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/assets/value/validator.rs`.

### [src/loader/api/src/env/game/guid.rs](../../execution/lua/bindings/game/guid.rs)

Vollständig gelesen, 69 Zeilen. SHA-256: `a760cdfae3015a1ab1c648f0ab1e62247629ee9e21aab16ba92c99f480747504`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet Nullkennung, GUID-/ContentHash-Umwandlung und GUID-Hash32 an Lua. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet Nullkennung, GUID-/ContentHash-Umwandlung und GUID-Hash32 an Lua. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile9`<br>`lua_from_content_hash Zeile21`<br>`lua_to_content_hash Zeile51`<br>`lua_hash Zeile64` |
| 4. Aufrufer und Verbraucher | env/game.rs create |
| 5. Verwendete Komponenten | ContentHash · CastLuaExt · add_function |
| 6. Eingaben und Ausgaben | Eingaben: ContentHash-Tabelle/Userdata oder GUID-String Ausgabe: GUID, Hashworttabelle oder Hash32 |
| 7. Seiteneffekte und Ressourcen | Lua-Tabellen-/Stringallokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: ContentHash-Feldauslese entspricht derselben size/hash0/hash1/hash2-Konvertierung in env/util.rs, dort als CastLua |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: ContentHash · CastLuaExt · add_function. Aufruferbezug: env/game.rs create |
| 11. Duplikation | size/hash0/hash1/hash2-Feldauslese parallel zu CastLua<ContentHash> in env/util.rs |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Historische GUID-/ContentHash-Repräsentation erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Assetkennungen. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile9 · lua_from_content_hash Zeile21 · lua_to_content_hash Zeile51 · lua_hash Zeile64

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/guid.rs`.

### [src/loader/api/src/env/game/types.rs](../../execution/lua/bindings/game/types.rs)

Vollständig gelesen, 518 Zeilen. SHA-256: `a88af4257d1168cdcf408d1b20cb39a0567c1a7bd084990055597cfdc1b554a6`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt das aktuelle KFC-Typregister in Lua bereit, löst Namen/Hashes/Indizes auf und erzeugt Typansichten mit Layout, Vererbung, Feldern, Enums und Attributen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt das aktuelle KFC-Typregister in Lua bereit, löst Namen/Hashes/Indizes auf und erzeugt Typansichten mit Layout, Vererbung, Feldern, Enums und Attributen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile18`<br>`lua_find Zeile153`<br>`Type Zeile232`<br>`UserData Zeile261`<br>`struct_field_to_lua Zeile425`<br>`enum_field_to_lua Zeile499` |
| 4. Aufrufer und Verbraucher | env/game.rs create · AppState::get_type · env-Bindungen mit Typselektoren |
| 5. Verwendete Komponenten | TypeHandle · AppState · registry · type_lookup · ReadOnlyMap/Array · OnceCell |
| 6. Eingaben und Ausgaben | Eingaben: Typregister und Lua-Name/Hash/Index/Filter Ausgabe: Typansichten und Reflexionsmetadaten |
| 7. Seiteneffekte und Ressourcen | Lazycaches für schreibgeschützte Metadatentabellen |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: TypeHandle · AppState · registry · type_lookup · ReadOnlyMap/Array · OnceCell. Aufruferbezug: env/game.rs create · AppState::get_type · env-Bindungen mit Typselektoren |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Typidentität, Vererbung und Hashkollisionen müssen konsistent bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Typreflexion. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile18 · lua_find Zeile153 · Type Zeile232 · UserData Zeile261 · struct_field_to_lua Zeile425 · enum_field_to_lua Zeile499

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/game/types.rs`.

### [src/loader/api/src/env/hasher.rs](../../execution/lua/bindings/hasher.rs)

Vollständig gelesen, 52 Zeilen. SHA-256: `e70392ddf67e72f6f8e84d65dd9aa890a8850d86d6359a369e6ed6ed4a17f2be`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt FNV-1a32, CRC32/ISO-HDLC und CRC64/ECMA-182 für Lua-Zeichenfolgen und Binärpuffer bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt FNV-1a32, CRC32/ISO-HDLC und CRC64/ECMA-182 für Lua-Zeichenfolgen und Binärpuffer bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile9`<br>`lua_fnv1a32 Zeile19`<br>`lua_crc32 Zeile30`<br>`lua_crc64 Zeile42` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | crc · kfc::hash::fnv_bytes · Buffer · FunctionArgs |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Stringbytes oder Buffer Ausgabe: 32-/64-Bit-Prüfsummen |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: crc · kfc::hash::fnv_bytes · Buffer · FunctionArgs. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Algorithmusvarianten und u64-Bitdarstellung erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Prüfsummenadapter. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile9 · lua_fnv1a32 Zeile19 · lua_crc32 Zeile30 · lua_crc64 Zeile42

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/hasher.rs`.

### [src/loader/api/src/env/image.rs](../../execution/lua/bindings/image.rs)

Vollständig gelesen, 104 Zeilen. SHA-256: `55d612353580aac39a608efcbdf5f0bd5d4049dc64d0906c03c13ceba9cff020`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet Bilddateidekodierung/-kodierung und KFC-Texturkonvertierung an Lua-Puffer und Image-Werte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet Bilddateidekodierung/-kodierung und KFC-Texturkonvertierung an Lua-Puffer und Image-Werte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile17`<br>`lua_decode Zeile36`<br>`lua_decode_texture Zeile56`<br>`lua_encode_texture Zeile91` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | image · kfc::content::image · Buffer · Image · format · pixel_format |
| 6. Eingaben und Ausgaben | Eingaben: Bild-/Texturbytes, Format, Dimensionen oder Image Ausgabe: Image oder kodierter Buffer |
| 7. Seiteneffekte und Ressourcen | Bild-/Pufferallokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: image · kfc::content::image · Buffer · Image · format · pixel_format. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Dateiformate und KFC-Texturcodec müssen gleich gebunden bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Bild- und Texturkonvertierung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile17 · lua_decode Zeile36 · lua_decode_texture Zeile56 · lua_encode_texture Zeile91

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/image.rs`.

### [src/loader/api/src/env/image/format.rs](../../execution/lua/bindings/image/format.rs)

Vollständig gelesen, 45 Zeilen. SHA-256: `b6817f6a77809b39530f9df771bf2884488b427e8cfda66a545d429c51312da9`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst die unterstützten Lua-Bildformatnamen einschließlich jpg/jpeg und PCX zu image::ImageFormat auf. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst die unterstützten Lua-Bildformatnamen einschließlich jpg/jpeg und PCX zu image::ImageFormat auf. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `CastLua<ImageFormat> Zeile5`<br>`Formatzuordnung Zeilen18–35` |
| 4. Aufrufer und Verbraucher | env/image.rs decode und encode |
| 5. Verwendete Komponenten | image::ImageFormat · CastLua |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Bildformatname Ausgabe: image::ImageFormat oder Ablehnung |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: image::ImageFormat · CastLua. Aufruferbezug: env/image.rs decode und encode |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Formatnamen sind öffentlicher Lua-Vertrag. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Bildformatkonvertierung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** CastLua<ImageFormat> Zeile5 · Formatzuordnung Zeilen18–35

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/image/format.rs`.

### [src/loader/api/src/env/image/pixel_format.rs](../../execution/lua/bindings/image/pixel_format.rs)

Vollständig gelesen, 175 Zeilen. SHA-256: `a2d2ff640c71ad2b96a50bcf11bd16ae5c1a6c4863562a8372d1f6791cfac86b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Übersetzt die konkreten keen-Pixelformatnamen von Lua nach KFC einschließlich gepackter, Float-, Tiefen- und BC-Texturformate. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Übersetzt die konkreten keen-Pixelformatnamen von Lua nach KFC einschließlich gepackter, Float-, Tiefen- und BC-Texturformate. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `CastLua<PixelFormat> Zeile5`<br>`vollständige Namenszuordnung Zeilen17–163` |
| 4. Aufrufer und Verbraucher | env/image.rs decode_texture und encode_texture |
| 5. Verwendete Komponenten | kfc::content::image::PixelFormat · CastLua |
| 6. Eingaben und Ausgaben | Eingaben: Lua-keen-Pixelformatname Ausgabe: KFC-PixelFormat oder Ablehnung |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: kfc::content::image::PixelFormat · CastLua. Aufruferbezug: env/image.rs decode_texture und encode_texture |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Vollständige Namensabbildung erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Spieltextur-Formatkonvertierung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** CastLua<PixelFormat> Zeile5 · vollständige Namenszuordnung Zeilen17–163

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/image/pixel_format.rs`.

### [src/loader/api/src/env/image/value.rs](../../execution/lua/bindings/image/value.rs)

Vollständig gelesen, 154 Zeilen. SHA-256: `61d5416720f8989ff500ecf54885d6b9e613297626c41a7f826092a8f7fc06e6`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Besitzt RGBA-Bilddaten und stellt Lua-Größe sowie geprüften einzelnen oder gepackten Pixelzugriff bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Besitzt RGBA-Bilddaten und stellt Lua-Größe sowie geprüften einzelnen oder gepackten Pixelzugriff bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Image Zeile6`<br>`UserData Zeile46`<br>`lua_get_pixel Zeile60`<br>`lua_set_pixel_packed Zeile125` |
| 4. Aufrufer und Verbraucher | env/image.rs |
| 5. Verwendete Komponenten | image::RgbaImage · mlua::UserData · MethodArgs |
| 6. Eingaben und Ausgaben | Eingaben: Bilddimensionen, RGBA-Daten und Pixelkoordinaten Ausgabe: Image, Größe oder Pixelwerte |
| 7. Seiteneffekte und Ressourcen | mutiert eigene RGBA-Daten |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: image::RgbaImage · mlua::UserData · MethodArgs. Aufruferbezug: env/image.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Pixelbyteordnung und Koordinatenprüfung erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Bilddatenmodell. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Image Zeile6 · UserData Zeile46 · lua_get_pixel Zeile60 · lua_set_pixel_packed Zeile125

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/image/value.rs`.

### [src/loader/api/src/env/integer.rs](../../execution/lua/bindings/integer.rs)

Vollständig gelesen, 330 Zeilen. SHA-256: `d8cabf442c34938a2d2d4499680b449b01e0bb5e384d4a302568a8fe8cecd565`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erzeugt Lua-Bibliotheken für acht feste Ganzzahltypen mit Konvertierungen, Bitoperationen und verschiedenen Überlaufregeln durch Rust-Makros. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erzeugt Lua-Bibliotheken für acht feste Ganzzahltypen mit Konvertierungen, Bitoperationen und verschiedenen Überlaufregeln durch Rust-Makros. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile3`<br>`impl_integer_fns Zeile72`<br>`impl_integer_fn_wrapper Zeile261`<br>`acht Instanziierungen Zeilen322–330` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | FunctionArgs · LuaValue · Rust-Ganzzahloperationen |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Zahlen, Strings und Rechenoperanden Ausgabe: Ganzzahlen, bool, optionale Werte oder Überlaufpaar |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: FunctionArgs · LuaValue · Rust-Ganzzahloperationen. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Überlaufregeln und u64-Darstellung sind API-Verhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Ganzzahlarithmetik. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile3 · impl_integer_fns Zeile72 · impl_integer_fn_wrapper Zeile261 · acht Instanziierungen Zeilen322–330

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/integer.rs`.

### [src/loader/api/src/env/io.rs](../../execution/lua/bindings/io.rs)

Vollständig gelesen, 426 Zeilen. SHA-256: `dae5385d43ab2ae8abef94de1e98f59402be011908b0974c8639dc9e585a48f4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt paketlokalen Moddateizugriff, reine Pfadoperationen und rechtgebundene Exportdateioperationen mit Pfadbegrenzung bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt paketlokalen Moddateizugriff, reine Pfadoperationen und rechtgebundene Exportdateioperationen mit Pfadbegrenzung bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create Zeile15`<br>`lua_read Zeile50`<br>`lua_export Zeile191`<br>`checked_export_file Zeile284`<br>`sanitize_path Zeile397` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | Mod::fs · AppFeatures::EXPORT · AppState::export_dir · camino · Buffer |
| 6. Eingaben und Ausgaben | Eingaben: Modrelative Dateipfade, Exportrelative Pfade und Bytes Ausgabe: Buffer/Strings/Dateilisten oder Exportoperationserfolg |
| 7. Seiteneffekte und Ressourcen | Moddateilesen, Exportdateien anlegen/kopieren/umbenennen/löschen und Logging |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Paketdateilesen, reine Pfadoperationen und Exportdateiverwaltung bilden drei sichtbare Zuständigkeiten in einer Datei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: Mod::fs · AppFeatures::EXPORT · AppState::export_dir · camino · Buffer. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Pfadbegrenzung, Berechtigungen und Paketdateisystem müssen unverändert bleiben. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Moddateien und Export. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create Zeile15 · lua_read Zeile50 · lua_export Zeile191 · checked_export_file Zeile284 · sanitize_path Zeile397

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/io.rs`.

### [src/loader/api/src/env/loader.rs](../../execution/lua/runtime/mod.rs)

Vollständig gelesen, 4257 Zeilen. SHA-256: `b721f1ae7bbc6b568bde9a7f6b1e10eaf7bd18c88c70300c51f141346359dbd5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erzeugt den kanonischen runtime.*-Luanamensraum und bindet die tatsächlich vorhandenen Runtimeoperationen mit Modcapability, Ingamephase, aktivem Modlebenszyklus, API-Verfügbarkeit und Providerbereitschaft. Implementiert Lua-Operationen für ECS-Katalog/Registry/Query/Handle/Template/Komponenten-/Feldzugriff, Weltkontext/Cursor/Voxel/Props/Entitätsbau und profilgebundene Patches. Berechnet reflektierte Änderungsbereiche und ruft Attributneuberechnung und Ownershipprüfung vor CAS auf. Enthält zugleich den vollständigen Windows-DLL-Adapter mit ABI-12-Prüfung, erforderlichen/optionalen Exporten, C-Structs, besitzenden Puffern, Ressourcen-/Item-/DI-/ECS-/Welt-/Codeinventaraufrufen und NichtWindows-Ablehnungsstubs. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erzeugt den kanonischen runtime.*-Luanamensraum und bindet die tatsächlich vorhandenen Runtimeoperationen mit Modcapability, Ingamephase, aktivem Modlebenszyklus, API-Verfügbarkeit und Providerbereitschaft. Implementiert Lua-Operationen für ECS-Katalog/Registry/Query/Handle/Template/Komponenten-/Feldzugriff, Weltkontext/Cursor/Voxel/Props/Entitätsbau und profilgebundene Patches. Berechnet reflektierte Änderungsbereiche und ruft Attributneuberechnung und Ownershipprüfung vor CAS auf. Enthält zugleich den vollständigen Windows-DLL-Adapter mit ABI-12-Prüfung, erforderlichen/optionalen Exporten, C-Structs, besitzenden Puffern, Ressourcen-/Item-/DI-/ECS-/Welt-/Codeinventaraufrufen und NichtWindows-Ablehnungsstubs. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create/create_functions/create_ecs/create_world/create_patch`<br>`available/operation_status/runtime_denial_reason/runtime_type_name`<br>`ecs_fields/lua_ecs_write/collect_changed_ranges`<br>`runtime_provider::{Provider,PropRecord,PropRecipe,RuntimeResourceId,provider,configure,registry,registration,functions,read_many,compare_exchange}`<br>`runtime_provider::{resource_scalar_batch,resource_resolve_item,resource_read,service_read,service_compare_exchange,resource_compare_exchange_scalar_for_session}` |
| 4. Aufrufer und Verbraucher | env/mod.rs:register ruft loader::create auf und ergänzt types/values/overrides. · env/mod.rs:runtime_provider_report und api/lib.rs:760 verwenden Providerreport. · env/app_state.rs nutzt Session-/Ressourcenlese-/CAS-Funktionen. · env/game/assets.rs und assets/loaded_index.rs nutzen Ressourcen-/Item-/Scalarbatch-Adapter und Zugriffsprüfung. · env/runtime_attributes.rs nutzt Komponentenauflösung, read_many, functions und CAS. · env/runtime_services.rs nutzt service_read/service_compare_exchange. · env/runtime_templates.rs nutzt Registrierung, UUID und Ressourcen. · env/runtime_ownership.rs nutzt Policy, Session, Komponentenauflösung und Lesen. · env/runtime_building.rs/runtime_networking.rs nutzen runtime_denial_reason. · eml/v1/loader.rs importiert available für Kompatibilität. |
| 5. Verwendete Komponenten | kfc::reflection und kfc::guid/container::StaticHash · shroudforge-compatibility::Availability · shroudforge-steam-networking::status · mod_loader::{Mod,Capability} · mlua und FunctionArgs/LuaError/LuaValue · AppState/RuntimePhase/AppFeatures · MappedValue und game::value-Konvertierung · runtime_functions, runtime_services, runtime_templates, runtime_attributes, runtime_building, runtime_networking, runtime_values, runtime_overrides, runtime_ownership, runtime_metrics · runtime-operations.json per include_str · Windows kernel32 GetModuleHandleW/GetProcAddress · kfc-runtime.dll ABI-Version12 und KfcRuntime*-Exporte |
| 6. Eingaben und Ausgaben | Eingaben: Lua-State mit AppState, aktuelles Modobjekt, Runtimephase, Modcapabilities und Lebenszyklusstatus. · Lua-Typselektion, Entityhandles, Feldpfade/-werte, erwartete Snapshots, Transformationen, Voxel-/Recipearrays und Patchnamen. · Parserreflection/ResourceId und geladene Native-Provider-DLL-Exporte. · Native Ergebniszähler, JSON-Kataloge, Codebytes, Sessionkennungen und Outcome-/Presencecodes. Ausgabe: Lua-Namensraum runtime mit ECS-, Welt-, Service-, Override-, Netzwerk-, Patch- und Functions-Tabellen. · Verfügbarkeits-/Ablehnungsgründe, registrierte Typen und readonly-/writefähige Metadaten, Handles, gemappte Werte, bestätigte Feldwerte und Prop-/Cursor-/Voxel-Luatabellen. · Rust-Providerwrapper liefern besitzende Byte-/Handle-/ResourceId-Vektoren, Katalog-JSON, geprüfte Ergebnisse oder Fehlermeldungen. · Providerdiagnose einschließlich ABI, wartender Welt, Schreibbereitschaft und ResourceService-Status. |
| 7. Seiteneffekte und Ressourcen | Konfiguriert beim ersten Ingame-Namespaceaufbau den nativen reflektierten Komponententypvertrag und markiert AppState.runtime_configured. · Bindet bereits geladene kfc-runtime.dll über Win32-Symbolauflösung und merkt erfolgreiche Provider-/Funktionskataloge in OnceLock. · Führt freigegebene native Ressourcen-/DI-/ECS-/Welt-/Patchoperationen aus. Diese lesen oder mutieren Spielspeicher beziehungsweise Weltzustand gemäß nativen Gates. · Meldet Runtimeeffekte in AppState und schreibt Tracing-/Runtime-Metriken. · NichtWindows-Implementierung liefert Ablehnung/False/None und führt keine native Integration aus. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration. Befunde: Die Datei verbindet Namespace-/Policypflege, ECS-Reflectionänderungslogik, konkrete Welt-/Prop-/Cursorumwandlung und sämtliche native Transport-/ABI-Anbindungen. Das sind vorhandene unabhängige Änderungsgründe und belegte Aufteilungskandidaten. · Native-Adapter liegen als Untermodul der Luanamespace-Datei, werden aber von AppState, Assets, Ownership, Attributen, Diensten und Templates direkt genutzt. Dadurch hängt die gemeinsame Native-Integration strukturell an der Luanamespaceimplementierung. · Statischer Plattformparitätsfehler: available:331 und operation_status:446 rufen runtime_provider::world_entity_query_props_ready unbedingte auf. Die Definition existiert ausschließlich cfg(windows):3018-3024, im vollständig gelesenen cfg(not(windows))-Modul4053-4257 fehlt sie. env/mod.rs und lib.rs binden den Code ohne Windowsgate ein. Build nicht ausgeführt. · Public-ABI12-Erweiterungen sind optional geladen, während zahlreiche Welt-/Prop-Exporte Pflicht für Provideraufbau sind. Diese tatsächlich implementierte Kopplung muss bei einer Modularisierung sichtbar bleiben. Keine zusätzliche Produkterhaltungsanforderung daraus abgeleitet. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Runtime-Luabindung und native ABI. Technische Kopplung: kfc::reflection und kfc::guid/container::StaticHash · shroudforge-compatibility::Availability · shroudforge-steam-networking::status · mod_loader::{Mod,Capability} · mlua und FunctionArgs/LuaError/LuaValue · AppState/RuntimePhase/AppFeatures · MappedValue und game::value-Konvertierung · runtime_functions, runtime_services, runtime_templates, runtime_attributes, runtime_building, runtime_networking, runtime_values, runtime_overrides, runtime_ownership, runtime_metrics · runtime-operations.json per include_str · Windows kernel32 GetModuleHandleW/GetProcAddress · kfc-runtime.dll ABI-Version12 und KfcRuntime*-Exporte. Aufruferbezug: env/mod.rs:register ruft loader::create auf und ergänzt types/values/overrides. · env/mod.rs:runtime_provider_report und api/lib.rs:760 verwenden Providerreport. · env/app_state.rs nutzt Session-/Ressourcenlese-/CAS-Funktionen. · env/game/assets.rs und assets/loaded_index.rs nutzen Ressourcen-/Item-/Scalarbatch-Adapter und Zugriffsprüfung. · env/runtime_attributes.rs nutzt Komponentenauflösung, read_many, functions und CAS. · env/runtime_services.rs nutzt service_read/service_compare_exchange. · env/runtime_templates.rs nutzt Registrierung, UUID und Ressourcen. · env/runtime_ownership.rs nutzt Policy, Session, Komponentenauflösung und Lesen. · env/runtime_building.rs/runtime_networking.rs nutzen runtime_denial_reason. · eml/v1/loader.rs importiert available für Kompatibilität. |
| 11. Duplikation | Prop-/Transform-zu-Lua-Projektion wird innerhalb der Datei wiederholt: lua_world_entity_query_props:731-771, lua_world_entity_query_props_in_bounds:864-902 und lua_world_entity_get_transform:929-965. · Availabilitybedingungen werden in available:238-339 und operation_status:380-468 mit teils abweichender Phasen-/Lifecyclebedeutung erneut formuliert. Keine vollständige Austauschbarkeit behauptet. · Rust-C-Layouts und Funktionssignaturen:2486-2630 bilden dieselbe öffentliche ABI wie runtime/native/runtime.h ab. Die NichtWindows-Structs:4098-4119 wiederholen zusätzlich PropRecord/PropRecipe. Diese Duplikation muss bei ABI-Änderungen synchron bleiben. · registration:3460-3479 und fetch_json:3847-3865 wiederholen den vollständigen JSON-Puffer-/Retryvertrag, jedoch mit unterschiedlicher Exportform. |
| 12. Nutzungsstatus | Aktiv. env/mod.rs:register erzeugt pro Mod den runtime-Namensraum. Zahlreiche API-Nebenmodule verwenden direkt das pub(super) runtime_provider-Modul und zentrale Zugriffspolicy. API-Cargo.toml erklärt die dafür verwendeten lokalen Komponenten und mlua/KFC-Abhängigkeiten. |
| 13. Risiko bei Änderung oder Verschiebung | Sehr hoch bei fachlicher Aufteilung, da viele Nachbarmodule pub(super) runtime_provider, available, runtime_denial_reason und runtime_type_name direkt importieren. Include_str-Pfad zur Operationenliste, Modulsichtbarkeit, Windows-/NichtWindows-Parität und exakt identische C-Struct-/Signaturverträge müssen mitgezogen werden. Reine Pfadänderung kann isoliert erfolgen, Policy-/ABI-Refactoring danach separat. |
| 14. Vorhandener Testschutz | Direkt in loader.rs:2445-2471 write_policy_tests::whole_component_writes_share_finite_and_reference_rules schützt collect_changed_ranges für nichtendliche Skalare und unveränderte/veränderte Objektverweise. · Direkt in Windowsmodul:3675-3726 resource_buffer_tests::reads_small_snapshots_without_a_size_only_probe_and_retries_larger_owned_buffers sowie never_publishes_failed_or_incomplete_resource_buffers schützen besitzende Puffer, Retrygrenzen und fehlende Teilausgabe. · API-Agent belegt gemeinsame Codec-/Attributtests in runtime_values.rs und runtime_attributes.rs, die von den ECS-Schreibwrappern verwendete Funktionen schützen. Diese sind keine nachgewiesene dynamische FFI-Gesamtprüfung. · Die vollständig gelesenen nativen Tests schützen das Gegenstück der ABI, jedoch nicht den Rust-Loader-DLL-Verbindungsweg. Eine echte DLL-Integrationstestabdeckung dieses vollständigen Moduls ist nicht nachgewiesen. Keine Tests ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Luaschnittstelle und nativer Provideradapter. Neue Einordnung: Runtime-Luabindung und native ABI. Besitzer: Modausführung und Runtime-Brücke. |

**Belege:** 1-127 Namespaceaufbau und Typvertragkonfiguration · 129-231 Functions-Verbindung mit geprüften bestehenden Operationen · 238-514 Availability/Status/Effectreport · 516-661 Namespacegruppen und Patchaufrufe · 663-1391 Welt-/Prop-/Cursor-/Voxel-Luawrapper · 1393-1930 ECS-Query/Katalog/Registry/Template · 1932-2258 Komponenten-/Feldlesen und CAS-Schreiben · 2260-2359 reflektierte Änderungsbereiche · 2406-2443 Capability-/Lifecycle-/Writefault-Ablehnung · 2473-2982 Windows-ABI, Providerstruktur und Symbolauflösung · 3000-3501 Status-/Welt-/ECS-/Codewrapper · 3503-3865 Ressourcen/Item/Service/CAS/JSON-Puffer · 3866-4051 ECS-FFI · 4053-4257 NichtWindows-Stubs

**Evidenzgrenze:** Vollständige Quelle in begrenzten lückenlosen Chunks1-4257 gelesen. Cargo.toml und env/mod.rs ergänzend vollständig gelesen, eingehende Verwendungen gezielt per rg belegt. Beschriebene Implementierung nachgewiesen, Aufteilungsgrenzen abgeleitet. Keine dynamischen Spiel-/Build-/Testbehauptungen.

**Maßnahme:** aufteilen. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/mod.rs`, `execution/lua/runtime/policy.rs`, `execution/lua/runtime/ecs/registry.rs`, `execution/lua/runtime/ecs/entities.rs`, `execution/lua/runtime/ecs/components.rs`, `execution/lua/runtime/ecs/queries.rs`, `execution/lua/runtime/ecs/props.rs`, `execution/lua/runtime/voxels.rs`, `execution/lua/runtime/building.rs`, `execution/lua/runtime/player_input/cursor.rs`, `execution/lua/runtime/engine_context.rs`, `execution/lua/runtime/patches.rs`, `kfc-runtime/bridge/lib.rs`, `kfc-runtime/bridge/windows.rs`, `kfc-runtime/bridge/unsupported.rs`.

### [src/loader/api/src/env/log.rs](../../execution/lua/bindings/log.rs)

Vollständig gelesen, 56 Zeilen. SHA-256: `a663293b0bb813be5a0aaabdb8485df81cf386b9995625d285be74651cc33f80`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Ersetzt print und warn pro Mod durch tabulatorgetrennte tracing-Ausgaben mit zugehöriger Mod-ID. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Ersetzt print und warn pro Mod durch tabulatorgetrennte tracing-Ausgaben mit zugehöriger Mod-ID. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `register Zeile8`<br>`print Zeilen9–30`<br>`warn Zeilen32–53` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | LuaValue::to_string · crate::log debug/warn · mod_id |
| 6. Eingaben und Ausgaben | Eingaben: Mod-ID und beliebige Lua-Ausgabeargumente Ausgabe: print-/warn-Funktionen |
| 7. Seiteneffekte und Ressourcen | tracing-Logausgaben mit Mod-ID |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: LuaValue::to_string · crate::log debug/warn · mod_id. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Modzuordnung und Lua-Stringkonvertierung erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Moddiagnostik. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** register Zeile8 · print Zeilen9–30 · warn Zeilen32–53

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/log.rs`.

### [src/loader/api/src/env/mod.rs](../../execution/lua/bindings/mod.rs)

Vollständig gelesen, 102 Zeilen. SHA-256: `ccc36728d93d1261680b0019f8b32ac7f04a96913e8af150b91c15d6c2208c91`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Baut je Mod die globalen Lua-Bibliotheken auf, verknüpft Typen und Werte mit runtime, ergänzt Overrideziele und registriert EML-/ShroudForge-Namensräume, Tabellenoperationen und Logging. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Baut je Mod die globalen Lua-Bibliotheken auf, verknüpft Typen und Werte mit runtime, ergänzt Overrideziele und registriert EML-/ShroudForge-Namensräume, Tabellenoperationen und Logging. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `register Zeile42`<br>`runtime.values Zeilen52–62`<br>`Overrideziel-Fabrik Zeilen64–72`<br>`builtin/global Zeilen77–96` |
| 4. Aufrufer und Verbraucher | runner/mod.rs setup_mod |
| 5. Verwendete Komponenten | AppState · alle env-Unterbibliotheken · eml/v1 · shroudforge/v1 · override_targets.lua |
| 6. Eingaben und Ausgaben | Eingaben: Lua-VM, Mod und Umwelttabelle Ausgabe: globale und builtin-Bibliothekstabellen |
| 7. Seiteneffekte und Ressourcen | Registriert Funktionen, lädt eingebettete Override-Lua-Fabrik und ändert table/Logging |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte, die Datei ist der ausdrücklich sichtbare Zusammenbau der Lua-Umgebung |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: AppState · alle env-Unterbibliotheken · eml/v1 · shroudforge/v1 · override_targets.lua. Aufruferbezug: runner/mod.rs setup_mod |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Registrierungsreihenfolge und Namensräume sind Mod-API. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Umgebungszusammenbau. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** register Zeile42 · runtime.values Zeilen52–62 · Overrideziel-Fabrik Zeilen64–72 · builtin/global Zeilen77–96

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/mod.rs`.

### [src/loader/api/src/env/registry.rs](../../execution/lua/bindings/registry.rs)

Vollständig gelesen, 177 Zeilen. SHA-256: `560dab9799593a5664a3de531773ec4ceabafa3da3bb538e5d176f4e250fedcf`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft Lua-Typselektoren auf aktuelle Registeridentität, exportiert Vererbung/Felder und wandelt Provider-JSON in getrennte Lua-Werte um. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft Lua-Typselektoren auf aktuelle Registeridentität, exportiert Vererbung/Felder und wandelt Provider-JSON in getrennte Lua-Werte um. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `resolve Zeile9`<br>`fields/inheritance-Reexport Zeile49`<br>`json_to_lua Zeile52`<br>`fixture und Tests Zeilen88–177` |
| 4. Aufrufer und Verbraucher | env/game/types.rs · env/runtime_values.rs · env/loader.rs · env/runtime_*.rs |
| 5. Verwendete Komponenten | TypeRegistry · type_lookup · Type · runtime_resolution::types · serde_json |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Typselektor und aktuelles Register oder Provider-JSON Ausgabe: Typindex oder abgelöste Lua-Tabelle |
| 7. Seiteneffekte und Ressourcen | Lua-Allokation, kein Providerzustandszugriff über Rückgabetabelle |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Typidentitätsprüfung und allgemeine Provider-JSON-Konvertierung teilen dieselbe Datei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: TypeRegistry · type_lookup · Type · runtime_resolution::types · serde_json. Aufruferbezug: env/game/types.rs · env/runtime_values.rs · env/loader.rs · env/runtime_*.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Registeridentität und Kollisionsablehnung sind zentrale Vorbedingungen. |
| 14. Vorhandener Testschutz | inheritance_is_not_element_containment_and_fields_keep_owners · selectors_reject_collisions_wrong_domains_and_invalid_numbers · cycles_are_reported_instead_of_looping |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Typregisterintegration. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** resolve Zeile9 · fields/inheritance-Reexport Zeile49 · json_to_lua Zeile52 · fixture und Tests Zeilen88–177

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/registry.rs`.

### [src/loader/api/src/env/runtime_attributes.rs](../../execution/lua/runtime/ecs/attributes.rs)

Vollständig gelesen, 1177 Zeilen. SHA-256: `f79cec5eb3b3e96c4cafae74bd4af3c4dc3ef33e0d7d7334a2501c7bda3791a5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verbindet aufgelöste KFC-Attribute mit genau einem initialisierten Entitykomponentenspeicher, bietet Lesen/Auswerten/Ändern und führt vor jedem AttributeComponent-Schreiben die gesamte originale Wurzelberechnung aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verbindet aufgelöste KFC-Attribute mit genau einem initialisierten Entitykomponentenspeicher, bietet Lesen/Auswerten/Ändern und führt vor jedem AttributeComponent-Schreiben die gesamte originale Wurzelberechnung aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `storage_range Zeile29`<br>`resolve_storage Zeile78`<br>`entity_storage Zeile111`<br>`recalculate_bound_component_update Zeile157`<br>`recalculate_component_update Zeile233`<br>`attach Zeile284`<br>`encode_attribute_scalar Zeile540`<br>`Tests Zeilen620–1177` |
| 4. Aufrufer und Verbraucher | env/loader.rs attach und recalculate_component_update · override_targets.lua Attributeadapter |
| 5. Verwendete Komponenten | runtime_resolution::attributes::Catalog · runtime_provider read_many/functions/compare_exchange · runtime_ownership · AppState::attribute_catalog · registry::json_to_lua · runtime_metrics |
| 6. Eingaben und Ausgaben | Eingaben: Attributname/-ID, Entity und optional neuer Wert/Erwartungswurzel · KFC-Katalog, reflektierter Header und komplette Komponentensnapshots Ausgabe: Attributskalar oder vollständige benannte Wurzel samt Berechnungstrace · korrigierter Kandidat und vollständige Blattmaske |
| 7. Seiteneffekte und Ressourcen | gebündelte Providerreads · nativer vollständiger Snapshot-CAS nach Besitzprüfung · ignorierte installierte Prüfungen können QA-JSON schreiben |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Lua-Bindungen und sicherheitsrelevante owned-data-Neuberechnung liegen gemeinsam, obwohl die reine Attributauflösung bereits separat ist · Mehr als die Hälfte der Datei sind synthetische und explizit ignorierte installierte Prüfungen |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: runtime_resolution::attributes::Catalog · runtime_provider read_many/functions/compare_exchange · runtime_ownership · AppState::attribute_catalog · registry::json_to_lua · runtime_metrics. Aufruferbezug: env/loader.rs attach und recalculate_component_update · override_targets.lua Attributeadapter |
| 11. Duplikation | update_attribute und write_attribute_storage teilen denselben recalculation core, getrennte Bindungen bleiben aus Kompatibilitätsgründen sichtbar |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Wurzelidentität, Initialisierungsflags, Signiertheitskonflikte, komplette Neubewertung und unveränderliche Header müssen gemeinsam erhalten bleiben. |
| 14. Vorhandener Testschutz | raw_attribute_write_recalculates_derived_values_and_refreshes_complete_scalar_masks · raw_attribute_write_rejects_headers_uninitialized_storage_and_unmasked_changes · raw_attribute_write_requires_verified_model_programs_and_scalar_domain · named_attribute_updates_validate_unselected_programs_and_derived_result_domains · raw_attribute_policy_follows_actual_reflection_inheritance_and_exact_storage · attribute_snapshot_accepts_compact_header_without_reading_storage_as_header · attribute_entity_join_requires_one_initialized_root_match · expected_attribute_values_reject_stale_bounds_and_other_roots · scalar_range_checks · program_literals_are_not_misreported_as_opcodes · zwei explizit ignorierte installierte Attributprüfungen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Geprüfte Entity-Attributoperationen. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** storage_range Zeile29 · resolve_storage Zeile78 · entity_storage Zeile111 · recalculate_bound_component_update Zeile157 · recalculate_component_update Zeile233 · attach Zeile284 · encode_attribute_scalar Zeile540 · Tests Zeilen620–1177

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/ecs/attributes.rs`.

### [src/loader/api/src/env/runtime_building.rs](../../execution/lua/runtime/player_input/building.rs)

Vollständig gelesen, 283 Zeilen. SHA-256: `600d48b099db8ec07f7fb548f7e0714e8163d2aadad3d143c8d81b79b4e4cb01`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet lokale Client-Baueingaben mit exklusivem Sequenzbesitzer an dynamisch gelöste KFC-Runtime-Exporte und stellt Submit/Status/Cancel bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet lokale Client-Baueingaben mit exklusivem Sequenzbesitzer an dynamisch gelöste KFC-Runtime-Exporte und stellt Submit/Status/Cancel bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `RequestOwner Zeile14`<br>`symbol Zeile59`<br>`submit Zeile76`<br>`attach Zeile150`<br>`Ownershiptest Zeile268` |
| 4. Aufrufer und Verbraucher | env/loader.rs Runtimeweltaufbau attach |
| 5. Verwendete Komponenten | loader::runtime_denial_reason · KfcRuntimeWorldBuildingInput/Status/Cancel · Windows DLL-Symbolauflösung · Arc/AtomicU32 |
| 6. Eingaben und Ausgaben | Eingaben: Spielerhandle, Bauaktion, optional Item/Material/Slot/Zielentity und Transform Ausgabe: Request-ID oder Fehler · Statuswort und Cancelresultat |
| 7. Seiteneffekte und Ressourcen | DLL-Symbolauflösung · native Baueingabequeue · globale exklusive Sequenzreservierung und Dropcancel |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration. Befunde: Lua-Eingabevertrag und eigener dynamischer DLL-ABIadapter liegen zusammen statt ausschließlich über loader::runtime_provider zu gehen |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Clientbaueingabe. Technische Kopplung: loader::runtime_denial_reason · KfcRuntimeWorldBuildingInput/Status/Cancel · Windows DLL-Symbolauflösung · Arc/AtomicU32. Aufruferbezug: env/loader.rs Runtimeweltaufbau attach |
| 11. Duplikation | GetModuleHandleW/GetProcAddress-Adaptermuster ist auch im zentralen Runtimeprovider vorhanden |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. ABI-Signaturen, Einzelbesitzer, Dropcancel und Dispatch-/Wirkungsunterscheidung erhalten. |
| 14. Vorhandener Testschutz | building_sequence_has_one_owner_until_cancel_or_drop |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Clientbaueingabe. Neue Einordnung: Clientbaueingabe. Besitzer: Modausführung und Runtime-Brücke. |

**Belege:** RequestOwner Zeile14 · symbol Zeile59 · submit Zeile76 · attach Zeile150 · Ownershiptest Zeile268

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** aufteilen. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/player_input/building.rs`, `kfc-runtime/bridge/player_input.rs`.

### [src/loader/api/src/env/runtime_functions.rs](../../execution/lua/runtime/code_inventory.rs)

Vollständig gelesen, 1086 Zeilen. SHA-256: `89591d0732b2a222e1add3ad647b6d0a67304bccafeabddc253c344c0189ecab`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verknüpft nachgewiesene native Funktionskandidaten, originale Enginebeschreibungen, Live-Registrierungsbesitzer und reflektierte Abhängigkeiten zu inspizierbaren Lua-Bindings, führt nur bewiesene eigene Pufferadapter oder vorhandene geprüfte Patchwrapper aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verknüpft nachgewiesene native Funktionskandidaten, originale Enginebeschreibungen, Live-Registrierungsbesitzer und reflektierte Abhängigkeiten zu inspizierbaren Lua-Bindings, führt nur bewiesene eigene Pufferadapter oder vorhandene geprüfte Patchwrapper aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Catalog Zeile19`<br>`refresh Zeile32`<br>`selector Zeile130`<br>`describe Zeile189`<br>`descriptor_to_lua Zeile283`<br>`binding Zeile350`<br>`attach Zeile391`<br>`bind_modifier Zeile500`<br>`Tests Zeilen608–1086` |
| 4. Aufrufer und Verbraucher | env/loader.rs runtime.functions attach |
| 5. Verwendete Komponenten | runtime_provider functions/registry/function_code · runtime_resolution::functions::Resolver · buffer_transform::Plan · AppState::attribute_catalog · registry::json_to_lua · Patchavailable/set_enabled |
| 6. Eingaben und Ausgaben | Eingaben: Providerfunktionsinventar, Live-Registrierungen und exakter EXE-Hash · Lua-Selektor, Suchfilter, Pagination oder Modifier-ID · eigene Quell-/Zielbytes Ausgabe: Lua-Funktionsdeskriptoren mit explizitem Evidenzstatus und Typobjekten · geprüfte Pufferaufrufbindung oder Patchadapter · Teilbindings mit Ablehnungsgrund |
| 7. Seiteneffekte und Ressourcen | Providerreads und begrenztes Registerrefresh · eigene Bytepuffertransformation · vorhandener nativer Patchbackend nur bei geprüfter Modifiernutzung · ignorierte Corpusprüfung schreibt QA-JSON |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration. Befunde: Inspektionskatalog, geprüfte Pufferaufrufe und historischer profilgebundener Modifieradapter teilen dieselbe Datei · Mehrere Regressionstests prüfen konkrete Referenzmods wie Stamina, FallDamage und Blueprintfreischaltung, obwohl ihr Produktcode nicht zum Funktionskatalog gehört |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Funktionsbelege und geprüfte Adapter. Technische Kopplung: runtime_provider functions/registry/function_code · runtime_resolution::functions::Resolver · buffer_transform::Plan · AppState::attribute_catalog · registry::json_to_lua · Patchavailable/set_enabled. Aufruferbezug: env/loader.rs runtime.functions attach |
| 11. Duplikation | keine belegte Produktionsduplikation, vorhandene Bindungen werden ausdrücklich wiederverwendet |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Provisorische Namen bleiben EXE-gebunden, reine Beschreibung darf keine rohe native Callfähigkeit erhalten. Tests referenzieren Mod- und Profilpfade. |
| 14. Vorhandener Testschutz | original_engine_names_are_resolved_without_inventing_helper_names · includes_leaf_callbacks_merges_owners_and_returns_partial_bindings · bundled_mods_bind_profile_modifiers_and_restore_on_unload · stamina_mod_uses_typed_roots_and_retries_without_a_modifier_patch · no_fall_mod_uses_actual_template_scalars_and_scoped_restoration · blueprint_mod_resolves_original_query_action_instead_of_fixed_id · production_callback_binding_corpus ist explizit ignoriert |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Native Funktionsinspektion und geprüfte Bindungen. Neue Einordnung: Funktionsbelege und geprüfte Adapter. Besitzer: Modausführung. |

**Belege:** Catalog Zeile19 · refresh Zeile32 · selector Zeile130 · describe Zeile189 · descriptor_to_lua Zeile283 · binding Zeile350 · attach Zeile391 · bind_modifier Zeile500 · Tests Zeilen608–1086

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** aufteilen. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/code_inventory.rs`, `execution/lua/runtime/patches/modifiers.rs`, `execution/tests/mod_contracts.rs`.

### [src/loader/api/src/env/runtime_metrics.rs](../../execution/lua/bindings/runtime_metrics.rs)

Vollständig gelesen, 171 Zeilen. SHA-256: `7b083be8cbb4e0303afe5ae61389a088739420c34968dc244af5f5a6560d3e99`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Misst optional begrenzt pro Loaderworker inklusive Operationszeiten mit saturierenden Summen und logarithmischen Histogrammobergrenzen, trennt Diagnosesitzungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Misst optional begrenzt pro Loaderworker inklusive Operationszeiten mit saturierenden Summen und logarithmischen Histogrammobergrenzen, trennt Diagnosesitzungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Measurement::record Zeile36`<br>`set_enabled Zeile83`<br>`Span Zeile92`<br>`snapshot Zeile117`<br>`Tests Zeilen142–171` |
| 4. Aufrufer und Verbraucher | env/runtime_*.rs span · lib.rs Diagnosestatus und set_enabled |
| 5. Verwendete Komponenten | thread_local Cell/RefCell · Instant · serde_json |
| 6. Eingaben und Ausgaben | Eingaben: Aktivierungsstatus und statischer Operationsname · Spanlebensdauer Ausgabe: JSON mit Zählung/Summen/Maxima und Quantilobergrenzen |
| 7. Seiteneffekte und Ressourcen | begrenzter threadlokaler Messspeicher und Uhrlesen nur aktiviert |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: thread_local Cell/RefCell · Instant · serde_json. Aufruferbezug: env/runtime_*.rs span · lib.rs Diagnosestatus und set_enabled |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Workerbindung, Generationsgrenze und abgeschalteter Nullpfad erhalten. |
| 14. Vorhandener Testschutz | histogram_bounds_and_saturated_totals_are_explicit · disabled_spans_and_old_session_spans_do_not_record |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Opt-in-Laufzeitmessung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Measurement::record Zeile36 · set_enabled Zeile83 · Span Zeile92 · snapshot Zeile117 · Tests Zeilen142–171

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/runtime_metrics.rs`.

### [src/loader/api/src/env/runtime_networking.rs](../../modules/transport/lua/network.rs)

Vollständig gelesen, 358 Zeilen. SHA-256: `168446c47736e3a98a2c6da6732b5c41f644259aa3135f902fcd12ca7ca0cb8e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt Steam-P2P-Nachrichten für aktive Mods bereit, bindet Status und Configfallbacks, überprüft Peer-/Kanalparameter und routet JSON-Hüllen modadressiert über deterministische Kanäle. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt Steam-P2P-Nachrichten für aktive Mods bereit, bindet Status und Configfallbacks, überprüft Peer-/Kanalparameter und routet JSON-Hüllen modadressiert über deterministische Kanäle. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `authorized_client_ids Zeile9`<br>`denied Zeile43`<br>`mod_channel Zeile61`<br>`create Zeile68`<br>`send_mod Zeile165`<br>`receive_mod Zeile298` |
| 4. Aufrufer und Verbraucher | env/loader.rs Runtimeaufbau create |
| 5. Verwendete Komponenten | shroudforge_steam_networking · loader::runtime_denial_reason · Mod · mod_loader::config::read_loader · serde_json |
| 6. Eingaben und Ausgaben | Eingaben: Mod, SteamID64, Payload, Optionen oder Empfangslimit · bestehende Network-Konfiguration Ausgabe: Netzwerkstatus, Send-/Acceptresultat, Peerlisten und Nachrichten |
| 7. Seiteneffekte und Ressourcen | Steam-Senden/Annehmen/Empfangen und Konfigurationsdateilesen |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Netzwerkkonfigurationsauslese, P2P-Rohzugriff und eigenes Modnachrichtenprotokoll sind in derselben Adapterdatei gebündelt |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebautes Modul. Technische Kopplung: shroudforge_steam_networking · loader::runtime_denial_reason · Mod · mod_loader::config::read_loader · serde_json. Aufruferbezug: env/loader.rs Runtimeaufbau create |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Protokollname, deterministische Zielkanäle, Envelopeprüfung und serverseitige Peerautorisierung erhalten. |
| 14. Vorhandener Testschutz | Worldeditorfixtures schützen Modprotokollnutzung mit simuliertem Netzwerk, keine direkte Rustadapterprüfung nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Modnachrichten über Steam. Neue Einordnung: Eingebautes Modul. Besitzer: Transportmodul. |

**Belege:** authorized_client_ids Zeile9 · denied Zeile43 · mod_channel Zeile61 · create Zeile68 · send_mod Zeile165 · receive_mod Zeile298

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Modkanäle, Envelope und Autorisierung beim Transportmodul. Lua-Host liefert begrenzten Kontext und registriert die Modulbindung, ohne Rückabhängigkeit des Moduls auf den gesamten Ausführungshost.

**Vorgeschlagene Zielpfade:** `modules/transport/lua/network.rs`.

### [src/loader/api/src/env/runtime_overrides.rs](../../execution/runtime/overrides.rs)

Vollständig gelesen, 264 Zeilen. SHA-256: `afbc10f1684a4efd6657a395f43b375eff6cbac033bcb08df397fd5e0fe0c66e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erzeugt je Mod einmal die eingebettete Lua-Overridefabrik, registriert alle Tracker und erzwingt automatische Wiederherstellung vor endgültigem Modunload, mit Busy- und Faultprüfung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erzeugt je Mod einmal die eingebettete Lua-Overridefabrik, registriert alle Tracker und erzwingt automatische Wiederherstellung vor endgültigem Modunload, mit Busy- und Faultprüfung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `create_scoped Zeile12`<br>`Trackerregistrierung Zeilen23–37`<br>`cleanup Zeile40`<br>`vier Tests Zeilen80–264` |
| 4. Aufrufer und Verbraucher | env/loader.rs create · lib.rs Lifecycle::try_unload |
| 5. Verwendete Komponenten | overrides.lua · Lua-Namensregistry · Mod-ID · runtime_provider::world_session_id · crate::Lifecycle |
| 6. Eingaben und Ausgaben | Eingaben: Lua-VM und Mod-ID · registrierte Tracker Ausgabe: modgebundene Fabriktabelle oder Cleanupstatus/-fehler |
| 7. Seiteneffekte und Ressourcen | Lua-Registry und automatische Adapterrestaurierung |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: overrides.lua · Lua-Namensregistry · Mod-ID · runtime_provider::world_session_id · crate::Lifecycle. Aufruferbezug: env/loader.rs create · lib.rs Lifecycle::try_unload |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Busycleanup muss den aktiven Schreibscope behalten, auch ohne on_unload oder bei Modunloadfehler. |
| 14. Vorhandener Testschutz | attribute_targets_reuse_names_without_reusing_live_root_snapshots · lifecycle_restores_api_values_without_a_mod_unload_callback · failed_mod_unload_still_runs_registered_api_restoration · automatic_cleanup_is_mod_scoped_and_retries_busy_writers |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Override-Modlebenszyklusintegration. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** create_scoped Zeile12 · Trackerregistrierung Zeilen23–37 · cleanup Zeile40 · vier Tests Zeilen80–264

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/runtime/overrides.rs`.

### [src/loader/api/src/env/runtime_ownership.rs](../../execution/runtime/ownership.rs)

Vollständig gelesen, 430 Zeilen. SHA-256: `f54c8e64ac14c7202b9c336284336fffb5471978aafcb7bd7111bfbd93cf4e3e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Koordiniert exklusive modgebundene Claims für originale ECS-, Ressourcen- und Dienstspeicheridentitäten, prüft alle typisierten Writer gegen fremden Besitz und verwaltet Sitzungsgültigkeit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Koordiniert exklusive modgebundene Claims für originale ECS-, Ressourcen- und Dienstspeicheridentitäten, prüft alle typisierten Writer gegen fremden Besitz und verwaltet Sitzungsgültigkeit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Target Zeile16`<br>`OwnershipStore Zeile60`<br>`check_write Zeile125`<br>`RuntimeWriteClaim Zeile141`<br>`attach Zeile243`<br>`Tests Zeilen367–430` |
| 4. Aufrufer und Verbraucher | env/mod.rs attach · env/loader.rs öffentliche ECS-Writer · env/game/assets.rs Writer · runtime_services.rs · runtime_templates.rs · override_targets.lua |
| 5. Verwendete Komponenten | AppState::runtime_ownership · loader::runtime_provider · RuntimeService · RuntimeTemplateComponent · Resource · Mod |
| 6. Eingaben und Ausgaben | Eingaben: Mod-ID, Speicherziel, Token und aktuelle Weltsitzung Ausgabe: Claim-Userdata, Erwerb-/Freigabe-/Schreibberechtigung |
| 7. Seiteneffekte und Ressourcen | gemeinsamer Besitzspeicher · Providerprüfung der Entity-/Schreibautorität · aktive verlorene Tokens behalten bewusst ihren Claim |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: AppState::runtime_ownership · loader::runtime_provider · RuntimeService · RuntimeTemplateComponent · Resource · Mod. Aufruferbezug: env/mod.rs attach · env/loader.rs öffentliche ECS-Writer · env/game/assets.rs Writer · runtime_services.rs · runtime_templates.rs · override_targets.lua |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Ressourcenzielidentität ist absichtlich sitzungsübergreifend, Tokenautorität dagegen weltgebunden. |
| 14. Vorhandener Testschutz | independent_mods_cannot_capture_another_mods_baseline · one_mods_storage_claim_survives_until_every_own_token_is_released · world_changes_clear_entity_and_service_claims_but_keep_shared_resources |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Speicherbesitz. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** Target Zeile16 · OwnershipStore Zeile60 · check_write Zeile125 · RuntimeWriteClaim Zeile141 · attach Zeile243 · Tests Zeilen367–430

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/runtime/ownership.rs`.

### [src/loader/api/src/env/runtime_services.rs](../../execution/lua/runtime/named_services.rs)

Vollständig gelesen, 264 Zeilen. SHA-256: `4c54197bd179eb278c96da4cbf072e10dc164c2a41d4cd9e3b1c6c8411b8da79`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet Original-DI-Dienste als welt- und modgebundene reflektierte Plain-Data-Ansichten, liest Felder frisch und schreibt mit vollständigem Snapshotvergleich, Besitzprüfung und Faultsperre. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet Original-DI-Dienste als welt- und modgebundene reflektierte Plain-Data-Ansichten, liest Felder frisch und schreibt mit vollständigem Snapshotvergleich, Besitzprüfung und Faultsperre. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `RuntimeService Zeile11`<br>`get Zeile44`<br>`snapshot Zeile111`<br>`read_fields Zeile134`<br>`write_field Zeile174`<br>`UserData Zeile235` |
| 4. Aufrufer und Verbraucher | env/loader.rs create · runtime_ownership.rs claim_service · override_targets.lua serviceadapter |
| 5. Verwendete Komponenten | AppState · loader::runtime_provider service_read/service_compare_exchange · registry · runtime_values · runtime_ownership |
| 6. Eingaben und Ausgaben | Eingaben: exakter Original-DI-Name · Feldpfade, neuer und erwarteter Wert, optionale abhängige Feldwerte Ausgabe: RuntimeService-Ansicht · abgelöste Feldwerte · Write-/Fehlerstatus |
| 7. Seiteneffekte und Ressourcen | Providerlesezugriff und vollständiger nativer CAS · lokale/Sessionfaultsperre nach ungewissem Ergebnis |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: AppState · loader::runtime_provider service_read/service_compare_exchange · registry · runtime_values · runtime_ownership. Aufruferbezug: env/loader.rs create · runtime_ownership.rs claim_service · override_targets.lua serviceadapter |
| 11. Duplikation | Dichte eindeutige Feldpfadlisten werden fast gleich in runtime_templates.rs read_fields geprüft · require_access/require_session entsprechen Templateadapter mit anderem Fehlertext |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Komplette Plain-Data-Prüfung, aktuelle Named-Bindung und ursprüngliche Scene-Gates bleiben Vorbedingungen. |
| 14. Vorhandener Testschutz | Keine direkte Rustadapterprüfung nachgewiesen · survival.lua simuliert den öffentlichen Servicevertrag |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Geprüfte benannte Engine-Dienste. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** RuntimeService Zeile11 · get Zeile44 · snapshot Zeile111 · read_fields Zeile134 · write_field Zeile174 · UserData Zeile235

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/named_services.rs`.

### [src/loader/api/src/env/runtime_templates.rs](../../execution/lua/runtime/ecs/templates.rs)

Vollständig gelesen, 663 Zeilen. SHA-256: `4042ac1f0e3b85cab209b7c3007e9991f565d2e0227524ccff0cf2f32ac4f169`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst das Originaltemplate einer aktuellen Entity auf, bindet genau eine live registrierte konkrete Konfigurationsvariante und bietet sitzungsgebundene Skalarreads/-writes auf der gehaltenen Ressourcenidentität. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst das Originaltemplate einer aktuellen Entity auf, bindet genau eine live registrierte konkrete Konfigurationsvariante und bietet sitzungsgebundene Skalarreads/-writes auf der gehaltenen Ressourcenidentität. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `RuntimeTemplateComponent Zeile20`<br>`template_registration Zeile52`<br>`select_variant Zeile123`<br>`get_component Zeile215`<br>`snapshot Zeile329`<br>`write_field Zeile391`<br>`Tests Zeilen499–663` |
| 4. Aufrufer und Verbraucher | env/loader.rs attach · runtime_ownership.rs claim_template · override_targets.lua templateadapter |
| 5. Verwendete Komponenten | runtime_provider registration/entity_template_uuid/resource_read · MappedValue · runtime_values::resource_field · runtime_ownership · AppState CAS · runtime_metrics |
| 6. Eingaben und Ausgaben | Eingaben: Entityhandle und Komponentenselektor · Originalregistrierungsbericht und Resourcesnapshot · Feldpfade/Skalare/Erwartungen Ausgabe: RuntimeTemplateComponent · typisierte Skalare oder eindeutiger Fehlergrund |
| 7. Seiteneffekte und Ressourcen | Providerlesungen · 32-Bit-Ressourcen-CAS unter ursprünglichen Gates · Schreibsperre nach ungewissem Ergebnis |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: runtime_provider registration/entity_template_uuid/resource_read · MappedValue · runtime_values::resource_field · runtime_ownership · AppState CAS · runtime_metrics. Aufruferbezug: env/loader.rs attach · runtime_ownership.rs claim_template · override_targets.lua templateadapter |
| 11. Duplikation | read_fields dichte eindeutige Pfadprüfung parallel runtime_services.rs · Erwartungsfeldvergleich ähnlich runtime_values::expect_inline_fields mit ressourcenrelativer Pfadabbildung |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Besitzer ist absichtlich ResourceId statt Entityhandle, damit Restore nach Entityverlust innerhalb derselben Sitzung funktioniert. |
| 14. Vorhandener Testschutz | concrete_variant_selection_uses_the_mapper_and_rejects_missing_or_duplicate_types · a_template_role_requires_one_exact_live_registration · survival.lua simuliert den öffentlichen Templatevertrag |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Geteilte Entity-Templatekonfiguration. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** RuntimeTemplateComponent Zeile20 · template_registration Zeile52 · select_variant Zeile123 · get_component Zeile215 · snapshot Zeile329 · write_field Zeile391 · Tests Zeilen499–663

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/ecs/templates.rs`.

### [src/loader/api/src/env/runtime_values.rs](../../execution/lua/runtime/values.rs)

Vollständig gelesen, 1411 Zeilen. SHA-256: `dd82dc86252125848b05c875fb26d84a4c2effa764d8bab6eb89f4618f84fe83`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Kodiert und dekodiert eigene reflektierte Lua-Werte, normalisiert Felder und berechnet geprüfte Inline-/Blobressourcenfeldpläne mit Blattmasken unter Erhalt von Padding und externen Deskriptoren. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Kodiert und dekodiert eigene reflektierte Lua-Werte, normalisiert Felder und berechnet geprüfte Inline-/Blobressourcenfeldpläne mit Blattmasken unter Erhalt von Padding und externen Deskriptoren. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `InlineField Zeile15`<br>`loaded_field Zeile35`<br>`InlineField::write_into Zeile86`<br>`expect_inline_fields Zeile131`<br>`inline_field Zeile414`<br>`resource_field Zeile523`<br>`create Zeile782`<br>`Tests Zeilen921–1411` |
| 4. Aufrufer und Verbraucher | env/mod.rs runtime.values · env/loader.rs ECS-Feldreads/-writes · runtime_services.rs · runtime_templates.rs · assets.rs · loaded_index.rs |
| 5. Verwendete Komponenten | TypeRegistry/TypeHandle · MappedValue · registry::fields/inheritance · value Validator/Converter · AppState Inlineplancache |
| 6. Eingaben und Ausgaben | Eingaben: Typselektor/Typregister · eigene Bytes oder Lua-Werte · reflektierter Feldpfad und erwartete Werte Ausgabe: typisierte Lua-Werte, kodierte Bytes, InlineField-Pläne und Schreibmasken |
| 7. Seiteneffekte und Ressourcen | eigene Byte-/Lua-Allokation · optionaler Zugriff auf vorhandenen Plancache · ignorierte explizite Exporttests schreiben QA-Berichte |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Allgemeine Lua-Wertbibliothek für eigene Daten, Inlinefeldplanauflösung und Blob-Ressourcenoffsetbestimmung teilen eine 1411-Zeilen-Datei mit ausführlichen Fixtures und Exportprüfungen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: TypeRegistry/TypeHandle · MappedValue · registry::fields/inheritance · value Validator/Converter · AppState Inlineplancache. Aufruferbezug: env/mod.rs runtime.values · env/loader.rs ECS-Feldreads/-writes · runtime_services.rs · runtime_templates.rs · assets.rs · loaded_index.rs |
| 11. Duplikation | resource_field muss BlobArray/BlobVariant-Offsets zusätzlich zum KFC-Mapper berechnen, weil dieser keine Quellposition liefert. Formatkopplung ist im Code ausdrücklich dokumentiert. |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Pinned-KFC-Blobformat, Masken, Padding, Deskriptorüberschneidungen und Endlichkeit müssen unverändert geprüft werden. |
| 14. Vorhandener Testschutz | field_normalization_uses_the_live_writer_codec_and_float_domain · resource_blob_paths_match_the_pinned_mapper_and_change_only_the_selected_word · resource_paths_reject_bad_indices_descriptors_truncation_and_nonfinite_values · resource_scalar_arrays_are_bounded_and_ds_strings_and_bool_words_are_rejected · lua_values_round_trip_inherited_fields_defaults_and_validate_errors · named_byte_enum_expected_fields_use_the_production_codec_and_preserve_siblings · zwei explizit ignorierte installierte Exportprüfungen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Reflektierte Wertcodecs und Feldselektion. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** InlineField Zeile15 · loaded_field Zeile35 · InlineField::write_into Zeile86 · expect_inline_fields Zeile131 · inline_field Zeile414 · resource_field Zeile523 · create Zeile782 · Tests Zeilen921–1411

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Lua-Anbindung nach derselben tatsächlichen Fähigkeit wie das native Backend gliedern. Öffentliche runtime.world.*-Namen als bestehende Registrierung erhalten, reine Strukturänderung ohne Modmigration.

**Vorgeschlagene Zielpfade:** `execution/lua/runtime/values.rs`.

### [src/loader/api/src/env/table.rs](../../execution/lua/bindings/table.rs)

Vollständig gelesen, 117 Zeilen. SHA-256: `d4be6f79d97cf6cf9ad9fa988f74b501b4c7a4f0e1ea2f5dac7bb2b74fcb1aa6`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erweitert Lua-table.insert/remove/clear auf die eigenen reflektierten Array-Userdata und delegiert normale Tabellen an die ursprünglichen Lua-Funktionen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erweitert Lua-table.insert/remove/clear auf die eigenen reflektierten Array-Userdata und delegiert normale Tabellen an die ursprünglichen Lua-Funktionen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `modify Zeile11`<br>`insert Zeilen19–53`<br>`remove Zeilen55–89`<br>`clear Zeilen91–114` |
| 4. Aufrufer und Verbraucher | env/mod.rs register |
| 5. Verwendete Komponenten | env::value push/insert/pop/remove/clear · mlua::Table · FunctionArgs |
| 6. Eingaben und Ausgaben | Eingaben: Lua-table-Bibliothek und eigene Array-Userdata Ausgabe: erweiterte insert/remove/clear-Funktionen |
| 7. Seiteneffekte und Ressourcen | mutiert Lua-Arrays oder Tabellen |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: env::value push/insert/pop/remove/clear · mlua::Table · FunctionArgs. Aufruferbezug: env/mod.rs register |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Fallback auf ursprüngliche Lua-Funktionen und einbasige Indizes erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Containerintegration. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** modify Zeile11 · insert Zeilen19–53 · remove Zeilen55–89 · clear Zeilen91–114

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/table.rs`.

### [src/loader/api/src/env/type_lookup.rs](../../execution/lua/bindings/type_lookup.rs)

Vollständig gelesen, 103 Zeilen. SHA-256: `d692519837560b95b39678eabd7ab6c2d95b0b29fbb701a7dd345a795efed29d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst Typen in vier Hashdomänen nur bei eindeutiger Übereinstimmung auf und prüft Kollisionsverhalten mit synthetischem Typregister. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst Typen in vier Hashdomänen nur bei eindeutiger Übereinstimmung auf und prüft Kollisionsverhalten mit synthetischem Typregister. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `HashKind Zeile4`<br>`by_hash Zeile34`<br>`unique_match Zeile44`<br>`Tests Zeilen54–103` |
| 4. Aufrufer und Verbraucher | env/game/types.rs · env/loader.rs Typauflösung |
| 5. Verwendete Komponenten | kfc::reflection::TypeRegistry · TypeMetadata |
| 6. Eingaben und Ausgaben | Eingaben: Typregister, Hash und Hashdomäne Ausgabe: eindeutige Typmetadaten, kein Treffer oder Kollisionsfehler |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: kfc::reflection::TypeRegistry · TypeMetadata. Aufruferbezug: env/game/types.rs · env/loader.rs Typauflösung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Hashkollisionen dürfen durch Umzug nicht zu beliebiger Typwahl werden. |
| 14. Vorhandener Testschutz | hash_collision_never_selects_an_arbitrary_type · qualified_hash_resolves_while_shared_internal_hash_is_rejected |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Eindeutige Typauflösung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** HashKind Zeile4 · by_hash Zeile34 · unique_match Zeile44 · Tests Zeilen54–103

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/type_lookup.rs`.

### [src/loader/api/src/env/util.rs](../../execution/lua/bindings/util.rs)

Vollständig gelesen, 164 Zeilen. SHA-256: `468bd566b224a7fce2c9c7a5fed861f284ae47e193ccea1948a186cf41d4f589`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst Lua-Typselektoren auf, bindet Rustfunktionen mit optionaler Modbindung ein und konvertiert GUID/ContentHash aus Strings, Tabellen oder Asset-Userdata. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst Lua-Typselektoren auf, bindet Rustfunktionen mit optionaler Modbindung ein und konvertiert GUID/ContentHash aus Strings, Tabellen oder Asset-Userdata. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `get_type Zeile18`<br>`add_function Zeile36`<br>`add_function_with_mod Zeile54`<br>`CastLua<Guid> Zeile74`<br>`CastLua<ContentHash> Zeile109` |
| 4. Aufrufer und Verbraucher | env/game/assets.rs · env/game/guid.rs · env/buffer.rs · env/loader.rs |
| 5. Verwendete Komponenten | TypeHandle · TypeRegistry · Mod · CastLua · Content · Resource |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Argumente, Typregister, Mod und Funktionsclosure Ausgabe: TypHandle, registrierte Lua-Funktionen und GUID/ContentHash |
| 7. Seiteneffekte und Ressourcen | Lua-Funktionsallokation |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Allgemeine Funktionsregistrierung und fachliche GUID-/ContentHash-Konvertierung sind in derselben Hilfsdatei gebündelt |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lua-Bindungen und Wertansichten. Technische Kopplung: TypeHandle · TypeRegistry · Mod · CastLua · Content · Resource. Aufruferbezug: env/game/assets.rs · env/game/guid.rs · env/buffer.rs · env/loader.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Traitimplementierungen und modgebundene Closures brauchen ihre bisherigen Identitäten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Bindung und Assettypkonvertierung. Neue Einordnung: Lua-Bindungen und Wertansichten. Besitzer: Modausführung. |

**Belege:** get_type Zeile18 · add_function Zeile36 · add_function_with_mod Zeile54 · CastLua<Guid> Zeile74 · CastLua<ContentHash> Zeile109

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/bindings/util.rs`.

### [src/loader/api/src/lib.rs](../../execution/lib.rs)

Vollständig gelesen, 1503 Zeilen. SHA-256: `33ad1336cfa01bc65cf719f3c8d78d3a8eabdcbbff5b3a0eb8f94195cdfe7484`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt ShroudForgeApi bereit, führt vor Spielstart Lua-Assetmods mit Commit/Cache aus und verwaltet langlebige Runtime inklusive Lifecycle, Abhängigkeiten, Livekonfiguration, nativen DLLs, Netzwerkdiagnose, UI-Aufträgen, Reload und Status. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt ShroudForgeApi bereit, führt vor Spielstart Lua-Assetmods mit Commit/Cache aus und verwaltet langlebige Runtime inklusive Lifecycle, Abhängigkeiten, Livekonfiguration, nativen DLLs, Netzwerkdiagnose, UI-Aufträgen, Reload und Status. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `38: fn network_probe_failure_started() -> bool {`<br>`46: fn network_probe_recovered() -> bool {`<br>`61: fn network_allowlist(config: &serde_json::Value) -> (Vec<String>, bool) {`<br>`76: pub fn runtime_operations(_root: &std::path::Path) -> anyhow::Result<Vec<String>> {`<br>`85: pub struct ShroudForgeApi {`<br>`90: pub fn new(contract: GameContract) -> Self {`<br>`96: pub fn game_version(&self) -> &str {`<br>`100: pub fn parser_id(&self) -> &str {`<br>`104: pub fn type_by_name(&self, name: &str) -> Option<&TypeDefinition> {`<br>`108: pub fn types(&self) -> impl Iterator<Item = &TypeDefinition> {`<br>`112: pub fn resource_types(&self) -> impl Iterator<Item = (&str, usize)> {`<br>`120: pub fn resources(&self, type_name: &str) -> &[ResourceReference] {`<br>`128: pub fn has_runtime(&self, operation: &str) -> bool {`<br>`132: pub fn runtime(&self, operation: &str) -> Availability {`<br>`143: pub enum RuntimePhase {`<br>`150: pub fn as_str(self) -> &'static str {`<br>`159: pub struct RunOptions {`<br>`178: pub struct RunArgs {`<br>`184: pub fn run(env: &ModEnvironment, api: ShroudForgeApi, args: RunArgs) -> anyhow::Result<()> {`<br>`190: pub fn run_with_local_schema(env: &ModEnvironment, args: RunArgs) -> anyhow::Result<()> {`<br>`194: fn run_with_api(`<br>`306: struct Lifecycle {`<br>`318: fn try_unload(&mut self, lua: &mlua::Lua) -> mlua::Result<()> {`<br>`340: fn settings_only_manifest_change(desired: &serde_json::Value, applied: &serde_json::Value) -> bool {`<br>`377: pub struct IngameRuntime {`<br>`394: pub fn active_mod_ids(&self) -> Vec<String> {`<br>`402: pub fn start(`<br>`410: pub fn start_live(env: &ModEnvironment, file_name: impl Into<String>) -> anyhow::Result<Self> {`<br>`414: fn start_with_schema(`<br>`758: fn publish_status(&self, running: bool) {`<br>`776: pub fn update(&mut self, delta_seconds: f64) {`<br>`910: fn process_runtime_reload_request(&mut self) {`<br>`954: fn unload_runtime_packages(&mut self, mod_ids: &[String]) -> Result<(), String> {`<br>`976: fn reload_runtime_packages(&mut self, mod_ids: &[String]) -> Result<(), String> {`<br>`1065: fn restore_previous_runtime(&mut self, active_ids: &[String]) {`<br>`1102: fn refresh_configuration(&mut self) {`<br>`1335: fn drop(&mut self) {`<br>`1360: fn callback(`<br>`1377: pub fn export_lua_definitions(game_dir: impl AsRef<Path>, file_name: &str, force: bool) -> bool {`<br>`1395: pub fn restore(game_dir: impl AsRef<Path>, file_name: &str) -> bool {`<br>`1404: pub fn backup_companion_file(path: impl AsRef<std::path::Path>) -> bool {`<br>`1414: fn failed_start_or_update_retains_cleanup_scope_until_unload_succeeds() -> mlua::Result<()> {`<br>`1453: fn setting_value_change_does_not_look_like_package_metadata_change() {`<br>`1479: fn setting_definition_change_still_requires_restart() {` |
| 4. Aufrufer und Verbraucher | workflow::ModLoader, pregame, CLI restore/types und API-Integrationstests |
| 5. Verwendete Komponenten | mlua, package/ModEnvironment, AppState, LuaModRunner, Compatibility, Parser, runtime-diagnostics, Steamtransport |
| 6. Eingaben und Ausgaben | Eingaben: ModEnvironment, GameContract, RunArgs, delta_seconds und Datei-Reload-/Manifeständerungen Ausgabe: Lua-Ergebnisse, aktualisierte Assets, Cachedateien, Runtimezustand und JSON-Status |
| 7. Seiteneffekte und Ressourcen | führt Lua aus, schreibt Spielassets/Cache/Status, lädt Mod-DLLs indirekt, pollt Reload und manifest revision |
| 8. Art der Verantwortung | Mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Lua-API-Einstieg bleibt klein. Assetausführung, Lifecycle/Restaurierung und Dateiaufträge werden nach ihrer Lebensdauer getrennt. Befunde: Mehrere Verantwortungen: APIvertrag, Assetcommit, Lua-Lifecycle, Netzwerkbeobachtung, Dateianfragen und Status in einem Einstieg. Reload baut ganze Runtime, modIds beschränkt hier die abschließende Prüfung, nicht die Neuerstellung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Mod-Lifecycle und Assetausführung. Technische Kopplung: mlua, package/ModEnvironment, AppState, LuaModRunner, Compatibility, Parser, runtime-diagnostics, Steamtransport. Aufruferbezug: workflow::ModLoader, pregame, CLI restore/types und API-Integrationstests |
| 11. Duplikation | Überschneidungen mit Workflow-Assetplanung und Providerzugängen, keine pauschale Gleichsetzung unterschiedlicher Lebenszeiten |
| 12. Nutzungsstatus | Statisch belegter Produktaufruf, keine Liveausführung im Audit |
| 13. Risiko bei Änderung oder Verschiebung | Öffentliche API-/Lifecycle-Signaturen, Reverse-Unload, Cleanup-Pending, Dependencyregeln und Restoreergebnis erhalten |
| 14. Vorhandener Testschutz | Eingebettet: failed_start_or_update_retains_cleanup_scope_until_unload_succeeds, setting_value_change_does_not_look_like_package_metadata_change, setting_definition_change_still_requires_restart. Integrationstests unter api/tests, im Audit nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: API-Einstieg, Assetausführung und Mod-Lifecycle. Neue Einordnung: Mod-Lifecycle und Assetausführung. Besitzer: Modausführung. |

**Belege:** ShroudForgeApi, run_with_api, Lifecycle::try_unload, IngameRuntime::start_with_schema/update/refresh_configuration/reload_runtime_packages, restore

**Evidenzgrenze:** Dateiinhalt und Aufrufe nachgewiesen, Verhalten aus statischem Kontrollfluss abgeleitet

**Maßnahme:** aufteilen. Lua-API-Einstieg bleibt klein. Assetausführung, Lifecycle/Restaurierung und Dateiaufträge werden nach ihrer Lebensdauer getrennt.

**Vorgeschlagene Zielpfade:** `execution/lib.rs`, `execution/preparation/run.rs`, `execution/runtime/lifecycle.rs`, `execution/runtime/reload.rs`, `execution/runtime/requests.rs`.

### [src/loader/api/src/load.rs](../../execution/preparation/load.rs)

Vollständig gelesen, 483 Zeilen. SHA-256: `2c1fee252d40194d42e0db7a2c539e1eed1ec8cdc87268c5b64f0307892f1198`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Öffnet KFC-Leser/-Writer, sichert und restauriert Originaldateien samt Begleitressourcen, lädt EXE-identitätsgebundene Typregistercaches und exportiert Lua-Typdefinitionen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Öffnet KFC-Leser/-Writer, sichert und restauriert Originaldateien samt Begleitressourcen, lädt EXE-identitätsgebundene Typregistercaches und exportiert Lua-Typdefinitionen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `load_kfc_file Zeile14`<br>`create_writer Zeile62`<br>`create_backup Zeile93`<br>`create_companion_backup Zeile194`<br>`load_type_registry Zeile280`<br>`export_lua_definitions Zeile432` |
| 4. Aufrufer und Verbraucher | env/app_state.rs load_type_registry, export_lua_definitions, KFC-Leser/Backups/Writer Zeilen177–228 · lib.rs export_lua_definitions und restore_game_files/ensure_companion_backup Zeilen1377–1405 |
| 5. Verwendete Komponenten | KFCFile/Reader/Writer · TypeRegistry::load_from_executable · Sha256 · mod_loader::config::write_json · definition::generator |
| 6. Eingaben und Ausgaben | Eingaben: Spiel-/Cachepfade · KFC-Basisname · aktives Typregister · EXE-Bytes und Versionskennung Ausgabe: KFC-Leser/-Writer · Backuppfade · TypeRegistry plus Dirtyflag · Lua-Definitionsdateien |
| 7. Seiteneffekte und Ressourcen | Spielbackup und Restaurierung · Typcache-/Identitäts-JSON · Lua-Dateiausgabe · Diagnoselogging |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: KFC-Dateiinfrastruktur, Originalbackups, EXE-basierter Reflexionscache und Editor-Definitionsexport sind getrennte Änderungsgründe in derselben Datei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: KFC-Ladevorbereitung und Definitionsexport. Technische Kopplung: KFCFile/Reader/Writer · TypeRegistry::load_from_executable · Sha256 · mod_loader::config::write_json · definition::generator. Aufruferbezug: env/app_state.rs load_type_registry, export_lua_definitions, KFC-Leser/Backups/Writer Zeilen177–228 · lib.rs export_lua_definitions und restore_game_files/ensure_companion_backup Zeilen1377–1405 |
| 11. Duplikation | create_backup/restore_backup und Begleitdateivarianten teilen Backupoperationen, KFC-Validierung unterscheidet sich belegt |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Backup-/Writer-Reihenfolge und verwendete Originaldateierweiterung sind verhaltensrelevant. |
| 14. Vorhandener Testschutz | load.rs companion_resource_backup_restores_original_bytes |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Spielasset-Vorbereitungsdateien. Neue Einordnung: KFC-Ladevorbereitung und Definitionsexport. Besitzer: Modausführung. |

**Belege:** load_kfc_file Zeile14 · create_writer Zeile62 · create_backup Zeile93 · create_companion_backup Zeile194 · load_type_registry Zeile280 · export_lua_definitions Zeile432

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/preparation/load.rs`.

### [src/loader/api/src/log.rs](../../execution/log.rs)

Vollständig gelesen, 73 Zeilen. SHA-256: `555654446071162ded96c7143b6e49d10b204872997f47f3af85d88e2199f613`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bündelt tracing-Makros und Spans unter dem Diagnoseziel shroudforge::api. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bündelt tracing-Makros und Spans unter dem Diagnoseziel shroudforge::api. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `info/debug/error-Makros Zeilen3–31`<br>`Span-Makros Zeilen33–61`<br>`Reexporte Zeilen63–73` |
| 4. Aufrufer und Verbraucher | cache/*.rs · load.rs · lib.rs · env/io.rs · env/log.rs |
| 5. Verwendete Komponenten | tracing |
| 6. Eingaben und Ausgaben | Eingaben: tracing-Makroargumente Ausgabe: Logs und Spans im Ziel shroudforge::api |
| 7. Seiteneffekte und Ressourcen | Diagnoseausgabe |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: tracing. Aufruferbezug: cache/*.rs · load.rs · lib.rs · env/io.rs · env/log.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Logziele und Makroimporte aktualisieren. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: API-Diagnoseinfrastruktur. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** info/debug/error-Makros Zeilen3–31 · Span-Makros Zeilen33–61 · Reexporte Zeilen63–73

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/log.rs`.

### [src/loader/api/src/lua/args.rs](../../execution/lua/args.rs)

Vollständig gelesen, 102 Zeilen. SHA-256: `bba76d7318a904f3c28bb1b85a92c67e0d8a84d2a14c8e74d235cf20424567b3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Wrappt Lua-Mehrfachargumente mit typgeprüftem Zugriff und unterscheidet Funktionsargumente von Methodenempfängern. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Wrappt Lua-Mehrfachargumente mit typgeprüftem Zugriff und unterscheidet Funktionsargumente von Methodenempfängern. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Args Zeile4`<br>`FunctionArgs Zeile13`<br>`MethodArgs Zeile54` |
| 4. Aufrufer und Verbraucher | Lua-Bindungen unter env · runner/mod.rs require-Bindung |
| 5. Verwendete Komponenten | mlua::MultiValue · CastLua · LuaError |
| 6. Eingaben und Ausgaben | Eingaben: mlua::MultiValue und gewünschter Zieltyp Ausgabe: typgeprüfte Argumente oder Lua-Fehler |
| 7. Seiteneffekte und Ressourcen | Userdata-Ausleihe |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: mlua::MultiValue · CastLua · LuaError. Aufruferbezug: Lua-Bindungen unter env · runner/mod.rs require-Bindung |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Methodenempfänger und Argumentindizes sind bindungsweit verwendet. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Argumentkonvertierung. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** Args Zeile4 · FunctionArgs Zeile13 · MethodArgs Zeile54

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/args.rs`.

### [src/loader/api/src/lua/cast.rs](../../execution/lua/cast.rs)

Vollständig gelesen, 446 Zeilen. SHA-256: `eb6cc670ea26c1b0c5a498aba5a6a15ab6d061c8d5835cf15e131e08e9e40dc4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Konvertiert Lua-Werte geprüft in primitive Rusttypen, geliehene Zeichenbytes und Userdata sowie optionale beziehungsweise alternative Argumenttypen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Konvertiert Lua-Werte geprüft in primitive Rusttypen, geliehene Zeichenbytes und Userdata sowie optionale beziehungsweise alternative Argumenttypen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `CastLua Zeile11`<br>`impl_primitive Zeile283`<br>`Userdata-Ausleihe Zeile322`<br>`Option Zeile379`<br>`Either Zeile398`<br>`CastLuaExt Zeile427` |
| 4. Aufrufer und Verbraucher | lua/args.rs · env/util.rs · env-Bindungen |
| 5. Verwendete Komponenten | mlua · half::f16 · short_type_name · LuaError |
| 6. Eingaben und Ausgaben | Eingaben: LuaValue und gewünschter Rusttyp Ausgabe: geprüfter Wert, Ausleihe oder kein Match |
| 7. Seiteneffekte und Ressourcen | Userdata-Ausleihe und Stringkonvertierung |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: mlua · half::f16 · short_type_name · LuaError. Aufruferbezug: lua/args.rs · env/util.rs · env-Bindungen |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Zahlenbereiche und Ausleihregeln wirken in allen Bindungen. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Wertkonvertierung. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** CastLua Zeile11 · impl_primitive Zeile283 · Userdata-Ausleihe Zeile322 · Option Zeile379 · Either Zeile398 · CastLuaExt Zeile427

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/cast.rs`.

### [src/loader/api/src/lua/error.rs](../../execution/lua/error.rs)

Vollständig gelesen, 119 Zeilen. SHA-256: `46bc7bee7fa5683927112a4c51ce14d03bca378b1d5d5793b36003794eddd28c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erzeugt konsistente Lua-Fehlermeldungen für Argumente, Empfänger, Typen, Grenzen, Module, Modkennungen und Inhalte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erzeugt konsistente Lua-Fehlermeldungen für Argumente, Empfänger, Typen, Grenzen, Module, Modkennungen und Inhalte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `LuaError Zeile3`<br>`bad_argument_type Zeile35`<br>`module_load Zeile86`<br>`external Zeile109` |
| 4. Aufrufer und Verbraucher | lua/args.rs · lua/cast.rs · env-Bindungen · runner/state.rs |
| 5. Verwendete Komponenten | mlua::Error · LuaValue |
| 6. Eingaben und Ausgaben | Eingaben: Fehlerkontext, Argument-/Typ-/Modulinfos Ausgabe: mlua::Error mit konsistenter Meldung |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: mlua::Error · LuaValue. Aufruferbezug: lua/args.rs · lua/cast.rs · env-Bindungen · runner/state.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Diagnoseformat und Fehlerursachen erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Fehlerdiagnostik. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** LuaError Zeile3 · bad_argument_type Zeile35 · module_load Zeile86 · external Zeile109

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/error.rs`.

### [src/loader/api/src/lua/mod.rs](../../execution/lua/mod.rs)

Vollständig gelesen, 171 Zeilen. SHA-256: `e46ad9f72bf93ab0adf80220309c64d4dee4c0e7d1871d42c4e53a1c18f5f2b3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erstellt die Lua-VM mit ausgewählten Standardbibliotheken und getrennten Basistabellen je Mod, ergänzt typeof und sammelt Lua-Warnungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erstellt die Lua-VM mit ausgewählten Standardbibliotheken und getrennten Basistabellen je Mod, ergänzt typeof und sammelt Lua-Warnungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `LuaVM::new Zeile20`<br>`create_base_environment Zeile41`<br>`copy_table Zeile121`<br>`register_warn_function Zeile147` |
| 4. Aufrufer und Verbraucher | runner/mod.rs LuaModRunner::new und create_mod_environment |
| 5. Verwendete Komponenten | mlua · args · cast · error · tracing |
| 6. Eingaben und Ausgaben | Eingaben: VM-Erstellung und Standardbibliothekstabellen Ausgabe: LuaVM und getrennte Basisumgebung |
| 7. Seiteneffekte und Ressourcen | Lua-VM-Allokation, entfernt dynamische Ladeglobals, sammelt Warnlogs |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: mlua · args · cast · error · tracing. Aufruferbezug: runner/mod.rs LuaModRunner::new und create_mod_environment |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Zugängliche Standardbibliotheken und Umgebungsisolation erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-VM und Basisumgebung. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** LuaVM::new Zeile20 · create_base_environment Zeile41 · copy_table Zeile121 · register_warn_function Zeile147

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/mod.rs`.

### [src/loader/api/src/runner/mod.rs](../../execution/runner/mod.rs)

Vollständig gelesen, 116 Zeilen. SHA-256: `de3d1e4986e36d98003bd8467d5a01b09f104cc384c06d058adca1d49afaf7d0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verwaltet die Lua-VM, richtet Modumgebungen ein und führt Vorbereitungs- oder Runtime-Entrypoints gemäß Fähigkeiten und Phase aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verwaltet die Lua-VM, richtet Modumgebungen ein und führt Vorbereitungs- oder Runtime-Entrypoints gemäß Fähigkeiten und Phase aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `LuaModRunner Zeile14`<br>`run Zeile29`<br>`runtime_mod_ids Zeile59`<br>`setup_mod Zeile92` |
| 4. Aufrufer und Verbraucher | lib.rs Api-Vorbereitung und Runtimezyklus |
| 5. Verwendete Komponenten | LuaVM · AppState · RunnerState · ModManifest::requires_pregame · Capability |
| 6. Eingaben und Ausgaben | Eingaben: AppState und registrierte Mods mit Fähigkeiten Ausgabe: vorbereitete Modumgebungen und Entrypointresultate |
| 7. Seiteneffekte und Ressourcen | führt Lua-Modcode phasenabhängig aus |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: LuaVM · AppState · RunnerState · ModManifest::requires_pregame · Capability. Aufruferbezug: lib.rs Api-Vorbereitung und Runtimezyklus |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Runtime-Entrypoints dürfen nicht durch Exportrecht zur Vorbereitung ausgeführt werden. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Modausführung. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** LuaModRunner Zeile14 · run Zeile29 · runtime_mod_ids Zeile59 · setup_mod Zeile92

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/runner/mod.rs`.

### [src/loader/api/src/runner/state.rs](../../execution/runner/state.rs)

Vollständig gelesen, 214 Zeilen. SHA-256: `bd440f36df8a4816d1319c9550d0cfc209381f6aa3bc5a294a39fcac8d78e86f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Hält Modumgebungen und Modulcaches, lädt Lua aus src-Modordnerdateien und verhindert zyklische require-Aufrufe mit Rücksetzschutz. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Hält Modumgebungen und Modulcaches, lädt Lua aus src-Modordnerdateien und verhindert zyklische require-Aufrufe mit Rücksetzschutz. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `RunnerState Zeile34`<br>`LoadingGuard Zeile40`<br>`require Zeile63`<br>`require_mod Zeile136`<br>`RunnerLocalState Zeile174` |
| 4. Aufrufer und Verbraucher | runner/mod.rs setup und run · RunnerLocalState::require |
| 5. Verwendete Komponenten | Mod::fs · RequirePath · LuaError · RefCell · Rc |
| 6. Eingaben und Ausgaben | Eingaben: Modulname, Mod-ID, Moddateisystem und Lua-Umgebung Ausgabe: gecachtes Modulergebnis |
| 7. Seiteneffekte und Ressourcen | liest Modquelldateien, führt Lua-Code aus, mutiert Modulcaches |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: Mod::fs · RequirePath · LuaError · RefCell · Rc. Aufruferbezug: runner/mod.rs setup und run · RunnerLocalState::require |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Exakte Manifest-IDs, require-Zyklen und Fehlerbereinigung erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Modmodulladen. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** RunnerState Zeile34 · LoadingGuard Zeile40 · require Zeile63 · require_mod Zeile136 · RunnerLocalState Zeile174

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/runner/state.rs`.

### [src/loader/api/src/runtime_resolution/attributes.rs](../../execution/reflection/attributes.rs)

Vollständig gelesen, 771 Zeilen. SHA-256: `1430cede3adc20def2696cee392b320b6470a2269ecf1881e7ee8d91142404c0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest originale Attribut-IDs, Namen und Programme aus KFC-Ressourcen, verbindet sie mit reflektierten Header-/Speicherlayouts, klassifiziert Skalar- und Signiertheitsvereinbarkeit und berechnet vollständige Wurzeln in Originalreihenfolge. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest originale Attribut-IDs, Namen und Programme aus KFC-Ressourcen, verbindet sie mit reflektierten Header-/Speicherlayouts, klassifiziert Skalar- und Signiertheitsvereinbarkeit und berechnet vollständige Wurzeln in Originalreihenfolge. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Catalog/CatalogIndexes Zeilen18–55`<br>`header_layout Zeile61`<br>`component_layouts Zeile188`<br>`Catalog::load Zeile311`<br>`from_resources Zeile340`<br>`root_entries Zeile542`<br>`recalculate Zeile552`<br>`definition_root Zeile590`<br>`decode_program Zeile610` |
| 4. Aufrufer und Verbraucher | AppState::attribute_catalog · env/runtime_attributes.rs · runtime_functions.rs Modifieranforderungen über AppState |
| 5. Verwendete Komponenten | KFCReader/File · KfcValue · TypeRegistry · types::fields/inheritance · calculation::evaluate · kfc_format |
| 6. Eingaben und Ausgaben | Eingaben: aktives Typregister und KFC-AttributeContainerResource/BaseAttributeResource · reflektierte Layouts, Originalprogramme und eigene Wurzelwerte Ausgabe: Attributkatalog mit Identitäts-/Layout-/Schreibstatus · benannte oder ID-basierte Auflösung · neue Wurzelwerte plus Trace |
| 7. Seiteneffekte und Ressourcen | Catalog::load liest KFC-Ressourcen über übergebenen Reader · unveränderlicher Lazyindex nach erstem Zugriff · keine Prozessmutation |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: KFC-Ressourceneinlese und reine Katalog-/Berechnungsableitung teilen dieselbe Datei. Der KFCReader ist injiziert, direkter Prozesszugriff ist hier nicht vorhanden. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Reine Reflectionauflösung. Technische Kopplung: KFCReader/File · KfcValue · TypeRegistry · types::fields/inheritance · calculation::evaluate · kfc_format. Aufruferbezug: AppState::attribute_catalog · env/runtime_attributes.rs · runtime_functions.rs Modifieranforderungen über AppState |
| 11. Duplikation | decode_program beschreibt dieselben Attributopcodes, calculation::evaluate führt deren verifiziertes Teilmodell aus. Darstellung und Auswertung sind getrennte Rollen, keine unbelegte Austauschbarkeit. |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Attribut-ID ist ausdrücklich kein FNV des Anzeigenamens, kompakte und breite Header werden aus Reflexion bestimmt und wurzelweise Reihenfolge darf nicht nach Hash sortiert werden. |
| 14. Vorhandener Testschutz | attribute_header_uses_reflected_kfc3_and_kfc4_widths · lookup_indexes_keep_original_ids_root_order_and_ambiguous_names · lookup_indexes_reject_incomplete_and_duplicate_root_positions · env/runtime_attributes.rs Entityjoin- und Berechnungstests |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Attributdefinitionen und Berechnungsauflösung. Neue Einordnung: Reine Reflectionauflösung. Besitzer: Modausführung. |

**Belege:** Catalog/CatalogIndexes Zeilen18–55 · header_layout Zeile61 · component_layouts Zeile188 · Catalog::load Zeile311 · from_resources Zeile340 · root_entries Zeile542 · recalculate Zeile552 · definition_root Zeile590 · decode_program Zeile610

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/reflection/attributes.rs`.

### [src/loader/api/src/runtime_resolution/calculation.rs](../../execution/reflection/calculation.rs)

Vollständig gelesen, 287 Zeilen. SHA-256: `6e27a86d491ad334181df59052fbf72cfb425d0d89bb70d8c4f04dd3bc4e66bc`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Interpretiert das begrenzte originale Attributprogramm auf eigenen32-Bit-Wurzeldaten mit signierter/unsignierter/Floatarithmetik, Stackgrenzen, Clamp und ScaleToNewMax-Referenzänderung, Fehler bleiben atomar. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Interpretiert das begrenzte originale Attributprogramm auf eigenen32-Bit-Wurzeldaten mit signierter/unsignierter/Floatarithmetik, Stackgrenzen, Clamp und ScaleToNewMax-Referenzänderung, Fehler bleiben atomar. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Scalar Zeile6`<br>`evaluate Zeile54`<br>`Push-Literalkonvertierung Zeile88`<br>`Clamp Zeile122`<br>`ScaleToNewMax Zeile151`<br>`atomarer Commit Zeile201`<br>`Tests Zeilen210–287` |
| 4. Aufrufer und Verbraucher | runtime_resolution/attributes.rs Catalog::recalculate |
| 5. Verwendete Komponenten | std ausschließlich im Produktpfad · serde_json und explizite Datei für ignorierten Differentialtest |
| 6. Eingaben und Ausgaben | Eingaben: Skalarart, eigene u32-Wurzelwerte und originale Programmwörter Ausgabe: optional berechneter Skalar und atomar aktualisierte Wurzelwerte oder Fehler |
| 7. Seiteneffekte und Ressourcen | mutiert nur übergebene eigene Werte nach vollständigem Erfolg · ignorierter Differentialtest liest explizite Vektordatei |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Reine Reflectionauflösung. Technische Kopplung: std ausschließlich im Produktpfad · serde_json und explizite Datei für ignorierten Differentialtest. Aufruferbezug: runtime_resolution/attributes.rs Catalog::recalculate |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. f32-Rundung,±0-Auswahl, wrapping-Arithmetik, absteigende externe Wurzelreihenfolge und Referenzmutation sind native Semantik. |
| 14. Vorhandener Testschutz | bounds_and_failure_are_atomic · float_push_is_a_numeric_integer_conversion · float_clamp_preserves_original_zero_bound_selection · original_engine_differential ist explizit ignoriert und erwartet unabhängig emulierte Enginevektoren |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Eigene Attributprogrammberechnung. Neue Einordnung: Reine Reflectionauflösung. Besitzer: Modausführung. |

**Belege:** Scalar Zeile6 · evaluate Zeile54 · Push-Literalkonvertierung Zeile88 · Clamp Zeile122 · ScaleToNewMax Zeile151 · atomarer Commit Zeile201 · Tests Zeilen210–287

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/reflection/calculation.rs`.

### [src/loader/api/src/runtime_resolution/functions.rs](../../execution/reflection/functions.rs)

Vollständig gelesen, 512 Zeilen. SHA-256: `e881a5660dd12e1a4dc934ef4e3cc392888da67df5736d85ab095e934edefdc4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verknüpft originale Engine-Systemabhängigkeitsdeskriptoren aus dem nativen Codeinventar mit Parserreflection und lebenden Runtime-/Template-Komponentenregistrierungen. Validiert exakte Schedulerbelege für sechs Slots, löst Typidentität, Layout und eindeutige Registrierungsrollen auf, ergänzt Funktionsdeskriptoren und Coverage und filtert Funktionen anhand Komponentennamen/Aliasbeziehungen. Die Auflösung erteilt keine native Aufrufberechtigung und hält ungeklärte Optionalität und Signatur ausdrücklich offen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verknüpft originale Engine-Systemabhängigkeitsdeskriptoren aus dem nativen Codeinventar mit Parserreflection und lebenden Runtime-/Template-Komponentenregistrierungen. Validiert exakte Schedulerbelege für sechs Slots, löst Typidentität, Layout und eindeutige Registrierungsrollen auf, ergänzt Funktionsdeskriptoren und Coverage und filtert Funktionen anhand Komponentennamen/Aliasbeziehungen. Die Auflösung erteilt keine native Aufrufberechtigung und hält ungeklärte Optionalität und Signatur ausdrücklich offen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Resolver::{new,set_inventory,set_registry,enrich,coverage,matches_component}`<br>`checked_live_type/dependency/slots_complete` |
| 4. Aufrufer und Verbraucher | env/runtime_functions.rs importiert Resolver:8. · Catalog erstellt Resolver und set_inventory:49. · Catalog::set_registry ruft resolution.set_registry:109 und coverage:117. · Funktionsbeschreibung nutzt enrich:203, Filter nutzt matches_component:489. |
| 5. Verwendete Komponenten | kfc::reflection::{TypeIndex,TypeMetadata,TypeRegistry} · serde_json::Value/json · runtime_resolution/types.rs:inheritance/fields · Rc und BTreeMap/BTreeSet · Testfixture per include_str aus runtime/profiles/enshrouded/client/25966634.json |
| 6. Eingaben und Ausgaben | Eingaben: Parser-TypeRegistry. · Native Inventar-JSON mit exakter Imageidentität und system_dependency_layout. · Live-Registry-JSON mit Registrierungsnamen, Hashes, Größen, Runtime-/Templatedeskriptoren. · Funktionsentries mit engine_descriptors/dependency_slots und Komponentenselektoren. Ausgabe: Angereicherte Deskriptoren mit aufgelösten Typ-/Registrierungsbeziehungen und zugelassenen Speicher-/Zugriffsrollen. · Coverage mit getrennten Reflection-, Registrierungs-, Slot- und Accessmodeauflösungen. · Boolesche Funktions-/Komponentenassoziation samt Aliasbeziehungen und explizite Gründe für fehlende Auflösung. |
| 7. Seiteneffekte und Ressourcen | Ändert ausschließlich den eigenen Resolverzustand und übergebenes JSON-Entry. Keine Prozess-/Datei-/Netzwerk-/Engineoperation im Produktcode. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung durch die vollständige Lektüre belegt. Validierung, Verknüpfung, Anreicherung und Coverage dienen derselben Belegauflösungsverantwortung. · Auflösung nutzt dynamische JSON-Verträge statt statisch typisierter Inventar-/Registrydaten. Änderungen an Feldnamen/Nullsemantik sind implizite Kopplungen mit nativem Inventar und Runtime-Katalog. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Reine Reflectionauflösung. Technische Kopplung: kfc::reflection::{TypeIndex,TypeMetadata,TypeRegistry} · serde_json::Value/json · runtime_resolution/types.rs:inheritance/fields · Rc und BTreeMap/BTreeSet · Testfixture per include_str aus runtime/profiles/enshrouded/client/25966634.json. Aufruferbezug: env/runtime_functions.rs importiert Resolver:8. · Catalog erstellt Resolver und set_inventory:49. · Catalog::set_registry ruft resolution.set_registry:109 und coverage:117. · Funktionsbeschreibung nutzt enrich:203, Filter nutzt matches_component:489. |
| 11. Duplikation | Normalisierung kurzer ECS-Namen zu keen::ecs::* erfolgt in dependency:222-227 und matches_component:377-387 erneut. Gleiche Alias-/Namensthematik, aber unterschiedliche Ergebnisform. · Validierte Registry-/Reflectionidentität wird zusätzlich auf Rustseite geprüft, nachdem die native Registry eigene Strukturprüfungen ausführt. Das ist eine Integrationsgrenze mit anderen Eingaben, keine als entbehrlich belegte Duplikation. |
| 12. Nutzungsstatus | Aktiv durch env/runtime_functions.rs Catalog, Belege siehe callers. Direkte Produktverwendung aller öffentlichen Resolvermethoden nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Modulpfad crate::runtime_resolution::functions::Resolver und Zugriff auf benachbarte types-Hilfen mitziehen. Test-include_str-Pfad zum exakten Profil ist verschiebungsempfindlich. Inventar-/Registry-JSON-Feldnamen und getrennte Auflösungsfakten müssen kompatibel bleiben. |
| 14. Vorhandener Testschutz | Im selben File: exact_image_scheduler_evidence_resolves_six_roles_but_no_native_signature:425-463. · different_image_or_missing_validation_never_inherits_roles:466-481. · incomplete_legacy_snapshot_keeps_roles_separate_from_complete_coverage:484-498. · duplicate_model_slot_is_rejected_and_previous_roles_are_cleared:501-511. · Aufrufer env/runtime_functions.rs enthält weitere Catalogtests, deren genaue Schutzwirkung hier nur über set_registry-Verwendung gezielt belegt ist, nicht vollständig in diesem Teilscope gelesen. Keine Tests ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Auflösung originaler Engine-Systemabhängigkeiten. Neue Einordnung: Reine Reflectionauflösung. Besitzer: Modausführung. |

**Belege:** 11-35 Resolverzustand · 38-90 exaktes Schedulerlayout · 123-220 Registry-/Reflectionvalidierung · 222-264 Abhängigkeitsauflösung · 266-320 enrich · 322-375 coverage · 377-391 matches_component · 394-512 lokale Tests

**Evidenzgrenze:** Alle512Zeilen vollständig in zwei lückenlosen begrenzten Chunks gelesen. Eingehende Runtimefunktionsverwendungen gezielt belegt. Profiltestpfad verfolgt. Implementierung nachgewiesen, fachliche Nachbarschaft abgeleitet, keine dynamische Prüfung.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/reflection/functions.rs`.

### [src/loader/api/src/runtime_resolution/mod.rs](../../execution/reflection/mod.rs)

Vollständig gelesen, 5 Zeilen. SHA-256: `9d73b0c2d32b4705dbedde04f88c03d90d6988d7eb48413e0a864a2db387ca01`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Fasst reine Auflösung von Attributen, Berechnungen, Funktionsabhängigkeiten und Typtraversierung ohne Lua-/Prozesszugriff zusammen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Fasst reine Auflösung von Attributen, Berechnungen, Funktionsabhängigkeiten und Typtraversierung ohne Lua-/Prozesszugriff zusammen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Moduldeklarationen Zeilen2–5` |
| 4. Aufrufer und Verbraucher | lib.rs Moduldeklaration · env/runtime_attributes.rs · env/runtime_functions.rs · env/registry.rs |
| 5. Verwendete Komponenten | attributes · calculation · functions · types |
| 6. Eingaben und Ausgaben | Eingaben: vier Unterdateien Ausgabe: interne Modulpfade |
| 7. Seiteneffekte und Ressourcen | keine eigene Laufzeitaktion |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Reine Reflectionauflösung. Technische Kopplung: attributes · calculation · functions · types. Aufruferbezug: lib.rs Moduldeklaration · env/runtime_attributes.rs · env/runtime_functions.rs · env/registry.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Beispielpfade zu functions/types beachten. |
| 14. Vorhandener Testschutz | Unterdateien enthalten jeweils eigene Tests beziehungsweise env/registry.rs Traversierungstests |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Reflektierte Runtime-Datenauflösung. Neue Einordnung: Reine Reflectionauflösung. Besitzer: Modausführung. |

**Belege:** Moduldeklarationen Zeilen2–5

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/reflection/mod.rs`.

### [src/loader/api/src/runtime_resolution/types.rs](../../execution/reflection/types.rs)

Vollständig gelesen, 44 Zeilen. SHA-256: `b1eb101bb81c4fe87467727f1ef12bcd4d9cf8db51b627035bae794b9b6652e2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Durchläuft ausschließlich Struct-/Typedef-Vererbung zyklussicher und liefert Basis-zuerst wirksame Felder mit deklarierendem Typ, überschreibbare Feldnamen bleiben eindeutig. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Durchläuft ausschließlich Struct-/Typedef-Vererbung zyklussicher und liefert Basis-zuerst wirksame Felder mit deklarierendem Typ, überschreibbare Feldnamen bleiben eindeutig. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `inheritance Zeile6`<br>`Struct/Typedefgrenze Zeilen20–27`<br>`fields Zeile32` |
| 4. Aufrufer und Verbraucher | env/registry.rs Reexport · runtime_resolution/functions.rs · examples/inspect_runtime_functions.rs via path |
| 5. Verwendete Komponenten | TypeRegistry · PrimitiveType · IndexMap · HashSet |
| 6. Eingaben und Ausgaben | Eingaben: Typregister und TypeIndex Ausgabe: Vererbungskette oder wirksame Felder samt Eigentümer |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Reine Reflectionauflösung. Technische Kopplung: TypeRegistry · PrimitiveType · IndexMap · HashSet. Aufruferbezug: env/registry.rs Reexport · runtime_resolution/functions.rs · examples/inspect_runtime_functions.rs via path |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Keine Arrayelemente als Basistypen interpretieren, Beispiel-path-Import nachziehen. |
| 14. Vorhandener Testschutz | env/registry.rs inheritance_is_not_element_containment_and_fields_keep_owners · env/registry.rs cycles_are_reported_instead_of_looping |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Reflektierte Vererbung und Feldauflösung. Neue Einordnung: Reine Reflectionauflösung. Besitzer: Modausführung. |

**Belege:** inheritance Zeile6 · Struct/Typedefgrenze Zeilen20–27 · fields Zeile32

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/reflection/types.rs`.

### [src/loader/api/src/shroudforge/v1/definitions/runtime.lua](../../execution/lua/shroudforge/v1/definitions/runtime.lua)

Vollständig gelesen, 899 Zeilen. SHA-256: `e22bb7142f480cda1d6313060b297783b1efb7f9da5850b86848c9bc9f41bc7e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert die gesamte ausführungsbezogene Lua-API mit Phasen/Fähigkeiten, Modlebenszyklus, Claims/Overrides, Netzwerk, Diensten, Werten, Funktionsinventar, ECS/Attributen, Templates und Weltoperationen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert die gesamte ausführungsbezogene Lua-API mit Phasen/Fähigkeiten, Modlebenszyklus, Claims/Overrides, Netzwerk, Diensten, Werten, Funktionsinventar, ECS/Attributen, Templates und Weltoperationen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `RuntimeApi Zeilen5–21`<br>`Overrides Zeilen28–131`<br>`Services Zeilen133–168`<br>`Network Zeilen170–232`<br>`Functions Zeilen263–377`<br>`ECS Zeilen403–682`<br>`World Zeilen684–863` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | env/loader.rs · env/runtime_*.rs · TypeRegistry · Resource · ShroudForge |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Typdefinitionen und dokumentierte aktuelle Runtimeoberflächen Ausgabe: eingebetteter Lua-Language-Server-Vertrag |
| 7. Seiteneffekte und Ressourcen | im belegten Buildpfad nur Definitionsquelle |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: ECS, Netzwerk, Ownership, Reflexion, eigene Werte und Weltinterventionen sind als ein sehr großer Vertrag zusammengeführt. · ShroudForge-Klasse und shroudforge-Zuweisung überlappen definitions/shroudforge.lua, das dokumentierte mod_kind wird von shroudforge.rs nicht registriert. · Website publicClassName bildet RuntimeLifecycle-Felder auf runtime.lifecycle.* ab. Die Runtime registriert keine solche Tabelle, die Lifecycle ist ein Rückgabetabellenvertrag der Mods und runtime.lifecycle eine Feature-ID. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: ShroudForge-Lua-Vertrag. Technische Kopplung: env/loader.rs · env/runtime_*.rs · TypeRegistry · Resource · ShroudForge. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | ShroudForge-Klassenbeschreibung und Zuweisung überschneidet sich mit shroudforge.lua |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Fachliche Aufteilung muss alle Signaturen und zusammengeführte Buildausgabe erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Laufzeitvertrag. Neue Einordnung: ShroudForge-Lua-Vertrag. Besitzer: Modausführung. |

**Belege:** RuntimeApi Zeilen5–21 · Overrides Zeilen28–131 · Services Zeilen133–168 · Network Zeilen170–232 · Functions Zeilen263–377 · ECS Zeilen403–682 · World Zeilen684–863

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/shroudforge/v1/definitions/runtime.lua`.

### [src/loader/api/src/shroudforge/v1/definitions/shroudforge.lua](../../execution/lua/shroudforge/v1/definitions/shroudforge.lua)

Vollständig gelesen, 59 Zeilen. SHA-256: `f9183c794010deeccf3ee9885938b93c0a0fe51f3e32fc4541fe4505b3892bfc`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Dokumentiert ShroudForge-Einstellungen, UI-Aktionscallbacks, lokale Nachrichten und fokussierte F1–F8-Abfragen für Lua-Mods. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Dokumentiert ShroudForge-Einstellungen, UI-Aktionscallbacks, lokale Nachrichten und fokussierte F1–F8-Abfragen für Lua-Mods. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `settings.get Zeile13`<br>`ui.on_action Zeile23`<br>`notifications.publish Zeile40`<br>`input.is_key_down Zeile49`<br>`ShroudForge Zeilen51–59` |
| 4. Aufrufer und Verbraucher | build.rs collect_lua_files |
| 5. Verwendete Komponenten | shroudforge.rs · extended.mod.json Einstellungs-/UIdeklarationen |
| 6. Eingaben und Ausgaben | Eingaben: dokumentierte ShroudForge-APItypen und Methoden Ausgabe: eingebetteter Lua-Werkzeugvertrag |
| 7. Seiteneffekte und Ressourcen | im belegten Buildpfad nur Definitionsquelle |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: mod_kind ist dokumentiert, create in shroudforge.rs setzt dieses Feld nicht |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: ShroudForge-Lua-Vertrag. Technische Kopplung: shroudforge.rs · extended.mod.json Einstellungs-/UIdeklarationen. Aufruferbezug: build.rs collect_lua_files |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Buildaufnahme und ausgeführte Bindung konsistent halten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ShroudForge-Lua-Vertrag. Neue Einordnung: ShroudForge-Lua-Vertrag. Besitzer: Modausführung. |

**Belege:** settings.get Zeile13 · ui.on_action Zeile23 · notifications.publish Zeile40 · input.is_key_down Zeile49 · ShroudForge Zeilen51–59

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/shroudforge/v1/definitions/shroudforge.lua`.

### [src/loader/api/src/shroudforge/v1/lua/override_targets.lua](../../execution/lua/shroudforge/v1/lua/override_targets.lua)

Vollständig gelesen, 161 Zeilen. SHA-256: `2799333fce05d712e33e7ca09557017547178e344c60032a8668ef61e04389cc`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erzeugt Override-Lese-/Schreibadapter für Attribute, Komponenten, Ressourcen, Services und Templates über gemeinsame geprüfte öffentliche APIs und native Besitzclaims. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erzeugt Override-Lese-/Schreibadapter für Attribute, Komponenten, Ressourcen, Services und Templates über gemeinsame geprüfte öffentliche APIs und native Besitzclaims. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `attribute Zeile14`<br>`view_target Zeile58`<br>`resource Zeile88`<br>`component Zeile123`<br>`Zieltypen Zeilen152–161` |
| 4. Aufrufer und Verbraucher | env/mod.rs register eval/call · flight.rs und survival.rs Targetfactory |
| 5. Verwendete Komponenten | runtime.overrides Claims · runtime.ecs · runtime.values · runtime.world.session_id · game.assets |
| 6. Eingaben und Ausgaben | Eingaben: Runtime-/Assets-API, Zielidentität, Feldpfad und Abhängigkeitserwartungen Ausgabe: Adapter mit key/claim/read/write/normalize |
| 7. Seiteneffekte und Ressourcen | delegierte Claims und überprüfte Runtimezugriffe bei Adapteraufruf · begrenzter Attributnamencache |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: ShroudForge-Lua-Vertrag. Technische Kopplung: runtime.overrides Claims · runtime.ecs · runtime.values · runtime.world.session_id · game.assets. Aufruferbezug: env/mod.rs register eval/call · flight.rs und survival.rs Targetfactory |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Sitzung, unabhängiger Readback und kanonische Normalisierung sind Teil des Änderungsvertrags. |
| 14. Vorhandener Testschutz | flight.rs/flight.lua · survival.rs/survival.lua |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Geprüfte temporäre Laufzeitziele. Neue Einordnung: ShroudForge-Lua-Vertrag. Besitzer: Modausführung. |

**Belege:** attribute Zeile14 · view_target Zeile58 · resource Zeile88 · component Zeile123 · Zieltypen Zeilen152–161

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/shroudforge/v1/lua/override_targets.lua`.

### [src/loader/api/src/shroudforge/v1/lua/overrides.lua](../../execution/lua/shroudforge/v1/lua/overrides.lua)

Vollständig gelesen, 198 Zeilen. SHA-256: `a3c8d3285aa7b3337eecb998b9abaa0094fddadd2989ee8d55b04ccf8e5741ef`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verfolgt temporäre Feldänderungen mit Baselines, modübergreifenden Claims, Konflikterhalt, Wiederherstellung, Sitzungswechseln und dauerhaftem Schreibstopp nach ungewissem Ausgang. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verfolgt temporäre Feldänderungen mit Baselines, modübergreifenden Claims, Konflikterhalt, Wiederherstellung, Sitzungswechseln und dauerhaftem Schreibstopp nach ungewissem Ausgang. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `new Zeile3`<br>`begin_update Zeile6`<br>`write Zeile36`<br>`apply Zeile67`<br>`restore Zeile139`<br>`reset Zeile177` |
| 4. Aufrufer und Verbraucher | env/runtime_overrides.rs eingebettete Trackerfabrik · tests/runtime_overrides.rs · Flug-/Survivalfixtures |
| 5. Verwendete Komponenten | Adapter mit read/write/claim/normalize · optionale Weltsitzungsabfrage |
| 6. Eingaben und Ausgaben | Eingaben: Adapter, gewünschte Wertfunktion, Einstellung und Sitzung Ausgabe: Tracker, Besitz-/Faultzustand und Zusammenfassung |
| 7. Seiteneffekte und Ressourcen | native Claims und Schreibzugriffe ausschließlich durch Adapter · gemeinsame active_keys und lokale Baselines |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: ShroudForge-Lua-Vertrag. Technische Kopplung: Adapter mit read/write/claim/normalize · optionale Weltsitzungsabfrage. Aufruferbezug: env/runtime_overrides.rs eingebettete Trackerfabrik · tests/runtime_overrides.rs · Flug-/Survivalfixtures |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Fault muss spätere Apply- und Restorewrites sperren und unaufgelösten Besitz erhalten. |
| 14. Vorhandener Testschutz | runtime_overrides.rs/runtime_overrides.lua · flight.rs/flight.lua · survival.rs/survival.lua |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Temporäre Laufzeitänderungen und Wiederherstellung. Neue Einordnung: ShroudForge-Lua-Vertrag. Besitzer: Modausführung. |

**Belege:** new Zeile3 · begin_update Zeile6 · write Zeile36 · apply Zeile67 · restore Zeile139 · reset Zeile177

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/shroudforge/v1/lua/overrides.lua`.

### [src/loader/api/src/shroudforge/v1/shroudforge.rs](../../execution/lua/shroudforge/v1/shroudforge.rs)

Vollständig gelesen, 320 Zeilen. SHA-256: `4a266b1b536b9a3eb0582f4da66b27f73167f761e55e28f3edfd555d217f8d0d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet Modsettings, strukturiertes Modlogging, fokussierte Funktionstasten, dateibasierte UI-Aktionszustellung und lokale Nachrichtenveröffentlichung an Lua. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet Modsettings, strukturiertes Modlogging, fokussierte Funktionstasten, dateibasierte UI-Aktionszustellung und lokale Nachrichtenveröffentlichung an Lua. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `set_settings Zeile12`<br>`create Zeile27`<br>`notifications Zeile126`<br>`is_game_key_down Zeile167`<br>`dispatch_ui_actions Zeile213`<br>`json_to_lua Zeile280` |
| 4. Aufrufer und Verbraucher | env/mod.rs register · lib.rs Runtimeaktualisierung und dispatch_ui_actions |
| 5. Verwendete Komponenten | Mod::setting_values · Lua-Namensregister · mod_loader::paths::ui_data_dir · mod_loader::news::write_event · Windows-Eingabe-/Fenster-API · tracing |
| 6. Eingaben und Ausgaben | Eingaben: Modsettings, Lua-Aktionen, Tastennamen und Nachrichtenpayloads · UI-actions-Dateien Ausgabe: shroudforge-Tabelle, Tastenzustand, Callbacks und Nachrichtenereignisse |
| 7. Seiteneffekte und Ressourcen | Logging · Windows-Tastatur-/Fokuslesung · liest und entfernt Aktionsdateien · schreibt lokale Newsevents · Lua-Registryupdates |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Settingscache, Logging, Windows-Eingabe, UI-Dateibrücke und lokale Nachrichten sind eigenständige Änderungsgründe in einer Datei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: ShroudForge-Lua-Vertrag. Technische Kopplung: Mod::setting_values · Lua-Namensregister · mod_loader::paths::ui_data_dir · mod_loader::news::write_event · Windows-Eingabe-/Fenster-API · tracing. Aufruferbezug: env/mod.rs register · lib.rs Runtimeaktualisierung und dispatch_ui_actions |
| 11. Duplikation | json_to_lua parallel env/registry.rs, hier absichtlich eingeschränkte Einstellungswertformen |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Registrykeys, Dateivertrag mit UI, Fokusprüfung und Liverefresh müssen gemeinsam erhalten bleiben. |
| 14. Vorhandener Testschutz | converts_json_arrays_to_lua_sequences |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Modbedienungsintegration. Neue Einordnung: ShroudForge-Lua-Vertrag. Besitzer: Modausführung. |

**Belege:** set_settings Zeile12 · create Zeile27 · notifications Zeile126 · is_game_key_down Zeile167 · dispatch_ui_actions Zeile213 · json_to_lua Zeile280

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/lua/shroudforge/v1/shroudforge.rs`.

### [src/loader/api/src/util/mod.rs](../../execution/util/mod.rs)

Vollständig gelesen, 7 Zeilen. SHA-256: `6b35b301048c80e3e1ec1a46f2af92445776c75bc5d6c647bf993ef3b1a1404e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert schreibgeschützte Lua-Container, Modmodulpfadparser und gekürzte Rust-Typnamen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert schreibgeschützte Lua-Container, Modmodulpfadparser und gekürzte Rust-Typnamen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Unterdateien Zeilen1–3`<br>`Reexporte Zeilen5–7` |
| 4. Aufrufer und Verbraucher | runner/state.rs · env/game/types.rs · env/game/assets/value/*.rs · lua/cast.rs |
| 5. Verwendete Komponenten | readonly · require_path · type_name |
| 6. Eingaben und Ausgaben | Eingaben: drei Lua-Hilfsmodule Ausgabe: gemeinsame Reexporte |
| 7. Seiteneffekte und Ressourcen | keine eigene Laufzeitaktion |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: readonly · require_path · type_name. Aufruferbezug: runner/state.rs · env/game/types.rs · env/game/assets/value/*.rs · lua/cast.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Alle Reexportnutzer aktualisieren. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gemeinsame Lua-Hilfsfunktionen. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** Unterdateien Zeilen1–3 · Reexporte Zeilen5–7

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/util/mod.rs`.

### [src/loader/api/src/util/readonly.rs](../../execution/util/readonly.rs)

Vollständig gelesen, 159 Zeilen. SHA-256: `ca1a1062651346ff7436381c0dcb45d148d491cba94463af165c57b6fa49bb2a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Implementiert unveränderliche Lua-Map- und Array-Userdata samt Länge, Indexierung und pairs-Iteratoren mit gehaltenen Besitzern. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Implementiert unveränderliche Lua-Map- und Array-Userdata samt Länge, Indexierung und pairs-Iteratoren mit gehaltenen Besitzern. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `ReadOnlyMap Zeile9`<br>`ReadOnlyMapIter Zeile44`<br>`ReadOnlyArray Zeile84`<br>`ReadOnlyArrayIter Zeile121` |
| 4. Aufrufer und Verbraucher | env/game/types.rs Typ-, Feld-, Enum-, Attribut- und Namespaceansichten |
| 5. Verwendete Komponenten | Rc · IndexMap · mlua::UserData · ouroboros |
| 6. Eingaben und Ausgaben | Eingaben: IndexMap oder Vec von Lua-Werten Ausgabe: schreibgeschützte Map-/Array-Userdata und Iteratoren |
| 7. Seiteneffekte und Ressourcen | Rc-Besitz und Lua-Userdata-Allokation |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: Rc · IndexMap · mlua::UserData · ouroboros. Aufruferbezug: env/game/types.rs Typ-, Feld-, Enum-, Attribut- und Namespaceansichten |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Iteratoren halten Datenbesitzer, Lebensdauervertrag erhalten. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Schreibgeschützte Lua-Container. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** ReadOnlyMap Zeile9 · ReadOnlyMapIter Zeile44 · ReadOnlyArray Zeile84 · ReadOnlyArrayIter Zeile121

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/util/readonly.rs`.

### [src/loader/api/src/util/require_path.rs](../../execution/util/require_path.rs)

Vollständig gelesen, 76 Zeilen. SHA-256: `4d43ae9ee388ce7afe62c614fa996e40a8d06681ff0079aa2888360c9def2fd1`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Zerlegt lokale und mods.ID.Modul-Modulnamen, ergänzt mod als Standardhauptmodul und erzeugt qualifizierte Cachekennungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Zerlegt lokale und mods.ID.Modul-Modulnamen, ergänzt mod als Standardhauptmodul und erzeugt qualifizierte Cachekennungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `RequirePath Zeile3`<br>`parse Zeile25`<br>`parse_qualified Zeile52`<br>`qualified_id Zeile73` |
| 4. Aufrufer und Verbraucher | runner/state.rs RunnerState::require und RunnerLocalState::require |
| 5. Verwendete Komponenten | LuaError::module_not_found |
| 6. Eingaben und Ausgaben | Eingaben: lokale oder qualifizierte Modulkennung und fallback Mod-ID Ausgabe: zerlegte Mod-ID/Pfad und kanonische Modulkennung |
| 7. Seiteneffekte und Ressourcen | keine externen Seiteneffekte |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: LuaError::module_not_found. Aufruferbezug: runner/state.rs RunnerState::require und RunnerLocalState::require |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Punkttrennung und Standardmodul mod sind require-Vertrag. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Modmodulauflösung. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** RequirePath Zeile3 · parse Zeile25 · parse_qualified Zeile52 · qualified_id Zeile73

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/util/require_path.rs`.

### [src/loader/api/src/util/type_name.rs](../../execution/util/type_name.rs)

Vollständig gelesen, 46 Zeilen. SHA-256: `91773d6fa9591cd0c056f44deb598edd619f79ea53a2539e1b0935cf14a37f7c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Kürzt Rust-Typnamen unter Erhalt generischer Struktur und zwischenspeichert sie nach TypeId für Lua-Fehlermeldungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Kürzt Rust-Typnamen unter Erhalt generischer Struktur und zwischenspeichert sie nach TypeId für Lua-Fehlermeldungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `CACHE Zeile5`<br>`short_type_name Zeile8`<br>`short_type_name_impl Zeile29` |
| 4. Aufrufer und Verbraucher | lua/cast.rs Typfehler |
| 5. Verwendete Komponenten | TypeId · LazyLock · parking_lot::RwLock |
| 6. Eingaben und Ausgaben | Eingaben: Rust-Typ T Ausgabe: verkürzter Typname |
| 7. Seiteneffekte und Ressourcen | globaler TypeId-Cache mit RwLock |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: TypeId · LazyLock · parking_lot::RwLock. Aufruferbezug: lua/cast.rs Typfehler |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Nur Diagnosen und Cacheidentität betroffen. |
| 14. Vorhandener Testschutz | nicht nachgewiesen |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lua-Diagnosehilfen. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** CACHE Zeile5 · short_type_name Zeile8 · short_type_name_impl Zeile29

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/util/type_name.rs`.

### [src/loader/api/tests/flight.lua](../../execution/tests/flight.lua)

Vollständig gelesen, 424 Zeilen. SHA-256: `940052beb17caff3434e286a614870572f1f1644b464f85c53fd1615c3c91c9a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Simuliert Spieler, Attribute, Gliderressourcen und Claims und prüft die reale Flugmod auf Kanonikauswahl, Steuergrenzen, Basiserhalt, Konflikte, Wiederherstellung und Weltwechsel. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Simuliert Spieler, Attribute, Gliderressourcen und Claims und prüft die reale Flugmod auf Kanonikauswahl, Steuergrenzen, Basiserhalt, Konflikte, Wiederherstellung und Weltwechsel. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `fixture Zeile26`<br>`kanonischer ItemRegistry-Test Zeile178`<br>`langes Updateintervall Zeile271`<br>`Baseline-/Restorefälle Zeile333`<br>`Sitzungsgrenze Zeile410` |
| 4. Aufrufer und Verbraucher | tests/flight.rs |
| 5. Verwendete Komponenten | flight_source · overrides_source · override_targets_factory · öffentlicher runtime/game-Vertrag |
| 6. Eingaben und Ausgaben | Eingaben: echte Flugmodquelle und synthetische Runtimewerte Ausgabe: Assertions über Einstellungen, Schreibzahlen und Lebenszykluszustände |
| 7. Seiteneffekte und Ressourcen | nur synthetische Lua-Modellwerte |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: flight_source · overrides_source · override_targets_factory · öffentlicher runtime/game-Vertrag. Aufruferbezug: tests/flight.rs |
| 11. Duplikation | Claim-/Trackerlebenszyklus und Statusfixture ähnlich survival.lua, Szenario unterscheidet sich |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Harnessglobals und echtes Modverhalten müssen gemeinsam erhalten bleiben. |
| 14. Vorhandener Testschutz | flight.rs unlimited_flight_uses_public_runtime_api_and_restores_owned_values |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Flugmod-Verhaltensfixture. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** fixture Zeile26 · kanonischer ItemRegistry-Test Zeile178 · langes Updateintervall Zeile271 · Baseline-/Restorefälle Zeile333 · Sitzungsgrenze Zeile410

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/flight.lua`.

### [src/loader/api/tests/flight.rs](../../execution/tests/flight.rs)

Vollständig gelesen, 33 Zeilen. SHA-256: `426e76a22e245145ac9d30c9c2d6b56704d95a47cbd4ad88ec32a3c4b2deb09e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt die tatsächliche Unlimited-Flight-Modquelle und Overrideadapter in eine isolierte Lua-VM und startet den Flug-Lebenszyklustest, prüft zusätzlich öffentliche API und fehlende Legacyabhängigkeiten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt die tatsächliche Unlimited-Flight-Modquelle und Overrideadapter in eine isolierte Lua-VM und startet den Flug-Lebenszyklustest, prüft zusätzlich öffentliche API und fehlende Legacyabhängigkeiten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `unlimited_flight_uses_public_runtime_api_and_restores_owned_values Zeile4`<br>`Quell-/Manifestassertionen Zeilen13–20`<br>`flight.lua Zeile30` |
| 4. Aufrufer und Verbraucher | Cargo-Integrationstest |
| 5. Verwendete Komponenten | mods/sf-unlimited-flight · overrides.lua · override_targets.lua · flight.lua · mlua |
| 6. Eingaben und Ausgaben | Eingaben: aktuelle Flugmodquelle und Manifest · Lua-Testfixture Ausgabe: Assertions über Modvertrag und Lebenszyklus |
| 7. Seiteneffekte und Ressourcen | liest Moddateien, führt isolierten Lua-Testcode aus |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: mods/sf-unlimited-flight · overrides.lua · override_targets.lua · flight.lua · mlua. Aufruferbezug: Cargo-Integrationstest |
| 11. Duplikation | Overridequell-/Targetfabrik-Aufbau parallel zu survival.rs |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Relative Mod- und include_str-Pfade müssen angepasst werden. |
| 14. Vorhandener Testschutz | definiert unlimited_flight_uses_public_runtime_api_and_restores_owned_values, nicht ausgeführt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Flugmod-Regressionsharness. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** unlimited_flight_uses_public_runtime_api_and_restores_owned_values Zeile4 · Quell-/Manifestassertionen Zeilen13–20 · flight.lua Zeile30

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/flight.rs`.

### [src/loader/api/tests/production_time.rs](../../execution/tests/production_time.rs)

Vollständig gelesen, 253 Zeilen. SHA-256: `f05112adf681899df0f38206ee81be0bfe7a43f55fecf7c6aa0a99f108cfe2da`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft die referenzierte Produktionszeitmod auf einzige Änderung positiver Rezeptdauer und Idempotenz, bietet zusätzlich ignorierte Snapshot- und echte isolierte KFC-Roundtriptests. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft die referenzierte Produktionszeitmod auf einzige Änderung positiver Rezeptdauer und Idempotenz, bietet zusätzlich ignorierte Snapshot- und echte isolierte KFC-Roundtriptests. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `check Zeile29`<br>`production_time_preserves_recipe_semantics_and_is_idempotent Zeile95`<br>`ignorierte Snapshottests Zeile108`<br>`production_time_real_kfc_roundtrip_on_isolated_copy Zeile123` |
| 4. Aufrufer und Verbraucher | Cargo-Integrationstests |
| 5. Verwendete Komponenten | reference-mods-0.9/sf-production-time · KFCParser · ShroudForgeApi · ModEnvironment · SF_RECIPE_SNAPSHOTS · SF_FACTORY_GAME_SOURCE |
| 6. Eingaben und Ausgaben | Eingaben: synthetische Rezeptdaten oder explizit bereitgestellte Spielkopie/Snapshots Ausgabe: Assertions über geänderte Dauer und unveränderte übrige Rezeptsemantik |
| 7. Seiteneffekte und Ressourcen | nur bei ignoriertem explizitem Roundtrip temporäre Spielkopien, Moddateien und echte Assetpipeline |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Synthetischer Lua-Vertragstest und optionaler echter KFC-/Modenvironment-Roundtrip mit Dateikopien liegen in einer Datei |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: reference-mods-0.9/sf-production-time · KFCParser · ShroudForgeApi · ModEnvironment · SF_RECIPE_SNAPSHOTS · SF_FACTORY_GAME_SOURCE. Aufruferbezug: Cargo-Integrationstests |
| 11. Duplikation | lokale JSON→Lua-Konvertierung parallel env/registry.rs mit weniger Datentypfällen |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Fixturepfade, Umgebungsvariablen und Ignoregrenzen erhalten. |
| 14. Vorhandener Testschutz | drei hier definierte Tests, zwei ignoriert, keiner ausgeführt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Assetvorbereitungs-Szenariotests. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** check Zeile29 · production_time_preserves_recipe_semantics_and_is_idempotent Zeile95 · ignorierte Snapshottests Zeile108 · production_time_real_kfc_roundtrip_on_isolated_copy Zeile123

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/production_time.rs`.

### [src/loader/api/tests/runtime_overrides.lua](../../execution/tests/runtime_overrides.lua)

Vollständig gelesen, 110 Zeilen. SHA-256: `fcad9f32640262bbbe495dd84548a9be674acb3d04e10af89be4b403f77601af`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft Override-Tracker auf Stopp nach ungewissen oder unerwarteten Ergebnissen, vollständigen Besitzerhalt, false/zero-Baselines und Claimfreigabe nach Berechnungsfehlern. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft Override-Tracker auf Stopp nach ungewissen oder unerwarteten Ergebnissen, vollständigen Besitzerhalt, false/zero-Baselines und Claimfreigabe nach Berechnungsfehlern. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `fixture Zeile6`<br>`Uncertainty-Fälle Zeilen29–54`<br>`Restorefehler Zeilen57–66`<br>`false/zero Zeilen69–77`<br>`No-change Zeilen99–110` |
| 4. Aufrufer und Verbraucher | tests/runtime_overrides.rs |
| 5. Verwendete Komponenten | overrides.new · synthetische Lese-/Schreibadapter und Claims |
| 6. Eingaben und Ausgaben | Eingaben: kontrollierte Adapterergebnisse und Fehler Ausgabe: Assertions über Fault, Ownership und Schreibstopp |
| 7. Seiteneffekte und Ressourcen | synthetische Modellwerte |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: overrides.new · synthetische Lese-/Schreibadapter und Claims. Aufruferbezug: tests/runtime_overrides.rs |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Rückgabesemantik der Adapter und Tracker identisch halten. |
| 14. Vorhandener Testschutz | runtime_overrides.rs reversible_overrides_stop_all_writes_after_uncertain_results |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Override-Fehlerverhaltenstests. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** fixture Zeile6 · Uncertainty-Fälle Zeilen29–54 · Restorefehler Zeilen57–66 · false/zero Zeilen69–77 · No-change Zeilen99–110

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/runtime_overrides.lua`.

### [src/loader/api/tests/runtime_overrides.rs](../../execution/tests/runtime_overrides.rs)

Vollständig gelesen, 14 Zeilen. SHA-256: `d77110aa3ae654fdf018ccd41ac4d50a84507d5529b45b3f95edb87df5d8f7ea`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt die produktive Override-Luaimplementierung und führt isolierte Fehlerrückfalltests gegen ungewisse Schreibresultate aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt die produktive Override-Luaimplementierung und führt isolierte Fehlerrückfalltests gegen ungewisse Schreibresultate aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reversible_overrides_stop_all_writes_after_uncertain_results Zeile4` |
| 4. Aufrufer und Verbraucher | Cargo-Integrationstest |
| 5. Verwendete Komponenten | overrides.lua · runtime_overrides.lua · mlua |
| 6. Eingaben und Ausgaben | Eingaben: produktiver Overridecode und Lua-Fixture Ausgabe: Testergebnis |
| 7. Seiteneffekte und Ressourcen | isolierte Lua-Ausführung |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: overrides.lua · runtime_overrides.lua · mlua. Aufruferbezug: Cargo-Integrationstest |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. include_str-Pfade aktualisieren. |
| 14. Vorhandener Testschutz | definiert reversible_overrides_stop_all_writes_after_uncertain_results, nicht ausgeführt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Override-Regressionsharness. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** reversible_overrides_stop_all_writes_after_uncertain_results Zeile4

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/runtime_overrides.rs`.

### [src/loader/api/tests/survival.lua](../../execution/tests/survival.lua)

Vollständig gelesen, 283 Zeilen. SHA-256: `7c49572a897354a0b38ce38bdced16beb6b53c991c112b164208d6e64097c3eb`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Simuliert Spielerattribute, Falldamage-Templates und GameSettings und prüft reale Survivalmod-Einstellungen auf Basiserhalt, Normalisierung, Konflikterhalt, Busycleanup, Fault und Weltwechsel. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Simuliert Spielerattribute, Falldamage-Templates und GameSettings und prüft reale Survivalmod-Einstellungen auf Basiserhalt, Normalisierung, Konflikterhalt, Busycleanup, Fault und Weltwechsel. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `fixture Zeile7`<br>`alle Einstellungen Zeile156`<br>`Busy-/Konfliktfälle Zeile198`<br>`Floatnormalisierung Zeile234`<br>`Cleanup Zeile250`<br>`Uncertainty und Sitzung Zeilen267–283` |
| 4. Aufrufer und Verbraucher | tests/survival.rs |
| 5. Verwendete Komponenten | survival_source · overrides_source · override_targets_factory · öffentliche ECS-/Service-/Templateverträge |
| 6. Eingaben und Ausgaben | Eingaben: reale Modquelle und synthetische Attribute/Templates/Services Ausgabe: Assertions über Werte, Writes und Besitzrestaurierung |
| 7. Seiteneffekte und Ressourcen | nur synthetische Lua-Modellwerte |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: survival_source · overrides_source · override_targets_factory · öffentliche ECS-/Service-/Templateverträge. Aufruferbezug: tests/survival.rs |
| 11. Duplikation | Fixture für Claims und Trackercleanup parallel zu flight.lua mit anderen fachlichen Inputs |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Native Berechnungen werden hier lediglich simuliert, Grenzen im Bericht erhalten. |
| 14. Vorhandener Testschutz | survival.rs survival_uses_public_api_and_restores_owned_inputs |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Survivalmod-Verhaltensfixture. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** fixture Zeile7 · alle Einstellungen Zeile156 · Busy-/Konfliktfälle Zeile198 · Floatnormalisierung Zeile234 · Cleanup Zeile250 · Uncertainty und Sitzung Zeilen267–283

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/survival.lua`.

### [src/loader/api/tests/survival.rs](../../execution/tests/survival.rs)

Vollständig gelesen, 24 Zeilen. SHA-256: `b644c73b2d99b2170249b64a31ac3aa0e4d4a362deba08b133f0bad781f77e3e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt die reale Survival-Modquelle mit produktiver Overrideimplementierung und Targetfabrik in eine isolierte Lua-Regressionsfixture. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt die reale Survival-Modquelle mit produktiver Overrideimplementierung und Targetfabrik in eine isolierte Lua-Regressionsfixture. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `survival_uses_public_api_and_restores_owned_inputs Zeile4` |
| 4. Aufrufer und Verbraucher | Cargo-Integrationstest |
| 5. Verwendete Komponenten | mods/sf-survival · overrides.lua · override_targets.lua · survival.lua |
| 6. Eingaben und Ausgaben | Eingaben: Survivalmodquelle und Lua-Fixture Ausgabe: Szenariotestergebnis |
| 7. Seiteneffekte und Ressourcen | Modquelldateilesen und isolierte Lua-Ausführung |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: mods/sf-survival · overrides.lua · override_targets.lua · survival.lua. Aufruferbezug: Cargo-Integrationstest |
| 11. Duplikation | Override-/Targetfactory-Harness parallel flight.rs |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Relative Modquelle und eingebettete Luaquellen mitziehen. |
| 14. Vorhandener Testschutz | definiert survival_uses_public_api_and_restores_owned_inputs, nicht ausgeführt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Survivalmod-Regressionsharness. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** survival_uses_public_api_and_restores_owned_inputs Zeile4

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/survival.rs`.

### [src/loader/api/tests/world_editor_p2p.lua](../../execution/tests/world_editor_p2p.lua)

Vollständig gelesen, 108 Zeilen. SHA-256: `59db1875da1282ca9039cf64a0e2945f642173d330bec37b47866faf7c9a36e2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft P2P-Blueprintübertragung auf Chunkzusammenbau, Requestdeduplizierung, Undo, unbekannten Timeoutausgang, Peer-/Modidentität, Prüfsumme und Größenlimit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft P2P-Blueprintübertragung auf Chunkzusammenbau, Requestdeduplizierung, Undo, unbekannten Timeoutausgang, Peer-/Modidentität, Prüfsumme und Größenlimit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `network Zeile13`<br>`Chunkfall Zeile52`<br>`Replay Zeile67`<br>`Timeout Zeile85`<br>`Autorisierung Zeile93`<br>`Größenlimit Zeile106` |
| 4. Aufrufer und Verbraucher | world_editor_p2p.rs world_editor_p2p_transport |
| 5. Verwendete Komponenten | p2p_factory · simulierte Netzwerkqueues · Blueprint-V7-Text |
| 6. Eingaben und Ausgaben | Eingaben: synthetische Steamnachrichten und Blueprinttext Ausgabe: Assertions zu übertragenem Inhalt und Callbackanzahl |
| 7. Seiteneffekte und Ressourcen | In-Memory-Netzwerksimulation, kein Steamzugriff |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: p2p_factory · simulierte Netzwerkqueues · Blueprint-V7-Text. Aufruferbezug: world_editor_p2p.rs world_editor_p2p_transport |
| 11. Duplikation | Queue-/Endpointsimulation ähnlich world_editor_server.lua |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Niedrig. Harnessfactory und Nachrichtenvertrag erhalten. |
| 14. Vorhandener Testschutz | world_editor_p2p.rs world_editor_p2p_transport |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Welteditor-Transporttests. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** network Zeile13 · Chunkfall Zeile52 · Replay Zeile67 · Timeout Zeile85 · Autorisierung Zeile93 · Größenlimit Zeile106

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/world_editor_p2p.lua`.

### [src/loader/api/tests/world_editor_p2p.rs](../../execution/tests/world_editor_p2p.rs)

Vollständig gelesen, 70 Zeilen. SHA-256: `c390aa53f652ec895e0892818710fed194581e4d712d3e22435a737b9d9eae57`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Startet vier Lua-Integrationstests für Worldeditor-P2P, serverseitige Weltänderung, gebündelte Lua-Syntax und Blueprint-Speicherfortschritt mit realen Referenzmodquellen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Startet vier Lua-Integrationstests für Worldeditor-P2P, serverseitige Weltänderung, gebündelte Lua-Syntax und Blueprint-Speicherfortschritt mit realen Referenzmodquellen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `world_editor_p2p_transport Zeile4`<br>`server_target_applies_and_undoes_through_native_world_api Zeile22`<br>`bundled_lua_compiles Zeile43`<br>`blueprint_save_progress_confirms_write_and_library Zeile61` |
| 4. Aufrufer und Verbraucher | Cargo-Integrationstests |
| 5. Verwendete Komponenten | reference-mods-0.9/world-editor · world_editor_p2p.lua · world_editor_server.lua · world_editor_save_progress.lua · mods/**/*.lua · mlua |
| 6. Eingaben und Ausgaben | Eingaben: Referenzmodquellen und drei Lua-Fixtures Ausgabe: Transport-/Welt-/Speicher- und Syntaxassertionen |
| 7. Seiteneffekte und Ressourcen | liest Moddateien und führt isolierte Lua-Fixtures aus |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Welteditor-Szenarien und repoübergreifende Syntaxprüfung aller gebündelten Mods teilen den Harness |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: reference-mods-0.9/world-editor · world_editor_p2p.lua · world_editor_server.lua · world_editor_save_progress.lua · mods/**/*.lua · mlua. Aufruferbezug: Cargo-Integrationstests |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Relative Referenzmod-/Modpfade und unsicherer Debug-Test-VM müssen auf Fixtures begrenzt bleiben. |
| 14. Vorhandener Testschutz | vier hier definierte Tests, nicht ausgeführt |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Welteditormod-Regressionsharness. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** world_editor_p2p_transport Zeile4 · server_target_applies_and_undoes_through_native_world_api Zeile22 · bundled_lua_compiles Zeile43 · blueprint_save_progress_confirms_write_and_library Zeile61

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/world_editor_p2p.rs`.

### [src/loader/api/tests/world_editor_save_progress.lua](../../execution/tests/world_editor_save_progress.lua)

Vollständig gelesen, 211 Zeilen. SHA-256: `79862e73ce308ddd7795dd35867804b419b61c53bb5050c74287459c50de797e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft Worldeditor-Speicherfortschritt und Fortsetzungen einschließlich Datei-/Bibliotheksfehler, Querywarten, Cancel, Weltwechselquarantäne, Routing und Undopriorität durch gezielte Debug-Upvalueseams. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft Worldeditor-Speicherfortschritt und Fortsetzungen einschließlich Datei-/Bibliotheksfehler, Querywarten, Cancel, Weltwechselquarantäne, Routing und Undopriorität durch gezielte Debug-Upvalueseams. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `Exportassertionen Zeilen16–56`<br>`upvalue Zeile59`<br>`Capturefortsetzung Zeile80`<br>`Weltwechsel Zeile149`<br>`Backendrouting Zeile165`<br>`Undopriorität Zeile187` |
| 4. Aufrufer und Verbraucher | world_editor_p2p.rs blueprint_save_progress_confirms_write_and_library |
| 5. Verwendete Komponenten | editor_source · Lua-debug-Upvalues · simulierte UI/Export/Runtime-API |
| 6. Eingaben und Ausgaben | Eingaben: reale Editorquelle und injizierte Fehler-/Sitzungszustände Ausgabe: Assertions über Fortschritt, Journale und Callbacks |
| 7. Seiteneffekte und Ressourcen | ersetzt Test-Upvalues und schreibt synthetische Exportzustände |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: editor_source · Lua-debug-Upvalues · simulierte UI/Export/Runtime-API. Aufruferbezug: world_editor_p2p.rs blueprint_save_progress_confirms_write_and_library |
| 11. Duplikation | keine belegte |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Hoch. Tests hängen ausdrücklich an Namen privater Lua-Upvalues, strukturelle Änderungen brauchen abgestimmte Seams. |
| 14. Vorhandener Testschutz | world_editor_p2p.rs blueprint_save_progress_confirms_write_and_library |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Welteditor-Fortschritt- und Lebenszyklustests. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** Exportassertionen Zeilen16–56 · upvalue Zeile59 · Capturefortsetzung Zeile80 · Weltwechsel Zeile149 · Backendrouting Zeile165 · Undopriorität Zeile187

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/world_editor_save_progress.lua`.

### [src/loader/api/tests/world_editor_server.lua](../../execution/tests/world_editor_server.lua)

Vollständig gelesen, 108 Zeilen. SHA-256: `b54522601e8df5bf2ffb104c495efd53ebb48283ce317f45a2a24be3ec566b12`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Führt die reale Worldeditormod mit simuliertem Servernetz und Voxel-API aus und prüft Paste, serverseitiges Undojournal und erneutes Paste nach erfolgreichem Undo. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Führt die reale Worldeditormod mit simuliertem Servernetz und Voxel-API aus und prüft Paste, serverseitiges Undojournal und erneutes Paste nach erfolgreichem Undo. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `server_runtime Zeile38`<br>`server_env Zeile61`<br>`Paste Zeile76`<br>`Undo Zeile88`<br>`zweites Paste Zeile99` |
| 4. Aufrufer und Verbraucher | world_editor_p2p.rs server_target_applies_and_undoes_through_native_world_api |
| 5. Verwendete Komponenten | editor_source · p2p_factory · simulierte Network-/World-API |
| 6. Eingaben und Ausgaben | Eingaben: reale Editorquelle und synthetischer Voxel-/Netzzustand Ausgabe: Assertions zu Weltwerten und Servertoken |
| 7. Seiteneffekte und Ressourcen | mutiert ausschließlich simulierte Voxel- und Netzwerkwerte |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: keine belegte |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modausführung und Lua-Verträge. Technische Kopplung: editor_source · p2p_factory · simulierte Network-/World-API. Aufruferbezug: world_editor_p2p.rs server_target_applies_and_undoes_through_native_world_api |
| 11. Duplikation | Queue-/Endpointaufbau parallel world_editor_p2p.lua |
| 12. Nutzungsstatus | belegt |
| 13. Risiko bei Änderung oder Verschiebung | Mittel. Echte Modquelle und Simulation müssen zusammenbleiben. |
| 14. Vorhandener Testschutz | world_editor_p2p.rs server_target_applies_and_undoes_through_native_world_api |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Welteditor-Serverablauftest. Neue Einordnung: Modausführung und Lua-Verträge. Besitzer: Modausführung. |

**Belege:** server_runtime Zeile38 · server_env Zeile61 · Paste Zeile76 · Undo Zeile88 · zweites Paste Zeile99

**Evidenzgrenze:** nachgewiesen

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `execution/tests/world_editor_server.lua`.

## Eingebaute Produktfunktionen und Oberflächen

29 Dateien, 13.629 Zeilen.

### [src/loader/modules/commands/src/lib.rs](../../modules/commands/lib.rs)

Vollständig gelesen, 4 Zeilen. SHA-256: `e74333d274a33f4f843eea4b464bd0151ba3e92615cc143cd07550c0bcad4c16`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt den Commands-Einstieg aus main.rs als Bibliothek für den Loader-Dispatcher bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt den Commands-Einstieg aus main.rs als Bibliothek für den Loader-Dispatcher bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:1-4 bindet main.rs ein und exportiert run_module.` |
| 4. Aufrufer und Verbraucher | workflow/main.rs:86 ruft shroudforge_commands::run_module auf. |
| 5. Verwendete Komponenten | main.rs |
| 6. Eingaben und Ausgaben | Eingaben: Kein Laufzeitinput Ausgabe: öffentlicher run_module-Export |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: main.rs. Aufruferbezug: workflow/main.rs:86 ruft shroudforge_commands::run_module auf. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | #[path] und Reexport müssen beim Verschieben zusammen angepasst werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modulaufruf und Einstieg. Neue Einordnung: Eingebaute Module. Besitzer: commands. |

**Belege:** lib.rs:1-4 bindet main.rs ein und exportiert run_module.

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/commands/lib.rs`.

### [src/loader/modules/commands/src/main.rs](../../modules/commands/main.rs)

Vollständig gelesen, 45 Zeilen. SHA-256: `f0fc591e297059efc9fa62711f8eb5eac7e66d54e303420a1deafa288eeb0c41`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Meldet ausschließlich die Verfügbarkeit des Commands-Moduls als JSON bei --status oder als Logeintrag. Es implementiert keine eigentlichen Spielbefehle. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Meldet ausschließlich die Verfügbarkeit des Commands-Moduls als JSON bei --status oder als Logeintrag. Es implementiert keine eigentlichen Spielbefehle. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.rs:13 run_module`<br>`main.rs:21 --status`<br>`main.rs:39 module_root` |
| 4. Aufrufer und Verbraucher | commands/src/lib.rs:1-4 · workflow/main.rs:86 |
| 5. Verwendete Komponenten | serde · serde_json · shroudforge_package::logging · Prozessargumente |
| 6. Eingaben und Ausgaben | Eingaben: --status · --root oder aktuelles Arbeitsverzeichnis Ausgabe: JSON ModuleStatus oder Verfügbarkeitslog |
| 7. Seiteneffekte und Ressourcen | stdout beziehungsweise Loaderlog |
| 8. Art der Verantwortung | Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: serde · serde_json · shroudforge_package::logging · Prozessargumente. Aufruferbezug: commands/src/lib.rs:1-4 · workflow/main.rs:86 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Status-/CLI-Vertrag und Rootauflösung erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Befehlszugang und Verfügbarkeitsmeldung. Neue Einordnung: Eingebaute Module. Besitzer: commands. |

**Belege:** main.rs:13 run_module · main.rs:21 --status · main.rs:39 module_root

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/commands/main.rs`.

### [src/loader/modules/debug-console/src/lib.rs](../../modules/debug-console/lib.rs)

Vollständig gelesen, 4 Zeilen. SHA-256: `e74333d274a33f4f843eea4b464bd0151ba3e92615cc143cd07550c0bcad4c16`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt den Debugkonsolen-Einstieg aus main.rs als Bibliothek bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt den Debugkonsolen-Einstieg aus main.rs als Bibliothek bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:1-4` |
| 4. Aufrufer und Verbraucher | workflow/main.rs:17 |
| 5. Verwendete Komponenten | main.rs |
| 6. Eingaben und Ausgaben | Eingaben: Kein Laufzeitinput Ausgabe: öffentlicher run_module-Export |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: main.rs. Aufruferbezug: workflow/main.rs:17 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | #[path] und Reexport erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modulaufruf und Einstieg. Neue Einordnung: Eingebaute Module. Besitzer: debug-console. |

**Belege:** lib.rs:1-4

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/debug-console/lib.rs`.

### [src/loader/modules/debug-console/src/main.rs](../../modules/debug-console/log_tail.rs)

Vollständig gelesen, 614 Zeilen. SHA-256: `c7e3d0107011e0f6572b75a69a25768214f1913f7f12f428d462b9c82dbc9a26`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Betreibt die Windows-Debugkonsole als WebView, liest begrenzte Logtails, reagiert auf F10/Fensterzustand/IPC, speichert Präferenzen und öffnet sich bei Loaderfehlern oder abnormalem Spielende zur Fehleranzeige. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Betreibt die Windows-Debugkonsole als WebView, liest begrenzte Logtails, reagiert auf F10/Fensterzustand/IPC, speichert Präferenzen und öffnet sich bei Loaderfehlern oder abnormalem Spielende zur Fehleranzeige. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.rs:80 run`<br>`main.rs:129-147 HTML-Einbettung`<br>`main.rs:172 Ereignisschleife`<br>`main.rs:267 Präferenzen schreiben`<br>`main.rs:401 open_for_alert`<br>`main.rs:443 arguments`<br>`main.rs:517 scan_log_for_alert`<br>`main.rs:551 read_tail` |
| 4. Aufrufer und Verbraucher | debug-console/src/lib.rs:1-4 · workflow/main.rs:17 · modloader-ui/src/main.rs:1694 startet --debug-console |
| 5. Verwendete Komponenten | tao/wry · Windows Prozess-, Event- und Tastatur-APIs · shroudforge_package::config, paths, logging, window_icon · ui/app.ts, index.html, styles.css und ui-shared/tokens.css |
| 6. Eingaben und Ausgaben | Eingaben: Loaderkonfiguration · Root, Spiel-/Owner-PID und Windows-Events · UI-IPC und Logdateien Ausgabe: Logtail-Snapshots · Fenster-/Fehleranzeige |
| 7. Seiteneffekte und Ressourcen | Fenster/WebView · Präferenz-/Fensterzustand und Logs schreiben · Prozess-/Eventhandles lesen |
| 8. Art der Verantwortung | Präsentationslogik und Windows-/Dateiintegration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Logbeobachtung bleibt Modulimplementierung. Fenster und UI-Ereignisse liegen im eigenen UI-Teil desselben Moduls. Befunde: Fensterhost, Prozessbeobachtung, Logzugriff und Präferenzen dienen derselben Diagnoseoberfläche. Keine unabhängige fachliche Vermischung belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: tao/wry · Windows Prozess-, Event- und Tastatur-APIs · shroudforge_package::config, paths, logging, window_icon · ui/app.ts, index.html, styles.css und ui-shared/tokens.css. Aufruferbezug: debug-console/src/lib.rs:1-4 · workflow/main.rs:17 · modloader-ui/src/main.rs:1694 startet --debug-console |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | include_str-Pfade, CLI/IPC, Prozesslebensdauer, Warnungsöffnung und Profiltrennung erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Protokollanzeige und Startfehlerbeobachtung. Neue Einordnung: Eingebaute Module. Besitzer: debug-console. |

**Belege:** main.rs:80 run · main.rs:129-147 HTML-Einbettung · main.rs:172 Ereignisschleife · main.rs:267 Präferenzen schreiben · main.rs:401 open_for_alert · main.rs:443 arguments · main.rs:517 scan_log_for_alert · main.rs:551 read_tail

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** aufteilen. Logbeobachtung bleibt Modulimplementierung. Fenster und UI-Ereignisse liegen im eigenen UI-Teil desselben Moduls.

**Vorgeschlagene Zielpfade:** `modules/debug-console/log_tail.rs`, `modules/debug-console/startup_alerts.rs`, `modules/debug-console/ui/window.rs`.

### [src/loader/modules/debug-console/ui/app.ts](../../modules/debug-console/ui/app.ts)

Vollständig gelesen, 159 Zeilen. SHA-256: `61f6bcca43d4378090145d8a469f4c0b370c1310a79e3588a9e176783c41ed77`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Rendert die beiden vom Host gelieferten Logtails mit Suche, Schweregrad, Pause, automatischem Scrollen und Kopieren. Sendet Anzeigepräferenzen sowie Hide- und Drag-Aktionen über IPC an den Host. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Rendert die beiden vom Host gelieferten Logtails mit Suche, Schweregrad, Pause, automatischem Scrollen und Kopieren. Sendet Anzeigepräferenzen sowie Hide- und Drag-Aktionen über IPC an den Host. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `app.ts:18 classify`<br>`app.ts:36 render`<br>`app.ts:72 __shroudforgeUpdate`<br>`app.ts:104-157 Ereignisbehandlung` |
| 4. Aufrufer und Verbraucher | debug-console/src/main.rs:129-136 bettet HTML, Styles und app.ts ein. |
| 5. Verwendete Komponenten | DOM · window.ipc · navigator.clipboard |
| 6. Eingaben und Ausgaben | Eingaben: Host-Logtails und Präferenzen · DOM-Benutzereingaben Ausgabe: DOM-Loganzeige · Präferenz-/Hide-/Drag-IPC |
| 7. Seiteneffekte und Ressourcen | Zwischenablage schreiben |
| 8. Art der Verantwortung | Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: DOM · window.ipc · navigator.clipboard. Aufruferbezug: debug-console/src/main.rs:129-136 bettet HTML, Styles und app.ts ein. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Callback-/Elementnamen und IPC-Verträge zusammen mit Host ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Protokollanzeige und Interaktion. Neue Einordnung: Eingebaute Module. Besitzer: debug-console. |

**Belege:** app.ts:18 classify · app.ts:36 render · app.ts:72 __shroudforgeUpdate · app.ts:104-157 Ereignisbehandlung

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/debug-console/ui/app.ts`.

### [src/loader/modules/debug-console/ui/index.html](../../modules/debug-console/ui/index.html)

Vollständig gelesen, 48 Zeilen. SHA-256: `54c3b690c9f9b70fcfbba47ae3b5960cf13bee854fca95752fa1fd400f3e25e5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert das Debugkonsolen-Fenster mit Logquellen, Suche, Filter, Kopieren, Pause und Scrollsteuerung sowie Platzhaltern für die eingebetteten Styles und Skripte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert das Debugkonsolen-Fenster mit Logquellen, Suche, Filter, Kopieren, Pause und Scrollsteuerung sowie Platzhaltern für die eingebetteten Styles und Skripte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `index.html:7 Style-Platzhalter`<br>`index.html:23-44 Bedienoberfläche`<br>`index.html:46 Script-Platzhalter` |
| 4. Aufrufer und Verbraucher | debug-console/src/main.rs:129-136 bettet HTML, Styles und app.ts ein. |
| 5. Verwendete Komponenten | __SHROUDFORGE_STYLES__ · __SHROUDFORGE_SCRIPT__ |
| 6. Eingaben und Ausgaben | Eingaben: eingebettete Styles und app.ts Ausgabe: DOM der Debugkonsole |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: __SHROUDFORGE_STYLES__ · __SHROUDFORGE_SCRIPT__. Aufruferbezug: debug-console/src/main.rs:129-136 bettet HTML, Styles und app.ts ein. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Platzhalter und DOM-IDs mit app.ts/Host gekoppelt. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Protokollansicht. Neue Einordnung: Eingebaute Module. Besitzer: debug-console. |

**Belege:** index.html:7 Style-Platzhalter · index.html:23-44 Bedienoberfläche · index.html:46 Script-Platzhalter

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/debug-console/ui/index.html`.

### [src/loader/modules/debug-console/ui/styles.css](../../modules/debug-console/ui/styles.css)

Vollständig gelesen, 4 Zeilen. SHA-256: `3c3ac312e0131c17d37b5865a9efc09a244e1e5c11dbfa3a1b6f0a854b06a157`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Gestaltet ausschließlich die Debugkonsole, ihre Titelzeile, Logfilter, Schweregradfarben und Logquell-Tabs mit den gemeinsamen sf-Designvariablen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Gestaltet ausschließlich die Debugkonsole, ihre Titelzeile, Logfilter, Schweregradfarben und Logquell-Tabs mit den gemeinsamen sf-Designvariablen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `styles.css:1-4 vollständige Layout- und Zustandsregeln` |
| 4. Aufrufer und Verbraucher | debug-console/src/main.rs:129-136 bettet HTML, Styles und app.ts ein. |
| 5. Verwendete Komponenten | ui-shared/tokens.css Variablen · debug-console/ui/index.html Klassen |
| 6. Eingaben und Ausgaben | Eingaben: DOM-Klassen · sf-CSS-Variablen Ausgabe: Debugkonsolenlayout |
| 7. Seiteneffekte und Ressourcen | Keine über Darstellung hinaus |
| 8. Art der Verantwortung | Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: ui-shared/tokens.css Variablen · debug-console/ui/index.html Klassen. Aufruferbezug: debug-console/src/main.rs:129-136 bettet HTML, Styles und app.ts ein. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Einbettung und gemeinsame Tokenreferenz erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gestaltung der Protokollansicht. Neue Einordnung: Eingebaute Module. Besitzer: debug-console. |

**Belege:** styles.css:1-4 vollständige Layout- und Zustandsregeln

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/debug-console/ui/styles.css`.

### [src/loader/modules/modloader-ui/src/lib.rs](../../modules/modloader/lib.rs)

Vollständig gelesen, 4 Zeilen. SHA-256: `acf62598245d8c042d6e183cdc93a154b9b344c803250fb76c329458cfc6b234`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert den Start der Modverwaltung sowie den separaten Katalog-Installationsworker für den Loader-Dispatcher. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert den Start der Modverwaltung sowie den separaten Katalog-Installationsworker für den Loader-Dispatcher. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:1-4` |
| 4. Aufrufer und Verbraucher | workflow/main.rs:11-13 |
| 5. Verwendete Komponenten | main.rs |
| 6. Eingaben und Ausgaben | Eingaben: Kein Laufzeitinput Ausgabe: run_module und run_catalog_install_worker |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: main.rs. Aufruferbezug: workflow/main.rs:11-13 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Beide öffentlichen Loader-Einstiege erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modulaufruf und Einstieg. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** lib.rs:1-4

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/lib.rs`.

### [src/loader/modules/modloader-ui/src/main.rs](../../modules/modloader/ui/host/window.rs)

Vollständig gelesen, 5951 Zeilen. SHA-256: `0ffb23982894d99f36afe64cb09f8f4819da9398fcf3484bee3f25416a5f2f43`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Betreibt Modverwaltung und Updaterfenster. Verarbeitet UI- und Control-Aufträge, erstellt Gesamtsnapshots, liest Mods/Abhängigkeiten/Icons, schreibt enabled und Einstellungen samt erstmaliger EML-Erweiterung und verwaltet Nachrichten/Logs/Konfiguration. Sucht im ShroudEdit-Katalog, lädt/verifiziert/entpackt Mods und ersetzt sie mit Runtime-Unload/Reload und Rollback. Prüft GitHub-Systemupdates und startet Updater-Aufträge. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Betreibt Modverwaltung und Updaterfenster. Verarbeitet UI- und Control-Aufträge, erstellt Gesamtsnapshots, liest Mods/Abhängigkeiten/Icons, schreibt enabled und Einstellungen samt erstmaliger EML-Erweiterung und verwaltet Nachrichten/Logs/Konfiguration. Sucht im ShroudEdit-Katalog, lädt/verifiziert/entpackt Mods und ersetzt sie mit Runtime-Unload/Reload und Rollback. Prüft GitHub-Systemupdates und startet Updater-Aufträge. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.rs:565 parse_ui_command`<br>`main.rs:738 run_with_arguments`<br>`main.rs:2301 snapshot`<br>`main.rs:2551 read_mods`<br>`main.rs:2765 read_package_file`<br>`main.rs:3013 read_notifications`<br>`main.rs:3370 save_mod_settings`<br>`main.rs:3399 set_mod_enabled`<br>`main.rs:3440 update_mod_extension`<br>`main.rs:3509 write_package_json`<br>`main.rs:3523 write_zip_entry`<br>`main.rs:3626 mod_ecosystem_for_package`<br>`main.rs:3668 claim_control_actions`<br>`main.rs:3908 save_setting`<br>`main.rs:4415 fetch_catalog`<br>`main.rs:4578 run_catalog_install_worker`<br>`main.rs:4655 install_mod_from_catalog`<br>`main.rs:4817 download_catalog_version`<br>`main.rs:4972 replace_catalog_mod`<br>`main.rs:5240 start_update_check`<br>`main.rs:5600 fetch_latest_github`<br>`main.rs:5820 extract_zip` |
| 4. Aufrufer und Verbraucher | modloader-ui/src/lib.rs:1-4 · workflow/main.rs:11-13 · updater/src/main.rs:2526-2529 und :2697-2700 starten --catalog-install-worker · updater/src/main.rs:1079-1085 startet --module-ui --updater-window |
| 5. Verwendete Komponenten | shroudforge_package::config, paths, news, backups, status, update_check, logging, window_icon · shroudforge_updater · shroudforge_runtime_diagnostics · username.rs · tao/wry · Windows COM/Shell/Prozess/Tastatur · reqwest · serde/serde_json · semver · sha2/base64 · zip · ui/dist/index.html |
| 6. Eingaben und Ausgaben | Eingaben: UI-/Control-Aufträge · bestehende Loaderkonfiguration · entpackte Modverzeichnisse mit mod.json und optionaler extended.mod.json · Katalog-/Release-HTTP-Antworten · Queue-/Runtime-/Nachrichtenzustände Ausgabe: Gesamtsnapshot und Befehlsresultate · aktualisierte extended.mod.json und Loaderkonfiguration · installierte Modverzeichnisse · Updater-/Runtime-Aufträge |
| 7. Seiteneffekte und Ressourcen | Dateien schreiben/verschieben · Netzwerk · Fenster/Prozesse · Logs/Nachrichten/Registry schreiben |
| 8. Art der Verantwortung | Geschäfts-, Integrations-, Infrastruktur- und Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. UI sendet Aufträge. Settings/EML bleiben Paketverantwortung, Modinstallation und Releasewahl werden von UI und headless gemeinsam aufgerufen. Archiv-I/O ist kein Runtime-Modmodell. Befunde: Belegt sind mehrere fachlich trennbare Aufgaben in einer Datei: Fenster/IPC und Snapshot, Modverzeichniszugriff und EML-Kompatibilität, Loaderkonfiguration/Nachrichten, ShroudEdit-Installation/Runtime-Rollback und GitHub-Systemupdateprüfung. · update_mod_extension ist für entpackte Mods erforderlich. save_mod_settings:3379 und set_mod_enabled:3400 verwenden ihn. Er erzeugt fehlende extended.mod.json mit launcher EML:3458-3460, validiert Erweiterung und kombiniertes Manifest:3489-3496 und schreibt über den Verzeichniszweig write_package_json:3515-3518. · Zusätzliche Archiv-Modzweige sind read_package_file:2776ff., package_has_file:3504ff. und write_zip_entry:3523ff. Sie sind vom laufenden Verzeichnisbetrieb abzugrenzen. Einmaliger Katalogdownload mit ZIP-Prüfung und Entpackung:4857-4968 und :5820ff. gehört zur Installation. · Keine neue Konfigurationsdatei oder Schemamigration für die Sortierung begründet. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: shroudforge_package::config, paths, news, backups, status, update_check, logging, window_icon · shroudforge_updater · shroudforge_runtime_diagnostics · username.rs · tao/wry · Windows COM/Shell/Prozess/Tastatur · reqwest · serde/serde_json · semver · sha2/base64 · zip · ui/dist/index.html. Aufruferbezug: modloader-ui/src/lib.rs:1-4 · workflow/main.rs:11-13 · updater/src/main.rs:2526-2529 und :2697-2700 starten --catalog-install-worker · updater/src/main.rs:1079-1085 startet --module-ui --updater-window |
| 11. Duplikation | Desktop Releaseprüfung:5240ff./5600ff. und Updater headless_check_system_updates:1720ff. führen denselben Releaseablauf separat aus. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | EML ohne extension, enabled/settings-Revisionen, Directory-Read/Write, CLI/IPC, JSON-/Dateiverträge und Installationsrollback erhalten. Archivzweige isoliert bewerten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modverwaltung, Kataloginstallation, Aktualisierungsprüfung und Desktop-Steuerung. Neue Einordnung: Eingebaute Module. Besitzer: Mehrere konkrete Besitzer. |

**Belege:** main.rs:565 parse_ui_command · main.rs:738 run_with_arguments · main.rs:2301 snapshot · main.rs:2551 read_mods · main.rs:2765 read_package_file · main.rs:3013 read_notifications · main.rs:3370 save_mod_settings · main.rs:3399 set_mod_enabled · main.rs:3440 update_mod_extension · main.rs:3509 write_package_json · main.rs:3523 write_zip_entry · main.rs:3626 mod_ecosystem_for_package · main.rs:3668 claim_control_actions · main.rs:3908 save_setting · main.rs:4415 fetch_catalog · main.rs:4578 run_catalog_install_worker · main.rs:4655 install_mod_from_catalog · main.rs:4817 download_catalog_version · main.rs:4972 replace_catalog_mod · main.rs:5240 start_update_check · main.rs:5600 fetch_latest_github · main.rs:5820 extract_zip

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** aufteilen. UI sendet Aufträge. Settings/EML bleiben Paketverantwortung, Modinstallation und Releasewahl werden von UI und headless gemeinsam aufgerufen. Archiv-I/O ist kein Runtime-Modmodell.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/host/window.rs`, `modules/modloader/ui/host/commands.rs`, `modules/modloader/ui/host/snapshot.rs`, `packages/settings/update.rs`, `packages/manifest/ensure_eml_extension.rs`, `modules/updater/mod-installation/catalog.rs`, `modules/updater/mod-installation/install.rs`, `modules/updater/system-updates/check.rs`, `execution/runtime/control.rs`, `modules/updater/ui/host/window.rs`, `modules/updater/ui/host/commands.rs`.

### [src/loader/modules/modloader-ui/src/username.rs](../../modules/modloader/username.rs)

Vollständig gelesen, 244 Zeilen. SHA-256: `b63c0d07eb1c1708909b2e5ab1c892836aa7e9eceac23a93e0d25c2278f2375d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Ermittelt lesend den zuletzt gespielten Charakternamen aus lokalen oder Steam-Spielständen. Prüft stabile Dateien, analysiert KSC1-Container und zstd-komprimierte CHAR/BDB1-Blobs, verwirft mehrdeutige Namen und cached die Erkennung für fünf Sekunden. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Ermittelt lesend den zuletzt gespielten Charakternamen aus lokalen oder Steam-Spielständen. Prüft stabile Dateien, analysiert KSC1-Container und zstd-komprimierte CHAR/BDB1-Blobs, verwirft mehrdeutige Namen und cached die Erkennung für fünf Sekunden. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `username.rs:22 detect_active_character_name`<br>`username.rs:70 character_save_directories`<br>`username.rs:102 read_latest_character`<br>`username.rs:132 parse_ksc1_characters`<br>`username.rs:217 select_most_recent_character` |
| 4. Aufrufer und Verbraucher | modloader-ui/src/main.rs:2418 username::detect_active_character_name in snapshot. |
| 5. Verwendete Komponenten | serde_json · zstd · USERPROFILE und Steam-Verzeichnisse · std::fs |
| 6. Eingaben und Ausgaben | Eingaben: lokale und Steam characters-index/characters-N Spielstände Ausgabe: eindeutiger jüngster Charaktername oder None |
| 7. Seiteneffekte und Ressourcen | Dateien lesen · prozessinterner Fünfsekundencache |
| 8. Art der Verantwortung | Integrationslogik und Binärformatanalyse |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: serde_json · zstd · USERPROFILE und Steam-Verzeichnisse · std::fs. Aufruferbezug: modloader-ui/src/main.rs:2418 username::detect_active_character_name in snapshot. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | KSC1-/BDB1-/zstd-Verarbeitung, Größen- und Stabilitätsprüfungen erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Spielername aus Spielständen. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** username.rs:22 detect_active_character_name · username.rs:70 character_save_directories · username.rs:102 read_latest_character · username.rs:132 parse_ksc1_characters · username.rs:217 select_most_recent_character

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/username.rs`.

### [src/loader/modules/modloader-ui/ui/index.html](../../modules/modloader/ui/index.html)

Vollständig gelesen, 12 Zeilen. SHA-256: `415757023681015f1c7ea84d8d241225c304a2afa4c7ba3505d44e5789736aa1`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt das HTML-Startdokument mit React-Mountpunkt und main.tsx-Moduleinstieg bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt das HTML-Startdokument mit React-Mountpunkt und main.tsx-Moduleinstieg bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `index.html:9 root`<br>`index.html:10 src/main.tsx` |
| 4. Aufrufer und Verbraucher | Vite verwendet index.html als Einstieg und bündelt main.tsx. modloader-ui/src/main.rs:890-895 bettet ui/dist/index.html ein. |
| 5. Verwendete Komponenten | src/main.tsx · Vite |
| 6. Eingaben und Ausgaben | Eingaben: Vite main.tsx Ausgabe: React-Mountpunkt |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Präsentationseinstieg |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: src/main.tsx · Vite. Aufruferbezug: Vite verwendet index.html als Einstieg und bündelt main.tsx. modloader-ui/src/main.rs:890-895 bettet ui/dist/index.html ein. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Vite- und Host-Einbettung beachten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Startdokument der Modverwaltung. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** index.html:9 root · index.html:10 src/main.tsx

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/index.html`.

### [src/loader/modules/modloader-ui/ui/scripts/check-locales.mjs](../../modules/modloader/ui/scripts/check-locales.mjs)

Vollständig gelesen, 40 Zeilen. SHA-256: `f58477459f4c1b027efdb709d971b32df752c89d46a6a4c59abaa507d942704e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft alle Übersetzungskataloge gegen Englisch auf identische Schlüssel, nicht leere Texte, passende Platzhalter und gültige Sprach-/Flaggenmetadaten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft alle Übersetzungskataloge gegen Englisch auf identische Schlüssel, nicht leere Texte, passende Platzhalter und gültige Sprach-/Flaggenmetadaten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `check-locales.mjs:7-13 Kataloge`<br>`check-locales.mjs:20-34 Prüfungen`<br>`check-locales.mjs:36 Fehlerstatus` |
| 4. Aufrufer und Verbraucher | ui/package.json scripts.locale:check, ausgeführt vor build. |
| 5. Verwendete Komponenten | node:fs/promises · node:path · src/locales/*.json |
| 6. Eingaben und Ausgaben | Eingaben: alle src/locales JSON-Kataloge Ausgabe: Prüfmeldung und Exitcode |
| 7. Seiteneffekte und Ressourcen | Dateien lesen · stdout/stderr |
| 8. Art der Verantwortung | Build- und Qualitätssicherung |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: node:fs/promises · node:path · src/locales/*.json. Aufruferbezug: ui/package.json scripts.locale:check, ausgeführt vor build. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | package.json-Skripte und relative Katalogpfade anpassen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Qualitätssicherung der Übersetzungen. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** check-locales.mjs:7-13 Kataloge · check-locales.mjs:20-34 Prüfungen · check-locales.mjs:36 Fehlerstatus

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/scripts/check-locales.mjs`.

### [src/loader/modules/modloader-ui/ui/scripts/generate-flags.mjs](../../modules/modloader/ui/scripts/generate-flags.mjs)

Vollständig gelesen, 19 Zeilen. SHA-256: `f85ff22cc252d7b7af497d8b84f70c742e06a188592e71d3d917211a296b38a3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest die Flaggenmetadaten sämtlicher Sprachkataloge und erzeugt daraus das TypeScript-Modul mit den benötigten flag-icons-SVG-Quellen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest die Flaggenmetadaten sämtlicher Sprachkataloge und erzeugt daraus das TypeScript-Modul mit den benötigten flag-icons-SVG-Quellen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `generate-flags.mjs:10-15 Flaggen lesen`<br>`generate-flags.mjs:17-18 generated-flags.ts schreiben` |
| 4. Aufrufer und Verbraucher | ui/package.json scripts.flags:build, ausgeführt vor build und dev. |
| 5. Verwendete Komponenten | node:fs/promises · src/locales/*.json · node_modules/flag-icons/flags/4x3/*.svg |
| 6. Eingaben und Ausgaben | Eingaben: Sprachkataloge · flag-icons SVG-Dateien Ausgabe: generated-flags.ts |
| 7. Seiteneffekte und Ressourcen | generierte Datei schreiben |
| 8. Art der Verantwortung | Buildwerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: node:fs/promises · src/locales/*.json · node_modules/flag-icons/flags/4x3/*.svg. Aufruferbezug: ui/package.json scripts.flags:build, ausgeführt vor build und dev. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | package.json und generierten Importpfad mitziehen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Erzeugung der Sprachflaggen. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** generate-flags.mjs:10-15 Flaggen lesen · generate-flags.mjs:17-18 generated-flags.ts schreiben

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/scripts/generate-flags.mjs`.

### [src/loader/modules/modloader-ui/ui/src/i18n.tsx](../../modules/modloader/ui/i18n.tsx)

Vollständig gelesen, 66 Zeilen. SHA-256: `db10065faeded5d097c61b07cbb23bee91ce8a3f4128d61e7701ef410f01c4db`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt die vorhandenen Sprachkataloge, bietet einen React-Übersetzungskontext mit Platzhalterersetzung und Englisch-Fallback, speichert die Sprachwahl lokal und setzt die Dokumentensprache. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt die vorhandenen Sprachkataloge, bietet einen React-Übersetzungskontext mit Platzhalterersetzung und Englisch-Fallback, speichert die Sprachwahl lokal und setzt die Dokumentensprache. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `i18n.tsx:16 Kataloge`<br>`i18n.tsx:42 I18nProvider`<br>`i18n.tsx:52 t`<br>`i18n.tsx:62 useI18n` |
| 4. Aufrufer und Verbraucher | main.tsx:4 importiert I18nProvider/useI18n · main.tsx:647 mountet I18nProvider. |
| 5. Verwendete Komponenten | React · Vite import.meta.glob · src/locales/*.json · localStorage |
| 6. Eingaben und Ausgaben | Eingaben: Sprachkataloge · sf-language storage · Sprache und Übersetzungsschlüssel/Parameter Ausgabe: Übersetzungskontext und lokalisierte Texte |
| 7. Seiteneffekte und Ressourcen | localStorage und html.lang ändern |
| 8. Art der Verantwortung | Präsentationsunterstützung |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: React · Vite import.meta.glob · src/locales/*.json · localStorage. Aufruferbezug: main.tsx:4 importiert I18nProvider/useI18n · main.tsx:647 mountet I18nProvider. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Vite glob, Katalogtyp und Übersetzungs-/Provider-Aufrufer mitziehen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Sprachauswahl und Übersetzung. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** i18n.tsx:16 Kataloge · i18n.tsx:42 I18nProvider · i18n.tsx:52 t · i18n.tsx:62 useI18n

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/i18n.tsx`.

### [src/loader/modules/modloader-ui/ui/src/main.tsx](../../modules/modloader/ui/app.tsx)

Vollständig gelesen, 647 Zeilen. SHA-256: `2c15d93934bd392a0c9d138ddf0e5d57185ab6861a2bb13d89ad1e154e1f073c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Implementiert die gesamte React-Modverwaltung samt separatem Updaterfenster. Hält Host-Snapshots und IPC-Verträge, Navigation, Logs, Nachrichten, Katalog, installierte Mods, Updatequeue, Kompatibilität und globale Einstellungen. Rendert deklarierte Modsettings mit Autosave/Revision, Modaktionen/Links, Sprachwahl/Flaggen und lokale Darstellungspräferenzen. Ruft GitHub-Mitwirkende direkt ab und cached sie. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Implementiert die gesamte React-Modverwaltung samt separatem Updaterfenster. Hält Host-Snapshots und IPC-Verträge, Navigation, Logs, Nachrichten, Katalog, installierte Mods, Updatequeue, Kompatibilität und globale Einstellungen. Rendert deklarierte Modsettings mit Autosave/Revision, Modaktionen/Links, Sprachwahl/Flaggen und lokale Darstellungspräferenzen. Ruft GitHub-Mitwirkende direkt ab und cached sie. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.tsx:9-31 Hostdaten-Typen`<br>`main.tsx:35 getGitHubContributors`<br>`main.tsx:45 post und :49 postRequest`<br>`main.tsx:63 App`<br>`main.tsx:87-105 Hostcallbacks`<br>`main.tsx:224 ActivityPage`<br>`main.tsx:245 NewsPage`<br>`main.tsx:252 Discover`<br>`main.tsx:283 UpdateQueuePanel`<br>`main.tsx:320 ReleaseNotes`<br>`main.tsx:345 UpdaterWindow`<br>`main.tsx:382 Compatibility`<br>`main.tsx:434 SettingsPage`<br>`main.tsx:483 ModulePreferencesEditor`<br>`main.tsx:524 ModPage`<br>`main.tsx:538,551,559,571 settings schreiben`<br>`main.tsx:574 enabled schreiben`<br>`main.tsx:596 ModSetting`<br>`main.tsx:638 iconPaths`<br>`main.tsx:647 React-Einstieg` |
| 4. Aufrufer und Verbraucher | ui/index.html:10 main.tsx · ui/vite.config.ts bündelt in dist/index.html · modloader-ui/src/main.rs:890-900 hostet dist/index.html |
| 5. Verwendete Komponenten | React/react-dom · i18n.tsx · generated-flags.ts · links/*.json und links/assets/*.svg · styles.css · window.ipc und __shroudforge-Callbacks · GitHub contributors API · localStorage und Clipboard |
| 6. Eingaben und Ausgaben | Eingaben: Host-Snapshots und Resultate · Nutzerinteraktionen · Sprach-/Linkdaten · GitHub-Mitwirkende Ausgabe: alle Modverwaltungs-/Updateransichten · IPC-Aufträge samt RequestID/Revision |
| 7. Seiteneffekte und Ressourcen | localStorage · GitHub-Netzwerk · Zwischenablage · DOM |
| 8. Art der Verantwortung | Präsentationslogik und Hostintegration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Modloader-, Update-, Diagnose- und Nachrichtenoberflächen bei ihren Modulen führen. Das Modloader-Menü verwendet deren öffentliche Aufträge und gegebenenfalls exportierte Ansichten. Befunde: Belegt sind mehrere fachliche Ansichten und ihre vollständige Zustands-/IPC-/Präsentationslogik in einer Datei. Zusammenhang über einen Snapshot rechtfertigt gemeinsame Koordination, aber keine gemeinsame Implementierung aller Seiten. · Hostdaten-Typen und Commandnamen sind parallel zu Rust UiCommand/Snapshot/ModInfo und vite-env.d.ts geschrieben. Änderungen müssen beide Seiten konsistent halten. · iconPaths enthält denselben Schlüssel heart zweimal:640 und :642, jeweils mit gleichem Pfad. Auswirkungen auf den Build wurden nicht geprüft. · Hilfskomponenten Rows:625, ModuleRow:628, CompatibilityCard:629 und Modal:635 haben in dieser vollständig gelesenen Datei keine Aufrufstelle. Da sie nicht exportiert werden, ist ihre Verwendung in diesem Einstieg nicht nachgewiesen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: React/react-dom · i18n.tsx · generated-flags.ts · links/*.json und links/assets/*.svg · styles.css · window.ipc und __shroudforge-Callbacks · GitHub contributors API · localStorage und Clipboard. Aufruferbezug: ui/index.html:10 main.tsx · ui/vite.config.ts bündelt in dist/index.html · modloader-ui/src/main.rs:890-900 hostet dist/index.html |
| 11. Duplikation | iconPaths enthält heart doppelt:640/:642. · Hosttypen und Commandnamen werden in Rust und TypeScript parallel gepflegt, belegt durch Typen:9-31 und Host UiCommand/Snapshot/ModInfo. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Hostvertrag, Autosave bei Wechsel/Unmount, EML-Settings/Enabled, Seitennavigation und Buildimporte erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modverwaltung, Update- und Diagnoseoberflächen mit Host-Interaktion. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** main.tsx:9-31 Hostdaten-Typen · main.tsx:35 getGitHubContributors · main.tsx:45 post und :49 postRequest · main.tsx:63 App · main.tsx:87-105 Hostcallbacks · main.tsx:224 ActivityPage · main.tsx:245 NewsPage · main.tsx:252 Discover · main.tsx:283 UpdateQueuePanel · main.tsx:320 ReleaseNotes · main.tsx:345 UpdaterWindow · main.tsx:382 Compatibility · main.tsx:434 SettingsPage · main.tsx:483 ModulePreferencesEditor · main.tsx:524 ModPage · main.tsx:538,551,559,571 settings schreiben · main.tsx:574 enabled schreiben · main.tsx:596 ModSetting · main.tsx:638 iconPaths · main.tsx:647 React-Einstieg

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** aufteilen. Modloader-, Update-, Diagnose- und Nachrichtenoberflächen bei ihren Modulen führen. Das Modloader-Menü verwendet deren öffentliche Aufträge und gegebenenfalls exportierte Ansichten.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/app.tsx`, `modules/modloader/ui/host.ts`, `modules/modloader/ui/mods.tsx`, `modules/modloader/ui/settings.tsx`, `modules/updater/ui/updates.tsx`, `modules/runtime-diagnostics/ui/controls.tsx`, `modules/news/ui/notices.tsx`.

### [src/loader/modules/modloader-ui/ui/src/styles.css](../../modules/modloader/ui/styles.css)

Vollständig gelesen, 198 Zeilen. SHA-256: `2d776bf0759bce1b3389b336f8a26618133760aed3fcec4b0284d18930b23f82`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Gestaltet die gesamte Modverwaltungs- und Updateroberfläche, Navigation, Logs, Nachrichten, Moddetails und deklarative Einstellungen, Kompatibilitätsstatus, Updatequeue, responsive Layouts und reduzierte Animation. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Gestaltet die gesamte Modverwaltungs- und Updateroberfläche, Navigation, Logs, Nachrichten, Moddetails und deklarative Einstellungen, Kompatibilitätsstatus, Updatequeue, responsive Layouts und reduzierte Animation. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `styles.css:1 tokens-Import`<br>`styles.css:13-20 Grundlayout und Ansichten`<br>`styles.css:67 Toasts`<br>`styles.css:70-75 Moddarstellung`<br>`styles.css:82-110 Updater und Queue`<br>`styles.css:112-171 Layoutanpassungen`<br>`styles.css:177-198 Chirper und Name` |
| 4. Aufrufer und Verbraucher | main.tsx:7 CSS-Import · Vite CSS-Bündelung und Host-Einbettung |
| 5. Verwendete Komponenten | ui-shared/tokens.css · main.tsx DOM/Klassen |
| 6. Eingaben und Ausgaben | Eingaben: DOM und sf-Token Ausgabe: Modverwaltungs-/Updaterlayout |
| 7. Seiteneffekte und Ressourcen | Keine über Darstellung hinaus |
| 8. Art der Verantwortung | Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Identische ui-fatal-error-Regeln stehen doppelt:3 und :5. Modstatus-, Modsetting- und Layoutregeln werden an mehreren Stellen erneut definiert und später überschrieben, etwa .mod-setting-control:70 und :72 sowie .table-row:17,124,149. · Das Stylesheet folgt mehreren fachlichen Ansichten. Ordnung nach tatsächlichen Ansichten und gemeinsam verwendeten Bedienelementen ist begründbar. Vollständige CSS-Nutzungsanalyse wurde nicht durchgeführt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: ui-shared/tokens.css · main.tsx DOM/Klassen. Aufruferbezug: main.tsx:7 CSS-Import · Vite CSS-Bündelung und Host-Einbettung |
| 11. Duplikation | Identische ui-fatal-error-Regeln:3 und :5. · Wiederholte Modsetting-/Layoutselektoren mit Kaskadenüberschreibungen:70/:72 und :17/:124/:149. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Kaskadenreihenfolge und responsive Überschreibungen bei Aufteilung beibehalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gestaltung der Modverwaltung und Aktualisierung. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** styles.css:1 tokens-Import · styles.css:13-20 Grundlayout und Ansichten · styles.css:67 Toasts · styles.css:70-75 Moddarstellung · styles.css:82-110 Updater und Queue · styles.css:112-171 Layoutanpassungen · styles.css:177-198 Chirper und Name

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/styles.css`.

### [src/loader/modules/modloader-ui/ui/src/vite-env.d.ts](../../modules/modloader/ui/vite-env.d.ts)

Vollständig gelesen, 9 Zeilen. SHA-256: `13b5294bc56573b7e256599a6a4d09478049e7c4453baffc9d89e8ce8c96ad9d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert die Host-IPC-Schnittstelle und die vom Rust-Host aufgerufenen Fenster-, Snapshot-, Befehls- und Logkopiercallbacks für TypeScript. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert die Host-IPC-Schnittstelle und die vom Rust-Host aufgerufenen Fenster-, Snapshot-, Befehls- und Logkopiercallbacks für TypeScript. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `vite-env.d.ts:3-9 Window-Erweiterung` |
| 4. Aufrufer und Verbraucher | TypeScript ambient Window-Erweiterung, konkrete Aufrufe main.tsx:45 und Hostcallbacks:87-105,230. |
| 5. Verwendete Komponenten | vite/client · Window |
| 6. Eingaben und Ausgaben | Eingaben: TypeScript-Code mit window-Hostcallbacks Ausgabe: statische Window-Typdeklarationen |
| 7. Seiteneffekte und Ressourcen | Keine Laufzeitseiteneffekte |
| 8. Art der Verantwortung | Schnittstellentypen |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: vite/client · Window. Aufruferbezug: TypeScript ambient Window-Erweiterung, konkrete Aufrufe main.tsx:45 und Hostcallbacks:87-105,230. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Callbacktypen synchron zu Rust und main.tsx halten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Typvertrag zwischen Oberfläche und Host. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** vite-env.d.ts:3-9 Window-Erweiterung

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/vite-env.d.ts`.

### [src/loader/modules/modloader-ui/ui/vite.config.ts](../../modules/modloader/ui/vite.config.ts)

Vollständig gelesen, 64 Zeilen. SHA-256: `d145fb51824c58eb24dd828884eeda43d43327e48ad14e9a46037539ed66fa54`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Konfiguriert den React-Build für die eingebettete Modverwaltung. Ein Nachbearbeitungsplugin schreibt erzeugtes JavaScript und CSS in das HTML und entfernt die externen Bundle-Dateien. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Konfiguriert den React-Build für die eingebettete Modverwaltung. Ein Nachbearbeitungsplugin schreibt erzeugtes JavaScript und CSS in das HTML und entfernt die externen Bundle-Dateien. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `vite.config.ts:5 inlineBuiltAssets`<br>`vite.config.ts:9 generateBundle`<br>`vite.config.ts:52 defineConfig` |
| 4. Aufrufer und Verbraucher | ui/package.json scripts.build und scripts.dev starten Vite. |
| 5. Verwendete Komponenten | vite · @vitejs/plugin-react · node:path |
| 6. Eingaben und Ausgaben | Eingaben: Vite-Bundle HTML/JS/CSS Ausgabe: einbettbares HTML |
| 7. Seiteneffekte und Ressourcen | Buildbundle verändert/Dateien entfernt |
| 8. Art der Verantwortung | Buildwerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: vite · @vitejs/plugin-react · node:path. Aufruferbezug: ui/package.json scripts.build und scripts.dev starten Vite. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Host erwartet dist/index.html mit allen Assets eingebettet. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Bündelung der eingebetteten Modverwaltung. Neue Einordnung: Eingebaute Module. Besitzer: Modverwaltung-Oberfläche. |

**Belege:** vite.config.ts:5 inlineBuiltAssets · vite.config.ts:9 generateBundle · vite.config.ts:52 defineConfig

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/modloader/ui/vite.config.ts`.

### [src/loader/modules/runtime-diagnostics/src/lib.rs](../../modules/runtime-diagnostics/lib.rs)

Vollständig gelesen, 415 Zeilen. SHA-256: `0c297458f5cb3b2a848de4f564fae888af3af4e149f1dcee1b366dea9128fa5f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verwaltet zeitlich begrenzte, explizit aktivierte Diagnosesitzungen. Liest die bestehende Loaderkonfiguration, sammelt begrenzte Modmessungen und native Runtime-Snapshots, protokolliert Änderungen und veröffentlicht diagnostics-status. Enthält dazu drei Tests. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verwaltet zeitlich begrenzte, explizit aktivierte Diagnosesitzungen. Liest die bestehende Loaderkonfiguration, sammelt begrenzte Modmessungen und native Runtime-Snapshots, protokolliert Änderungen und veröffentlicht diagnostics-status. Enthält dazu drei Tests. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:33 Session`<br>`lib.rs:79 measure`<br>`lib.rs:142 tick`<br>`lib.rs:190 sample`<br>`lib.rs:250 publish`<br>`lib.rs:272 request`<br>`lib.rs:292 status`<br>`lib.rs:313 native_snapshot`<br>`lib.rs:340 Tests` |
| 4. Aufrufer und Verbraucher | api/src/lib.rs:378,428 Session · modloader-ui/src/main.rs:1807 request und :2420 status · runtime-diagnostics/src/main.rs:13,19 |
| 5. Verwendete Komponenten | shroudforge_package::config · serde_json · tracing · KfcRuntimeDiagnostics Export · tempfile in Tests |
| 6. Eingaben und Ausgaben | Eingaben: Loader-Diagnoseeinstellungen · Messzeiten/Fehler · KfcRuntimeDiagnostics JSON Ausgabe: Logs und diagnostics-status · aktiv/fresh-Diagnosestatus |
| 7. Seiteneffekte und Ressourcen | Loaderkonfiguration bei Request/Abschluss ändern · Statusdatei und Logs schreiben · native DLL-Funktion aufrufen |
| 8. Art der Verantwortung | Diagnose-/Messlogik und Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: shroudforge_package::config · serde_json · tracing · KfcRuntimeDiagnostics Export · tempfile in Tests. Aufruferbezug: api/src/lib.rs:378,428 Session · modloader-ui/src/main.rs:1807 request und :2420 status · runtime-diagnostics/src/main.rs:13,19 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | bestehende config-Felder, RequestID-Wettlaufprüfung, Dauer-/Größenlimits und native ABI erhalten. |
| 14. Vorhandener Testschutz | lib.rs:344 disabled_means_no_measurements_and_snapshot_finishes · lib.rs:369 deadline_is_independent_of_sample_interval_and_restart_works · lib.rs:404 invalid_settings_are_rejected · Tests vollständig gelesen, nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Laufzeitmessung und Diagnosezustand. Neue Einordnung: Eingebaute Module. Besitzer: runtime-diagnostics. |

**Belege:** lib.rs:33 Session · lib.rs:79 measure · lib.rs:142 tick · lib.rs:190 sample · lib.rs:250 publish · lib.rs:272 request · lib.rs:292 status · lib.rs:313 native_snapshot · lib.rs:340 Tests

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/runtime-diagnostics/lib.rs`.

### [src/loader/modules/runtime-diagnostics/src/main.rs](../../modules/runtime-diagnostics/main.rs)

Vollständig gelesen, 30 Zeilen. SHA-256: `618ea26161c979d55fabc38bff828c7a77c3cf621084d80f0291441cfdb11d34`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Übersetzt CLI-Argumente mit verpflichtendem --root in Diagnoseaktionen start, stop, snapshot oder die Ausgabe des gespeicherten Status. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Übersetzt CLI-Argumente mit verpflichtendem --root in Diagnoseaktionen start, stop, snapshot oder die Ausgabe des gespeicherten Status. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.rs:3 run_from_args`<br>`main.rs:12 --status`<br>`main.rs:17 Aktionsschleife` |
| 4. Aufrufer und Verbraucher | runtime-diagnostics/src/lib.rs:2-5 · workflow/main.rs:90 |
| 5. Verwendete Komponenten | crate::request · crate::status · std::env |
| 6. Eingaben und Ausgaben | Eingaben: --root und --start/--stop/--snapshot/--status Ausgabe: CLI-Meldung oder Status-JSON |
| 7. Seiteneffekte und Ressourcen | über crate::request Loaderkonfiguration ändern · stdout |
| 8. Art der Verantwortung | Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: crate::request · crate::status · std::env. Aufruferbezug: runtime-diagnostics/src/lib.rs:2-5 · workflow/main.rs:90 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CLI-Vertrag und öffentliche run_module-Funktion erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Diagnosesteuerung über Kommandozeile. Neue Einordnung: Eingebaute Module. Besitzer: runtime-diagnostics. |

**Belege:** main.rs:3 run_from_args · main.rs:12 --status · main.rs:17 Aktionsschleife

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/runtime-diagnostics/main.rs`.

### [src/loader/modules/steam-networking/src/lib.rs](../../modules/transport/lib.rs)

Vollständig gelesen, 291 Zeilen. SHA-256: `fab4b875951b1c0a4a02349efa532349f0d00c02909e70acbf3dc3d315d8a542`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bietet den Rust-Zugang zu den bereits geladenen KfcRuntimeNetwork-Exporten für Steam-Status, Senden, Empfangen, Autorisierung, verbundene Spieler und Probe-Statistik. Begrenzt Payload und Kanal und initialisiert Steam nicht selbst. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bietet den Rust-Zugang zu den bereits geladenen KfcRuntimeNetwork-Exporten für Steam-Status, Senden, Empfangen, Autorisierung, verbundene Spieler und Probe-Statistik. Begrenzt Payload und Kanal und initialisiert Steam nicht selbst. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:7 MAX_MESSAGE_BYTES`<br>`lib.rs:83 send`<br>`lib.rs:123 service_tick`<br>`lib.rs:165 accept`<br>`lib.rs:200 connected_peers`<br>`lib.rs:218 receive`<br>`lib.rs:277 symbol` |
| 4. Aufrufer und Verbraucher | api/src/lib.rs:455,789 service_tick, :790 status, :803 service_stats · api/src/env/loader.rs:9 status |
| 5. Verwendete Komponenten | kfc-runtime.dll C-Exporte · windows_sys::LibraryLoader · std::sync Empfangspuffer |
| 6. Eingaben und Ausgaben | Eingaben: SteamID/Kanal/Nachricht · autorisierte Spieler · native KFC-Exporte Ausgabe: Transportstatus, Nachrichten und Statistik |
| 7. Seiteneffekte und Ressourcen | Native Netzwerkoperationen · DLL-Symbolauflösung · wiederverwendeter Empfangspuffer |
| 8. Art der Verantwortung | Transportintegration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: kfc-runtime.dll C-Exporte · windows_sys::LibraryLoader · std::sync Empfangspuffer. Aufruferbezug: api/src/lib.rs:455,789 service_tick, :790 status, :803 service_stats · api/src/env/loader.rs:9 status |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Exportnamen/C-ABI, Payload-/Kanalgrenzen und Spiel-eigene Steaminitialisierung beachten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Steam-Nachrichtentransport und Sitzungsstatus. Neue Einordnung: Eingebaute Module. Besitzer: Steam-Transport. |

**Belege:** lib.rs:7 MAX_MESSAGE_BYTES · lib.rs:83 send · lib.rs:123 service_tick · lib.rs:165 accept · lib.rs:200 connected_peers · lib.rs:218 receive · lib.rs:277 symbol

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/transport/lib.rs`.

### [src/loader/modules/ui-shared/tokens.css](../../modules/ui-resources/tokens.css)

Vollständig gelesen, 87 Zeilen. SHA-256: `7501c0f430de60f6032dda435879654ec7fdb256d2e395d10a4546d85fea28b0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert die gemeinsamen Farben, Abstände, Schriften, Übergänge und elementaren Fokus-/Scrollregeln für die eingebetteten Oberflächen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert die gemeinsamen Farben, Abstände, Schriften, Übergänge und elementaren Fokus-/Scrollregeln für die eingebetteten Oberflächen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `tokens.css:1-40 Designvariablen`<br>`tokens.css:42-87 Grundregeln` |
| 4. Aufrufer und Verbraucher | debug-console/src/main.rs:131 · world-editor-ui/src/main.rs:257 · modloader-ui/ui/src/styles.css:1 |
| 5. Verwendete Komponenten | CSS Custom Properties |
| 6. Eingaben und Ausgaben | Eingaben: CSS-Verbraucher Ausgabe: gemeinsame Designvariablen und Grundregeln |
| 7. Seiteneffekte und Ressourcen | Keine über Darstellung hinaus |
| 8. Art der Verantwortung | Präsentationsunterstützung |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: CSS Custom Properties. Aufruferbezug: debug-console/src/main.rs:131 · world-editor-ui/src/main.rs:257 · modloader-ui/ui/src/styles.css:1 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | alle drei UI-Einbettungen und CSS-Importe zusammen anpassen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gemeinsame Oberflächengestaltung. Neue Einordnung: Eingebaute Module. Besitzer: Gemeinsame UI-Ressourcen. |

**Belege:** tokens.css:1-40 Designvariablen · tokens.css:42-87 Grundregeln

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/ui-resources/tokens.css`.

### [src/loader/modules/updater/src/bin.rs](../../modules/updater/bin.rs)

Vollständig gelesen, 64 Zeilen. SHA-256: `6f0c75cbbfbd1330dca2bdc5377b9976dcf18e483ad85b4125b7bb7c19e49287`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Startet den eigenständigen Updater, verarbeitet den Bootstrap-Startupcheck und das Einreihen nach Spielende und bildet Fehler auf Exitcodes beziehungsweise Windows-Dialoge ab. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Startet den eigenständigen Updater, verarbeitet den Bootstrap-Startupcheck und das Einreihen nach Spielende und bildet Fehler auf Exitcodes beziehungsweise Windows-Dialoge ab. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `bin.rs:3 main`<br>`bin.rs:5 --startup-check`<br>`bin.rs:16 --queue-install`<br>`bin.rs:35 Fehlerausgabe` |
| 4. Aufrufer und Verbraucher | updater/Cargo.toml [[bin]] |
| 5. Verwendete Komponenten | shroudforge_updater Bibliothek · windows_sys::MessageBoxW |
| 6. Eingaben und Ausgaben | Eingaben: Startup-/Queue-/Worker-CLI Ausgabe: Exitcodes und Fehlerdialog |
| 7. Seiteneffekte und Ressourcen | Bibliotheksaktionen · MessageBox/prozessbeendigung |
| 8. Art der Verantwortung | Prozesseinstieg und Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: shroudforge_updater Bibliothek · windows_sys::MessageBoxW. Aufruferbezug: updater/Cargo.toml [[bin]] |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Bootstrap interpretiert Exitcode 2 als blockierenden Update-/Reparaturbedarf. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Updater-Prozessstart und Bootstrap-Protokoll. Neue Einordnung: Eingebaute Module. Besitzer: Updates und Installation. |

**Belege:** bin.rs:3 main · bin.rs:5 --startup-check · bin.rs:16 --queue-install · bin.rs:35 Fehlerausgabe

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/updater/bin.rs`.

### [src/loader/modules/updater/src/lib.rs](../../modules/updater/lib.rs)

Vollständig gelesen, 17 Zeilen. SHA-256: `5462e7eae0f53a70a4b572440a7dd032775dd9c3e53ebdd2717fb5bff6eaad3c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert die Updater-Anwendungsfunktionen für System-, Mod-, Reparatur-, Runtime- und Queue-Aufträge aus main.rs. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert die Updater-Anwendungsfunktionen für System-, Mod-, Reparatur-, Runtime- und Queue-Aufträge aus main.rs. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:4-17 reexports` |
| 4. Aufrufer und Verbraucher | modloader-ui/src/main.rs:1288 ff.,4524 ff.,5304 ff. · workflow/lib.rs:113 · workflow/main.rs:94 · updater/src/bin.rs:12,28,33 |
| 5. Verwendete Komponenten | main.rs |
| 6. Eingaben und Ausgaben | Eingaben: Kein eigener Laufzeitinput Ausgabe: Updater-API-Reexports |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Öffentliche Integrationsschnittstelle |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: main.rs. Aufruferbezug: modloader-ui/src/main.rs:1288 ff.,4524 ff.,5304 ff. · workflow/lib.rs:113 · workflow/main.rs:94 · updater/src/bin.rs:12,28,33 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | öffentliche Reexports und alle Bibliotheksabhängigkeiten mitziehen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Öffentliche Aktualisierungsschnittstelle. Neue Einordnung: Eingebaute Module. Besitzer: Updates und Installation. |

**Belege:** lib.rs:4-17 reexports

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/updater/lib.rs`.

### [src/loader/modules/updater/src/main.rs](../../modules/updater/system-updates/check.rs)

Vollständig gelesen, 2864 Zeilen. SHA-256: `e4061919bcd21358464a9adfaeb19fde4e61429d215fcdfaf61c0f1888939be3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft erforderliche Installationsdateien und GitHub-Releases, verwaltet persistierte System-/Mod-Updatequeues, startet unabhängige/scheduled Windows-Worker, lädt/verifiziert/entpackt Systemreleases, wartet auf Spielende und ersetzt nur verwaltete Dateien mit Backup/Rollback. Startet außerdem Kataloginstallation im Loader, Runtime-Unload/Reload über Request/Result-Dateien, headless Config-Steuerung und Wiederherstellung originaler Spieldateien. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft erforderliche Installationsdateien und GitHub-Releases, verwaltet persistierte System-/Mod-Updatequeues, startet unabhängige/scheduled Windows-Worker, lädt/verifiziert/entpackt Systemreleases, wartet auf Spielende und ersetzt nur verwaltete Dateien mit Backup/Rollback. Startet außerdem Kataloginstallation im Loader, Runtime-Unload/Reload über Request/Result-Dateien, headless Config-Steuerung und Wiederherstellung originaler Spieldateien. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.rs:3 parse_release_notes`<br>`main.rs:26 refresh_cached_repair_status`<br>`main.rs:80 restore_gamefiles_worker`<br>`main.rs:128 run_install`<br>`main.rs:332 hand_off_installed_worker`<br>`main.rs:446 managed_paths`<br>`main.rs:718 stage_system_update`<br>`main.rs:903 wait_for_game_processes`<br>`main.rs:1102 ensure_task`<br>`main.rs:1270 request_runtime_mod_action`<br>`main.rs:1354 start_headless_control_worker`<br>`main.rs:1540 execute_headless_control_action`<br>`main.rs:1720 headless_check_system_updates`<br>`main.rs:1926 startup_check`<br>`main.rs:2076 update_update_queue`<br>`main.rs:2218 start_update_queue`<br>`main.rs:2415 run_update_queue`<br>`main.rs:2526-2529 Katalogworker-Aufruf`<br>`main.rs:2619 run_scheduled_worker`<br>`main.rs:2762 run_module`<br>`main.rs:2812 request_gamefiles_restore` |
| 4. Aufrufer und Verbraucher | updater/src/bin.rs:12,28,33 · updater/src/lib.rs:4-17 · modloader-ui/src/main.rs Update-, Queue-, Reparatur- und Runtime-Aufträge · workflow/lib.rs:113 start_headless_control_worker · workflow/main.rs:94 run_module |
| 5. Verwendete Komponenten | mod_loader_bridge::missing_required_files · shroudforge_package::config, paths, backups, update_check, logging · Windows Toolhelp/Prozess-APIs · schtasks.exe · Loader --module-ui und --catalog-install-worker · reqwest · serde_json · semver · sha2 · zip |
| 6. Eingaben und Ausgaben | Eingaben: Release-/Queue-/Control-Requests · GitHub/Systemrelease-HTTP · verwaltete Dateiliste · Spielprozesse · Runtime-Resultate Ausgabe: Queue-/Progressstatus · verifizierte Stagingdateien · installierte Release-/restaurierte Originaldateien |
| 7. Seiteneffekte und Ressourcen | Netzwerk · Dateien ersetzen/Backup/Rollback · Windowsprozesse und geplante Aufgabe · Config/Log/Status schreiben |
| 8. Art der Verantwortung | Geschäfts- und Integrationslogik mit Prozess-/Dateiinfrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Doppelte Releaseprüfung zusammenführen. Worker bleiben wegen Prozess-/Dateisperrgrenze getrennt. Runtimekontrolle und Spielasset-Restore gehören ihren Datenbesitzern. Befunde: Mehrere fachlich trennbare Aufgaben sind belegt: Systemreleaseprüfung, Download/Staging/Dateiinstallation, persistierte Queue/Prozesssteuerung, headless Konfigurationsaufträge, Runtime-Modaktionen und Originaldatei-Wiederherstellung. · GitHub-Releaseauswahl, Tag-/Buildvergleich, Prüfsummenabruf und latestRelease-Persistierung in headless_check_system_updates:1720ff. duplizieren die Desktoppfade modloader-ui/src/main.rs:5240ff./5600ff. Gemeinsam genutzt werden lediglich update_check-Koordination und parse_release_notes. · Katalog-Modinstallation bleibt im Modverwaltungsmodul, das der Updater über den Loaderprozess aufruft:2526-2529/2697-2700. Das ist eine reale Zuständigkeitskopplung über eine CLI-Dateivertragsgrenze. · ZIP-Verarbeitung hier gehört zum einmaligen Systemrelease-Download/Staging, nicht zum laufenden entpackten Modbetrieb. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: mod_loader_bridge::missing_required_files · shroudforge_package::config, paths, backups, update_check, logging · Windows Toolhelp/Prozess-APIs · schtasks.exe · Loader --module-ui und --catalog-install-worker · reqwest · serde_json · semver · sha2 · zip. Aufruferbezug: updater/src/bin.rs:12,28,33 · updater/src/lib.rs:4-17 · modloader-ui/src/main.rs Update-, Queue-, Reparatur- und Runtime-Aufträge · workflow/lib.rs:113 start_headless_control_worker · workflow/main.rs:94 run_module |
| 11. Duplikation | headless_check_system_updates:1720ff. dupliziert GitHub-Auswahl/Checksumme/Releasepersistierung mit modloader-ui/src/main.rs:5240ff./5600ff. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CLI-/Queue-/Runtime-Dateiverträge, schreibgeschützte Nutzer-/Spielpfade, Dateipfadprüfungen, vollständiges Backup und Rollback sowie UI-/Bootstrap-Aufrufe erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Installation, Systemaktualisierung, Mod-Queue und Wiederherstellung. Neue Einordnung: Eingebaute Module. Besitzer: Mehrere konkrete Besitzer. |

**Belege:** main.rs:3 parse_release_notes · main.rs:26 refresh_cached_repair_status · main.rs:80 restore_gamefiles_worker · main.rs:128 run_install · main.rs:332 hand_off_installed_worker · main.rs:446 managed_paths · main.rs:718 stage_system_update · main.rs:903 wait_for_game_processes · main.rs:1102 ensure_task · main.rs:1270 request_runtime_mod_action · main.rs:1354 start_headless_control_worker · main.rs:1540 execute_headless_control_action · main.rs:1720 headless_check_system_updates · main.rs:1926 startup_check · main.rs:2076 update_update_queue · main.rs:2218 start_update_queue · main.rs:2415 run_update_queue · main.rs:2526-2529 Katalogworker-Aufruf · main.rs:2619 run_scheduled_worker · main.rs:2762 run_module · main.rs:2812 request_gamefiles_restore

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** aufteilen. Doppelte Releaseprüfung zusammenführen. Worker bleiben wegen Prozess-/Dateisperrgrenze getrennt. Runtimekontrolle und Spielasset-Restore gehören ihren Datenbesitzern.

**Vorgeschlagene Zielpfade:** `modules/updater/system-updates/check.rs`, `modules/updater/system-updates/stage.rs`, `modules/updater/system-updates/install.rs`, `modules/updater/queue/store.rs`, `modules/updater/queue/execute.rs`, `modules/updater/windows/worker.rs`, `execution/runtime/control.rs`, `game-assets/restore_request.rs`.

### [src/loader/modules/world-editor-ui/src/lib.rs](../../modules/world-editor/lib.rs)

Vollständig gelesen, 4 Zeilen. SHA-256: `e74333d274a33f4f843eea4b464bd0151ba3e92615cc143cd07550c0bcad4c16`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert den Start der Welteditor-Oberfläche aus main.rs für den Loader-Dispatcher. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert den Start der Welteditor-Oberfläche aus main.rs für den Loader-Dispatcher. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `lib.rs:1-4` |
| 4. Aufrufer und Verbraucher | workflow/main.rs:65 |
| 5. Verwendete Komponenten | main.rs |
| 6. Eingaben und Ausgaben | Eingaben: Kein eigener Laufzeitinput Ausgabe: run_module-Reexport |
| 7. Seiteneffekte und Ressourcen | Keine eigenen |
| 8. Art der Verantwortung | Integrationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Vermischung unabhängiger fachlicher Aufgaben belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: main.rs. Aufruferbezug: workflow/main.rs:65 |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | #[path] und Loader-Dispatcher mitziehen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modulaufruf und Einstieg. Neue Einordnung: Eingebaute Module. Besitzer: world-editor. |

**Belege:** lib.rs:1-4

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/world-editor/lib.rs`.

### [src/loader/modules/world-editor-ui/src/main.rs](../../modules/world-editor/library.rs)

Vollständig gelesen, 1470 Zeilen. SHA-256: `383358fa46d2527e117f087e9c2c291d5b814976d1076b42d950df3a6e032a9e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Betreibt das Windows-Welteditor-Overlay mit F2, Fenster-/Prozesslebensdauer und WebView-IPC. Liest Editorzustand und Blueprintbibliothek aus Exports, erzeugt Vorschaubilder, findet/liest Screenshotordner, setzt und restauriert Blueprintcover und übergibt übrige Editoraktionen per atomar umbenannter JSON-Datei an das world-editor-Mod. Enthält zusätzlich einen JavaScript-Fallback für die Oberfläche. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Betreibt das Windows-Welteditor-Overlay mit F2, Fenster-/Prozesslebensdauer und WebView-IPC. Liest Editorzustand und Blueprintbibliothek aus Exports, erzeugt Vorschaubilder, findet/liest Screenshotordner, setzt und restauriert Blueprintcover und übergibt übrige Editoraktionen per atomar umbenannter JSON-Datei an das world-editor-Mod. Enthält zusätzlich einen JavaScript-Fallback für die Oberfläche. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `main.rs:126 run_module`<br>`main.rs:198 IPC-Parsing`<br>`main.rs:302 Statusworker`<br>`main.rs:316 Ereignisschleife`<br>`main.rs:639-797 JavaScript-Fallbackrenderer`<br>`main.rs:919 view_state`<br>`main.rs:1069 choose_screenshot_folder`<br>`main.rs:1177 load_screenshot_list`<br>`main.rs:1248 load_screenshot_image`<br>`main.rs:1313 apply_screenshot`<br>`main.rs:1346 backup_cover`<br>`main.rs:1373 undo_screenshot`<br>`main.rs:1412 dispatch_action` |
| 4. Aufrufer und Verbraucher | world-editor-ui/src/lib.rs:1-4 · workflow/main.rs:65 |
| 5. Verwendete Komponenten | shroudforge_package::config, paths, logging, window_icon · tao/wry · Windows COM/Prozess/Fenster/Tastatur · image · resvg · base64 · serde_json · ui/index.html, styles.css und gemeinsames tokens.css · exports/world-editor und ui/actions Dateiverträge |
| 6. Eingaben und Ausgaben | Eingaben: Editor-state.txt und .sfbp-Bibliothek · Screenshotordner/-bilder · Windows-/UI-Eingaben Ausgabe: ViewState/Screenshotlisten · Blueprintcover und Action-JSON |
| 7. Seiteneffekte und Ressourcen | Dateien/Undo-Cover schreiben · Fenster-/Prozess-/COM-Zugriff · Logs |
| 8. Art der Verantwortung | Präsentations- und Integrationslogik mit Bildverarbeitung |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Fenster/IPC, Blueprintdateien und Cover haben eigene Seiteneffekte. JS-Fallback erst nach Zweckprüfung konsolidieren. Befunde: Fenster-/IPC-Steuerung, Blueprint-/Screenshotdateizugriff, Bildverarbeitung und Präsentations-Fallback stehen in derselben Datei. Die Coververwaltung ist fachlich geschlossen, kann aber unabhängig vom Windows-Fensterhost organisiert werden. · Belegt ist parallele Präsentationslogik: main.rs:639-797 baut Prozessanzeige, Blueprintkarten und Screenshotmanager nach, die ui/index.html:52-65,75-158 bereits implementiert. Eine Reduzierung muss den bestehenden Fallbackzweck prüfen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: shroudforge_package::config, paths, logging, window_icon · tao/wry · Windows COM/Prozess/Fenster/Tastatur · image · resvg · base64 · serde_json · ui/index.html, styles.css und gemeinsames tokens.css · exports/world-editor und ui/actions Dateiverträge. Aufruferbezug: world-editor-ui/src/lib.rs:1-4 · workflow/main.rs:65 |
| 11. Duplikation | Backend-Fallbackrenderer main.rs:639-797 und HTML-Renderer index.html:52-65,75-158 haben parallele Prozess-/Karten-/Managerdarstellung. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Lua-Mod-Dateiverträge, Actionnamen, HTML-/Fallbackcallbacks, Bildlimits und Cover-Undo erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Welteditor-Overlay, Blueprintbibliothek und Coververwaltung. Neue Einordnung: Eingebaute Module. Besitzer: world-editor. |

**Belege:** main.rs:126 run_module · main.rs:198 IPC-Parsing · main.rs:302 Statusworker · main.rs:316 Ereignisschleife · main.rs:639-797 JavaScript-Fallbackrenderer · main.rs:919 view_state · main.rs:1069 choose_screenshot_folder · main.rs:1177 load_screenshot_list · main.rs:1248 load_screenshot_image · main.rs:1313 apply_screenshot · main.rs:1346 backup_cover · main.rs:1373 undo_screenshot · main.rs:1412 dispatch_action

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** aufteilen. Fenster/IPC, Blueprintdateien und Cover haben eigene Seiteneffekte. JS-Fallback erst nach Zweckprüfung konsolidieren.

**Vorgeschlagene Zielpfade:** `modules/world-editor/library.rs`, `modules/world-editor/covers.rs`, `modules/world-editor/ui/window.rs`, `modules/world-editor/ui/commands.rs`.

### [src/loader/modules/world-editor-ui/ui/index.html](../../modules/world-editor/ui/index.html)

Vollständig gelesen, 161 Zeilen. SHA-256: `ebc62a92c4ce7c97ef70c58642382c11aeee5758ba15a9f03ac20ab782767d70`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert und rendert die Blueprintbibliothek, Fortschrittsanzeige und den Blueprint-/Screenshotmanager. Verwaltet Auswahl, zuletzt verwendete Blueprints, Wartebestätigung und Bibliothekserweiterung, sendet Aktionen an den Rust-Host und protokolliert sichtbare UI-Flächen für Diagnose. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert und rendert die Blueprintbibliothek, Fortschrittsanzeige und den Blueprint-/Screenshotmanager. Verwaltet Auswahl, zuletzt verwendete Blueprints, Wartebestätigung und Bibliothekserweiterung, sendet Aktionen an den Rust-Host und protokolliert sichtbare UI-Flächen für Diagnose. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `index.html:26 Zustand`<br>`index.html:27 ipc`<br>`index.html:29 reportVisibleSurface`<br>`index.html:52 renderProcessProgress`<br>`index.html:73 requestBlueprint`<br>`index.html:75 renderCards`<br>`index.html:150 openManage`<br>`index.html:156 Screenshotliste`<br>`index.html:157 __worldEditorUpdate`<br>`index.html:158 Ereignisbindung` |
| 4. Aufrufer und Verbraucher | world-editor-ui/src/main.rs:262-267 HTML-Einbettung · world-editor-ui/src/main.rs:639ff. ruft __worldEditorUpdate auf |
| 5. Verwendete Komponenten | window.ipc · localStorage · DOM · Hostcallbacks __worldEditorUpdate und Screenshotcallbacks · eingebettete Styles und Icon |
| 6. Eingaben und Ausgaben | Eingaben: ViewState und Screenshotcallbacks · Benutzereingaben Ausgabe: Blueprint-/Manager-DOM · IPC-Aktionen und sichtbare UI-Diagnose |
| 7. Seiteneffekte und Ressourcen | localStorage · Bestätigungsdialoge · DOM |
| 8. Art der Verantwortung | Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: HTML-Struktur und umfangreiche JavaScript-Interaktion sind gemeinsam enthalten. Sie bilden eine Oberfläche, eine Aufteilung nach Bibliotheksdarstellung, Manager und IPC wäre durch verschiedene Änderungsgründe begründbar. · Renderer überschneidet sich mit JavaScript-Fallback world-editor-ui/src/main.rs:639-797. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: window.ipc · localStorage · DOM · Hostcallbacks __worldEditorUpdate und Screenshotcallbacks · eingebettete Styles und Icon. Aufruferbezug: world-editor-ui/src/main.rs:262-267 HTML-Einbettung · world-editor-ui/src/main.rs:639ff. ruft __worldEditorUpdate auf |
| 11. Duplikation | Backend-Fallbackrenderer main.rs:639-797 und HTML-Renderer index.html:52-65,75-158 haben parallele Prozess-/Karten-/Managerdarstellung. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Hostcallbacks, Fallbackrenderer, Actionnamen und Tab-separierte Aktionswerte koppeln. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Welteditor-Interaktion und Bibliotheksanzeige. Neue Einordnung: Eingebaute Module. Besitzer: world-editor. |

**Belege:** index.html:26 Zustand · index.html:27 ipc · index.html:29 reportVisibleSurface · index.html:52 renderProcessProgress · index.html:73 requestBlueprint · index.html:75 renderCards · index.html:150 openManage · index.html:156 Screenshotliste · index.html:157 __worldEditorUpdate · index.html:158 Ereignisbindung

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/world-editor/ui/index.html`.

### [src/loader/modules/world-editor-ui/ui/styles.css](../../modules/world-editor/ui/styles.css)

Vollständig gelesen, 94 Zeilen. SHA-256: `91965993e0eb1195bb30a28eb3f6bf225aba971eb8b169e044554254bdb6d356`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Gestaltet das Welteditor-Overlay, Blueprintkarten und Auswahlzustände, den Screenshotmanager, Hinweise und die Prozessfortschrittsanzeige. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Gestaltet das Welteditor-Overlay, Blueprintkarten und Auswahlzustände, den Screenshotmanager, Hinweise und die Prozessfortschrittsanzeige. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `styles.css:5 shell`<br>`styles.css:21 cards`<br>`styles.css:56 manage`<br>`styles.css:70 screenshot-queue`<br>`styles.css:84 process-progress` |
| 4. Aufrufer und Verbraucher | world-editor-ui/src/main.rs:255-263 Einbettung |
| 5. Verwendete Komponenten | ui-shared/tokens.css · world-editor-ui/ui/index.html und Backend-Fallbackrenderer |
| 6. Eingaben und Ausgaben | Eingaben: HTML-/Fallback-DOM und sf-Token Ausgabe: Welteditorlayout |
| 7. Seiteneffekte und Ressourcen | Keine über Darstellung hinaus |
| 8. Art der Verantwortung | Präsentationslogik |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Verantwortlichkeit belegt. Einzelne Regeln für brand/keys/actions sind im aktuellen HTML nicht vorhanden, Verwendung im sonstigen Fallback nur gezielt geprüft. Nicht pauschal als unbenutzt einzustufen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebaute Module. Technische Kopplung: ui-shared/tokens.css · world-editor-ui/ui/index.html und Backend-Fallbackrenderer. Aufruferbezug: world-editor-ui/src/main.rs:255-263 Einbettung |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv verwendet, durch die angegebenen Aufrufer oder Build-/Einbettungspfade nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Host-Einbettung und CSS-Selektoren mitziehen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Es wurden keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Gestaltung von Welteditor und Blueprintbibliothek. Neue Einordnung: Eingebaute Module. Besitzer: world-editor. |

**Belege:** styles.css:5 shell · styles.css:21 cards · styles.css:56 manage · styles.css:70 screenshot-queue · styles.css:84 process-progress

**Evidenzgrenze:** Verhalten und konkrete Symbole aus vollständiger Dateilektüre nachgewiesen. Fachliche Kategorien und Verschiebungsrisiken daraus abgeleitet. Nicht belegte Nutzung oder Tests ausdrücklich markiert.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `modules/world-editor/ui/styles.css`.

## Native Runtime und Profilwerkzeuge

64 Dateien, 17.638 Zeilen.

### [src/loader/runtime/native/ecs/component_registry.h](../../kfc-runtime/native/ecs/registry/component_registry.h)

Vollständig gelesen, 192 Zeilen. SHA-256: `693f661263252f7c6e85a17b7d2bc7cfc023c81a98233d02be6bbfd15dd75725`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest und validiert die originale Komponentenregistrierung über eine injizierte Speicherlesefunktion. Trennt Laufzeitspeicher und Template-Konfiguration, prüft Namen, Hashes, Parallelarrays und wiederholte Identitätslesung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest und validiert die originale Komponentenregistrierung über eine injizierte Speicherlesefunktion. Trennt Laufzeitspeicher und Template-Konfiguration, prüft Namen, Hashes, Parallelarrays und wiederholte Identitätslesung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `40-174 ReadOwner`<br>`176-190 Read` |
| 4. Aufrufer und Verbraucher | ecs_runtime.cpp, registry_discovery.h und externe Registry-Verifikatoren |
| 5. Verwendete Komponenten | STL Reader-Funktion |
| 6. Eingaben und Ausgaben | Eingaben: Imagebasis/-größe, originaler Registryowner/Manager und injizierter Reader Ausgabe: Validierter Registry-Snapshot oder Fehlergrund |
| 7. Seiteneffekte und Ressourcen | Führt injizierte Leseoperationen aus, hält keine Enginezeiger als Besitz. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: STL Reader-Funktion. Aufruferbezug: ecs_runtime.cpp, registry_discovery.h und externe Registry-Verifikatoren |
| 11. Duplikation | Python discovery/components.py:29-100 validiert dasselbe Registrylayout in getrennter Sprache. Externe C++-Prüfung teilt dagegen diesen Produktionsheader. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | ecs_read_test.cpp:166-190,237-263 Registryfixture/ABI · enshrouded/native/verify-component-registry.cpp:168-190 lokale Fehlerinjektion am Produktionsleser |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Registrierungsmetadaten. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 40-174 ReadOwner · 176-190 Read

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/ecs/registry/component_registry.h`.

### [src/loader/runtime/native/ecs/ecs_runtime.cpp](../../kfc-runtime/native/ecs/entities.cpp)

Vollständig gelesen, 2442 Zeilen. SHA-256: `c1752e36a16b647a702302e4386911268336260318e62b46c8b4d51bd9111ac3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Implementiert generische ECS-Registrierung, laufende Registryaufnahme, Build-/Owner-/Dispatcher-Bindung, nicht wiederverwendete Handles mit Generation und Epochenschutz, inkrementelle Queries und ID-Auflösung, geprüfte Einzel-/Stapellesungen sowie maskierte Schreib-/CAS-Operationen. Enthält zusätzlich konkrete Propabfragen, Rezept-Bounding-Box-Geometrie, Transformation und Skalierung, umfassende Diagnosen und Patch-ABI-Weiterleitung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Implementiert generische ECS-Registrierung, laufende Registryaufnahme, Build-/Owner-/Dispatcher-Bindung, nicht wiederverwendete Handles mit Generation und Epochenschutz, inkrementelle Queries und ID-Auflösung, geprüfte Einzel-/Stapellesungen sowie maskierte Schreib-/CAS-Operationen. Enthält zusätzlich konkrete Propabfragen, Rezept-Bounding-Box-Geometrie, Transformation und Skalierung, umfassende Diagnosen und Patch-ABI-Weiterleitung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `299-532 Identität/Handles`<br>`619-1038 Queries und Lesen`<br>`1059-1185 Schreiben`<br>`1301-1470 Registrierung/Discovery`<br>`1545-1852 Lebenszyklus/Diagnose`<br>`2081-2291 Prop-Rezepte und Transformation`<br>`2436-2442 Patchwrapper` |
| 4. Aufrufer und Verbraucher | provider.cpp · world_runtime.cpp · building_input.cpp · resource_runtime.cpp · service_runtime.cpp · Host über KfcRuntimeEcs*/WorldEntity*-ABI |
| 5. Verwendete Komponenten | component_registry.h · registry_discovery.h · dispatcher.h · profile.h · engine_context.h · world_runtime.h · patch_runtime.h · Win32 · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Reflektierter Typvertrag, profil-/ownergebundene Entitytabellen, Querynamen, Handles, Payloads und Masken Ausgabe: Registry-/Diagnose-JSON, opaque Handles, Komponentenbytes, Props und Schreibresultate |
| 7. Seiteneffekte und Ressourcen | Liest und schreibt Enginekomponenten unter Gates, registriert und invalidiert Sitzungen, koordiniert Dispatcherhooks und Profilzustand. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Belegt vermischt sind generischer ECS-Zugriff, konkrete Prop-/Rezeptgeometrie, Diagnostik/Discovery, Laufzeitkoordination und fachfremde Patchwrapper. Als vorhandene Aufgaben aufteilen, keine neuen Dienste voraussetzen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entitäten und Weltabfragen. Technische Kopplung: component_registry.h · registry_discovery.h · dispatcher.h · profile.h · engine_context.h · world_runtime.h · patch_runtime.h · Win32 · nlohmann JSON. Aufruferbezug: provider.cpp · world_runtime.cpp · building_input.cpp · resource_runtime.cpp · service_runtime.cpp · Host über KfcRuntimeEcs*/WorldEntity*-ABI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | ecs_read_test.cpp vollständig · prop_query_test.cpp vollständig · service_runtime_test.cpp SessionGuard-Integration |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Zugriff und Propabfragen. Neue Einordnung: Entitäten und Weltabfragen. Besitzer: KFC-Runtime. |

**Belege:** 299-532 Identität/Handles · 619-1038 Queries und Lesen · 1059-1185 Schreiben · 1301-1470 Registrierung/Discovery · 1545-1852 Lebenszyklus/Diagnose · 2081-2291 Prop-Rezepte und Transformation · 2436-2442 Patchwrapper

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/ecs/entities.cpp`, `kfc-runtime/native/ecs/components.cpp`, `kfc-runtime/native/ecs/queries.cpp`, `kfc-runtime/native/ecs/props.cpp`.

### [src/loader/runtime/native/ecs/ecs_runtime.h](../../kfc-runtime/native/ecs/entities.h)

Vollständig gelesen, 43 Zeilen. SHA-256: `41569b4cc101f6f294303eecdfc080162eb59890a5b348fdfee83fbcd4685909`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert ECS-Lebenszyklus und Sitzungswache sowie interne Identitäts- und Prop-Hilfsoperationen. Die Sitzungswache fixiert die ECS-Bindung vor originalen Engine-Sperren. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert ECS-Lebenszyklus und Sitzungswache sowie interne Identitäts- und Prop-Hilfsoperationen. Die Sitzungswache fixiert die ECS-Bindung vor originalen Engine-Sperren. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `19-32 SessionGuard`<br>`33-42 interne ECS-Operationen` |
| 4. Aufrufer und Verbraucher | provider.cpp, world_runtime.cpp, resource_runtime.cpp, service_runtime.cpp |
| 5. Verwendete Komponenten | runtime.h · engine_context.h |
| 6. Eingaben und Ausgaben | Eingaben: ECS-Sitzung, Entitätshandle und interne Aufrufargumente Ausgabe: Deklarationen der ECS-Lebenszyklus-, Handle- und Sessionguard-Schnittstelle |
| 7. Seiteneffekte und Ressourcen | Keine durch den Header selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: runtime.h · engine_context.h. Aufruferbezug: provider.cpp, world_runtime.cpp, resource_runtime.cpp, service_runtime.cpp |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | ecs_read_test.cpp und service_runtime_test.cpp verwenden SessionGuard-/ECS-Verträge |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Zugriff. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 19-32 SessionGuard · 33-42 interne ECS-Operationen

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/ecs/entities.h`, `kfc-runtime/native/ecs/components.h`, `kfc-runtime/native/ecs/queries.h`, `kfc-runtime/native/ecs/props.h`.

### [src/loader/runtime/native/ecs/registry_discovery.h](../../kfc-runtime/native/ecs/registry/discovery.h)

Vollständig gelesen, 365 Zeilen. SHA-256: `e422550cbf6e51bbc4a877bb214aa311575129253162c3be9a8402b350be4b2b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Sucht inkrementell ausschließlich in residentem privatem Speicher nach strukturell validen Komponentenregistrierungen. Budgetiert Scans, führt äquivalente Kataloge zusammen und verwirft Mehrdeutigkeit oder Änderungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Sucht inkrementell ausschließlich in residentem privatem Speicher nach strukturell validen Komponentenregistrierungen. Budgetiert Scans, führt äquivalente Kataloge zusammen und verwirft Mehrdeutigkeit oder Änderungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `47-218 Tick`<br>`280-304 Kandidaten`<br>`344-363 Veröffentlichung` |
| 4. Aufrufer und Verbraucher | ecs_runtime.cpp und externer Registry-Verifikator |
| 5. Verwendete Komponenten | component_registry.h · Win32 VirtualQuery und WorkingSetEx |
| 6. Eingaben und Ausgaben | Eingaben: Imageidentität, Reader, Typvertragvalidator, optional Region-/Residencyabfragen Ausgabe: Budgetierter Suchfortschritt und validierter eindeutiger Katalog |
| 7. Seiteneffekte und Ressourcen | Liest residenten privaten Prozessspeicher, hält Suchzustand und kopierte Kataloge. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: component_registry.h · Win32 VirtualQuery und WorkingSetEx. Aufruferbezug: ecs_runtime.cpp und externer Registry-Verifikator |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | enshrouded/native/verify-component-registry.cpp:39-120 externe Ausführung des Produktionsscanners, keine isolierte Unitprüfung im gelesenen Scope |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Registrierungsentdeckung. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 47-218 Tick · 280-304 Kandidaten · 344-363 Veröffentlichung

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/ecs/registry/discovery.h`.

### [src/loader/runtime/native/game-thread/dispatcher.cpp](../../kfc-runtime/native/execution/dispatcher.cpp)

Vollständig gelesen, 590 Zeilen. SHA-256: `7f2a76f6e47ab80c62a1e68c86ee65d31577d206b04c3473ac7d9763e7ee8c1e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Installiert geprüfte x64-Trampoline für kontinuierlichen Dispatcher, EntityManager und Weltcallbacks. Führt besitzende Jobs ausschließlich in aktiver Callbackphase mit Queue-/Zeitgrenzen aus, verwaltet Timeout/Abbruch und schreibt Code bei pausierten unbeteiligten Threads nach Byteprüfung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Installiert geprüfte x64-Trampoline für kontinuierlichen Dispatcher, EntityManager und Weltcallbacks. Führt besitzende Jobs ausschließlich in aktiver Callbackphase mit Queue-/Zeitgrenzen aus, verwaltet Timeout/Abbruch und schreibt Code bei pausierten unbeteiligten Threads nach Byteprüfung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `142-191 Codeaustausch`<br>`193-234 drain`<br>`274-400 Trampoline und Hookinstallation`<br>`409-486 Hooklebenszyklus`<br>`522-558 Invoke` |
| 4. Aufrufer und Verbraucher | ECS-, Welt- und Patchoperationen · EcsRuntime::Initialize/Shutdown |
| 5. Verwendete Komponenten | profile.h · world_runtime.h · patch_runtime.h · Win32 Threads/Events/VirtualProtect · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Profilhooks, besitzende Jobs, Ausführungskontexte und erwartete/ersetzende Codebytes Ausgabe: Aktuelle Dispatcherbereitschaft, Jobergebnis und Diagnosen |
| 7. Seiteneffekte und Ressourcen | Installiert/entfernt Spielhooks, allokiert Trampoline, hält Threads für Codeänderungen an, signalisiert Events, führt Jobs im Enginecallback aus. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Allgemeine Jobqueue und Ausführungsphase, x64-Hooktechnik und die konkrete Verdrahtung aller Weltcallbacks sind gemeinsam implementiert. Diese drei tatsächlichen Aufgaben lassen sich fachlich trennen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Engine-Ausführungsphase. Technische Kopplung: profile.h · world_runtime.h · patch_runtime.h · Win32 Threads/Events/VirtualProtect · nlohmann JSON. Aufruferbezug: ECS-, Welt- und Patchoperationen · EcsRuntime::Initialize/Shutdown |
| 11. Duplikation | parse_signature:66-78 und ausführbare Signatursuche:80-105 wiederholen patch_runtime.cpp:31-74, allocate_near:115-140 wiederholt patch_runtime.cpp:107-128. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | prop_query_test.cpp:128-140 WriteCode-Byteguard auf privater Seite · world_context_test.cpp:104-142 Managerbeobachtung/Sessionwechsel |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Ausführungsphase. Neue Einordnung: Engine-Ausführungsphase. Besitzer: KFC-Runtime. |

**Belege:** 142-191 Codeaustausch · 193-234 drain · 274-400 Trampoline und Hookinstallation · 409-486 Hooklebenszyklus · 522-558 Invoke

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** refaktorieren. Enginephasenbesitz erhalten. Gleiche PE-/Signatur-/Nahallokationsprimitive mit Patchbackend gezielt gemeinsam nutzen.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/execution/dispatcher.cpp`.

### [src/loader/runtime/native/game-thread/dispatcher.h](../../kfc-runtime/native/execution/dispatcher.h)

Vollständig gelesen, 29 Zeilen. SHA-256: `0d0b5ca932fc32a8c5136d8e0eee728447cee6109c39055ac4bda857dc8e0071`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert den an kontinuierliche Engine-Callbacks gebundenen Dispatcher mit Jobbesitz, Timeout, Kontextbeobachtung und kontrolliertem Codeaustausch. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert den an kontinuierliche Engine-Callbacks gebundenen Dispatcher mit Jobbesitz, Timeout, Kontextbeobachtung und kontrolliertem Codeaustausch. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `8-28 Dispatcher-Vertrag` |
| 4. Aufrufer und Verbraucher | ecs_runtime.cpp, world_runtime.cpp, patch_runtime.cpp |
| 5. Verwendete Komponenten | STL shared_ptr |
| 6. Eingaben und Ausgaben | Eingaben: Operation und besitzender shared_ptr-Kontext, Timeout Ausgabe: Dispatcherdeklarationen und Ausführungs-/Besitzvertrag |
| 7. Seiteneffekte und Ressourcen | Keine durch den Header selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Engine-Ausführungsphase. Technische Kopplung: STL shared_ptr. Aufruferbezug: ecs_runtime.cpp, world_runtime.cpp, patch_runtime.cpp |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | prop_query_test.cpp und world_context_test.cpp |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Ausführungsphase. Neue Einordnung: Engine-Ausführungsphase. Besitzer: KFC-Runtime. |

**Belege:** 8-28 Dispatcher-Vertrag

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/execution/dispatcher.h`.

### [src/loader/runtime/native/patch/patch_runtime.cpp](../../kfc-runtime/native/patches/patch_runtime.cpp)

Vollständig gelesen, 272 Zeilen. SHA-256: `b4dc55eff900ec737ead2131ea6f57b7ee9fd6167f680589440cc2d30d7fc7ae`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst ausschließlich profilregistrierte Bytepatches oder Detours über eindeutige Signaturen und PE-Funktionsbereiche auf. Baut nahe Trampoline, schaltet mit kontrolliertem Codeaustausch um und liefert Zuordnungsdiagnosen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst ausschließlich profilregistrierte Bytepatches oder Detours über eindeutige Signaturen und PE-Funktionsbereiche auf. Baut nahe Trampoline, schaltet mit kontrolliertem Codeaustausch um und liefert Zuordnungsdiagnosen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `130-190 resolve`<br>`194-245 Lebenszyklus und Schaltung`<br>`247-270 Diagnose` |
| 4. Aufrufer und Verbraucher | EcsRuntime::Initialize/Shutdown und native Patch-ABI in world_runtime.cpp |
| 5. Verwendete Komponenten | profile.h · dispatcher.h · Win32 PE/VirtualAlloc · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Profilregistrierte Signatur-/Funktions-/Payloadverträge und Aktivierungswunsch Ausgabe: Patchverfügbarkeit, Umschaltresultat und JSON-Diagnose |
| 7. Seiteneffekte und Ressourcen | Allokiert ausführbare Trampoline und ändert profilgebundene Spielcodebytes über den Dispatcher. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Profilgebundene Codeeingriffe. Technische Kopplung: profile.h · dispatcher.h · Win32 PE/VirtualAlloc · nlohmann JSON. Aufruferbezug: EcsRuntime::Initialize/Shutdown und native Patch-ABI in world_runtime.cpp |
| 11. Duplikation | Signaturparser/-scan und nahe Allokation wiederholen entsprechende Helfer in dispatcher.cpp. Belegt an 31-74,107-128 gegenüber dispatcher.cpp:66-140. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Profilgebundene Codeeingriffe. Neue Einordnung: Profilgebundene Codeeingriffe. Besitzer: KFC-Runtime. |

**Belege:** 130-190 resolve · 194-245 Lebenszyklus und Schaltung · 247-270 Diagnose

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** refaktorieren. Patchinhalt bleibt eigenes Backend. Nur nachgewiesen gleiche technische Primitive mit Dispatcher teilen.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/patches/patch_runtime.cpp`.

### [src/loader/runtime/native/patch/patch_runtime.h](../../kfc-runtime/native/patches/patch_runtime.h)

Vollständig gelesen, 12 Zeilen. SHA-256: `9aee1dc09bdadb2ffaa5ed9087e410b855c21cc11c953f057e030ad16774ab85`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert Initialisierung, Verfügbarkeit, Umschalten und Diagnose profilregistrierter Maschinenpatches. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert Initialisierung, Verfügbarkeit, Umschalten und Diagnose profilregistrierter Maschinenpatches. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5-11 PatchRuntime` |
| 4. Aufrufer und Verbraucher | ecs_runtime.cpp und eigene C-ABI-Exporte |
| 5. Verwendete Komponenten | stdint · string |
| 6. Eingaben und Ausgaben | Eingaben: Patchname und Zielaktivierung Ausgabe: PatchRuntime-Deklarationen |
| 7. Seiteneffekte und Ressourcen | Keine durch den Header selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Profilgebundene Codeeingriffe. Technische Kopplung: stdint · string. Aufruferbezug: ecs_runtime.cpp und eigene C-ABI-Exporte |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Profilgebundene Codeeingriffe. Neue Einordnung: Profilgebundene Codeeingriffe. Besitzer: KFC-Runtime. |

**Belege:** 5-11 PatchRuntime

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/patches/patch_runtime.h`.

### [src/loader/runtime/native/provider/engine_context.cpp](../../kfc-runtime/native/engine-context/owners.cpp)

Vollständig gelesen, 732 Zeilen. SHA-256: `1f5d638f96b071679c75ee022f2d1516e222780129189038ef9e47fa1767a180`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Validiert ein exakt zum geladenen Spiel passendes Besitz-, ECS-, Scheduler- und Schreibprofil. Verfolgt Root/Client/Controller/Scene/ExecutionOwner/Registry/EntityManager und implementiert originale nichtblockierende Shared- und Scene-Sperren samt Ressourcenworker-Lebenszeit, Untracked-Komponentenpolicy und geprüftem Trackeraufruf. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Validiert ein exakt zum geladenen Spiel passendes Besitz-, ECS-, Scheduler- und Schreibprofil. Verfolgt Root/Client/Controller/Scene/ExecutionOwner/Registry/EntityManager und implementiert originale nichtblockierende Shared- und Scene-Sperren samt Ressourcenworker-Lebenszeit, Untracked-Komponentenpolicy und geprüftem Trackeraufruf. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `123-213 Besitzerauflösung`<br>`217-426 Profilprüfung`<br>`461-569 ResourceLifeGuard`<br>`570-731 ECS-Schreibwache` |
| 4. Aufrufer und Verbraucher | ResourceRuntime::Initialize konfiguriert · ECS-, Ressourcen- und Dienstimplementierungen nutzen ResolveEcs und Guards |
| 5. Verwendete Komponenten | profile.h · Win32 SEH/Interlocked · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Exaktes Buildprofil, originale Besitzerketten und erwartete Sichten Ausgabe: Validierte Owners/EcsView und gehaltene Lebenszeit-/Scene-Gates |
| 7. Seiteneffekte und Ressourcen | Liest originale Daten, erwirbt originale Interlocked-Sperren und kann den separat geprüften Tracker aufrufen. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Profilmodellprüfung, Besitzauflösung und zwei unterschiedliche originale Sperrprotokolle liegen in einer Datei. Eine Trennung nach Besitzerauflösung, Scene-Gate und Ressourcenlebenszeit ist an vorhandenen Klassen belegbar. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Enginebesitz und Sperrprotokolle. Technische Kopplung: profile.h · Win32 SEH/Interlocked · nlohmann JSON. Aufruferbezug: ResourceRuntime::Initialize konfiguriert · ECS-, Ressourcen- und Dienstimplementierungen nutzen ResolveEcs und Guards |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | ecs_read_test.cpp Owner-/Scene-Gates · resource_runtime_test.cpp Ressourcenlebenszeit · service_runtime_test.cpp Scene-/DI-Gates |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Besitz und Lebenszeit. Neue Einordnung: Enginebesitz und Sperrprotokolle. Besitzer: KFC-Runtime. |

**Belege:** 123-213 Besitzerauflösung · 217-426 Profilprüfung · 461-569 ResourceLifeGuard · 570-731 ECS-Schreibwache

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/engine-context/owners.cpp`, `kfc-runtime/native/engine-context/resource_lifetime.cpp`, `kfc-runtime/native/engine-context/scene_gate.cpp`.

### [src/loader/runtime/native/provider/engine_context.h](../../kfc-runtime/native/engine-context/owners.h)

Vollständig gelesen, 109 Zeilen. SHA-256: `4197c13ed250464472c9b1616da22c5039c32438ffcd9b582678b2394c1456e5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beschreibt originale Engine-Besitzerketten und ECS-Sichten. Deklariert nichtblockierende Lebenszeit- und Scene-Sperren für Ressourcen, Komponenten und benannte Dienste. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beschreibt originale Engine-Besitzerketten und ECS-Sichten. Deklariert nichtblockierende Lebenszeit- und Scene-Sperren für Ressourcen, Komponenten und benannte Dienste. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-34 Owners/EcsView`<br>`48-73 ResourceLifeGuard`<br>`81-108 EcsWriteGuard` |
| 4. Aufrufer und Verbraucher | ecs_runtime.cpp, resource_runtime.cpp, service_runtime.cpp, profile.cpp |
| 5. Verwendete Komponenten | Win32-Typen indirekt in Implementierung · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Exaktes Profil, erwartete Besitzer-/ECS-Sicht und Ressourcenworker Ausgabe: Besitzer-/Layouttypen und Guard-Schnittstellen |
| 7. Seiteneffekte und Ressourcen | Keine durch die Deklarationen selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: Win32-Typen indirekt in Implementierung · nlohmann JSON. Aufruferbezug: ecs_runtime.cpp, resource_runtime.cpp, service_runtime.cpp, profile.cpp |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | ecs_read_test.cpp, resource_runtime_test.cpp und service_runtime_test.cpp |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Besitz und Lebenszeit. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 14-34 Owners/EcsView · 48-73 ResourceLifeGuard · 81-108 EcsWriteGuard

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/engine-context/owners.h`, `kfc-runtime/native/engine-context/resource_lifetime.h`, `kfc-runtime/native/engine-context/scene_gate.h`.

### [src/loader/runtime/native/provider/function_inventory.h](../../kfc-runtime/native/code-inventory/functions.h)

Vollständig gelesen, 213 Zeilen. SHA-256: `2be24c56f5aa4be33e7f16e256bb951e8bd7c2bdc4633bab601ce3e2b324ee08`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erstellt einen lesenden JSON-Katalog nativer Codebereiche aus PE-Unwinddaten und Codezeigern. Ergänzt originale Engine-Ausführungsdeskriptoren, Profiloperationen und zugeordnete Modifikatoren ohne native Aufrufbarkeit abzuleiten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erstellt einen lesenden JSON-Katalog nativer Codebereiche aus PE-Unwinddaten und Codezeigern. Ergänzt originale Engine-Ausführungsdeskriptoren, Profiloperationen und zugeordnete Modifikatoren ohne native Aufrufbarkeit abzuleiten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-57 Unwindgruppen`<br>`73-157 Zeiger und Deskriptoren`<br>`160-211 Profile und Ausgabe` |
| 4. Aufrufer und Verbraucher | provider.cpp:KfcRuntimeFunctions und ECS-Lesetest |
| 5. Verwendete Komponenten | profile.h · Win32 PE-Strukturen · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Geladenes oder kopiertes PE-Image, Profilzustand und Schedulerbelege Ausgabe: JSON-Codeinventar mit Grenzen und Profilzuordnungen |
| 7. Seiteneffekte und Ressourcen | Liest ausführbares Image und nichtausführbare Zeigerslots, keine nativen Zielaufrufe. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: profile.h · Win32 PE-Strukturen · nlohmann JSON. Aufruferbezug: provider.cpp:KfcRuntimeFunctions und ECS-Lesetest |
| 11. Duplikation | Python discovery/pe.py und discovery/functions.py lesen ebenfalls PE-Unwind-/Codezeigerbelege. Unterschiedlicher Laufzeit-/Offlinekontext und unterschiedliche Ergebnismodelle, keine Austauschbarkeit nachgewiesen. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | enshrouded/native/verify-component-registry.cpp:191-245 Produktionsinventar auf Prozesssnapshot, keine isolierte Unitprüfung im Scope |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Native Codebelege. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 14-57 Unwindgruppen · 73-157 Zeiger und Deskriptoren · 160-211 Profile und Ausgabe

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/code-inventory/functions.h`.

### [src/loader/runtime/native/provider/profile.cpp](../../kfc-runtime/native/profiles/select.cpp)

Vollständig gelesen, 437 Zeilen. SHA-256: `05f9099061e35ab510ae28a41f8b5f3ec568e2c0d10820ab58ebe90abcf9f4df`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Wählt ein eingebettetes oder externes Buildprofil anhand Zielprozess, PE-Identität, SHA-256, KFC-Version und Provenienz. Lädt ECS-Layouts, Hooks, Weltgrids, Operationen und Maschinenpatches mit Größen-, Eindeutigkeits- und Instruktionsprüfungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Wählt ein eingebettetes oder externes Buildprofil anhand Zielprozess, PE-Identität, SHA-256, KFC-Version und Provenienz. Lädt ECS-Layouts, Hooks, Weltgrids, Operationen und Maschinenpatches mit Größen-, Eindeutigkeits- und Instruktionsprüfungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `47-188 Profilwahl`<br>`191-313 Layouts und Komponenten`<br>`314-429 Weltoperationen und Patches` |
| 4. Aufrufer und Verbraucher | EcsRuntime::Initialize |
| 5. Verwendete Komponenten | profile.h · eingebettete RC-Profile · modloader-config.json Pfade/runtime.profileId · BCrypt SHA-256 · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Geladenes EXE-Image, EXE-Datei und benachbarte KFC-Datei, externe/eingebettete Profile, optionale Loaderkonfiguration Ausgabe: Ausgewählter Profilzustand mit Layouts, Hooks, Weltoperationen und Patches |
| 7. Seiteneffekte und Ressourcen | Liest Dateien/RC-Ressourcen und aktualisiert globale Profilwerte, Debugausgabe bei Fehler. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: profile.h · eingebettete RC-Profile · modloader-config.json Pfade/runtime.profileId · BCrypt SHA-256 · nlohmann JSON. Aufruferbezug: EcsRuntime::Initialize |
| 11. Duplikation | sha256_file:15-43 ist nahezu identisch mit dev-console/main.cpp:114-142 BCrypt-Dateihash. Produktions- und Entwicklerkontext sind verschieden. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. dev-console validate_profile ist eine getrennte Implementierung und kein Test dieses Load-Codes. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Build-Kompatibilitätsprofil. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 47-188 Profilwahl · 191-313 Layouts und Komponenten · 314-429 Weltoperationen und Patches

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/profiles/select.cpp`.

### [src/loader/runtime/native/provider/profile.h](../../kfc-runtime/native/profiles/contract.h)

Vollständig gelesen, 63 Zeilen. SHA-256: `445437cdd7de0c0a138def19eaa2224d68b1197b8784313ac0cfcf7be00014af`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert die geladenen Build-Kompatibilitätsdaten für Komponentenlayouts, Weltoperationen, Hooks, Grids und Patches und hält den aktuellen Profilzustand. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert die geladenen Build-Kompatibilitätsdaten für Komponentenlayouts, Weltoperationen, Hooks, Grids und Patches und hält den aktuellen Profilzustand. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `8-38 Profilstrukturen`<br>`39-62 geladener Zustand und Load` |
| 4. Aufrufer und Verbraucher | profile.cpp, ecs_runtime.cpp, dispatcher.cpp, world_runtime.cpp, patch_runtime.cpp und function_inventory.h |
| 5. Verwendete Komponenten | STL |
| 6. Eingaben und Ausgaben | Eingaben: Geladene Buildprofilwerte Ausgabe: Globale Profilstrukturen und Load-Deklaration |
| 7. Seiteneffekte und Ressourcen | Inline-Profilzustand wird von der Implementierung verändert. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: STL. Aufruferbezug: profile.cpp, ecs_runtime.cpp, dispatcher.cpp, world_runtime.cpp, patch_runtime.cpp und function_inventory.h |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Build-Kompatibilitätsprofil. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 8-38 Profilstrukturen · 39-62 geladener Zustand und Load

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/profiles/contract.h`.

### [src/loader/runtime/native/provider/provider.cpp](../../kfc-runtime/native/host/lifecycle.cpp)

Vollständig gelesen, 93 Zeilen. SHA-256: `3cef5275eea21861ec30db4cbaf49e7d5fd0511c616882723ad0a4264e4eb8b9`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Implementiert ABI-Erkennung und übergreifenden Provider-Lebenszyklus. Liefert begrenzte ausführbare Codebelege, den Funktionskatalog und zusammengeführte ECS-/Ressourcendiagnosen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Implementiert ABI-Erkennung und übergreifenden Provider-Lebenszyklus. Liefert begrenzte ausführbare Codebelege, den Funktionskatalog und zusammengeführte ECS-/Ressourcendiagnosen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-45 Code und Funktionsinventar`<br>`46-66 Lebenszyklus`<br>`67-92 Diagnosen` |
| 4. Aufrufer und Verbraucher | Host lädt KfcRuntime*-Exporte dynamisch, direkter C++-Aufrufer nicht nachgewiesen |
| 5. Verwendete Komponenten | EcsRuntime · ResourceRuntime · EngineContext · FunctionInventory · Win32 · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Hostinitialisierung/Tick/Shutdown, Code-RVA und Ausgabepuffer Ausgabe: ABI-Version, Funktionskatalog, Codebytes und Gesamtdiagnose |
| 7. Seiteneffekte und Ressourcen | Initialisiert/stopt ECS/Ressourcen/Netzwerkcallback, liest Prozesscode und schreibt Hostpuffer. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: EcsRuntime · ResourceRuntime · EngineContext · FunctionInventory · Win32 · nlohmann JSON. Aufruferbezug: Host lädt KfcRuntime*-Exporte dynamisch, direkter C++-Aufrufer nicht nachgewiesen |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Native Provider-Fassade. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 14-45 Code und Funktionsinventar · 46-66 Lebenszyklus · 67-92 Diagnosen

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/host/lifecycle.cpp`, `kfc-runtime/native/host/diagnostics.cpp`, `kfc-runtime/native/code-inventory/exports.cpp`.

### [src/loader/runtime/native/provider/steam_network.cpp](../../modules/transport/native/steam_network.cpp)

Vollständig gelesen, 547 Zeilen. SHA-256: `d5579f1425a9bf0dcf90a9c59fda1f3d834038c75b1161e597ad6a515358cd77`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet die vorhandene steam_api64.dll für NetworkingMessages ein. Liest authentifizierte aktive Dedicated-Server-Peers aus dem Spiellog, filtert optional freigegebene Clients, registriert Sessionrequests im Spielcallback, tauscht Probe/Pong aus, publiziert lokale Serveridentität und exportiert Nachrichtenversand/-empfang. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet die vorhandene steam_api64.dll für NetworkingMessages ein. Liest authentifizierte aktive Dedicated-Server-Peers aus dem Spiellog, filtert optional freigegebene Clients, registriert Sessionrequests im Spielcallback, tauscht Probe/Pong aus, publiziert lokale Serveridentität und exportiert Nachrichtenversand/-empfang. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `130-151 Steam-Schnittstellen`<br>`186-219 Peerlog`<br>`236-281 Callback`<br>`318-403 Probeprotokoll`<br>`424-546 Discovery und Transport` |
| 4. Aufrufer und Verbraucher | Native Network-ABI, Host-Aufrufe dynamisch |
| 5. Verwendete Komponenten | runtime.h · SteamAPI-Exporte · Win32 Mapping · enshrouded_server.log |
| 6. Eingaben und Ausgaben | Eingaben: Peer-IDs, Kanal-/Payloadargumente, optional erlaubte Clients und aktive Serverlogzeilen Ausgabe: Transportstatus, Identitäten, verbundene Peers, Empfangsdaten und Probestatistik |
| 7. Seiteneffekte und Ressourcen | Steam-Nachrichten/Sessionannahme, Callbackregistrierung, Spielprotokolllesen und lokales Memorymapping. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Belegt zusammengeführt sind Transportbindung, Spielprotokollauswertung zur Peerautorisation, lokales Identitätsmapping und Probeprotokoll. Diese vorhandenen Verantwortlichkeiten bieten interne Dateigrenzen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Eingebautes Modul. Technische Kopplung: runtime.h · SteamAPI-Exporte · Win32 Mapping · enshrouded_server.log. Aufruferbezug: Native Network-ABI, Host-Aufrufe dynamisch |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Steam-Netzwerktransport. Neue Einordnung: Eingebautes Modul. Besitzer: Transportmodul. |

**Belege:** 130-151 Steam-Schnittstellen · 186-219 Peerlog · 236-281 Callback · 318-403 Probeprotokoll · 424-546 Discovery und Transport

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Native Steam-Implementierung gehört zum Transportmodul. Weiterhin in dieselbe Runtime-DLL linken, Modulbesitz erzeugt keine zusätzliche DLL.

**Vorgeschlagene Zielpfade:** `modules/transport/native/steam_network.cpp`.

### [src/loader/runtime/native/resource/resource_runtime.cpp](../../kfc-runtime/native/loaded-resources/profile.cpp)

Vollständig gelesen, 1205 Zeilen. SHA-256: `61c89aada46c9d8d6b584a50682bf18ae9b5ed6f92c52269737d8a53c1afdd9f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Wählt eingebettete exakt validierte Ressourcenprofile, konfiguriert EngineContext und validiert ItemRegistry/ResourceService/Reader/Index/Chunks. Listet und liest Ressourcen mit Identitätswiederholung, projiziert skalare Stapelwerte, löst kanonische ItemIds über die originale Map auf und führt begrenzten atomaren 32-Bit-CAS mit optionalem vollständigem Sitzungssnapshot und Schutzwiederherstellung aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Wählt eingebettete exakt validierte Ressourcenprofile, konfiguriert EngineContext und validiert ItemRegistry/ResourceService/Reader/Index/Chunks. Listet und liest Ressourcen mit Identitätswiederholung, projiziert skalare Stapelwerte, löst kanonische ItemIds über die originale Map auf und führt begrenzten atomaren 32-Bit-CAS mit optionalem vollständigem Sitzungssnapshot und Schutzwiederherstellung aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `192-290 Layoutprüfung`<br>`293-457 Ressourcenbesitz und Lookup`<br>`483-525 Itemmap`<br>`673-740 Initialisierung`<br>`824-997 Batch und Itemauflösung`<br>`999-1105 CAS` |
| 4. Aufrufer und Verbraucher | provider.cpp Lebenszyklus · KfcRuntimeResource*-ABI |
| 5. Verwendete Komponenten | embedded_profile_ids.h · EngineContext · EcsRuntime::SessionGuard · profile.h · Win32 RPM/VirtualProtect/Interlocked · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Eingebettete Ressourcenprofile, Engine-Ressourcenidentitäten, Typ-/Feld-/Sitzungsverträge und erwartete/neue Skalarwerte Ausgabe: Ressourcenidentitäten/-bytes, skalare Stapelwerte, ItemResourceId und CAS-Status |
| 7. Seiteneffekte und Ressourcen | Liest originalen Ressourcenbulk, erwirbt Worker-/Scene-Gates, ändert vorübergehend private Datenseitenrechte und schreibt atomar 32-Bit-Skalare. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Ressourcenprofilwahl einschließlich globaler EngineContext-Konfiguration, Ressourcenindexzugriff, originale ItemRegistry-Relation und Schutz-/CAS-Protokoll sind in einer Datei gebündelt. Fachliche Grenzen für kleinere Dateien bestehen zwischen Profilzulassung, Index/Lesen, Itemrelation und skalarem Schreiben. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Ressourcen und skalare Änderungen. Technische Kopplung: embedded_profile_ids.h · EngineContext · EcsRuntime::SessionGuard · profile.h · Win32 RPM/VirtualProtect/Interlocked · nlohmann JSON. Aufruferbezug: provider.cpp Lebenszyklus · KfcRuntimeResource*-ABI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | resource_runtime_test.cpp vollständig. SessionGuard ist hier gestubbt und erlaubt keine sitzungsgebundenen Writes. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Ressourcen. Neue Einordnung: Live-Ressourcen und skalare Änderungen. Besitzer: KFC-Runtime. |

**Belege:** 192-290 Layoutprüfung · 293-457 Ressourcenbesitz und Lookup · 483-525 Itemmap · 673-740 Initialisierung · 824-997 Batch und Itemauflösung · 999-1105 CAS

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/loaded-resources/profile.cpp`, `kfc-runtime/native/loaded-resources/index.cpp`, `kfc-runtime/native/loaded-resources/item_registry.cpp`, `kfc-runtime/native/loaded-resources/scalar_write.cpp`.

### [src/loader/runtime/native/resource/resource_runtime.h](../../kfc-runtime/native/loaded-resources/api.h)

Vollständig gelesen, 35 Zeilen. SHA-256: `b6d8a8fa0066fe1b1bba9f63144a73105a01135b5516d165eda5bdff6f2b54be`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert profilgebundenen Ressourcenzugriff, skalare Stapellesung, ItemRegistry-Auflösung und atomaren skalaren Vergleichsaustausch mit optionaler Sitzung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert profilgebundenen Ressourcenzugriff, skalare Stapellesung, ItemRegistry-Auflösung und atomaren skalaren Vergleichsaustausch mit optionaler Sitzung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `8-35 Ressourcenvertrag` |
| 4. Aufrufer und Verbraucher | provider.cpp und C-ABI-Wrapper in resource_runtime.cpp |
| 5. Verwendete Komponenten | runtime.h |
| 6. Eingaben und Ausgaben | Eingaben: ResourceId, Typ/hash, Rootlayout, Sitzung und erwartete/neue Werte Ausgabe: Ressourcenoperationsdeklarationen |
| 7. Seiteneffekte und Ressourcen | Keine durch den Header selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: runtime.h. Aufruferbezug: provider.cpp und C-ABI-Wrapper in resource_runtime.cpp |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | resource_runtime_test.cpp |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Ressourcen. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 8-35 Ressourcenvertrag

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/loaded-resources/api.h`.

### [src/loader/runtime/native/runtime.h](../../kfc-runtime/native/abi/runtime.h)

Vollständig gelesen, 292 Zeilen. SHA-256: `23bc42531227a0ac3dcd4f04ce016b215f5983e23257bef8679d7a539be5742e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Definiert die versionierte C-ABI des nativen Laufzeitproviders mit Ressourcen-, ECS-, Welt-, Patch- und Netzwerkoperationen und ihren Ergebnisverträgen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Definiert die versionierte C-ABI des nativen Laufzeitproviders mit Ressourcen-, ECS-, Welt-, Patch- und Netzwerkoperationen und ihren Ergebnisverträgen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `25-31 ABI-Version und Lebenszyklus`<br>`71-156 Ressourcen- und Dienstverträge`<br>`158-288 ECS, Welt und Patches` |
| 4. Aufrufer und Verbraucher | provider/provider.cpp und weitere native Implementierungen exportieren diese Symbole |
| 5. Verwendete Komponenten | C-ABI, stdint/stddef |
| 6. Eingaben und Ausgaben | Eingaben: ABI-Argumente und Host-Lebenszyklusaufrufe Ausgabe: C-Deklarationen, Structlayouts und Status-/Ergebnisverträge |
| 7. Seiteneffekte und Ressourcen | Keine zur Laufzeit, reiner Deklarationsheader. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: C-ABI, stdint/stddef. Aufruferbezug: provider/provider.cpp und weitere native Implementierungen exportieren diese Symbole |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Native Laufzeitschnittstelle. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 25-31 ABI-Version und Lebenszyklus · 71-156 Ressourcen- und Dienstverträge · 158-288 ECS, Welt und Patches

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/abi/runtime.h`.

### [src/loader/runtime/native/service/service_runtime.cpp](../../kfc-runtime/native/named-services/registry.cpp)

Vollständig gelesen, 258 Zeilen. SHA-256: `aa32501a2f6d5c347ca9ecd61d73ec896f0cac6224caf546ad54aeb84a4bd78c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst originale benannte Plain-Data-DI-Dienste anhand freigegebener Profilverträge auf. Prüft Registryname, Typ, Hashmap, doppelte Namen und mutable Systembindung. Liest oder aktualisiert maskiert unter ECS-Sitzungs- und originaler Scene-Sperre mit Rücklesung und Sperre bei unklarem Ergebnis. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst originale benannte Plain-Data-DI-Dienste anhand freigegebener Profilverträge auf. Prüft Registryname, Typ, Hashmap, doppelte Namen und mutable Systembindung. Liest oder aktualisiert maskiert unter ECS-Sitzungs- und originaler Scene-Sperre mit Rücklesung und Sperre bei unklarem Ergebnis. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `50-95 Vertragswahl`<br>`103-182 Besitzauflösung`<br>`199-258 Lesen und Vergleichsaustausch` |
| 4. Aufrufer und Verbraucher | KfcRuntimeServiceRead/KfcRuntimeServiceCompareExchange dynamische ABI |
| 5. Verwendete Komponenten | runtime.h · EcsRuntime::SessionGuard · EngineContext::ServiceLayout/EcsWriteGuard · Win32 · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Benannter Dienstvertrag, Typ/Größe/Sitzung, erwarteter Payload, Maske und Zielbytes Ausgabe: Vollständige Dienstkopie oder Vergleichsaustauschstatus |
| 7. Seiteneffekte und Ressourcen | Liest/schreibt originale DI-Plaindaten unter Session-/Scene-Gates, sperrt Writes bei unklarem Ergebnis. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: runtime.h · EcsRuntime::SessionGuard · EngineContext::ServiceLayout/EcsWriteGuard · Win32 · nlohmann JSON. Aufruferbezug: KfcRuntimeServiceRead/KfcRuntimeServiceCompareExchange dynamische ABI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | service_runtime_test.cpp vollständig |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Benannte Engine-Dienstdaten. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 50-95 Vertragswahl · 103-182 Besitzauflösung · 199-258 Lesen und Vergleichsaustausch

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/named-services/registry.cpp`, `kfc-runtime/native/named-services/plain_data.cpp`.

### [src/loader/runtime/native/world/building_input.cpp](../../kfc-runtime/native/player-input/building.cpp)

Vollständig gelesen, 180 Zeilen. SHA-256: `32afc8e9f7f86db1d617e7c37d08311eb640356c41852b03a6ef00faabf587bd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verwaltet einen einzelnen zeitbegrenzten Bau-Eingabebefehl. Prüft exakten Clientbuild und Spielerkomponentenadresse, schreibt normale Auswahl-/Tasten-/Cursorfelder im originalen Cursorcallback und löst gedrückte Eingaben später wieder. Exportiert Queue-, Status- und Abbruchfunktionen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verwaltet einen einzelnen zeitbegrenzten Bau-Eingabebefehl. Prüft exakten Clientbuild und Spielerkomponentenadresse, schreibt normale Auswahl-/Tasten-/Cursorfelder im originalen Cursorcallback und löst gedrückte Eingaben später wieder. Exportiert Queue-, Status- und Abbruchfunktionen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `32-111 Verfügbarkeit und Anwendung`<br>`117-129 Callback`<br>`134-180 ABI-Befehle` |
| 4. Aufrufer und Verbraucher | WorldRuntime::OnCursorUpdate und KfcRuntimeWorldBuildingInput*-ABI |
| 5. Verwendete Komponenten | world_runtime.h · profile.h · EcsRuntime::MatchesComponentAddress · Win32 SEH |
| 6. Eingaben und Ausgaben | Eingaben: Spielerhandle, Auswahl-/Baubefehl, Transformation und originaler Cursorcallback Ausgabe: Befehls-ID und Status/Abbruchergebnis |
| 7. Seiteneffekte und Ressourcen | Ändert freigegebene Clientinput-/Cursorfelder im Spielcallback. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: world_runtime.h · profile.h · EcsRuntime::MatchesComponentAddress · Win32 SEH. Aufruferbezug: WorldRuntime::OnCursorUpdate und KfcRuntimeWorldBuildingInput*-ABI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | building_input_test.cpp vollständig · enshrouded/verify-building-input.py externer statischer Buildvertrag |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Client-Baueingabe. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 32-111 Verfügbarkeit und Anwendung · 117-129 Callback · 134-180 ABI-Befehle

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/player-input/building.cpp`.

### [src/loader/runtime/native/world/building_input.h](../../kfc-runtime/native/player-input/building.h)

Vollständig gelesen, 9 Zeilen. SHA-256: `6fcccdf2c01314115f5d9f7a85581e84ca4f956252b8f3e35f1447eb7d75adaf`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert die Beobachtung des validierten Client-Cursor-Callbacks, Verfügbarkeit und Zurücksetzen der Bau-Eingabeansteuerung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert die Beobachtung des validierten Client-Cursor-Callbacks, Verfügbarkeit und Zurücksetzen der Bau-Eingabeansteuerung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `4-8 BuildingInput` |
| 4. Aufrufer und Verbraucher | world_runtime.cpp und building_input.cpp |
| 5. Verwendete Komponenten | stdint |
| 6. Eingaben und Ausgaben | Eingaben: Livecursor und ExecutionView aus validiertem Hook Ausgabe: Interne BuildingInput-Deklarationen |
| 7. Seiteneffekte und Ressourcen | Keine durch den Header selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: stdint. Aufruferbezug: world_runtime.cpp und building_input.cpp |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | building_input_test.cpp |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Client-Baueingabe. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 4-8 BuildingInput

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/player-input/building.h`.

### [src/loader/runtime/native/world/world_runtime.cpp](../../kfc-runtime/native/engine-context/session.cpp)

Vollständig gelesen, 1178 Zeilen. SHA-256: `09b1a63a7d0e58ce154550d9c9aa1e1e3c08da36ff553c853c5d7fc749833808`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beobachtet direkte und lesende Client-Weltkontexte über originale Hooks, erfasst Cursorzustand und führt Voxelzugriff sowie profilgebundene Spawn-/Place-/Destroy-/Finish-Operationen aus. Prüft Kontextalter und Generation, sichert Voxeländerungen mit Rücklesung/Rollback, wartet auf Hookausführung und verifiziert Entitätswirkung im ECS. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beobachtet direkte und lesende Client-Weltkontexte über originale Hooks, erfasst Cursorzustand und führt Voxelzugriff sowie profilgebundene Spawn-/Place-/Destroy-/Finish-Operationen aus. Prüft Kontextalter und Generation, sichert Voxeländerungen mit Rücklesung/Rollback, wartet auf Hookausführung und verifiziert Entitätswirkung im ECS. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `130-233 Weltauflösung`<br>`308-539 Entitätsaufträge`<br>`542-621 Voxeloperation`<br>`685-939 Kontexte/Callbacks/Cursor`<br>`941-1137 öffentliche Operationen` |
| 4. Aufrufer und Verbraucher | dispatcher.cpp Hookcallbacks · ECS-Diagnosen · Host über KfcRuntimeWorld*-ABI |
| 5. Verwendete Komponenten | profile.h · dispatcher.h · ecs_runtime.h · building_input.h · Win32 Events/Interlocked/RPM · nlohmann JSON |
| 6. Eingaben und Ausgaben | Eingaben: Originale Weltframes und profilgebundene Cursor-/Voxel-/Entityaufrufe Ausgabe: Kontextart/-status, Cursor-Snapshot, Voxelwerte, Entitätsergebnis und Outcome |
| 7. Seiteneffekte und Ressourcen | Ruft profilvalidierte Enginefunktionen auf, verändert Voxel/Entitätsbauqueues, wartet auf Hooks und prüft Wirkung. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Kontextbeobachtung und Cursor-Mailbox, Voxelzugriff, Entitätsbauaufträge samt zwei Queueprotokollen und Wirkungsprüfung, Diagnose und ABI-Fassade sind gemeinsam implementiert. Aufteilung nach diesen vorhandenen Weltverantwortlichkeiten ist belegbar. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Weltkontext und Operationen. Technische Kopplung: profile.h · dispatcher.h · ecs_runtime.h · building_input.h · Win32 Events/Interlocked/RPM · nlohmann JSON. Aufruferbezug: dispatcher.cpp Hookcallbacks · ECS-Diagnosen · Host über KfcRuntimeWorld*-ABI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | world_context_test.cpp vollständig · enshrouded/verify-world-event-abi.py und verify-client-voxel-context.py prüfen externe Buildfakten |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Weltzugriff. Neue Einordnung: Weltkontext und Operationen. Besitzer: KFC-Runtime. |

**Belege:** 130-233 Weltauflösung · 308-539 Entitätsaufträge · 542-621 Voxeloperation · 685-939 Kontexte/Callbacks/Cursor · 941-1137 öffentliche Operationen

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/engine-context/session.cpp`, `kfc-runtime/native/voxels/cells.cpp`, `kfc-runtime/native/voxels/grids.cpp`, `kfc-runtime/native/building/placement.cpp`, `kfc-runtime/native/building/removal.cpp`, `kfc-runtime/native/building/spawn.cpp`, `kfc-runtime/native/player-input/cursor.cpp`.

### [src/loader/runtime/native/world/world_runtime.h](../../kfc-runtime/native/engine-context/session.h)

Vollständig gelesen, 36 Zeilen. SHA-256: `67b804af7488d2604f90612d95461d6fcdbae4213fcb0a2b60885ff0ad24a924`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Deklariert Weltkontextbeobachtung aus originalen Hooks, Cursor-Lesung, Voxel- und Entitätsoperationen sowie deren Status. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Deklariert Weltkontextbeobachtung aus originalen Hooks, Cursor-Lesung, Voxel- und Entitätsoperationen sowie deren Status. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `8-21 Kontext und Hooks`<br>`22-35 Voxel und Entitäten` |
| 4. Aufrufer und Verbraucher | dispatcher.cpp, ecs_runtime.cpp und world_runtime.cpp |
| 5. Verwendete Komponenten | ecs_runtime.h |
| 6. Eingaben und Ausgaben | Eingaben: Originale Hookframes, Cursor-/Voxel-/Entitätsargumente Ausgabe: Weltoperations- und Kontextdeklarationen |
| 7. Seiteneffekte und Ressourcen | Keine durch den Header selbst. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten. Befunde: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Live-Spielzugriff. Technische Kopplung: ecs_runtime.h. Aufruferbezug: dispatcher.cpp, ecs_runtime.cpp und world_runtime.cpp |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aktiv in kfc-runtime oder von aktiven nativen Implementierungen eingebunden, CMakeLists.txt:86-122. |
| 13. Risiko bei Änderung oder Verschiebung | Includepfade und CMake-Includeverzeichnisse müssen mitgezogen werden. Bei runtime.h zusätzlich externe ABI-Bindungen und Structlayout unverändert prüfen. |
| 14. Vorhandener Testschutz | world_context_test.cpp |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Weltzugriff. Neue Einordnung: Live-Spielzugriff. Besitzer: KFC-Runtime. |

**Belege:** 8-21 Kontext und Hooks · 22-35 Voxel und Entitäten

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** aufteilen. Nach realem Datenbesitz ordnen: ECS-Komponenten/Props, geladene KFC-Ressourcen, benannte DI-Daten, Voxelzellen, Bauereignisse oder Client-Eingaben. Bestehende ABI, Lebensdauer und Guards erhalten.

**Vorgeschlagene Zielpfade:** `kfc-runtime/native/engine-context/session.h`, `kfc-runtime/native/voxels/api.h`, `kfc-runtime/native/building/api.h`, `kfc-runtime/native/player-input/cursor.h`.

### [src/loader/runtime/profile-tools/dev/examples/registry-overview.lua](../../examples/runtime/registry-overview.lua)

Vollständig gelesen, 41 Zeilen. SHA-256: `e260262d51ccd49cca966564f9b2abd96dd044a7338254d5551d6bb93d7c38b3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beispielmod liest einmal nach verfügbarem Provider den vollständigen Komponentenregistrierungskatalog, zählt Layoutarten, zeigt Namensauflösung und protokolliert native Callbackbindungen ohne harte Komponentenindizes. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beispielmod liest einmal nach verfügbarem Provider den vollständigen Komponentenregistrierungskatalog, zählt Layoutarten, zeigt Namensauflösung und protokolliert native Callbackbindungen ohne harte Komponentenindizes. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5-40 on_update` |
| 4. Aufrufer und Verbraucher | Manuell als Mod-Einstieg installierbar, konkretes Manifest im gelesenen Scope nicht nachgewiesen |
| 5. Verwendete Komponenten | runtime.ecs · runtime.types · runtime.functions · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Runtime-Registry-/Typ-/Funktions-API des Hosts. Ausgabe: Einmalige Registryübersicht im Modlog. |
| 7. Seiteneffekte und Ressourcen | Protokolliert über shroudforge.log. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modentwicklung. Technische Kopplung: runtime.ecs · runtime.types · runtime.functions · shroudforge.log. Aufruferbezug: Manuell als Mod-Einstieg installierbar, konkretes Manifest im gelesenen Scope nicht nachgewiesen |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Explizites Entwicklerbeispiel. Konkrete Installation oder automatischer Aufruf nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Beispielinstallations-/Einstiegspfad bei vorhandenen Nutzern, im Scope kein Manifest nachgewiesen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-API-Beispiel. Neue Einordnung: Modentwicklung. Besitzer: Runtime-Beispiele. |

**Belege:** 5-40 on_update

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `examples/runtime/registry-overview.lua`.

### [src/loader/runtime/profile-tools/dev/tools/dev-console/live-ecs-capture.cpp](../../tools/runtime-profiles/console/live-ecs-capture.cpp)

Vollständig gelesen, 171 Zeilen. SHA-256: `1141731abe90ac52e7c618c86db368713416637ec43ce766c8eec3d2affc2892`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Entdeckt zwei benannte Reflectiondeskriptoren im geladenen Image, sucht daraus große mögliche ECS-Zeigertabellen und gibt Tabellen-/Metadaten-/Indexstatistik und TSV-Zeilen als Entwicklungscapture aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Entdeckt zwei benannte Reflectiondeskriptoren im geladenen Image, sucht daraus große mögliche ECS-Zeigertabellen und gibt Tabellen-/Metadaten-/Indexstatistik und TSV-Zeilen als Entwicklungscapture aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `39-63 Metadaten`<br>`79-170 Scan/Ausgabe` |
| 4. Aufrufer und Verbraucher | Native CMake kfc-runtime-capture-ecs, dev-console import-ecs-capture konsumiert Ausgabe |
| 5. Verwendete Komponenten | Win32 RPM/VirtualQueryEx/Toolhelp |
| 6. Eingaben und Ausgaben | Eingaben: PID Ausgabe: Konsolen- und TSV-Capture möglicher Komponentenindizes |
| 7. Seiteneffekte und Ressourcen | Lesender Prozessspeicherscan. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32 RPM/VirtualQueryEx/Toolhelp. Aufruferbezug: Native CMake kfc-runtime-capture-ecs, dev-console import-ecs-capture konsumiert Ausgabe |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Tabellensondierung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilkonsole. |

**Belege:** 39-63 Metadaten · 79-170 Scan/Ausgabe

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/console/live-ecs-capture.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/dev-console/main.cpp](../../tools/runtime-profiles/console/main.cpp)

Vollständig gelesen, 970 Zeilen. SHA-256: `a4e8b53568bb0010d2bf662b2e5ff3bb01a36b7c943ce59857c43a842502c8d7`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bietet native Entwickler-CLI für PE-/Build-/Prozess-/Providerinspektion, heuristische Funktionssuche und Signaturscan, Captureimport, Profilstrukturprüfung, Entwurfserzeugung und interaktive Provenienzfreigabe. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bietet native Entwickler-CLI für PE-/Build-/Prozess-/Providerinspektion, heuristische Funktionssuche und Signaturscan, Captureimport, Profilstrukturprüfung, Entwurfserzeugung und interaktive Provenienzfreigabe. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `46-155 PE/Hash`<br>`274-367 Profilprüfung`<br>`369-716 Capture-/Scanbefehle`<br>`719-855 Provider/Freigabe`<br>`858-970 Generierung/Dispatch` |
| 4. Aufrufer und Verbraucher | Native CMake kfc-runtime-dev, manuelle CLI |
| 5. Verwendete Komponenten | Win32/BCrypt · runtime.h · nlohmann JSON · Profile/Captures |
| 6. Eingaben und Ausgaben | Eingaben: CLI-Subkommando, EXE/Provider/Profile/Captures und Optionen Ausgabe: JSON-Inventare, Scans, Komponentenkataloge und Profilentwürfe/-freigaben |
| 7. Seiteneffekte und Ressourcen | Liest Dateien/Prozessmodule, lädt Provider-DLL und ruft ABI ab, kann Profile/Captures schreiben und nach zwei VERIFY-Eingaben Provenienz ändern. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: PE-/Hash-/Signaturtechnik, Prozess-/Providerinspektion, Captureimport und Profilgenerierung/-freigabe sind in einer CLI-Datei gebündelt. Vorhandene Subkommandos bieten klare Aufteilungsgrenzen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32/BCrypt · runtime.h · nlohmann JSON · Profile/Captures. Aufruferbezug: Native CMake kfc-runtime-dev, manuelle CLI |
| 11. Duplikation | BCrypt-Dateihash:114-142 wiederholt profile.cpp:15-43. Profilstrukturprüfung:274-367 wiederholt teilweise Runtimeprofilprüfungen in profile.cpp, mit abweichenden Regeln. Signaturscan wird zusätzlich innerhalb select_build_command:437-455 und scan_command:477-497 nahezu gleich umgesetzt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Profilentwicklungs-Konsole. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilkonsole. |

**Belege:** 46-155 PE/Hash · 274-367 Profilprüfung · 369-716 Capture-/Scanbefehle · 719-855 Provider/Freigabe · 858-970 Generierung/Dispatch

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/console/main.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/audit-mod-origins.py](../../tools/runtime-profiles/audit-mod-origins.py)

Vollständig gelesen, 205 Zeilen. SHA-256: `ac6a18c2c80eb6743e22398b0fececd0211cac77ecd5c60caa07df4d82ec3fd2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft für gebündelte Mods exakte EXE-/Profil-/Callgraph-/Ressourcenbelege. Validiert Patchpayloads, Funktionsbesitz, Argumentpositionen, Engine-Namen, Attribute und Aufruferpfade und gleicht tatsächliche Modmanifeste/Lua-Einstiege gegen diese Herkunft ab. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft für gebündelte Mods exakte EXE-/Profil-/Callgraph-/Ressourcenbelege. Validiert Patchpayloads, Funktionsbesitz, Argumentpositionen, Engine-Namen, Attribute und Aufruferpfade und gleicht tatsächliche Modmanifeste/Lua-Einstiege gegen diese Herkunft ab. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `19-35 Payload`<br>`38-150 Herkunft`<br>`153-184 audit_packages`<br>`187-205 CLI` |
| 4. Aufrufer und Verbraucher | run-runtime-discovery.ps1 und test_mod_origins.py |
| 5. Verwendete Komponenten | capstone · PE · discovery.systems · Modmanifeste/Lua · Ressourcenexport |
| 6. Eingaben und Ausgaben | Eingaben: EXE, Profil, Funktions- und Ressourcenexport, Modordner Ausgabe: JSON-Belegbericht je gebündeltem Mod |
| 7. Seiteneffekte und Ressourcen | Liest EXE/Profile/Modquellen, schreibt Bericht, führt Spielcode nicht aus. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: capstone · PE · discovery.systems · Modmanifeste/Lua · Ressourcenexport. Aufruferbezug: run-runtime-discovery.ps1 und test_mod_origins.py |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_mod_origins.py:14-31 schützt validate_payload, keine Gesamtworkflowprüfung. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modherkunftsprüfung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 19-35 Payload · 38-150 Herkunft · 153-184 audit_packages · 187-205 CLI

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/audit-mod-origins.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/capture-factory-state.py](../../tools/runtime-profiles/capture-factory-state.py)

Vollständig gelesen, 149 Zeilen. SHA-256: `2b70c8dba7b8b0dbee13fd721c5ab48b64c9ad00e14c0bbc1a93bd3bbbdfff8d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erfasst lesend FactoryStation-Zustände, Inventarslots und verknüpfte Inventare aus einem revalidierten Live-Registry-/Managercapture. Nutzt Reflectionfelder und wiederholte Entitäts-/Payloadlesung und kennzeichnet Beobachtungscharakter. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erfasst lesend FactoryStation-Zustände, Inventarslots und verknüpfte Inventare aus einem revalidierten Live-Registry-/Managercapture. Nutzt Reflectionfelder und wiederholte Entitäts-/Payloadlesung und kennzeichnet Beobachtungscharakter. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `17-110 capture`<br>`113-149 CLI` |
| 4. Aufrufer und Verbraucher | Manuelle CLI, weiterer Aufrufer nicht nachgewiesen |
| 5. Verwendete Komponenten | Process · PE · discovery.components |
| 6. Eingaben und Ausgaben | Eingaben: EXE, PID, Reflection und Registrycapture oder Manageradressen Ausgabe: JSON mit Stations-/Inventarzuständen und Managerbelegen |
| 7. Seiteneffekte und Ressourcen | Lesender Prozesszugriff, schreibt Capturedatei. |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Process · PE · discovery.components. Aufruferbezug: Manuelle CLI, weiterer Aufrufer nicht nachgewiesen |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Produktionsstationsdiagnose. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 17-110 capture · 113-149 CLI

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/capture-factory-state.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/compare-recipe-resources.py](../../tools/runtime-profiles/compare-recipe-resources.py)

Vollständig gelesen, 83 Zeilen. SHA-256: `da53ecc0bd42942c366a7dc1976f592a69424af4665b1e403406b6e2fd0976fd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Vergleicht exportierte RecipeRegistryResource-Rezepte von Client, Server und optional Backups anhand originaler RecipeIds. Meldet Feldänderungen und exakte Zeitverhältnisse mit Quelldateihashes. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Vergleicht exportierte RecipeRegistryResource-Rezepte von Client, Server und optional Backups anhand originaler RecipeIds. Meldet Feldänderungen und exakte Zeitverhältnisse mit Quelldateihashes. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10-52 load/compare`<br>`55-83 CLI` |
| 4. Aufrufer und Verbraucher | Manuelle CLI, weiterer Aufrufer nicht nachgewiesen |
| 5. Verwendete Komponenten | json · Fraction · Counter |
| 6. Eingaben und Ausgaben | Eingaben: Frische Ressourcenexporte für Client/Server/Backups Ausgabe: JSON-Differenzbericht und Konsolensummen |
| 7. Seiteneffekte und Ressourcen | Liest Exportdateien, schreibt Bericht. |
| 8. Art der Verantwortung | fachlich |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: json · Fraction · Counter. Aufruferbezug: Manuelle CLI, weiterer Aufrufer nicht nachgewiesen |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Rezeptressourcenvergleich. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 10-52 load/compare · 55-83 CLI

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/compare-recipe-resources.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discover-components.py](../../tools/runtime-profiles/discover-components.py)

Vollständig gelesen, 58 Zeilen. SHA-256: `4eee085d9618c3d257ee88f56c2f2d40a44adad98b7db45c517f46c38a94bd40`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | CLI validiert Captureidentität, entdeckt Registrierungen im gleichen laufenden Prozess oder exportiert bestehende validierte Captures ohne Prozesszugriff als Komponentenprofilkatalog. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: CLI validiert Captureidentität, entdeckt Registrierungen im gleichen laufenden Prozess oder exportiert bestehende validierte Captures ohne Prozesszugriff als Komponentenprofilkatalog. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10-54 main` |
| 4. Aufrufer und Verbraucher | Manuelle CLI, kein weiterer Aufrufer im gelesenen Code nachgewiesen |
| 5. Verwendete Komponenten | PE · discovery.components · Process |
| 6. Eingaben und Ausgaben | Eingaben: EXE, Captureordner, Exportoptionen und Profil-ID Ausgabe: components.json und optional Komponentenprofilkatalog |
| 7. Seiteneffekte und Ressourcen | Optional lesender Prozesszugriff und Schreiben der Berichtdateien. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · discovery.components · Process. Aufruferbezug: Manuelle CLI, kein weiterer Aufrufer im gelesenen Code nachgewiesen |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Katalog-CLI. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 10-54 main

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discover-components.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discover-runtime.py](../../tools/runtime-profiles/discover-runtime.py)

Vollständig gelesen, 108 Zeilen. SHA-256: `eb07c096588600455e623cd5342d38f27b5e50778c63f140f40c4ef344831f4c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | CLI orchestriert reproduzierbare EXE-/Reflection-/Funktionsaufnahme für Client und Server und optional lesenden Live- und Registrycapture. Verknüpft Berichte anhand exakter Imageidentität und liest Buildversion aus benachbarter KFC-Datei. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: CLI orchestriert reproduzierbare EXE-/Reflection-/Funktionsaufnahme für Client und Server und optional lesenden Live- und Registrycapture. Verknüpft Berichte anhand exakter Imageidentität und liest Buildversion aus benachbarter KFC-Datei. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `19-32 build_version`<br>`35-108 main` |
| 4. Aufrufer und Verbraucher | run-runtime-discovery.ps1 |
| 5. Verwendete Komponenten | discovery.pe/reflection/functions/live/components/systems · argparse |
| 6. Eingaben und Ausgaben | Eingaben: EXE, optional Profil/PID, Ausgabeordner, Functions-Schalter Ausgabe: reflection.json, profile-audit.json, live.json, components.json, functions.json, summary.json |
| 7. Seiteneffekte und Ressourcen | Liest EXE/KFC/optional Prozess, erzeugt Capturedateien. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: discovery.pe/reflection/functions/live/components/systems · argparse. Aufruferbezug: run-runtime-discovery.ps1 |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Aufnahme-CLI. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 19-32 build_version · 35-108 main

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discover-runtime.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/__init__.py](../../tools/runtime-profiles/discovery/__init__.py)

Vollständig gelesen, 1 Zeilen. SHA-256: `133e41f4f0e3fcf14e810a1ace01c757f3221f04e9b3117da450cc48b244822a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Kennzeichnet das Python-Paket der lesenden buildbezogenen Discoverywerkzeuge. Enthält ausschließlich den Paketdocstring. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Kennzeichnet das Python-Paket der lesenden buildbezogenen Discoverywerkzeuge. Enthält ausschließlich den Paketdocstring. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `1 Paketdocstring` |
| 4. Aufrufer und Verbraucher | Python-Importe discovery.* |
| 5. Verwendete Komponenten | Keine zusätzliche Angabe im Leseprotokoll. |
| 6. Eingaben und Ausgaben | Eingaben: Python-Paketimport. Ausgabe: Paketdocstring. |
| 7. Seiteneffekte und Ressourcen | Keine. |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Keine zusätzliche Angabe im Leseprotokoll.. Aufruferbezug: Python-Importe discovery.* |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Paketmarker der nachgewiesenen discovery-Importe. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Discovery-Paket. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 1 Paketdocstring

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/__init__.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/components.py](../../tools/runtime-profiles/discovery/components.py)

Vollständig gelesen, 253 Zeilen. SHA-256: `4e5ce6c353af94d2eba958e08860d946d04004db32ff1baa7a1d4c475a2eaaca`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Findet originale Komponentenregistries und EntityManager aus laufenden Speicherbelegen. Validiert Runtime-/Template-Deskriptoren und Parallelarrays, prüft archetypische Strides und exportiert erst aus validierten Captures einen Buildkatalog. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Findet originale Komponentenregistries und EntityManager aus laufenden Speicherbelegen. Validiert Runtime-/Template-Deskriptoren und Parallelarrays, prüft archetypische Strides und exportiert erst aus validierten Captures einen Buildkatalog. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `29-100 read_registry`<br>`103-188 export_catalog`<br>`191-253 Manager und Entdeckung` |
| 4. Aufrufer und Verbraucher | discover-runtime.py, discover-components.py, capture-factory-state.py |
| 5. Verwendete Komponenten | Process · PE · numpy · struct |
| 6. Eingaben und Ausgaben | Eingaben: Prozess, PE, Reflection, Livecapture oder validierter Registrybericht Ausgabe: Registry-/Managerbelege und Komponentenprofilkatalog |
| 7. Seiteneffekte und Ressourcen | Lesender Prozessspeicherscan, keine Engineaufrufe. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Process · PE · numpy · struct. Aufruferbezug: discover-runtime.py, discover-components.py, capture-factory-state.py |
| 11. Duplikation | read_registry:29-100 implementiert denselben Registry-v1-Vertrag wie native/ecs/component_registry.h:40-174 in Python. Tool- und Produktkontext verschieden. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_components.py:58-91 schützt read_registry. export_catalog/discover/validate_manager nicht durch diesen Test abgedeckt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Registrierungsanalyse. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 29-100 read_registry · 103-188 export_catalog · 191-253 Manager und Entdeckung

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/components.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/functions.py](../../tools/runtime-profiles/discovery/functions.py)

Vollständig gelesen, 367 Zeilen. SHA-256: `e26e38bc634d8d63ec01df19aa19f966e96f33b8b76424f884727ca55afb60bb`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Disassembliert compilerbeschriebene native Bereiche, sammelt direkte Sprünge, Metadaten- und Hashbelege, normalisiert Adressoperanden, gruppiert Unwindfragmente, auditiert Profilbyteanker und erzeugt eindeutige instruktionsgebundene Signaturen ohne ABI-Freigabe. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Disassembliert compilerbeschriebene native Bereiche, sammelt direkte Sprünge, Metadaten- und Hashbelege, normalisiert Adressoperanden, gruppiert Unwindfragmente, auditiert Profilbyteanker und erzeugt eindeutige instruktionsgebundene Signaturen ohne ABI-Freigabe. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `20-194 generate_signature`<br>`197-217 Gruppierung`<br>`220-267 profile_audit`<br>`270-367 discover` |
| 4. Aufrufer und Verbraucher | discover-runtime.py, inspect-runtime.py, matching.py |
| 5. Verwendete Komponenten | PE · capstone · numpy · hashlib |
| 6. Eingaben und Ausgaben | Eingaben: PE, Reflection und optionales Buildprofil Ausgabe: Codeinventar, Coverage, Querverweise und Signaturkandidaten |
| 7. Seiteneffekte und Ressourcen | Lesende Disassemblierung, keine Codeausführung. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · capstone · numpy · hashlib. Aufruferbezug: discover-runtime.py, inspect-runtime.py, matching.py |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py, konkrete Import-/Assertionfälle vollständig gelesen. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Native Codeanalyse. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 20-194 generate_signature · 197-217 Gruppierung · 220-267 profile_audit · 270-367 discover

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/functions.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/live.py](../../tools/runtime-profiles/discovery/live.py)

Vollständig gelesen, 267 Zeilen. SHA-256: `fcb3548ee5330490ad1775188399a1f4c7859d6c462fbf5f58998caf712afe13`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Öffnet einen Windowsprozess ausschließlich zum Lesen, prüft EXE-Pfad/SHA-/PE-Identität und erfasst validierte Reflectionzeiger und mögliche ECS-Tabellen. Wiederholt Kandidatenlesungen und dokumentiert fehlende Snapshotatomarität. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Öffnet einen Windowsprozess ausschließlich zum Lesen, prüft EXE-Pfad/SHA-/PE-Identität und erfasst validierte Reflectionzeiger und mögliche ECS-Tabellen. Wiederholt Kandidatenlesungen und dokumentiert fehlende Snapshotatomarität. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `23-88 Process`<br>`91-267 capture` |
| 4. Aufrufer und Verbraucher | discover-runtime.py, discover-components.py, capture-factory-state.py und Verifikationsskripte |
| 5. Verwendete Komponenten | ctypes Win32 · numpy · PE · hashlib |
| 6. Eingaben und Ausgaben | Eingaben: PID, exakte EXE, Reflection und optionales Reviewprofil Ausgabe: Livebericht mit Regionen, Zeigerläufen, Kandidatentabellen und Grenzen |
| 7. Seiteneffekte und Ressourcen | Öffnet Prozesshandle mit QUERY_INFORMATION/VM_READ und liest Speicher. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: ctypes Win32 · numpy · PE · hashlib. Aufruferbezug: discover-runtime.py, discover-components.py, capture-factory-state.py und Verifikationsskripte |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lesende Prozessaufnahme. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 23-88 Process · 91-267 capture

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/live.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/matching.py](../../tools/runtime-profiles/discovery/matching.py)

Vollständig gelesen, 529 Zeilen. SHA-256: `ea24df50246c58a588f3b908d409f04455bf9f228fcd32cc6015ae783c118cf3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Indiziert native Funktionsgruppen, Code-/Deskriptor-/Callbackbelege und Callgraph. Bewertet buildübergreifende Kandidaten mit erhaltenen Konflikten und verfolgt frische Byteanker bis zu originalen Prozeduren und benannten Aufrufern, ohne ABI oder Wirkung zu behaupten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Indiziert native Funktionsgruppen, Code-/Deskriptor-/Callbackbelege und Callgraph. Bewertet buildübergreifende Kandidaten mit erhaltenen Konflikten und verfolgt frische Byteanker bis zu originalen Prozeduren und benannten Aufrufern, ohne ABI oder Wirkung zu behaupten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `6-110 FunctionIndex/Anker`<br>`195-373 Ranking/Vergleich`<br>`376-529 trace_anchors` |
| 4. Aufrufer und Verbraucher | inspect-runtime.py und test_discovery.py |
| 5. Verwendete Komponenten | discovery.functions · capstone · bisect · collections |
| 6. Eingaben und Ausgaben | Eingaben: Quell-/Zielcaptures, Profileanker, Scans und optionale Registrycallbacks Ausgabe: Bewertete Kandidaten, statische Aufruferpfade und Beleggrenzen |
| 7. Seiteneffekte und Ressourcen | Nur Analyse, liest PE-Code für frische Anker. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: discovery.functions · capstone · bisect · collections. Aufruferbezug: inspect-runtime.py und test_discovery.py |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py, konkrete Import-/Assertionfälle vollständig gelesen. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Buildübergreifende Codezuordnung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 6-110 FunctionIndex/Anker · 195-373 Ranking/Vergleich · 376-529 trace_anchors

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/matching.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/pe.py](../../tools/runtime-profiles/discovery/pe.py)

Vollständig gelesen, 126 Zeilen. SHA-256: `1522073d021a923e97fd182abb8c4c8f642ceb9b069fee50c95646e3f7b45a21`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt eine PE32+-Datei mit Bereichsprüfungen, extrahiert Sektionen, Imageidentität und verkettete Unwind-Funktionsbereiche und bietet RVA-/VA-/Dateioffset-, Such- und Funktionszuordnung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt eine PE32+-Datei mit Bereichsprüfungen, extrahiert Sektionen, Imageidentität und verkettete Unwind-Funktionsbereiche und bietet RVA-/VA-/Dateioffset-, Such- und Funktionszuordnung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `8-66 PE.__init__`<br>`68-126 Lese-/Suchschnittstelle` |
| 4. Aufrufer und Verbraucher | discovery/reflection.py, functions.py und CLI-/Verifikationswerkzeuge |
| 5. Verwendete Komponenten | Python hashlib/struct/bisect |
| 6. Eingaben und Ausgaben | Eingaben: EXE-Dateipfad Ausgabe: PE-Sicht, SHA-256-Identität und Funktionsbereiche |
| 7. Seiteneffekte und Ressourcen | Liest EXE-Datei in den Speicher. |
| 8. Art der Verantwortung | infrastruktur |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Python hashlib/struct/bisect. Aufruferbezug: discovery/reflection.py, functions.py und CLI-/Verifikationswerkzeuge |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py, konkrete Import-/Assertionfälle vollständig gelesen. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Statische PE-Analyse. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 8-66 PE.__init__ · 68-126 Lese-/Suchschnittstelle

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/pe.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/reflection.py](../../tools/runtime-profiles/discovery/reflection.py)

Vollständig gelesen, 134 Zeilen. SHA-256: `99d8181b6f8a0d29b12cc274543151e9b639078e37f65983de88c06e57f1e1ae`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Entdeckt die originale Reflectionregistry über binäre String-/Deskriptoranker und extrahiert Typen, Felder, Attribute, Defaults und Hashkollisionen mit referenzieller Integritätsprüfung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Entdeckt die originale Reflectionregistry über binäre String-/Deskriptoranker und extrahiert Typen, Felder, Attribute, Defaults und Hashkollisionen mit referenzieller Integritätsprüfung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `21-49 find_registry`<br>`52-134 extract` |
| 4. Aufrufer und Verbraucher | discover-runtime.py und inspect-runtime.py:trace |
| 5. Verwendete Komponenten | discovery.pe.PE · struct |
| 6. Eingaben und Ausgaben | Eingaben: PE-Sicht Ausgabe: Reflectionbericht mit Adressen, Typen, Feldern und Kollisionen |
| 7. Seiteneffekte und Ressourcen | Keine Mutation, liest ausschließlich über PE. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: discovery.pe.PE · struct. Aufruferbezug: discover-runtime.py und inspect-runtime.py:trace |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py, konkrete Import-/Assertionfälle vollständig gelesen. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Reflectionanalyse. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 21-49 find_registry · 52-134 extract

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/reflection.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/discovery/systems.py](../../tools/runtime-profiles/discovery/systems.py)

Vollständig gelesen, 38 Zeilen. SHA-256: `53222f2f844a96ed8e594992dd1c3559fcd2672a23b42c02138300a98ac279a1`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Extrahiert benannte originale ECS-Ausführungsdeskriptoren aus statischen Codezeigerslots. Prüft ASCII-Namen und begrenzte Abhängigkeitsarrays und erhält rohe Slotpositionen für spätere Profilanalyse. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Extrahiert benannte originale ECS-Ausführungsdeskriptoren aus statischen Codezeigerslots. Prüft ASCII-Namen und begrenzte Abhängigkeitsarrays und erhält rohe Slotpositionen für spätere Profilanalyse. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `5-38 discover` |
| 4. Aufrufer und Verbraucher | discover-runtime.py |
| 5. Verwendete Komponenten | re · PE-Abstraktion und Funktionsinventar |
| 6. Eingaben und Ausgaben | Eingaben: PE-Sicht und erfasste Funktionsgruppen/Codezeigerslots. Ausgabe: Sortierte originale Engine-Systemdeskriptoren samt sechs rohen Abhängigkeitsslots. |
| 7. Seiteneffekte und Ressourcen | Lesende PE-Abfragen, keine Mutation. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: re · PE-Abstraktion und Funktionsinventar. Aufruferbezug: discover-runtime.py |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py, konkrete Import-/Assertionfälle vollständig gelesen. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Engine-Systemdeskriptoren. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 5-38 discover

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/discovery/systems.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/export-runtime-bindings.py](../../tools/runtime-profiles/export-runtime-bindings.py)

Vollständig gelesen, 92 Zeilen. SHA-256: `28a4acaf248a2675418e37ce54ba84cde85e36d667bb0dd09908cc2f36c198a9`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Führt Reflection-, Registry-, Provider- und optionale differenziell geprüfte Bufferadapterbelege zu einem beobachteten Typ-/Komponenten-/Codekatalog zusammen. Kennzeichnet ungeklärte Namen und trennt Bufferaufrufbarkeit von nativer Engineaufrufbarkeit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Führt Reflection-, Registry-, Provider- und optionale differenziell geprüfte Bufferadapterbelege zu einem beobachteten Typ-/Komponenten-/Codekatalog zusammen. Kennzeichnet ungeklärte Namen und trennt Bufferaufrufbarkeit von nativer Engineaufrufbarkeit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `11-67 export`<br>`70-92 CLI/Index` |
| 4. Aufrufer und Verbraucher | run-runtime-discovery.ps1 |
| 5. Verwendete Komponenten | PE · JSON-Captures |
| 6. Eingaben und Ausgaben | Eingaben: EXE, Captureordner, optionale Adapterprüfung Ausgabe: runtime-bindings.json und optional kompakter Index |
| 7. Seiteneffekte und Ressourcen | Liest Belegdateien, schreibt Katalog und Index. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · JSON-Captures. Aufruferbezug: run-runtime-discovery.ps1 |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Belegkatalog. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 11-67 export · 70-92 CLI/Index

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/export-runtime-bindings.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/inspect-runtime.py](../../tools/runtime-profiles/inspect-runtime.py)

Vollständig gelesen, 204 Zeilen. SHA-256: `d4816c3a1d2b34e882e8e13427fbcb63cd88ec46e9183bccf6e270c26c12d6f4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | CLI fragt erfasste Typen nach Namen/Hash ab, vergleicht Layouts und native Kandidaten zweier Builds und erzeugt frische Anker-/Aufruferbelege mit optional gespeicherten Captures. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: CLI fragt erfasste Typen nach Namen/Hash ab, vergleicht Layouts und native Kandidaten zweier Builds und erzeugt frische Anker-/Aufruferbelege mit optional gespeicherten Captures. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `29-52 lookup`<br>`64-122 compare`<br>`125-156 trace`<br>`159-204 CLI` |
| 4. Aufrufer und Verbraucher | run-runtime-discovery.ps1 und manuelle CLI |
| 5. Verwendete Komponenten | discovery.matching · PE/reflection/functions/systems |
| 6. Eingaben und Ausgaben | Eingaben: Captureordner, Abfrage oder Quell-/Zielbuild und optionale Anker Ausgabe: JSON-Lookup, Layoutdiff, Kandidaten oder Ankertrace |
| 7. Seiteneffekte und Ressourcen | Liest Captures/EXE, schreibt optional Ausgabe/Captures. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: discovery.matching · PE/reflection/functions/systems. Aufruferbezug: run-runtime-discovery.ps1 und manuelle CLI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py, konkrete Import-/Assertionfälle vollständig gelesen. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Beleginspektion. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 29-52 lookup · 64-122 compare · 125-156 trace · 159-204 CLI

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/inspect-runtime.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/native/audit-live-component-registry.cpp](../../tools/runtime-profiles/native/audit-live-component-registry.cpp)

Vollständig gelesen, 427 Zeilen. SHA-256: `36fc0eb01fabee7369277ad6d048f226bdce60a203334c15384af9b3b361c34f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft aus Typkatalog und Profil bekannte Metadaten im laufenden Prozess, auditiert Weltguards/Patchsignaturen und gewinnt durch Index-Abstimmung mögliche zusätzliche Tabellenzuordnungen. Bietet explizite Rangeinspektion und Zeigerlaufdiagnose. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft aus Typkatalog und Profil bekannte Metadaten im laufenden Prozess, auditiert Weltguards/Patchsignaturen und gewinnt durch Index-Abstimmung mögliche zusätzliche Tabellenzuordnungen. Bietet explizite Rangeinspektion und Zeigerlaufdiagnose. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `71-147 Codeaudit`<br>`180-224 Rangeinspektion`<br>`249-427 Metadaten-/Tabellenscan` |
| 4. Aufrufer und Verbraucher | run-live-component-registry-audit.ps1 und CMake-Entwicklertarget |
| 5. Verwendete Komponenten | Win32 RPM · nlohmann JSON · Typkatalog/Buildprofil |
| 6. Eingaben und Ausgaben | Eingaben: PID, Typkatalog und Profil oder Rangeparameter Ausgabe: Konsolenlog mit Belegen und new_mapping-Zeilen |
| 7. Seiteneffekte und Ressourcen | Lesender Prozessscan, liest Katalog/Profile. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32 RPM · nlohmann JSON · Typkatalog/Buildprofil. Aufruferbezug: run-live-component-registry-audit.ps1 und CMake-Entwicklertarget |
| 11. Duplikation | Eigenständige Win32-Lese-/Prozessscanhelfer wiederholen Lesetechnik anderer Entwicklungsprogramme. live-entity-managers.cpp und discovery/components.py validieren zudem ähnliche Manager-/Stridebelege in C++ und Python, ohne identischen Umfang. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Profilgestützte Registrysuche. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 71-147 Codeaudit · 180-224 Rangeinspektion · 249-427 Metadaten-/Tabellenscan

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/native/audit-live-component-registry.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/native/inspect-component-metadata.cpp](../../tools/runtime-profiles/native/inspect-component-metadata.cpp)

Vollständig gelesen, 80 Zeilen. SHA-256: `358d7033b94552328c603a837190b4cec1fb67f84ec0c6c60f6b2429d42c6bd7`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest bis zu 1024 Deskriptoren einer expliziten Live-Tabelle und meldet Metadatenoffsets, deren 16-/32-Bit-Wert häufig dem Tabellenindex entspricht. Liefert Untersuchungshinweise. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest bis zu 1024 Deskriptoren einer expliziten Live-Tabelle und meldet Metadatenoffsets, deren 16-/32-Bit-Wert häufig dem Tabellenindex entspricht. Liefert Untersuchungshinweise. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `26-80 main` |
| 4. Aufrufer und Verbraucher | Native CMake EXCLUDE_FROM_ALL, manueller CLI-Aufruf |
| 5. Verwendete Komponenten | Win32 RPM |
| 6. Eingaben und Ausgaben | Eingaben: PID und Tabellenadresse Ausgabe: Konsolenstatistik möglicher Indexfelder |
| 7. Seiteneffekte und Ressourcen | Öffnet Prozess lesend und liest Deskriptoren. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32 RPM. Aufruferbezug: Native CMake EXCLUDE_FROM_ALL, manueller CLI-Aufruf |
| 11. Duplikation | Eigenständige Win32-Lese-/Prozessscanhelfer wiederholen Lesetechnik anderer Entwicklungsprogramme. live-entity-managers.cpp und discovery/components.py validieren zudem ähnliche Manager-/Stridebelege in C++ und Python, ohne identischen Umfang. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Metadatenindex-Sondierung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 26-80 main

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/native/inspect-component-metadata.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/native/live-entity-manager-sample.cpp](../../tools/runtime-profiles/native/live-entity-manager-sample.cpp)

Vollständig gelesen, 93 Zeilen. SHA-256: `e0cde992828c4dd2a0171f8fb440d9ebbfac79ec87833eca89746d72e1d04b94`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Sampelt für ein hart codiertes geprüftes Codefenster kurz angehaltene Threadregister, leitet über R15 den EntityManager ab und meldet lesbare Tabellenkandidaten. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Sampelt für ein hart codiertes geprüftes Codefenster kurz angehaltene Threadregister, leitet über R15 den EntityManager ab und meldet lesbare Tabellenkandidaten. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `42-47 Byteguard`<br>`49-91 Sampling` |
| 4. Aufrufer und Verbraucher | Native CMake EXCLUDE_FROM_ALL, manueller CLI-Aufruf |
| 5. Verwendete Komponenten | Win32 Threadkontext/RPM |
| 6. Eingaben und Ausgaben | Eingaben: PID eines passenden Spielbuilds Ausgabe: Konsolenbelege von Managern und Stichproben |
| 7. Seiteneffekte und Ressourcen | Hält fremde Spielthreads kurz an und setzt sie fort, liest Speicher, kein Injectioncode. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32 Threadkontext/RPM. Aufruferbezug: Native CMake EXCLUDE_FROM_ALL, manueller CLI-Aufruf |
| 11. Duplikation | Eigenständige Win32-Lese-/Prozessscanhelfer wiederholen Lesetechnik anderer Entwicklungsprogramme. live-entity-managers.cpp und discovery/components.py validieren zudem ähnliche Manager-/Stridebelege in C++ und Python, ohne identischen Umfang. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: EntityManager-Registersampel. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 42-47 Byteguard · 49-91 Sampling

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/native/live-entity-manager-sample.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/native/live-entity-managers.cpp](../../tools/runtime-profiles/native/live-entity-managers.cpp)

Vollständig gelesen, 138 Zeilen. SHA-256: `a59677363d63c311d2496894fac75e4a94d5588ee1b6a8116111e73dfd58b214`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Scannt privaten beschreibbaren Prozessspeicher nach Managerheadern und bestätigt Kandidaten über Registry-Parallelarrays und mehrere Entitäten mit passenden Layoutbits/Strides. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Scannt privaten beschreibbaren Prozessspeicher nach Managerheadern und bestätigt Kandidaten über Registry-Parallelarrays und mehrere Entitäten mit passenden Layoutbits/Strides. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `36-87 validate_manager`<br>`90-138 Scan` |
| 4. Aufrufer und Verbraucher | Native CMake EXCLUDE_FROM_ALL, manuelle CLI |
| 5. Verwendete Komponenten | Win32 RPM/VirtualQueryEx |
| 6. Eingaben und Ausgaben | Eingaben: PID Ausgabe: Konsolenkandidaten mit Strukturbelegen |
| 7. Seiteneffekte und Ressourcen | Lesender Prozessspeicherscan. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32 RPM/VirtualQueryEx. Aufruferbezug: Native CMake EXCLUDE_FROM_ALL, manuelle CLI |
| 11. Duplikation | Eigenständige Win32-Lese-/Prozessscanhelfer wiederholen Lesetechnik anderer Entwicklungsprogramme. live-entity-managers.cpp und discovery/components.py validieren zudem ähnliche Manager-/Stridebelege in C++ und Python, ohne identischen Umfang. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Strukturelle EntityManager-Suche. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 36-87 validate_manager · 90-138 Scan

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/native/live-entity-managers.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/native/live-type-references.cpp](../../tools/runtime-profiles/native/live-type-references.cpp)

Vollständig gelesen, 82 Zeilen. SHA-256: `70a331e6b629aca52179d3bfb24213c77155652d5ee04e052298da1bf0b2e96d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Sucht im lesbaren Prozessspeicher Zeiger auf fünf fest codierte ECS-Metadaten-RVAs und meldet Referenzadressen und Summen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Sucht im lesbaren Prozessspeicher Zeiger auf fünf fest codierte ECS-Metadaten-RVAs und meldet Referenzadressen und Summen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-21 Targets`<br>`33-82 Scan` |
| 4. Aufrufer und Verbraucher | Native CMake EXCLUDE_FROM_ALL, manuelle CLI |
| 5. Verwendete Komponenten | Win32 RPM/VirtualQueryEx/Toolhelp |
| 6. Eingaben und Ausgaben | Eingaben: PID, fest codierte Build-RVAs Ausgabe: Konsolenreferenzen je Typ |
| 7. Seiteneffekte und Ressourcen | Lesender Prozessscan, keine vollständige Buildidentitätsprüfung implementiert. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Win32 RPM/VirtualQueryEx/Toolhelp. Aufruferbezug: Native CMake EXCLUDE_FROM_ALL, manuelle CLI |
| 11. Duplikation | Eigenständige Win32-Lese-/Prozessscanhelfer wiederholen Lesetechnik anderer Entwicklungsprogramme. live-entity-managers.cpp und discovery/components.py validieren zudem ähnliche Manager-/Stridebelege in C++ und Python, ohne identischen Umfang. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Buildspezifische Typzeigersuche. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 14-21 Targets · 33-82 Scan

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/native/live-type-references.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/native/verify-component-registry.cpp](../../tools/runtime-profiles/native/verify-component-registry.cpp)

Vollständig gelesen, 251 Zeilen. SHA-256: `32220e18e4ae6e92190a26910559d1b655de0eb557accf90095007f420f8c0cd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verwendet dieselben Registry-/Discovery-/Funktionsleser wie der Provider für einen externen lesenden Prozesscapture, exportiert Callbackcode und Codeinventar und injiziert Fehler ausschließlich in lokalen Lesergebnissen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verwendet dieselben Registry-/Discovery-/Funktionsleser wie der Provider für einen externen lesenden Prozesscapture, exportiert Callbackcode und Codeinventar und injiziert Fehler ausschließlich in lokalen Lesergebnissen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `39-127 Discovery/Owner/Manager`<br>`139-190 Export/Fehlerinjektion`<br>`191-245 Funktionsleser` |
| 4. Aufrufer und Verbraucher | run-runtime-discovery.ps1 und CMake-Entwicklertarget |
| 5. Verwendete Komponenten | component_registry.h · registry_discovery.h · function_inventory.h · Win32 RPM · JSON |
| 6. Eingaben und Ausgaben | Eingaben: PID, Manager/Owner oder Reflection und optional Profil Ausgabe: Registrybericht und .functions.json |
| 7. Seiteneffekte und Ressourcen | Lesender Prozessscan, lokale Fehlermanipulation, Berichtdateien. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: component_registry.h · registry_discovery.h · function_inventory.h · Win32 RPM · JSON. Aufruferbezug: run-runtime-discovery.ps1 und CMake-Entwicklertarget |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfad, Includes und externe Tool-/ABI-Aufrufe müssen mitgezogen werden. Reine Verschiebung darf keine Buildprofil-RVAs oder ABI-Verträge ändern. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Produktionsleser-Verifikator. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 39-127 Discovery/Owner/Manager · 139-190 Export/Fehlerinjektion · 191-245 Funktionsleser

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/native/verify-component-registry.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/run-live-component-registry-audit.ps1](../../tools/runtime-profiles/run-live-component-registry-audit.ps1)

Vollständig gelesen, 64 Zeilen. SHA-256: `acde9bcca6389435bafef8f852503c4f651fa8b49674d7c38db63622c49d6776`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Wählt einen laufenden Enshrouded-Client und das native Registryauditwerkzeug, baut dieses bei vorhandenem Quellbaum, startet lesendes Audit und speichert Log sowie neue_mapping-Kandidaten als TSV. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Wählt einen laufenden Enshrouded-Client und das native Registryauditwerkzeug, baut dieses bei vorhandenem Quellbaum, startet lesendes Audit und speichert Log sowie neue_mapping-Kandidaten als TSV. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `10-43 Tool-/Prozesswahl`<br>`45-64 Log/Kandidaten` |
| 4. Aufrufer und Verbraucher | Manueller PowerShell-Einstieg |
| 5. Verwendete Komponenten | cmake · kfc-runtime-audit-live-component-registry.exe · PowerShell Get-Process |
| 6. Eingaben und Ausgaben | Eingaben: Typkatalog, Buildprofil, optionale PID/Ausgabe Ausgabe: Auditlog und optionale Kandidaten-TSV |
| 7. Seiteneffekte und Ressourcen | Kann Tool bauen, erstellt Ordner, startet externen Leser, schreibt Logs. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: cmake · kfc-runtime-audit-live-component-registry.exe · PowerShell Get-Process. Aufruferbezug: Manueller PowerShell-Einstieg |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | PSScriptRoot-basierte Traversierung, Build-/Pakettoolpfade und aufgerufene Dateinamen sind verschiebungsempfindlich. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Registryaudit-Ausführungswerkzeug. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 10-43 Tool-/Prozesswahl · 45-64 Log/Kandidaten

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/run-live-component-registry-audit.ps1`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/run-runtime-discovery.ps1](../../tools/runtime-profiles/run-runtime-discovery.ps1)

Vollständig gelesen, 106 Zeilen. SHA-256: `ca8d5aba299bb65e04fc0c7ee8c1f7e8e5221c14fa3d42a42a8b0328d8e12e2a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Orchestriert Client-/Server-Discovery mit optionalem Livecapture, nativem Produktionsverifikator, Rust-Callbackkorpus, isolierter Adapterprüfung und Modherkunftsprüfung. Exportiert Kataloge und Client-Server-Vergleich. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Orchestriert Client-/Server-Discovery mit optionalem Livecapture, nativem Produktionsverifikator, Rust-Callbackkorpus, isolierter Adapterprüfung und Modherkunftsprüfung. Exportiert Kataloge und Client-Server-Vergleich. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `32-67 Capture/Herkunft`<br>`68-98 Liveprüfung/Export`<br>`100-106 Vergleich` |
| 4. Aufrufer und Verbraucher | Manueller PowerShell-Einstieg |
| 5. Verwendete Komponenten | Python-CLI-Werkzeuge · NativeVerifier · cargo · Spielinstallationen |
| 6. Eingaben und Ausgaben | Eingaben: Installationspfade, Target, Offline-/Prüfschalter und Werkzeuge Ausgabe: Capturebaum samt Prüfberichten und optionalem Crossbuildvergleich |
| 7. Seiteneffekte und Ressourcen | Startet Tools, kann Rust bauen/testen, setzt Testumgebung vorübergehend, schreibt Berichte. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: Python-CLI-Werkzeuge · NativeVerifier · cargo · Spielinstallationen. Aufruferbezug: Manueller PowerShell-Einstieg |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | PSScriptRoot-basierte Traversierung, Build-/Pakettoolpfade und aufgerufene Dateinamen sind verschiebungsempfindlich. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Aufnahme-Workflow. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 32-67 Capture/Herkunft · 68-98 Liveprüfung/Export · 100-106 Vergleich

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/run-runtime-discovery.ps1`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_components.py](../../tools/runtime-profiles/tests/test_components.py)

Vollständig gelesen, 94 Zeilen. SHA-256: `213c65fc5c1fc52114bf22066f3c8d8317331ee3f2a3fb08ec420a2e93192582`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft den Python-Registryleser mit unabhängigen synthetischen Speicherfixtures auf Template-only-Registrierung, Flags, Hash-/Parallelarrayabweichung, ausführbare Callbackadressen und Arraygrößen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft den Python-Registryleser mit unabhängigen synthetischen Speicherfixtures auf Template-only-Registrierung, Flags, Hash-/Parallelarrayabweichung, ausführbare Callbackadressen und Arraygrößen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `32-55 fixture`<br>`58-91 Tests` |
| 4. Aufrufer und Verbraucher | Python unittest-Einstieg |
| 5. Verwendete Komponenten | unittest · discovery.components.read_registry · reflection.fnv |
| 6. Eingaben und Ausgaben | Eingaben: Synthetische Speicherobjekte Ausgabe: Assertions/Teststatus |
| 7. Seiteneffekte und Ressourcen | Nur privater Speicher. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: unittest · discovery.components.read_registry · reflection.fnv. Aufruferbezug: Python unittest-Einstieg |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Registrierungsanalyse-Tests. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 32-55 fixture · 58-91 Tests

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/tests/test_components.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_discovery.py](../../tools/runtime-profiles/tests/test_discovery.py)

Vollständig gelesen, 265 Zeilen. SHA-256: `e5bf8fb1dd7f891c7ac58dbb00d75e2e7771f97562dc5d0c6b7a2daaca48fdf5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft PE-/Reflectionextraktion, Grenzen, Hashkollisionen, sechsten Systemdeskriptorslot, Disassemblierungsnormalisierung, verkettetes Unwind und frische Ankertraces mit synthetischer PE einschließlich Mehrdeutigkeit und Crossimageablehnung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft PE-/Reflectionextraktion, Grenzen, Hashkollisionen, sechsten Systemdeskriptorslot, Disassemblierungsnormalisierung, verkettetes Unwind und frische Ankertraces mit synthetischer PE einschließlich Mehrdeutigkeit und Crossimageablehnung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `18-44 fixture`<br>`55-171 Parser/Code`<br>`173-261 Inspektion/Trace` |
| 4. Aufrufer und Verbraucher | Python unittest-Einstieg |
| 5. Verwendete Komponenten | unittest · PE/reflection/functions/systems/matching · optionales capstone/numpy |
| 6. Eingaben und Ausgaben | Eingaben: Synthetische PE und Captures Ausgabe: Assertions/Teststatus |
| 7. Seiteneffekte und Ressourcen | Schreibt temporäre Fixtures und Captures, keine Spielinteraktion. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: unittest · PE/reflection/functions/systems/matching · optionales capstone/numpy. Aufruferbezug: Python unittest-Einstieg |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Statische Discovery-Tests. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 18-44 fixture · 55-171 Parser/Code · 173-261 Inspektion/Trace

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/tests/test_discovery.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/tests/test_mod_origins.py](../../tools/runtime-profiles/tests/test_mod_origins.py)

Vollständig gelesen, 35 Zeilen. SHA-256: `05871d0058ecb682f7946cc10ba0508843e72c48030dd4c558226e0091623543`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Sichert Patchpayload-Herkunftsprüfungen gegen die frühere falsche RIP-Konstantendistanz und gegen Addition statt Unterdrückung des Fallschadensstores. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Sichert Patchpayload-Herkunftsprüfungen gegen die frühere falsche RIP-Konstantendistanz und gegen Addition statt Unterdrückung des Fallschadensstores. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-31 zwei Regressionen` |
| 4. Aufrufer und Verbraucher | Python unittest-Einstieg |
| 5. Verwendete Komponenten | unittest · audit-mod-origins.validate_payload |
| 6. Eingaben und Ausgaben | Eingaben: Synthetische Patchpayloads Ausgabe: Assertions/Teststatus |
| 7. Seiteneffekte und Ressourcen | Keine externen Mutationen. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: unittest · audit-mod-origins.validate_payload. Aufruferbezug: Python unittest-Einstieg |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Modherkunfts-Regressionstests. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 14-31 zwei Regressionen

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/tests/test_mod_origins.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/verify-attribute-vm.py](../../tools/runtime-profiles/verify-attribute-vm.py)

Vollständig gelesen, 177 Zeilen. SHA-256: `f61562f720daa3106fe5136833c8fd37185bf4ec11d271d18a3dbe354e4ea99e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Emuliert originale hashidentifizierte Attributinterpreter in Unicorn und erzeugt deterministische Differentialtestvektoren einschließlich Randwerten und absteigender Wurzelneuberechnung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Emuliert originale hashidentifizierte Attributinterpreter in Unicorn und erzeugt deterministische Differentialtestvektoren einschließlich Randwerten und absteigender Wurzelneuberechnung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `57-98 Oracle`<br>`101-177 Vektorgenerator` |
| 4. Aufrufer und Verbraucher | Manuelle CLI, Vektoren für Rust-Oracletests laut Datenvertrag |
| 5. Verwendete Komponenten | PE · Unicorn · Ressourcenexport |
| 6. Eingaben und Ausgaben | Eingaben: EXE, Attributressourcenexport, Stichprobenzahlen Ausgabe: JSON mit Originalergebnissen und Coverage |
| 7. Seiteneffekte und Ressourcen | Isolierte Emulatorausführung, Dateioutput, kein Spielprozess. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · Unicorn · Ressourcenexport. Aufruferbezug: Manuelle CLI, Vektoren für Rust-Oracletests laut Datenvertrag |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Attributmodell-Verifikationswerkzeug. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 57-98 Oracle · 101-177 Vektorgenerator

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/verify-attribute-vm.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/verify-buffer-adapters.py](../../tools/runtime-profiles/verify-buffer-adapters.py)

Vollständig gelesen, 83 Zeilen. SHA-256: `63d3683318c759cc8860436048a35162afc5435d725228112b196f58abe92ba8`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Vergleicht zuvor geprüfte MOV/MOVZX/RET-Callbackbodies mit Bufferadapter-Kopierplänen auf privaten randomisierten Puffern einschließlich Guardbytes und Quellschutz. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Vergleicht zuvor geprüfte MOV/MOVZX/RET-Callbackbodies mit Bufferadapter-Kopierplänen auf privaten randomisierten Puffern einschließlich Guardbytes und Quellschutz. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `33-75 256 Versuche je Adapter` |
| 4. Aufrufer und Verbraucher | run-runtime-discovery.ps1 |
| 5. Verwendete Komponenten | PE · capstone · ctypes Win32 |
| 6. Eingaben und Ausgaben | Eingaben: EXE, Registrycodebelege und Adapterplan Ausgabe: JSON-Ergebnis je Adapter |
| 7. Seiteneffekte und Ressourcen | Allokiert lokalen ausführbaren Code und führt begrenzte Kopierbodies auf privaten Puffern aus, schreibt Bericht. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · capstone · ctypes Win32. Aufruferbezug: run-runtime-discovery.ps1 |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Aufruf/Einbindung über die angegebenen gelesenen Werkzeuge nachgewiesen. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Bufferadapter-Differentialprüfung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 33-75 256 Versuche je Adapter

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/verify-buffer-adapters.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/verify-building-input.py](../../tools/runtime-profiles/verify-building-input.py)

Vollständig gelesen, 71 Zeilen. SHA-256: `0b93cb1dd58e3d372bd624dc103184168a538c33d023407a36bb52d5e686f8cf`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft statisch exakte reflektierte ClientPlayerInput-Layouts, Tastenbits, Profilguards, Cursorregister und originalen Versionshelper für den Baueingabeadapter. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft statisch exakte reflektierte ClientPlayerInput-Layouts, Tastenbits, Profilguards, Cursorregister und originalen Versionshelper für den Baueingabeadapter. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-57 verify` |
| 4. Aufrufer und Verbraucher | Manuelle CLI |
| 5. Verwendete Komponenten | PE · capstone · Profil/Reflection |
| 6. Eingaben und Ausgaben | Eingaben: EXE, exaktes Profil und Reflection Ausgabe: JSON-Vertragsbeleg |
| 7. Seiteneffekte und Ressourcen | Liest EXE und Eingabedateien, schreibt Bericht. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · capstone · Profil/Reflection. Aufruferbezug: Manuelle CLI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Baueingabe-Vertragsprüfung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 14-57 verify

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/verify-building-input.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/verify-client-voxel-context.py](../../tools/runtime-profiles/verify-client-voxel-context.py)

Vollständig gelesen, 59 Zeilen. SHA-256: `415cfb243c0d039ecf104bda4b39d2cc71467e60bde343cb17ab69222c97ee51`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft statisch den originalen Datenfluss vom Clientcursorframe über ServiceView/VoxelWorld zum vorhandenen Voxelreader und die zusätzliche Callbackargumentkodierung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft statisch den originalen Datenfluss vom Clientcursorframe über ServiceView/VoxelWorld zum vorhandenen Voxelreader und die zusätzliche Callbackargumentkodierung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `13-48 verify` |
| 4. Aufrufer und Verbraucher | Manuelle CLI |
| 5. Verwendete Komponenten | PE · capstone · Buildprofil |
| 6. Eingaben und Ausgaben | Eingaben: EXE und exaktes Clientprofil Ausgabe: JSON mit Leser-/Framebelegen |
| 7. Seiteneffekte und Ressourcen | Liest EXE, schreibt Bericht. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · capstone · Buildprofil. Aufruferbezug: Manuelle CLI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Client-Weltkontextprüfung. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 13-48 verify

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/verify-client-voxel-context.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/verify-inventory-zero.py](../../tools/runtime-profiles/verify-inventory-zero.py)

Vollständig gelesen, 100 Zeilen. SHA-256: `e427c8ef0f54008470fef795af63b6e74c821ca115f7239d5df2a81de38ed067`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Emuliert zwei originale Inventar-Decrementhelper für festgelegte Builds und Mengen inklusive Null. Stubbt Lookup/Allokation und prüft Reststapel und Journalmenge. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Emuliert zwei originale Inventar-Decrementhelper für festgelegte Builds und Mengen inklusive Null. Stubbt Lookup/Allokation und prüft Reststapel und Journalmenge. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `23-80 run`<br>`83-100 Fälle` |
| 4. Aufrufer und Verbraucher | Manuelle CLI |
| 5. Verwendete Komponenten | PE · capstone · Unicorn |
| 6. Eingaben und Ausgaben | Eingaben: EXE eines bekannten SHA-Builds Ausgabe: JSON mit acht Originalcodefällen |
| 7. Seiteneffekte und Ressourcen | Isolierte Emulatorausführung und Berichtdatei. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · capstone · Unicorn. Aufruferbezug: Manuelle CLI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Inventarargument-Verifikation. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 23-80 run · 83-100 Fälle

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/verify-inventory-zero.py`.

### [src/loader/runtime/profile-tools/dev/tools/enshrouded/verify-world-event-abi.py](../../tools/runtime-profiles/verify-world-event-abi.py)

Vollständig gelesen, 119 Zeilen. SHA-256: `51b7c177a8ff0a17c040e9d3dce15305029d7f7656efc86ab7e733b268129b98`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Emuliert originale Place-/Destroy-Eventwriter mit gestubbten Queue-/Zeithelpern und prüft anhand Reflection die Zuordnung von Material, TrackingItemId, Owner, Timestamp und Position. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Emuliert originale Place-/Destroy-Eventwriter mit gestubbten Queue-/Zeithelpern und prüft anhand Reflection die Zuordnung von Material, TrackingItemId, Owner, Timestamp und Position. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `28-95 verify`<br>`98-119 Fälle` |
| 4. Aufrufer und Verbraucher | Manuelle CLI |
| 5. Verwendete Komponenten | PE · Unicorn · Reflection |
| 6. Eingaben und Ausgaben | Eingaben: EXE und Reflection desselben Builds Ausgabe: JSON mit vier ABI-Fällen und Grenzen |
| 7. Seiteneffekte und Ressourcen | Isolierte Emulatorausführung und Berichtdatei. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklungswerkzeuge. Technische Kopplung: PE · Unicorn · Reflection. Aufruferbezug: Manuelle CLI |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Manueller Entwickler-/Prüfeinstieg vorhanden. Regelmäßige Verwendung oder Veraltung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Python-Importpfade, sys.path/CLI-Aufrufe und relative Capture-/Werkzeugpfade müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Weltbau-ABI-Verifikation. Neue Einordnung: Entwicklungswerkzeuge. Besitzer: Runtime-Profilwerkzeuge. |

**Belege:** 28-95 verify · 98-119 Fälle

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `tools/runtime-profiles/verify-world-event-abi.py`.

### [src/loader/runtime/profile-tools/dev/tools/tests/building_input_test.cpp](../../kfc-runtime/tests/building_input_test.cpp)

Vollständig gelesen, 90 Zeilen. SHA-256: `03ac19c6b3dc50bd0163e8b9feb0a2b07fd0378b3d80e3a555d4f0f08eeb00dd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echten Baueingabecode mit privaten Eingabebytes und gestubbten Welt-/ECS-Grenzen auf Auswahl, Press/Release-Ticks, Spielerbindung, Dismantlehaltezeit, Abbruch, Werte-/Buildgates und physische Benutzereingabe. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echten Baueingabecode mit privaten Eingabebytes und gestubbten Welt-/ECS-Grenzen auf Auswahl, Press/Release-Ticks, Spielerbindung, Dismantlehaltezeit, Abbruch, Werte-/Buildgates und physische Benutzereingabe. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `25-89 main` |
| 4. Aufrufer und Verbraucher | CMake kfc-runtime-test-building-input |
| 5. Verwendete Komponenten | building_input.cpp · ECS-/Weltstubs · Win32 |
| 6. Eingaben und Ausgaben | Eingaben: Private Inputfixture und Profilzustand Ausgabe: Assertions und Konsolensumme |
| 7. Seiteneffekte und Ressourcen | Ändert private Bytes, wartet für Haltezeittest. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: building_input.cpp · ECS-/Weltstubs · Win32. Aufruferbezug: CMake kfc-runtime-test-building-input |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfade und direkte .cpp-Includes/interne Testzugriffe müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Baueingabe-Tests. Neue Einordnung: Verhaltensprüfung. Besitzer: Native Runtime-Tests. |

**Belege:** 25-89 main

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/tests/building_input_test.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/tests/ecs_read_test.cpp](../../kfc-runtime/tests/ecs_read_test.cpp)

Vollständig gelesen, 727 Zeilen. SHA-256: `f2edebdd1b66f40f0733bccd273b1767ff12a525325875736967f5ba6dc38bf2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echten ECS-Code und original-owner EngineContext mit privaten imagegebundenen Reflection- und Entitätsfixtures. Deckt Registrierung/Aliase, Handles, inkrementelle Querycache-Revalidierung, Hashmap-Mitgliedschaft, UUIDs, Einzel-/Stapellesung ohne Teilausgabe, Seitenwechsel und autoritative maskierte CAS-Schreibgates inklusive Tracker und sperrendem unklarem Ergebnis ab. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echten ECS-Code und original-owner EngineContext mit privaten imagegebundenen Reflection- und Entitätsfixtures. Deckt Registrierung/Aliase, Handles, inkrementelle Querycache-Revalidierung, Hashmap-Mitgliedschaft, UUIDs, Einzel-/Stapellesung ohne Teilausgabe, Seitenwechsel und autoritative maskierte CAS-Schreibgates inklusive Tracker und sperrendem unklarem Ergebnis ab. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `111-240 Fixture/Replica`<br>`241-380 Registry/Query/Membership`<br>`381-578 UUID/Lesung/Session`<br>`579-725 Schreibgates/CAS/Fehler` |
| 4. Aufrufer und Verbraucher | CMake kfc-runtime-test-ecs-read |
| 5. Verwendete Komponenten | ecs_runtime.cpp direkt eingebunden · engine_context.cpp · Win32-Mess-/Fehlerinjektion |
| 6. Eingaben und Ausgaben | Eingaben: Private synthetische Engineobjekte, Profile und gezielte Fehlerzustände. Ausgabe: Assertions, Teststatus und Konsolenzähler. |
| 7. Seiteneffekte und Ressourcen | Ändert ausschließlich Objekte und allokierte Seiten des Testprozesses. Führt lokale Testcallbacks und gegebenenfalls Seitenschutzänderungen aus. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Produktimplementierung wird als .cpp direkt eingebunden und interne Zustände werden verändert. Dadurch hängt der Test strukturell an Dateipfad und anonymen Interna. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: ecs_runtime.cpp direkt eingebunden · engine_context.cpp · Win32-Mess-/Fehlerinjektion. Aufruferbezug: CMake kfc-runtime-test-ecs-read |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfade und direkte .cpp-Includes/interne Testzugriffe müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Zugriffs- und Lebenszeittests. Neue Einordnung: Verhaltensprüfung. Besitzer: Native Runtime-Tests. |

**Belege:** 111-240 Fixture/Replica · 241-380 Registry/Query/Membership · 381-578 UUID/Lesung/Session · 579-725 Schreibgates/CAS/Fehler

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/tests/ecs_read_test.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/tests/prop_query_test.cpp](../../kfc-runtime/tests/prop_query_test.cpp)

Vollständig gelesen, 142 Zeilen. SHA-256: `57ca604919d0c7f547f93863168d98556620cae5e4f3ab34bfb4d5d2322123f3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echten ECS-Propquerycode mit synthetischen Tabellen auf inkrementellen Fortschritt, Sessionwechsel, Deadlines, Boundswechsel, Duplikate und stale Handles. Prüft zusätzlich generische Queries und Byteguards des Codeaustauschs. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echten ECS-Propquerycode mit synthetischen Tabellen auf inkrementellen Fortschritt, Sessionwechsel, Deadlines, Boundswechsel, Duplikate und stale Handles. Prüft zusätzlich generische Queries und Byteguards des Codeaustauschs. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `14-127 Queries/Identität`<br>`128-140 Codeaustausch` |
| 4. Aufrufer und Verbraucher | CMake kfc-runtime-test-prop-query |
| 5. Verwendete Komponenten | ecs_runtime.cpp direkt eingebunden · dispatcher.cpp · Win32 |
| 6. Eingaben und Ausgaben | Eingaben: Private Entitäts-/Recipe-/Codefixtures Ausgabe: Assertions und Konsolensumme |
| 7. Seiteneffekte und Ressourcen | Allokiert private ausführbare Seite und tauscht dort Bytes. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Einbindung der gesamten ecs_runtime.cpp und Zugriff auf anonyme Interna koppeln den Test strukturell. Zusätzlich liegt der fachfremde WriteCode-Byteguardtest in diesem Propquerytest (128-140). |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: ecs_runtime.cpp direkt eingebunden · dispatcher.cpp · Win32. Aufruferbezug: CMake kfc-runtime-test-prop-query |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfade und direkte .cpp-Includes/interne Testzugriffe müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Propquery- und Dispatcher-Tests. Neue Einordnung: Verhaltensprüfung. Besitzer: Native Runtime-Tests. |

**Belege:** 14-127 Queries/Identität · 128-140 Codeaustausch

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/tests/prop_query_test.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/tests/resource_runtime_test.cpp](../../kfc-runtime/tests/resource_runtime_test.cpp)

Vollständig gelesen, 843 Zeilen. SHA-256: `480e9a3f12af7300556d38774ef21bd368aa866d55b41f8dba8046f0bbba05d7`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echten Ressourcenruntimecode und Ressourcenlebenszeit mit synthetischer originaler Besitzkette und Index-/Chunkfixtures. Deckt Ressourcenliste/Lesen, unlesbare Lücken, begrenzte Abfragekosten, skalare Stapelprojektion, Replica-Fallback, kanonische Hashvektoren, originale Itemmap, 32-Bit-CAS, Schutzwiederherstellung und Fehlerblockade ab. Die Sitzung wird gezielt gestubbt und erlaubt keine sitzungsgebundenen Writes. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echten Ressourcenruntimecode und Ressourcenlebenszeit mit synthetischer originaler Besitzkette und Index-/Chunkfixtures. Deckt Ressourcenliste/Lesen, unlesbare Lücken, begrenzte Abfragekosten, skalare Stapelprojektion, Replica-Fallback, kanonische Hashvektoren, originale Itemmap, 32-Bit-CAS, Schutzwiederherstellung und Fehlerblockade ab. Die Sitzung wird gezielt gestubbt und erlaubt keine sitzungsgebundenen Writes. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `48-70 Sessionstub`<br>`134-365 Besitz/Lesen/Performance`<br>`367-548 Lifetime/Batch/Replica`<br>`550-681 Hash/Itemmap`<br>`682-843 Snapshot/CAS/Schutzfehler` |
| 4. Aufrufer und Verbraucher | CMake kfc-runtime-test-resource |
| 5. Verwendete Komponenten | resource_runtime.cpp direkt eingebunden · engine_context.cpp · gestubbter EcsRuntime::SessionGuard · Win32-Zähl-/Fehlerwrapper |
| 6. Eingaben und Ausgaben | Eingaben: Private synthetische Engineobjekte, Profile und gezielte Fehlerzustände. Ausgabe: Assertions, Teststatus und Konsolenzähler. |
| 7. Seiteneffekte und Ressourcen | Ändert ausschließlich Objekte und allokierte Seiten des Testprozesses. Führt lokale Testcallbacks und gegebenenfalls Seitenschutzänderungen aus. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Produktimplementierung wird als .cpp direkt eingebunden und interne Zustände werden verändert. Dadurch hängt der Test strukturell an Dateipfad und anonymen Interna. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: resource_runtime.cpp direkt eingebunden · engine_context.cpp · gestubbter EcsRuntime::SessionGuard · Win32-Zähl-/Fehlerwrapper. Aufruferbezug: CMake kfc-runtime-test-resource |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfade und direkte .cpp-Includes/interne Testzugriffe müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Ressourcen- und Lebenszeittests. Neue Einordnung: Verhaltensprüfung. Besitzer: Native Runtime-Tests. |

**Belege:** 48-70 Sessionstub · 134-365 Besitz/Lesen/Performance · 367-548 Lifetime/Batch/Replica · 550-681 Hash/Itemmap · 682-843 Snapshot/CAS/Schutzfehler

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/tests/resource_runtime_test.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/tests/service_runtime_test.cpp](../../kfc-runtime/tests/service_runtime_test.cpp)

Vollständig gelesen, 243 Zeilen. SHA-256: `268ba26e0ebf81f97e10d48fe2429ded9fe0c53370c37bcf07f1a5066e374d75`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echte benannte Dienstexports und ECS-/Scene-Gates gegen private Besitzer-/Registry-/DI-Objekte. Deckt maskiertes CAS, Konflikte, stale Sessions, gestoppte/busy Gates, Namenskollisionen/Duplikate, mutable Bindungen, Metadaten und Seitenschutz ab. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echte benannte Dienstexports und ECS-/Scene-Gates gegen private Besitzer-/Registry-/DI-Objekte. Deckt maskiertes CAS, Konflikte, stale Sessions, gestoppte/busy Gates, Namenskollisionen/Duplikate, mutable Bindungen, Metadaten und Seitenschutz ab. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `64-170 Fixturekonfiguration`<br>`171-241 Positiv-/Negativfälle` |
| 4. Aufrufer und Verbraucher | CMake kfc-runtime-test-service |
| 5. Verwendete Komponenten | ecs_runtime.cpp direkt · service_runtime.cpp · engine_context.cpp |
| 6. Eingaben und Ausgaben | Eingaben: Synthetische Engine-Besitzkette und Dienstpayload Ausgabe: Assertions und Konsolensumme |
| 7. Seiteneffekte und Ressourcen | Allokiert private Seite, ändert lokale Daten und Schutzrechte. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine fachfremde Vermischung in der vollständigen Lektüre belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: ecs_runtime.cpp direkt · service_runtime.cpp · engine_context.cpp. Aufruferbezug: CMake kfc-runtime-test-service |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfade und direkte .cpp-Includes/interne Testzugriffe müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Benannte Engine-Dienstdaten-Tests. Neue Einordnung: Verhaltensprüfung. Besitzer: Native Runtime-Tests. |

**Belege:** 64-170 Fixturekonfiguration · 171-241 Positiv-/Negativfälle

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/tests/service_runtime_test.cpp`.

### [src/loader/runtime/profile-tools/dev/tools/tests/world_context_test.cpp](../../kfc-runtime/tests/world_context_test.cpp)

Vollständig gelesen, 221 Zeilen. SHA-256: `25774ffd2ee587e60ac6c9a76a5b100d8c3452761a87421da0a03a7b5740487b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echten Weltcode an privaten Frames und Funktionsstubs auf Dispatcher-Reentranz, unveränderte laufende Removalphasen, Singleton-/Actor-/Cursorweltquellen, Session-/Managerwechsel und Priorität des lokalen Spielerkontexts. Prüft lesenden Clientcursor-Voxelzugriff ohne Schreibpromotion und Dedicated-Server-Destroy über den Propupdatehook. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echten Weltcode an privaten Frames und Funktionsstubs auf Dispatcher-Reentranz, unveränderte laufende Removalphasen, Singleton-/Actor-/Cursorweltquellen, Session-/Managerwechsel und Priorität des lokalen Spielerkontexts. Prüft lesenden Clientcursor-Voxelzugriff ohne Schreibpromotion und Dedicated-Server-Destroy über den Propupdatehook. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `27-60 Removalphasen`<br>`62-142 Kontext/Session`<br>`143-185 lesender Clientkontext`<br>`187-218 Server-Destroy` |
| 4. Aufrufer und Verbraucher | CMake kfc-runtime-test-world-context |
| 5. Verwendete Komponenten | world_runtime.cpp direkt eingebunden · ECS/Dispatcher/Profile · private Native-Callbacks |
| 6. Eingaben und Ausgaben | Eingaben: Private synthetische Engineobjekte, Profile und gezielte Fehlerzustände. Ausgabe: Assertions, Teststatus und Konsolenzähler. |
| 7. Seiteneffekte und Ressourcen | Ändert ausschließlich Objekte und allokierte Seiten des Testprozesses. Führt lokale Testcallbacks und gegebenenfalls Seitenschutzänderungen aus. |
| 8. Art der Verantwortung | integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Produktimplementierung wird als .cpp direkt eingebunden und interne Zustände werden verändert. Dadurch hängt der Test strukturell an Dateipfad und anonymen Interna. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: world_runtime.cpp direkt eingebunden · ECS/Dispatcher/Profile · private Native-Callbacks. Aufruferbezug: CMake kfc-runtime-test-world-context |
| 11. Duplikation | Keine weitergehende semantische Duplikation durch vollständige Lektüre dieser Datei belegt. |
| 12. Nutzungsstatus | Vorhandener ausführbarer Test. Native CMake-Tests sind EXCLUDE_FROM_ALL, Python-Tests haben unittest-Einstieg. In diesem Audit nicht ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | CMake-Quellpfade und direkte .cpp-Includes/interne Testzugriffe müssen mitgezogen werden. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. Dies ist keine Aussage, dass keine Tests existieren. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Weltkontext-Tests. Neue Einordnung: Verhaltensprüfung. Besitzer: Native Runtime-Tests. |

**Belege:** 27-60 Removalphasen · 62-142 Kontext/Session · 143-185 lesender Clientkontext · 187-218 Server-Destroy

**Evidenzgrenze:** Tatsächliche Dateifunktion vollständig im Quellcode nachgewiesen. Rolle und fachliche Nachbarschaft daraus abgeleitet. Dynamische Spielwirkung nicht ausgeführt.

**Maßnahme:** verschieben. Die zusammenhängende Verantwortung erhält einen eindeutigen Besitzer. Aufrufer und Ressourcen folgen der Implementierung.

**Vorgeschlagene Zielpfade:** `kfc-runtime/tests/world_context_test.cpp`.

## Mods, Proben und Vorlage

31 Dateien, 9.264 Zeilen.

### [diagnostic-mods/live-engine-flight-probe-check.py](../../diagnostic-mods/live-engine-flight-probe-check.py)

Vollständig gelesen, 228 Zeilen. SHA-256: `696e310529a31ddaa2e05471f51df69d307b9b356e391763e2a399eb282f024e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft echte Flugprobe gegen 24 Lua-Lifecycle-Szenarien inklusive CAS-Rennen, Welt-/Generationwechsel, fremder Änderungen, unsicherer Writes und Gravity-Timeouts. Prüft zusätzlich die Enum-Zeichenkettendarstellung mit Textassertions in Produktionscodecquellen. Lädt benutzerspezifische lokale Lua-DLL per ctypes. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft echte Flugprobe gegen 24 Lua-Lifecycle-Szenarien inklusive CAS-Rennen, Welt-/Generationwechsel, fremder Änderungen, unsicherer Writes und Gravity-Timeouts. Prüft zusätzlich die Enum-Zeichenkettendarstellung mit Textassertions in Produktionscodecquellen. Lädt benutzerspezifische lokale Lua-DLL per ctypes. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-engine-flight-probe-check.py:15-22 Codec-Quellassertions`<br>`diagnostic-mods/live-engine-flight-probe-check.py:35 bootstrap`<br>`diagnostic-mods/live-engine-flight-probe-check.py:129 checks`<br>`diagnostic-mods/live-engine-flight-probe-check.py:206 Szenarien` |
| 4. Aufrufer und Verbraucher | Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 5. Verwendete Komponenten | ctypes/json/pathlib · lua54.dll · echte Probe/Manifeste · converter.rs und validator.rs als Quelltext |
| 6. Eingaben und Ausgaben | Eingaben: 24 simulierte Szenarien und echte Lua-Quelle Ausgabe: Assertion-/Erfolgsergebnisse |
| 7. Seiteneffekte und Ressourcen | lokale DLL laden · Dateien lesen |
| 8. Art der Verantwortung | Test-/Exportwerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Quelltextassertions ersetzen keinen ausgeführten Produktionscodec. Lokaler DLL-Pfad:24 begrenzt Portabilität. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: ctypes/json/pathlib · lua54.dll · echte Probe/Manifeste · converter.rs und validator.rs als Quelltext. Aufruferbezug: Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 11. Duplikation | Phasen-/Session-/Cleanup-Grundmuster ähnelt anderen Diagnoseproben. Geprüfte Daten-/Schreibpfade und Kriterien unterscheiden sich. |
| 12. Nutzungsstatus | Direkt ausführbares Werkzeug, Automatisierung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Quellen-/Exportpfade und gegebenenfalls benutzerspezifische Lua-DLL beachten. Nachbearbeiter überschreibt Exporte, nicht ausführen. |
| 14. Vorhandener Testschutz | Datei ist selbst Fixture, nicht ausgeführt. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lokale Fixture der Flugzustandsprobe. Neue Einordnung: Verhaltensprüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-engine-flight-probe-check.py:15-22 Codec-Quellassertions · diagnostic-mods/live-engine-flight-probe-check.py:35 bootstrap · diagnostic-mods/live-engine-flight-probe-check.py:129 checks · diagnostic-mods/live-engine-flight-probe-check.py:206 Szenarien

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-engine-flight-probe-check.py`.

### [diagnostic-mods/live-engine-flight-probe/src/mod.lua](../../diagnostic-mods/live-engine-flight-probe/src/mod.lua)

Vollständig gelesen, 244 Zeilen. SHA-256: `5766889d77aaec1c1fdc014bdea569d68ed2d212c0ce44ba714ecc39416dfec5`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Führt eine begrenzte Enginezustandsprobe durch: wartet auf genau einen autoritativen Spieler und reflektiertes BuildingFlying-Enum, setzt ausschließlich DynamicLocomotion.state mit expected_fields und beobachtet unabhängig Gravity.isActive über drei Samples. Restauriert nur den eigenen unveränderten Zustand, höchstens zwei erfolgreiche Writes, stoppt bei Sessionwechsel/unsicherem Ergebnis. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Führt eine begrenzte Enginezustandsprobe durch: wartet auf genau einen autoritativen Spieler und reflektiertes BuildingFlying-Enum, setzt ausschließlich DynamicLocomotion.state mit expected_fields und beobachtet unabhängig Gravity.isActive über drei Samples. Restauriert nur den eigenen unveränderten Zustand, höchstens zwei erfolgreiche Writes, stoppt bei Sessionwechsel/unsicherem Ergebnis. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-engine-flight-probe/src/mod.lua:79 write_state`<br>`diagnostic-mods/live-engine-flight-probe/src/mod.lua:102 restore_once`<br>`diagnostic-mods/live-engine-flight-probe/src/mod.lua:123 step`<br>`diagnostic-mods/live-engine-flight-probe/src/mod.lua:229 Lifecycle` |
| 4. Aufrufer und Verbraucher | mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 5. Verwendete Komponenten | runtime.ecs.read_fields/write_fields/query · runtime.world/types/has · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Live-Locomotion/Gravity und Enum-Metadaten Ausgabe: Phasen-/Summary-Logs und Restaurierungsnachweis |
| 7. Seiteneffekte und Ressourcen | Live-Locomotion.state schreiben und restaurieren · Logs |
| 8. Art der Verantwortung | Diagnoselogik und öffentliche Runtime-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Einzelnen Spielzustand und unabhängigen Gravityeffekt nicht mit umfassender Flugfunktion gleichsetzen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Runtime-Beobachtung und Prüfung. Technische Kopplung: runtime.ecs.read_fields/write_fields/query · runtime.world/types/has · shroudforge.log. Aufruferbezug: mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 11. Duplikation | Phasen-/Session-/Cleanup-Grundmuster ähnelt anderen Diagnoseproben. Geprüfte Daten-/Schreibpfade und Kriterien unterscheiden sich. |
| 12. Nutzungsstatus | Ladepfad über Modmanifest vorhanden, reale Liveinstallation nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel/Manifest und öffentlicher API-Vertrag zusammen erhalten. Diagnose-Cleanup-, Session-, Writegrenzen und Exportnamen beibehalten. |
| 14. Vorhandener Testschutz | diagnostic-mods/live-engine-flight-probe-check.py prüft 24 Szenarien:206-213. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Kontrollierte Flugzustandsprobe. Neue Einordnung: Runtime-Beobachtung und Prüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-engine-flight-probe/src/mod.lua:79 write_state · diagnostic-mods/live-engine-flight-probe/src/mod.lua:102 restore_once · diagnostic-mods/live-engine-flight-probe/src/mod.lua:123 step · diagnostic-mods/live-engine-flight-probe/src/mod.lua:229 Lifecycle

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-engine-flight-probe/src/mod.lua`.

### [diagnostic-mods/live-resource-scalar-probe-check.py](../../diagnostic-mods/live-resource-scalar-probe-check.py)

Vollständig gelesen, 232 Zeilen. SHA-256: `a5d0be2f6f731dd0d2698182123bb3a06089e0005e35be3223d20b9e638d7475`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Führt echte Ressourcenprobe gegen 27 simulierte Lua-Szenarien aus. Erzwingt unveränderliche eigene Snapshots, separates frisches Readback und maximal zwei physische Writes. Deckt Inventar-/Quell-/Enumfehler, Sessions, Rennen, fremde Änderungen und Cleanup-/Readback-Timeouts ab. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Führt echte Ressourcenprobe gegen 27 simulierte Lua-Szenarien aus. Erzwingt unveränderliche eigene Snapshots, separates frisches Readback und maximal zwei physische Writes. Deckt Inventar-/Quell-/Enumfehler, Sessions, Rennen, fremde Änderungen und Cleanup-/Readback-Timeouts ab. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-resource-scalar-probe-check.py:27 bootstrap`<br>`diagnostic-mods/live-resource-scalar-probe-check.py:40 readonly`<br>`diagnostic-mods/live-resource-scalar-probe-check.py:101 Schreibmock`<br>`diagnostic-mods/live-resource-scalar-probe-check.py:138 checks`<br>`diagnostic-mods/live-resource-scalar-probe-check.py:210 Szenarien` |
| 4. Aufrufer und Verbraucher | Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 5. Verwendete Komponenten | ctypes/json/pathlib · lokale lua54.dll · echte Probe/Manifeste |
| 6. Eingaben und Ausgaben | Eingaben: 27 Szenarien und echte Lua-Quelle Ausgabe: Assertion-/Erfolgsergebnisse |
| 7. Seiteneffekte und Ressourcen | lokale DLL laden · Dateien lesen |
| 8. Art der Verantwortung | Test-/Exportwerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine Produktionsnative-API oder Spielprozess wird im Test verwendet. Benutzerspezifischer DLL-Pfad:16. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: ctypes/json/pathlib · lokale lua54.dll · echte Probe/Manifeste. Aufruferbezug: Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 11. Duplikation | Phasen-/Session-/Cleanup-Grundmuster ähnelt anderen Diagnoseproben. Geprüfte Daten-/Schreibpfade und Kriterien unterscheiden sich. |
| 12. Nutzungsstatus | Direkt ausführbares Werkzeug, Automatisierung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Quellen-/Exportpfade und gegebenenfalls benutzerspezifische Lua-DLL beachten. Nachbearbeiter überschreibt Exporte, nicht ausführen. |
| 14. Vorhandener Testschutz | Datei ist selbst Fixture, nicht ausgeführt. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lokale Fixture der Ressourcenprobe. Neue Einordnung: Verhaltensprüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-resource-scalar-probe-check.py:27 bootstrap · diagnostic-mods/live-resource-scalar-probe-check.py:40 readonly · diagnostic-mods/live-resource-scalar-probe-check.py:101 Schreibmock · diagnostic-mods/live-resource-scalar-probe-check.py:138 checks · diagnostic-mods/live-resource-scalar-probe-check.py:210 Szenarien

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-resource-scalar-probe-check.py`.

### [diagnostic-mods/live-resource-scalar-probe/src/mod.lua](../../diagnostic-mods/live-resource-scalar-probe/src/mod.lua)

Vollständig gelesen, 243 Zeilen. SHA-256: `ea3e4f29b8476b29f028f2d555d6ca2a8c34110b7a89b412f4a4586129bb068a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Inventarisiert echte geladene ItemInfo-Ressourcen in Batches, validiert Glider-Slot und ResourceId und wählt deterministisch den ersten Glider. Erhöht ausschließlich yawAngleSpeed exakt um Float32-eins und restauriert es mit erwarteten Werten. Prüft unabhängig frische Resource-Snapshots und stoppt bei Fremdänderung/Sitzungswechsel/Unsicherheit. Beansprucht keinen Gameplaynachweis. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Inventarisiert echte geladene ItemInfo-Ressourcen in Batches, validiert Glider-Slot und ResourceId und wählt deterministisch den ersten Glider. Erhöht ausschließlich yawAngleSpeed exakt um Float32-eins und restauriert es mit erwarteten Werten. Prüft unabhängig frische Resource-Snapshots und stoppt bei Fremdänderung/Sitzungswechsel/Unsicherheit. Beansprucht keinen Gameplaynachweis. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-resource-scalar-probe/src/mod.lua:55 inspect`<br>`diagnostic-mods/live-resource-scalar-probe/src/mod.lua:76 read_current`<br>`diagnostic-mods/live-resource-scalar-probe/src/mod.lua:89 write_scalar`<br>`diagnostic-mods/live-resource-scalar-probe/src/mod.lua:108 restore_once`<br>`diagnostic-mods/live-resource-scalar-probe/src/mod.lua:129 step`<br>`diagnostic-mods/live-resource-scalar-probe/src/mod.lua:228 Lifecycle` |
| 4. Aufrufer und Verbraucher | mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 5. Verwendete Komponenten | game.assets get_loaded_resources/get_loaded_resource/write_loaded_field · runtime.world/types/has · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: geladene ItemInfos und EquipmentSlot-Enum Ausgabe: Ressourcen-Readback-/Summarylogs |
| 7. Seiteneffekte und Ressourcen | Live-ItemInfo.yawAngleSpeed schreiben/restaurieren · Logs |
| 8. Art der Verantwortung | Diagnoselogik und öffentliche Runtime-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Ressourcenskalarnachweis ist ausdrücklich von Gameplaywirkung getrennt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Runtime-Beobachtung und Prüfung. Technische Kopplung: game.assets get_loaded_resources/get_loaded_resource/write_loaded_field · runtime.world/types/has · shroudforge.log. Aufruferbezug: mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 11. Duplikation | Phasen-/Session-/Cleanup-Grundmuster ähnelt anderen Diagnoseproben. Geprüfte Daten-/Schreibpfade und Kriterien unterscheiden sich. |
| 12. Nutzungsstatus | Ladepfad über Modmanifest vorhanden, reale Liveinstallation nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel/Manifest und öffentlicher API-Vertrag zusammen erhalten. Diagnose-Cleanup-, Session-, Writegrenzen und Exportnamen beibehalten. |
| 14. Vorhandener Testschutz | diagnostic-mods/live-resource-scalar-probe-check.py prüft 27 Szenarien:210-216. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Kontrollierte geladene Ressourcenprobe. Neue Einordnung: Runtime-Beobachtung und Prüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-resource-scalar-probe/src/mod.lua:55 inspect · diagnostic-mods/live-resource-scalar-probe/src/mod.lua:76 read_current · diagnostic-mods/live-resource-scalar-probe/src/mod.lua:89 write_scalar · diagnostic-mods/live-resource-scalar-probe/src/mod.lua:108 restore_once · diagnostic-mods/live-resource-scalar-probe/src/mod.lua:129 step · diagnostic-mods/live-resource-scalar-probe/src/mod.lua:228 Lifecycle

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-resource-scalar-probe/src/mod.lua`.

### [diagnostic-mods/live-runtime-probe-check.py](../../diagnostic-mods/live-runtime-probe-check.py)

Vollständig gelesen, 120 Zeilen. SHA-256: `a1d5411f585d03a21a07b7f6aac7df0d5e28058c56ed41983dc656f8f1354949`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Lädt echte Probequelle und Manifeste und führt sie per ctypes in einer lokalen Lua-5.4-DLL gegen simulierte Registry/ECS-/Ressourcenantworten aus. Prüft Startup ohne Reads, resumierte Queries, alle Ergebnisklassen und isolierten Lazy-Decodefehler bei null Writes. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Lädt echte Probequelle und Manifeste und führt sie per ctypes in einer lokalen Lua-5.4-DLL gegen simulierte Registry/ECS-/Ressourcenantworten aus. Prüft Startup ohne Reads, resumierte Queries, alle Ergebnisklassen und isolierten Lazy-Decodefehler bei null Writes. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-runtime-probe-check.py:6 Pfade`<br>`diagnostic-mods/live-runtime-probe-check.py:25 bootstrap`<br>`diagnostic-mods/live-runtime-probe-check.py:82 checks`<br>`diagnostic-mods/live-runtime-probe-check.py:106 Szenarioschleife` |
| 4. Aufrufer und Verbraucher | Direkter Python-Aufruf, Automatisierung nicht nachgewiesen. |
| 5. Verwendete Komponenten | Python ctypes/json/pathlib · hart codierter lokaler lua54.dll-Pfad · echte Probe und Manifeste |
| 6. Eingaben und Ausgaben | Eingaben: Probequelle und zwei Lazy-Fehlerszenarien Ausgabe: Assertion-Ergebnis und Erfolgsmeldungen |
| 7. Seiteneffekte und Ressourcen | lokale native Lua-DLL laden · Dateien lesen |
| 8. Art der Verantwortung | Test- und Diagnosewerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: Python ctypes/json/pathlib · hart codierter lokaler lua54.dll-Pfad · echte Probe und Manifeste. Aufruferbezug: Direkter Python-Aufruf, Automatisierung nicht nachgewiesen. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Direkt ausführbares Diagnosewerkzeug, automatischer CI-Aufruf nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Probe-/Manifestpfade und hart codierter benutzerspezifischer Lua-DLL-Pfad:13 begrenzen Portabilität. |
| 14. Vorhandener Testschutz | Die Datei ist selbst ein Test. Nicht ausgeführt. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lokale Fixture der Laufzeit-Leseprobe. Neue Einordnung: Verhaltensprüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-runtime-probe-check.py:6 Pfade · diagnostic-mods/live-runtime-probe-check.py:25 bootstrap · diagnostic-mods/live-runtime-probe-check.py:82 checks · diagnostic-mods/live-runtime-probe-check.py:106 Szenarioschleife

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-runtime-probe-check.py`.

### [diagnostic-mods/live-runtime-probe/src/mod.lua](../../diagnostic-mods/live-runtime-probe/src/mod.lua)

Vollständig gelesen, 357 Zeilen. SHA-256: `b5429fbe52683684bd7f625316d544f36e5d954ef8c1b434d815157ccbd5f716`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft schrittweise und ausschließlich lesend Registry-, Attribut-/Funktionskatalog, geladene ItemInfos, Spielerwurzeln und alle registrierten ECS-Speichertypen. Resumiert Queries, prüft Rohbytes und typisierte/owned/selektive Reads und klassifiziert beobachtet, nicht beobachtet, teilweise lesbar und Queryfehler mit begrenztem Budget. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft schrittweise und ausschließlich lesend Registry-, Attribut-/Funktionskatalog, geladene ItemInfos, Spielerwurzeln und alle registrierten ECS-Speichertypen. Resumiert Queries, prüft Rohbytes und typisierte/owned/selektive Reads und klassifiziert beobachtet, nicht beobachtet, teilweise lesbar und Queryfehler mit begrenztem Budget. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-runtime-probe/src/mod.lua:58 inline_selections`<br>`diagnostic-mods/live-runtime-probe/src/mod.lua:96 query`<br>`diagnostic-mods/live-runtime-probe/src/mod.lua:142 registry_step`<br>`diagnostic-mods/live-runtime-probe/src/mod.lua:197 player_step`<br>`diagnostic-mods/live-runtime-probe/src/mod.lua:278 type_step`<br>`diagnostic-mods/live-runtime-probe/src/mod.lua:327 update`<br>`diagnostic-mods/live-runtime-probe/src/mod.lua:341 Lifecycle` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.ecs/values/functions · game.assets · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Live-Registry, Player und Ressourcen Ausgabe: strukturierte Probe-Logs und SUMMARY |
| 7. Seiteneffekte und Ressourcen | Loaderlog schreiben · keine Enginewrites/native Enginecalls |
| 8. Art der Verantwortung | Diagnoselogik mit öffentlicher Runtime-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Runtime-Beobachtung und Prüfung. Technische Kopplung: runtime.ecs/values/functions · game.assets · shroudforge.log. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Diagnose-Runtime-Mod laut Manifest. Zugehörige Fixture verwendet echte Quelle. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | diagnostic-mods/live-runtime-probe-check.py:82-104 prüft Lifecycle, Ergebnisklassen und null Writes. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Breite Laufzeit-Leseprobe. Neue Einordnung: Runtime-Beobachtung und Prüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-runtime-probe/src/mod.lua:58 inline_selections · diagnostic-mods/live-runtime-probe/src/mod.lua:96 query · diagnostic-mods/live-runtime-probe/src/mod.lua:142 registry_step · diagnostic-mods/live-runtime-probe/src/mod.lua:197 player_step · diagnostic-mods/live-runtime-probe/src/mod.lua:278 type_step · diagnostic-mods/live-runtime-probe/src/mod.lua:327 update · diagnostic-mods/live-runtime-probe/src/mod.lua:341 Lifecycle

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-runtime-probe/src/mod.lua`.

### [diagnostic-mods/live-runtime-write-probe-check.py](../../diagnostic-mods/live-runtime-write-probe-check.py)

Vollständig gelesen, 183 Zeilen. SHA-256: `ce34ad524cf3711a59abee9f8f13bb4daf7fb656325ce96127624fff3c68bb8e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Führt echte Attributprobe gegen 14 Lua-Mockszenarien aus. Simuliert Attributneuberechnung/Reskalierung, verzögerte Netzwerkreplikation, busy/CAS-Ausgänge, Session-/Fremdänderung, unsichere Outcomes und Unload. Prüft, dass Restaurierung fortgeschrittene aktuelle Stamina nicht blind auf alte Werte setzt. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Führt echte Attributprobe gegen 14 Lua-Mockszenarien aus. Simuliert Attributneuberechnung/Reskalierung, verzögerte Netzwerkreplikation, busy/CAS-Ausgänge, Session-/Fremdänderung, unsichere Outcomes und Unload. Prüft, dass Restaurierung fortgeschrittene aktuelle Stamina nicht blind auf alte Werte setzt. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-runtime-write-probe-check.py:23 bootstrap`<br>`diagnostic-mods/live-runtime-write-probe-check.py:72 Schreibmock`<br>`diagnostic-mods/live-runtime-write-probe-check.py:111 checks`<br>`diagnostic-mods/live-runtime-write-probe-check.py:169 Szenarien` |
| 4. Aufrufer und Verbraucher | Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 5. Verwendete Komponenten | ctypes/json/pathlib · lokale lua54.dll · echte Probe/Manifeste |
| 6. Eingaben und Ausgaben | Eingaben: 14 simulierte Szenarien Ausgabe: Assertion-/Erfolgsergebnisse |
| 7. Seiteneffekte und Ressourcen | lokale DLL laden · Dateien lesen |
| 8. Art der Verantwortung | Test-/Exportwerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Mock-Lifecycle ersetzt keinen Spiel-/Produktionscodec-Test. Benutzerspezifischer DLL-Pfad:13. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Verhaltensprüfung. Technische Kopplung: ctypes/json/pathlib · lokale lua54.dll · echte Probe/Manifeste. Aufruferbezug: Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 11. Duplikation | Phasen-/Session-/Cleanup-Grundmuster ähnelt anderen Diagnoseproben. Geprüfte Daten-/Schreibpfade und Kriterien unterscheiden sich. |
| 12. Nutzungsstatus | Direkt ausführbares Werkzeug, Automatisierung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Quellen-/Exportpfade und gegebenenfalls benutzerspezifische Lua-DLL beachten. Nachbearbeiter überschreibt Exporte, nicht ausführen. |
| 14. Vorhandener Testschutz | Datei ist selbst Fixture, nicht ausgeführt. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lokale Fixture der Attributprobe. Neue Einordnung: Verhaltensprüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-runtime-write-probe-check.py:23 bootstrap · diagnostic-mods/live-runtime-write-probe-check.py:72 Schreibmock · diagnostic-mods/live-runtime-write-probe-check.py:111 checks · diagnostic-mods/live-runtime-write-probe-check.py:169 Szenarien

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-runtime-write-probe-check.py`.

### [diagnostic-mods/live-runtime-write-probe/src/mod.lua](../../diagnostic-mods/live-runtime-write-probe/src/mod.lua)

Vollständig gelesen, 297 Zeilen. SHA-256: `4ce3a964a63f07851ba5bcab6c68f7075956685a1d205ce3c43ceb7077b660dd`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Prüft den typisierten Attributschreibpfad mit Stamina_Max_Adder+1 und Restaurierung. Validiert vollständige sint32-Rootstruktur und Base/Adder/Max-Beziehung, wartet auf volle Stamina/einen Spieler, prüft Berechnungstrace und separat replizierte NetworkPlayerAttributes. Begrenzte Wiederholungen, Besitz-/Sitzungskontrolle und Stop bei unsicherem Write. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Prüft den typisierten Attributschreibpfad mit Stamina_Max_Adder+1 und Restaurierung. Validiert vollständige sint32-Rootstruktur und Base/Adder/Max-Beziehung, wartet auf volle Stamina/einen Spieler, prüft Berechnungstrace und separat replizierte NetworkPlayerAttributes. Begrenzte Wiederholungen, Besitz-/Sitzungskontrolle und Stop bei unsicherem Write. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-runtime-write-probe/src/mod.lua:52 valid_root`<br>`diagnostic-mods/live-runtime-write-probe/src/mod.lua:81 unchanged_scope`<br>`diagnostic-mods/live-runtime-write-probe/src/mod.lua:103 update`<br>`diagnostic-mods/live-runtime-write-probe/src/mod.lua:122 restore_once`<br>`diagnostic-mods/live-runtime-write-probe/src/mod.lua:161 step`<br>`diagnostic-mods/live-runtime-write-probe/src/mod.lua:279 Lifecycle` |
| 4. Aufrufer und Verbraucher | mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 5. Verwendete Komponenten | runtime.ecs update_attribute/read_attributes/read_fields/get_attribute/query · runtime.world/has · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Stamina-Root und Netzwerkmaximum Ausgabe: Trace-/Readback-/Summarylogs |
| 7. Seiteneffekte und Ressourcen | Live-Stamina_Max_Adder schreiben/restaurieren · Logs |
| 8. Art der Verantwortung | Diagnoselogik und öffentliche Runtime-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Beweist nur den beschriebenen Attribut-/Netzwerkpfad. Keine breitere Runtime-Funktion aus der Probe ableiten. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Runtime-Beobachtung und Prüfung. Technische Kopplung: runtime.ecs update_attribute/read_attributes/read_fields/get_attribute/query · runtime.world/has · shroudforge.log. Aufruferbezug: mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 11. Duplikation | Phasen-/Session-/Cleanup-Grundmuster ähnelt anderen Diagnoseproben. Geprüfte Daten-/Schreibpfade und Kriterien unterscheiden sich. |
| 12. Nutzungsstatus | Ladepfad über Modmanifest vorhanden, reale Liveinstallation nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel/Manifest und öffentlicher API-Vertrag zusammen erhalten. Diagnose-Cleanup-, Session-, Writegrenzen und Exportnamen beibehalten. |
| 14. Vorhandener Testschutz | diagnostic-mods/live-runtime-write-probe-check.py prüft 14 Szenarien:169-171. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Kontrollierte Attribut- und Replikationsprobe. Neue Einordnung: Runtime-Beobachtung und Prüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-runtime-write-probe/src/mod.lua:52 valid_root · diagnostic-mods/live-runtime-write-probe/src/mod.lua:81 unchanged_scope · diagnostic-mods/live-runtime-write-probe/src/mod.lua:103 update · diagnostic-mods/live-runtime-write-probe/src/mod.lua:122 restore_once · diagnostic-mods/live-runtime-write-probe/src/mod.lua:161 step · diagnostic-mods/live-runtime-write-probe/src/mod.lua:279 Lifecycle

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-runtime-write-probe/src/mod.lua`.

### [diagnostic-mods/live-survival-probe/src/mod.lua](../../diagnostic-mods/live-survival-probe/src/mod.lua)

Vollständig gelesen, 78 Zeilen. SHA-256: `a852db78f95e2e66d2a870823df019d010911020565fdfeecbf029810d63a4aa`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Beobachtet lesend Survival-Wurzeln, FallDamage-Template, Haltbarkeitsservice und aktive Glider-Items. Durchläuft pro Tick maximal 64 Itemressourcen und protokolliert nur geänderte Werte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Beobachtet lesend Survival-Wurzeln, FallDamage-Template, Haltbarkeitsservice und aktive Glider-Items. Durchläuft pro Tick maximal 64 Itemressourcen und protokolliert nur geänderte Werte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `diagnostic-mods/live-survival-probe/src/mod.lua:16 observe_gliders`<br>`diagnostic-mods/live-survival-probe/src/mod.lua:43 Lifecycle`<br>`diagnostic-mods/live-survival-probe/src/mod.lua:51 Spielerquery`<br>`diagnostic-mods/live-survival-probe/src/mod.lua:61 Wurzeln`<br>`diagnostic-mods/live-survival-probe/src/mod.lua:72 GameSettings` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.ecs/world/services · game.assets · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: geladene ItemInfo · Spielerattribute und Vorlagen/Service Ausgabe: SF_SURVIVAL_READ_PROBE Logs |
| 7. Seiteneffekte und Ressourcen | Loaderlog schreiben · keine Enginewrites im Code |
| 8. Art der Verantwortung | Diagnoselogik mit öffentlicher Runtime-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Runtime-Beobachtung und Prüfung. Technische Kopplung: runtime.ecs/world/services · game.assets · shroudforge.log. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Diagnose-Runtime-Mod laut Manifest, tatsächlicher Einsatz nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Survival-Laufzeitbeobachtung. Neue Einordnung: Runtime-Beobachtung und Prüfung. Besitzer: Runtime-Proben. |

**Belege:** diagnostic-mods/live-survival-probe/src/mod.lua:16 observe_gliders · diagnostic-mods/live-survival-probe/src/mod.lua:43 Lifecycle · diagnostic-mods/live-survival-probe/src/mod.lua:51 Spielerquery · diagnostic-mods/live-survival-probe/src/mod.lua:61 Wurzeln · diagnostic-mods/live-survival-probe/src/mod.lua:72 GameSettings

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `diagnostic-mods/live-survival-probe/src/mod.lua`.

### [eml-mods/XHL-Auto-Loot/src/mod.lua](../../eml-mods/XHL-Auto-Loot/src/mod.lua)

Vollständig gelesen, 438 Zeilen. SHA-256: `11d4cc8c27c6273ea0d96ccfee23ca9c4c34ff1be976d259dfc3fbcc4888cd85`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Klassifiziert Loot-Templates anhand Namen und ECS-Komponenten. Kopiert vorhandene Pickup-Zonen aus Base_LootTouch_Orb, setzt Radius/Intervallmarker, entfernt Interaction-/optional Spawn-/Spark-Komponenten und mildert Gravitation. Optional stabilisiert es MiningNode-Emitter. Eine native Sidecar wird über einen präzisen Zeitmarker angesprochen, ihre Umsetzung steht nicht in dieser Datei. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Klassifiziert Loot-Templates anhand Namen und ECS-Komponenten. Kopiert vorhandene Pickup-Zonen aus Base_LootTouch_Orb, setzt Radius/Intervallmarker, entfernt Interaction-/optional Spawn-/Spark-Komponenten und mildert Gravitation. Optional stabilisiert es MiningNode-Emitter. Eine native Sidecar wird über einen präzisen Zeitmarker angesprochen, ihre Umsetzung steht nicht in dieser Datei. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/XHL-Auto-Loot/src/mod.lua:46 loadSettings`<br>`eml-mods/XHL-Auto-Loot/src/mod.lua:180 tuneZone`<br>`eml-mods/XHL-Auto-Loot/src/mod.lua:240 classifyLoot`<br>`eml-mods/XHL-Auto-Loot/src/mod.lua:288 stabilizeResourceEmitter`<br>`eml-mods/XHL-Auto-Loot/src/mod.lua:310 disableDropGravity`<br>`eml-mods/XHL-Auto-Loot/src/mod.lua:324 Templates`<br>`eml-mods/XHL-Auto-Loot/src/mod.lua:356/378 Patchschleifen` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | game.assets · io.exists/read_to_string · Lua Tabellen |
| 6. Eingaben und Ausgaben | Eingaben: config.txt Sammeloptionen/Radius · TemplateResource Ausgabe: angepasste Loot-/Emittertemplates · Logstatistik |
| 7. Seiteneffekte und Ressourcen | Assetänderungen im Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: game.assets · io.exists/read_to_string · Lua Tabellen. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Ähnliche config.txt-Trim-/Boolean-/KeyValue-Parser in mehreren XHL-Patchmods, im Scope vollständig gelesen. Unterschiede bei Inlinekommentaren und Grenzen verhindern eine ungeprüfte Vereinheitlichung. |
| 12. Nutzungsstatus | Patchmod laut Manifest. Native Sidecar-Verhalten nur dokumentiert/Marker gesetzt. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lootvorbereitung und automatische Sammelzonen. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** eml-mods/XHL-Auto-Loot/src/mod.lua:46 loadSettings · eml-mods/XHL-Auto-Loot/src/mod.lua:180 tuneZone · eml-mods/XHL-Auto-Loot/src/mod.lua:240 classifyLoot · eml-mods/XHL-Auto-Loot/src/mod.lua:288 stabilizeResourceEmitter · eml-mods/XHL-Auto-Loot/src/mod.lua:310 disableDropGravity · eml-mods/XHL-Auto-Loot/src/mod.lua:324 Templates · eml-mods/XHL-Auto-Loot/src/mod.lua:356/378 Patchschleifen

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/XHL-Auto-Loot/src/mod.lua`.

### [eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua](../../eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua)

Vollständig gelesen, 197 Zeilen. SHA-256: `3d1c3280625857352f33695e273f67fdda79dd0bff5328b4ecb1b873e1b1a12e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest config.txt und erhöht die Zugdistanz dreier exakt identifizierter Grappling-Hook-Items über ihre Impact-Konfigurations-ID. Validiert den Swing-Wert, schreibt ihn aber nicht. Prüft Identitäten, Dubletten, numerische Werte, Readback und alle drei Treffer. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest config.txt und erhöht die Zugdistanz dreier exakt identifizierter Grappling-Hook-Items über ihre Impact-Konfigurations-ID. Validiert den Swing-Wert, schreibt ihn aber nicht. Prüft Identitäten, Dubletten, numerische Werte, Readback und alle drei Treffer. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:33 load_config`<br>`eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:97 Ressourcen`<br>`eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:103 Zielschleife`<br>`eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:131 Pullzweig`<br>`eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:179 Vollständigkeit` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | game.assets · io.exists/read_to_string · Lua Standardbibliothek |
| 6. Eingaben und Ausgaben | Eingaben: config.txt enabled/pullDistanceMultiplier · ItemInfo Ausgabe: geänderte Zugwerte · Zusammenfassungslog |
| 7. Seiteneffekte und Ressourcen | Assetänderungen im Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: game.assets · io.exists/read_to_string · Lua Standardbibliothek. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Ähnliche config.txt-Trim-/Boolean-/KeyValue-Parser in mehreren XHL-Patchmods, im Scope vollständig gelesen. Unterschiede bei Inlinekommentaren und Grenzen verhindern eine ungeprüfte Vereinheitlichung. |
| 12. Nutzungsstatus | Patchmod laut Manifest, lokale Installation nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Grappling-Hook-Zugdistanz. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:33 load_config · eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:97 Ressourcen · eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:103 Zielschleife · eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:131 Pullzweig · eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua:179 Vollständigkeit

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/XHL-Grappling-Hook-Pull-2x/src/mod.lua`.

### [eml-mods/XHL-Max-Stack-65535/src/mod.lua](../../eml-mods/XHL-Max-Stack-65535/src/mod.lua)

Vollständig gelesen, 112 Zeilen. SHA-256: `a4cf4dea3dc3083423ea250538a392ca6b9e6eb6e280abcc0879b4cad651a46b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest config.txt, normalisiert die Stapelobergrenze auf 1..65535 und setzt ItemInfo.maxStackSize für stapelbare Items oder optional auch Einzelitems. Zählt übersprungene Fälle. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest config.txt, normalisiert die Stapelobergrenze auf 1..65535 und setzt ItemInfo.maxStackSize für stapelbare Items oder optional auch Einzelitems. Zählt übersprungene Fälle. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/XHL-Max-Stack-65535/src/mod.lua:29 normalize_stack_size`<br>`eml-mods/XHL-Max-Stack-65535/src/mod.lua:45 parse_config`<br>`eml-mods/XHL-Max-Stack-65535/src/mod.lua:92 Items`<br>`eml-mods/XHL-Max-Stack-65535/src/mod.lua:101 Zuweisung` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | game.assets · io.exists/read_to_string |
| 6. Eingaben und Ausgaben | Eingaben: config.txt enabled/maxStackSize/preserveSingleStackItems · ItemInfo Ausgabe: maxStackSize-Änderungen · Logzähler |
| 7. Seiteneffekte und Ressourcen | Assetänderungen im Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: game.assets · io.exists/read_to_string. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Ähnliche config.txt-Trim-/Boolean-/KeyValue-Parser in mehreren XHL-Patchmods, im Scope vollständig gelesen. Unterschiede bei Inlinekommentaren und Grenzen verhindern eine ungeprüfte Vereinheitlichung. |
| 12. Nutzungsstatus | Patchmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Itemstapelgrenzen. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** eml-mods/XHL-Max-Stack-65535/src/mod.lua:29 normalize_stack_size · eml-mods/XHL-Max-Stack-65535/src/mod.lua:45 parse_config · eml-mods/XHL-Max-Stack-65535/src/mod.lua:92 Items · eml-mods/XHL-Max-Stack-65535/src/mod.lua:101 Zuweisung

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/XHL-Max-Stack-65535/src/mod.lua`.

### [eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua](../../eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua)

Vollständig gelesen, 118 Zeilen. SHA-256: `6254d29d2fd693d3da0b31b407adcba5e82da04889ba034837a11f4a19c8a74a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Setzt die UI-Anzeigegrenzen der Flame-Level auf 255 und entfernt requiresAltarSlot für durch Item-/Entity-GUID identifizierte Flammenaltäre. Der Kommentar nennt eine native Sidecar für dynamische Kapazitäten, deren Implementierung in dieser Lua-Datei nicht enthalten ist. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Setzt die UI-Anzeigegrenzen der Flame-Level auf 255 und entfernt requiresAltarSlot für durch Item-/Entity-GUID identifizierte Flammenaltäre. Der Kommentar nennt eine native Sidecar für dynamische Kapazitäten, deren Implementierung in dieser Lua-Datei nicht enthalten ist. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:27 read_enabled`<br>`eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:56 patch_display_limit`<br>`eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:75 remove_placement_slot_gate`<br>`eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:106 Aufrufe` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | game.assets · io.exists/read_to_string |
| 6. Eingaben und Ausgaben | Eingaben: config.txt enabled · BalancingTable und ItemInfo Ausgabe: Anzeige-/requiresAltarSlot-Änderungen · Logzähler |
| 7. Seiteneffekte und Ressourcen | Assetänderungen im Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: game.assets · io.exists/read_to_string. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Ähnliche config.txt-Trim-/Boolean-/KeyValue-Parser in mehreren XHL-Patchmods, im Scope vollständig gelesen. Unterschiede bei Inlinekommentaren und Grenzen verhindern eine ungeprüfte Vereinheitlichung. |
| 12. Nutzungsstatus | Patchmod laut Manifest. Sidecar-Verhalten hier nur dokumentiert. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Flammenaltar-Platzierungsregel. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:27 read_enabled · eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:56 patch_display_limit · eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:75 remove_placement_slot_gate · eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua:106 Aufrufe

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/XHL-Unlimited-Flame-Altars/src/mod.lua`.

### [eml-mods/XHL-Vein-Mining/src/mod.lua](../../eml-mods/XHL-Vein-Mining/src/mod.lua)

Vollständig gelesen, 65 Zeilen. SHA-256: `955ec2e087a37ad331cc968469e15f3f6c534f393345d402ae923279c0d9455e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Setzt ausschließlich die Reichweite von sieben per GUID ausgewählten Spitzhacken auf zehn Meter und meldet Treffer. Die eigentliche Aderabbau-/Multiplayerfunktion wird nur im Kommentar und langen Logtext einer externen nativen Sidecar zugeschrieben. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Setzt ausschließlich die Reichweite von sieben per GUID ausgewählten Spitzhacken auf zehn Meter und meldet Treffer. Die eigentliche Aderabbau-/Multiplayerfunktion wird nur im Kommentar und langen Logtext einer externen nativen Sidecar zugeschrieben. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/XHL-Vein-Mining/src/mod.lua:11 PICKAXE_ITEM_GUIDS`<br>`eml-mods/XHL-Vein-Mining/src/mod.lua:31 patch_pickaxe_reach`<br>`eml-mods/XHL-Vein-Mining/src/mod.lua:44 Zuweisung`<br>`eml-mods/XHL-Vein-Mining/src/mod.lua:61 Sidecar-Logtext` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | game.assets |
| 6. Eingaben und Ausgaben | Eingaben: ItemInfo von sieben GUIDs Ausgabe: equipment.maxDistance=10 und Log |
| 7. Seiteneffekte und Ressourcen | Assetänderungen im Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: game.assets. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Patchmod laut Manifest. Sidecar-Funktion in diesem Lua-Code nicht implementiert. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Spitzhackenreichweite. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** eml-mods/XHL-Vein-Mining/src/mod.lua:11 PICKAXE_ITEM_GUIDS · eml-mods/XHL-Vein-Mining/src/mod.lua:31 patch_pickaxe_reach · eml-mods/XHL-Vein-Mining/src/mod.lua:44 Zuweisung · eml-mods/XHL-Vein-Mining/src/mod.lua:61 Sidecar-Logtext

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/XHL-Vein-Mining/src/mod.lua`.

### [eml-mods/XHL-Workshop-Speed-20x/src/mod.lua](../../eml-mods/XHL-Workshop-Speed-20x/src/mod.lua)

Vollständig gelesen, 121 Zeilen. SHA-256: `23d276f1c2891c7c74cceded4c1c7b826432c8352a3059afe2c871a8d1280420`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest config.txt und teilt alle positiven Rezeptdauern durch einen validierten Multiplikator. Rundet auf mindestens eine Nanosekunde, zählt Rezepte und lehnt einen leeren Patch ab. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest config.txt und teilt alle positiven Rezeptdauern durch einen validierten Multiplikator. Rundet auf mindestens eine Nanosekunde, zählt Rezepte und lehnt einen leeren Patch ab. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:24 loadConfig`<br>`eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:68 registries`<br>`eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:77 Rezeptschleife`<br>`eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:89 Rundung`<br>`eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:106 Leerschutz` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | game.assets · io.exists/read_to_string |
| 6. Eingaben und Ausgaben | Eingaben: config.txt enabled/multiplier · RecipeRegistryResource Ausgabe: geänderte craftingDuration-Werte · Logstatistik |
| 7. Seiteneffekte und Ressourcen | Assetänderungen im Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: game.assets · io.exists/read_to_string. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Ähnliche config.txt-Trim-/Boolean-/KeyValue-Parser in mehreren XHL-Patchmods, im Scope vollständig gelesen. Unterschiede bei Inlinekommentaren und Grenzen verhindern eine ungeprüfte Vereinheitlichung. |
| 12. Nutzungsstatus | Patchmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Relative Produktionsbeschleunigung. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:24 loadConfig · eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:68 registries · eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:77 Rezeptschleife · eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:89 Rundung · eml-mods/XHL-Workshop-Speed-20x/src/mod.lua:106 Leerschutz

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/XHL-Workshop-Speed-20x/src/mod.lua`.

### [eml-mods/fishing-exporter/src/mod.lua](../../eml-mods/fishing-exporter/src/mod.lua)

Vollständig gelesen, 582 Zeilen. SHA-256: `b9330ba8692d25df83e1c30a18759c03cffaacf4132f31832067fb2cf5313ac8`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verknüpft Lokalisierung/Itemregistrierung, FishingSpot-Templates, Spawn-/Trash-Tabellen, Angelruten und Köder zu drei versionierten CSV-Exporten. Berechnet Gewichte/Chancen und exportiert sowohl namensheuristische Biome als auch teilweise aufgelöste Scatterbeziehungen. Liest zahlreiche Ressourcentypen, ohne Assets zu schreiben. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verknüpft Lokalisierung/Itemregistrierung, FishingSpot-Templates, Spawn-/Trash-Tabellen, Angelruten und Köder zu drei versionierten CSV-Exporten. Berechnet Gewichte/Chancen und exportiert sowohl namensheuristische Biome als auch teilweise aufgelöste Scatterbeziehungen. Liest zahlreiche Ressourcentypen, ohne Assets zu schreiben. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/fishing-exporter/src/mod.lua:88 ItemResolver:init`<br>`eml-mods/fishing-exporter/src/mod.lua:150 get_name`<br>`eml-mods/fishing-exporter/src/mod.lua:191 get_safe_timestamp`<br>`eml-mods/fishing-exporter/src/mod.lua:245 Templates`<br>`eml-mods/fishing-exporter/src/mod.lua:291 process_scatter_node`<br>`eml-mods/fishing-exporter/src/mod.lua:314 scan_table_recursive`<br>`eml-mods/fishing-exporter/src/mod.lua:352 fish_tables`<br>`eml-mods/fishing-exporter/src/mod.lua:416/515/580 io.export` |
| 4. Aufrufer und Verbraucher | mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 5. Verwendete Komponenten | loader.features.export · game.assets/guid · integer · io.export · os.date oder game.get_time |
| 6. Eingaben und Ausgaben | Eingaben: Lokalisierung/ItemInfo/ItemRegistry · Fishing-/Trash-Tabellen · Template-/Scatterressourcen Ausgabe: Spawn-, Ruten- und Köder/Trash-CSV |
| 7. Seiteneffekte und Ressourcen | Exportdateien und Logs schreiben |
| 8. Art der Verantwortung | Exportlogik und Assetintegration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Biome werden teils heuristisch aus Namen abgeleitet:62ff. und Scattercrawler ist auf Lua-Tabellen/Tiefe6 beschränkt:314ff. Exportspalten unterscheiden NameBasedBiome und ConnectedBiome, tatsächliche Weltbiome nicht allgemein bewiesen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: loader.features.export · game.assets/guid · integer · io.export · os.date oder game.get_time. Aufruferbezug: mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 11. Duplikation | Dateinamenssäuberung, CSV-Escaping und Teile der Lokalisierung wiederholen sich in den drei EML-Exportmods. |
| 12. Nutzungsstatus | Ladepfad über Modmanifest vorhanden, reale Liveinstallation nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel/Manifest und öffentlicher API-Vertrag zusammen erhalten. Diagnose-Cleanup-, Session-, Writegrenzen und Exportnamen beibehalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Fischerei-Datenexport. Neue Einordnung: Modpakete als Produkte. Besitzer: Datenexport-Mods. |

**Belege:** eml-mods/fishing-exporter/src/mod.lua:88 ItemResolver:init · eml-mods/fishing-exporter/src/mod.lua:150 get_name · eml-mods/fishing-exporter/src/mod.lua:191 get_safe_timestamp · eml-mods/fishing-exporter/src/mod.lua:245 Templates · eml-mods/fishing-exporter/src/mod.lua:291 process_scatter_node · eml-mods/fishing-exporter/src/mod.lua:314 scan_table_recursive · eml-mods/fishing-exporter/src/mod.lua:352 fish_tables · eml-mods/fishing-exporter/src/mod.lua:416/515/580 io.export

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/fishing-exporter/src/mod.lua`.

### [eml-mods/item-translator/src/mod.lua](../../eml-mods/item-translator/src/mod.lua)

Vollständig gelesen, 141 Zeilen. SHA-256: `d629a87edfddcafc35697ba2a5b340926c0356186b5af33317a0f246cfef5685`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liest englische Lokalisierungsdaten und Items, ordnet Namenshashes den Texten zu und exportiert zwei versionierte CSV-Dateien für Iteminfos und Übersetzungen. Die Abschlussmeldung spricht von drei Dateien, im Code sind zwei io.export-Aufrufe vorhanden. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liest englische Lokalisierungsdaten und Items, ordnet Namenshashes den Texten zu und exportiert zwei versionierte CSV-Dateien für Iteminfos und Übersetzungen. Die Abschlussmeldung spricht von drei Dateien, im Code sind zwei io.export-Aufrufe vorhanden. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/item-translator/src/mod.lua:16 Exportgate`<br>`eml-mods/item-translator/src/mod.lua:23 sanitize_filename_tag`<br>`eml-mods/item-translator/src/mod.lua:32 safe_to_string`<br>`eml-mods/item-translator/src/mod.lua:80 translations`<br>`eml-mods/item-translator/src/mod.lua:112 all_items`<br>`eml-mods/item-translator/src/mod.lua:127/138 io.export` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | loader.features.export · game.assets/guid · integer.to_string · io.export |
| 6. Eingaben und Ausgaben | Eingaben: LocaTagCollectionResource und En_Us-Content · ItemInfo · Spielversion Ausgabe: item_info_export_FULL CSV · translation_enus_export_FULL CSV |
| 7. Seiteneffekte und Ressourcen | Exportdateien schreiben · Konsolenlog |
| 8. Art der Verantwortung | Exportlogik und Assetintegration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Log behauptet drei exportierte Dateien:141, tatsächliche io.export-Aufrufe sind nur :127 und :138. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: loader.features.export · game.assets/guid · integer.to_string · io.export. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Exportmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Item- und Übersetzungsexport. Neue Einordnung: Modpakete als Produkte. Besitzer: Datenexport-Mods. |

**Belege:** eml-mods/item-translator/src/mod.lua:16 Exportgate · eml-mods/item-translator/src/mod.lua:23 sanitize_filename_tag · eml-mods/item-translator/src/mod.lua:32 safe_to_string · eml-mods/item-translator/src/mod.lua:80 translations · eml-mods/item-translator/src/mod.lua:112 all_items · eml-mods/item-translator/src/mod.lua:127/138 io.export

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/item-translator/src/mod.lua`.

### [eml-mods/kfc-parser-mimic/batch_format_assets.py](../../eml-mods/kfc-parser-mimic/batch_format_assets.py)

Vollständig gelesen, 259 Zeilen. SHA-256: `990414fd08c3b226025bd876d45bedc609a40993bdf953261d5f120fdf566f7e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Formatiert nachträglich exportierte Asset-JSONs parallel. Sucht einen lexikografisch zuletzt sortierten Typen-CSV, repariert bestimmte eingebettete KeyValue-Strings rekursiv, entpackt data-Objekte, ergänzt Typ/GUID/Part aus Ordner-/Dateinamen und überschreibt veränderte JSON-Dateien. Ignoriert Verzeichnisse mit archive im Namen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Formatiert nachträglich exportierte Asset-JSONs parallel. Sucht einen lexikografisch zuletzt sortierten Typen-CSV, repariert bestimmte eingebettete KeyValue-Strings rekursiv, entpackt data-Objekte, ergänzt Typ/GUID/Part aus Ordner-/Dateinamen und überschreibt veränderte JSON-Dateien. Ignoriert Verzeichnisse mit archive im Namen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/kfc-parser-mimic/batch_format_assets.py:17 load_type_mappings`<br>`eml-mods/kfc-parser-mimic/batch_format_assets.py:51 sanitize_malformed_components`<br>`eml-mods/kfc-parser-mimic/batch_format_assets.py:81 decode_embedded_json`<br>`eml-mods/kfc-parser-mimic/batch_format_assets.py:105 process_file`<br>`eml-mods/kfc-parser-mimic/batch_format_assets.py:169 main`<br>`eml-mods/kfc-parser-mimic/batch_format_assets.py:225 ProcessPoolExecutor` |
| 4. Aufrufer und Verbraucher | Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 5. Verwendete Komponenten | Python concurrent.futures/csv/json/pathlib/re/os · exported_assets-Verzeichnisse · 0_resource_types_export CSV |
| 6. Eingaben und Ausgaben | Eingaben: exportierte JSONs und Typen-CSV unter aktuellem Arbeitsverzeichnis Ausgabe: formatierte und metadatenangereicherte JSONs · Fortschritts-/Fehlerlogs |
| 7. Seiteneffekte und Ressourcen | Dateien überschreiben · Workerprozesse starten |
| 8. Art der Verantwortung | Test-/Exportwerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: ROOT_DIR='.' macht den Startordner zum Such-/Schreibumfang. Regel für malformed Komponententext:76 ist eine Heuristik, keine universelle Konvertierung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: Python concurrent.futures/csv/json/pathlib/re/os · exported_assets-Verzeichnisse · 0_resource_types_export CSV. Aufruferbezug: Direkter Python-Aufruf, CI-Aufruf nicht nachgewiesen. |
| 11. Duplikation | Dateinamenssäuberung, CSV-Escaping und Teile der Lokalisierung wiederholen sich in den drei EML-Exportmods. |
| 12. Nutzungsstatus | Direkt ausführbares Werkzeug, Automatisierung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Quellen-/Exportpfade und gegebenenfalls benutzerspezifische Lua-DLL beachten. Nachbearbeiter überschreibt Exporte, nicht ausführen. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Nachbearbeitung von Assetexporten. Neue Einordnung: Modpakete als Produkte. Besitzer: Datenexport-Mods. |

**Belege:** eml-mods/kfc-parser-mimic/batch_format_assets.py:17 load_type_mappings · eml-mods/kfc-parser-mimic/batch_format_assets.py:51 sanitize_malformed_components · eml-mods/kfc-parser-mimic/batch_format_assets.py:81 decode_embedded_json · eml-mods/kfc-parser-mimic/batch_format_assets.py:105 process_file · eml-mods/kfc-parser-mimic/batch_format_assets.py:169 main · eml-mods/kfc-parser-mimic/batch_format_assets.py:225 ProcessPoolExecutor

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/kfc-parser-mimic/batch_format_assets.py`.

### [eml-mods/kfc-parser-mimic/src/mod.lua](../../eml-mods/kfc-parser-mimic/src/mod.lua)

Vollständig gelesen, 548 Zeilen. SHA-256: `911ab7fea0cef855411342342c2156be0e8f8d4e7ac6ddda72b32220767fcbdc`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Exportiert einen Ressourcentyp-Katalog als CSV und pro verfügbarer Ressource JSON in versionierte Typordner. Enthält eigene JSON-Säuberung/Serialisierung, Version-/Typ-/GUID-/Part-Fallbacks und begrenztes Dereferenzieren. Überspringt standardmäßig vorhandene Exporte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Exportiert einen Ressourcentyp-Katalog als CSV und pro verfügbarer Ressource JSON in versionierte Typordner. Enthält eigene JSON-Säuberung/Serialisierung, Version-/Typ-/GUID-/Part-Fallbacks und begrenztes Dereferenzieren. Überspringt standardmäßig vorhandene Exporte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `eml-mods/kfc-parser-mimic/src/mod.lua:14 sanitize_filename_tag`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:22 get_game_version`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:133 JSON.clean_structure`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:154 JSON.encode`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:275 export_resource_types`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:362 local_fnv1a_32_hex`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:414 local_dereference_deep`<br>`eml-mods/kfc-parser-mimic/src/mod.lua:485 export_kfc_exact_format` |
| 4. Aufrufer und Verbraucher | mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 5. Verwendete Komponenten | loader.features.export · game.assets · integer · io.export/exists/open · Lua bit/bit32 optional |
| 6. Eingaben und Ausgaben | Eingaben: Ressourcentypen und Assetdaten · Spielversion Ausgabe: 0_resource_types_export CSV · exported_assets_VERSION/TYPE/*.json |
| 7. Seiteneffekte und Ressourcen | Exportdateien und Logs schreiben |
| 8. Art der Verantwortung | Exportlogik und Assetintegration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: pcall(export_json_file):529 verwirft das Erfolgsergebnis und erhöht saved auch bei Fehler:530. Die Logzahl beweist erfolgreiche Exporte daher nicht. · Hashfallback local_fnv1a_32_hex multipliziert vor XOR:368-380. Eine Übereinstimmung mit dem behaupteten Formatnamen ist nicht durch diesen Scope geprüft. · Eigene JSON-Serialisierung, CSV-Katalog und Exportdriver sind verschiedene technische Aufgaben innerhalb desselben Exportzwecks. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: loader.features.export · game.assets · integer · io.export/exists/open · Lua bit/bit32 optional. Aufruferbezug: mod.json deklariert runtime beziehungsweise export, Loader runner/state.rs:154 lädt src/mod.lua. Diagnose-Fixtures lesen die echte Probequelle. |
| 11. Duplikation | Dateinamenssäuberung, CSV-Escaping und Teile der Lokalisierung wiederholen sich in den drei EML-Exportmods. |
| 12. Nutzungsstatus | Ladepfad über Modmanifest vorhanden, reale Liveinstallation nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel/Manifest und öffentlicher API-Vertrag zusammen erhalten. Diagnose-Cleanup-, Session-, Writegrenzen und Exportnamen beibehalten. |
| 14. Vorhandener Testschutz | Nicht nachgewiesen. · Nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Allgemeiner Ressourcen- und Metadatenexport. Neue Einordnung: Modpakete als Produkte. Besitzer: Datenexport-Mods. |

**Belege:** eml-mods/kfc-parser-mimic/src/mod.lua:14 sanitize_filename_tag · eml-mods/kfc-parser-mimic/src/mod.lua:22 get_game_version · eml-mods/kfc-parser-mimic/src/mod.lua:133 JSON.clean_structure · eml-mods/kfc-parser-mimic/src/mod.lua:154 JSON.encode · eml-mods/kfc-parser-mimic/src/mod.lua:275 export_resource_types · eml-mods/kfc-parser-mimic/src/mod.lua:362 local_fnv1a_32_hex · eml-mods/kfc-parser-mimic/src/mod.lua:414 local_dereference_deep · eml-mods/kfc-parser-mimic/src/mod.lua:485 export_kfc_exact_format

**Evidenzgrenze:** Implementierung vollständig gelesen. Reale Engine-/Gameplaywirkung nicht ausgeführt und nicht durch lokale Mocks bewiesen.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `eml-mods/kfc-parser-mimic/src/mod.lua`.

### [mods/sf-survival/src/mod.lua](../../mods/sf-survival/src/mod.lua)

Vollständig gelesen, 109 Zeilen. SHA-256: `c715856a47e4a80b7a5e8da7eccba1da31e14425cfbaa9f2094d6420d113986e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verändert im autoritativen Weltzustand Survival-Eingänge über runtime.overrides. Hebt bei gewählten Optionen Stamina/Mana/Oxygen/BodyHeat-Minima auf ihr Maximum, verstärkt positive Regeneration, deaktiviert Fallschaden über Templategrenze und Werkzeughaltbarkeit über GameSettings. Restauriert deaktivierte eigene Overrides und meldet den Effektzustand. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verändert im autoritativen Weltzustand Survival-Eingänge über runtime.overrides. Hebt bei gewählten Optionen Stamina/Mana/Oxygen/BodyHeat-Minima auf ihr Maximum, verstärkt positive Regeneration, deaktiviert Fallschaden über Templategrenze und Werkzeughaltbarkeit über GameSettings. Restauriert deaktivierte eigene Overrides und meldet den Effektzustand. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `mods/sf-survival/src/mod.lua:4 attributes`<br>`mods/sf-survival/src/mod.lua:27 attribute_target`<br>`mods/sf-survival/src/mod.lua:43 update`<br>`mods/sf-survival/src/mod.lua:73 FallDamage`<br>`mods/sf-survival/src/mod.lua:88 enableDurability`<br>`mods/sf-survival/src/mod.lua:103 Lifecycle` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.overrides · runtime.ecs · runtime.services · runtime.world · shroudforge.settings |
| 6. Eingaben und Ausgaben | Eingaben: Survival-Einstellungen · PlayerInput-Entities · Attribut-/Template-/Servicestände Ausgabe: bestätigte Overrides und Effektstatus |
| 7. Seiteneffekte und Ressourcen | Live-Spielattribute/Template/GameSettings schreiben |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.overrides · runtime.ecs · runtime.services · runtime.world · shroudforge.settings. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Vom Modmanifest als runtime deklariert. Aktueller Runtime-Ladepfad nachgewiesen, konkrete Installation außerhalb des Repos ungeprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | src/loader/api/tests/survival.rs referenziert die echte Moddatei:7. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Survival-Spielregeln. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** mods/sf-survival/src/mod.lua:4 attributes · mods/sf-survival/src/mod.lua:27 attribute_target · mods/sf-survival/src/mod.lua:43 update · mods/sf-survival/src/mod.lua:73 FallDamage · mods/sf-survival/src/mod.lua:88 enableDurability · mods/sf-survival/src/mod.lua:103 Lifecycle

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `mods/sf-survival/src/mod.lua`.

### [mods/sf-unlimited-flight/src/mod.lua](../../mods/sf-unlimited-flight/src/mod.lua)

Vollständig gelesen, 334 Zeilen. SHA-256: `12924e62d8744de823ba0be246e5d3fa4ad5b31f684a16a9db2a6ed0fdc40fea`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Verändert Glider-Ausdauer und Geschwindigkeitskoeffizienten über eigene rücksetzbare Overrides. Erkennt die aktive Gleiteraktion aus Actor-Zustandsmasken, löst die echte ItemRegistry-Beziehung mit kompatiblem geladenem Indexfallback auf und steuert pitchAngleSpeed anhand Quaternion, Gravitation, Eingabe und beobachtetem Updateintervall. Begrenzung für einen autoritativen Spieler und invertierte Lage schützt die gemeinsame ItemInfo-Ressource. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Verändert Glider-Ausdauer und Geschwindigkeitskoeffizienten über eigene rücksetzbare Overrides. Erkennt die aktive Gleiteraktion aus Actor-Zustandsmasken, löst die echte ItemRegistry-Beziehung mit kompatiblem geladenem Indexfallback auf und steuert pitchAngleSpeed anhand Quaternion, Gravitation, Eingabe und beobachtetem Updateintervall. Begrenzung für einen autoritativen Spieler und invertierte Lage schützt die gemeinsame ItemInfo-Ressource. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `mods/sf-unlimited-flight/src/mod.lua:55 validate_glider`<br>`mods/sf-unlimited-flight/src/mod.lua:69 current_glider`<br>`mods/sf-unlimited-flight/src/mod.lua:117 active_item`<br>`mods/sf-unlimited-flight/src/mod.lua:159 pitch_command`<br>`mods/sf-unlimited-flight/src/mod.lua:209 update`<br>`mods/sf-unlimited-flight/src/mod.lua:325 Lifecycle` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.overrides · runtime.ecs/world · game.assets geladene Ressourcen und ItemRegistry · shroudforge.settings |
| 6. Eingaben und Ausgaben | Eingaben: unlimitedFlying und Prozentmultiplikatoren · Actor-/PlayerInput-/Transform-/Gravity-Felder · ItemInfo Ausgabe: Glider-Overrides und Effektstatus |
| 7. Seiteneffekte und Ressourcen | Live-Attribute und geladene ItemInfo-Skalare schreiben · Vorbereitungsindex anfordern |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.overrides · runtime.ecs/world · game.assets geladene Ressourcen und ItemRegistry · shroudforge.settings. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Runtime-Mod laut Manifest, Aufruf durch Runtime-Loader. Liveinstallation nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | src/loader/api/tests/flight.rs referenziert echte Moddatei:7. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Flugsteuerung und Gleiterregeln. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** mods/sf-unlimited-flight/src/mod.lua:55 validate_glider · mods/sf-unlimited-flight/src/mod.lua:69 current_glider · mods/sf-unlimited-flight/src/mod.lua:117 active_item · mods/sf-unlimited-flight/src/mod.lua:159 pitch_command · mods/sf-unlimited-flight/src/mod.lua:209 update · mods/sf-unlimited-flight/src/mod.lua:325 Lifecycle

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `mods/sf-unlimited-flight/src/mod.lua`.

### [reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua](../../reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua)

Vollständig gelesen, 68 Zeilen. SHA-256: `35eaf48f77e00538e3f51877bb9d801b535863e569cb3283e83c63e80f4d5a16`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Füllt alle beobachteten PlayerInput/Stamina-Entities periodisch auf das reflektierte Stamina_Max und wartet bei fehlender Welt/Schreibberechtigung oder instabilem Snapshot. Berichtet nur Zustandsänderungen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Füllt alle beobachteten PlayerInput/Stamina-Entities periodisch auf das reflektierte Stamina_Max und wartet bei fehlender Welt/Schreibberechtigung oder instabilem Snapshot. Berichtet nur Zustandsänderungen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:10 update`<br>`reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:19 Query`<br>`reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:44 update_attribute`<br>`reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:59 Lifecycle` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.ecs.query/read_attributes/update_attribute · runtime.has · shroudforge.settings |
| 6. Eingaben und Ausgaben | Eingaben: autoRefill · Stamina und Maximum Ausgabe: Effektstatus |
| 7. Seiteneffekte und Ressourcen | Live-Stamina schreiben |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.ecs.query/read_attributes/update_attribute · runtime.has · shroudforge.settings. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Runtime-Referenzmod laut Manifest, lokale Installation nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | runtime_functions.rs:735 stamina_mod_uses_typed_roots_and_retries_without_a_modifier_patch. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Ausdauerautomatik. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:10 update · reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:19 Query · reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:44 update_attribute · reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua:59 Lifecycle

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua`.

### [reference-mods-0.9/sf-infinite-item-split/src/mod.lua](../../reference-mods-0.9/sf-infinite-item-split/src/mod.lua)

Vollständig gelesen, 42 Zeilen. SHA-256: `d42cc741629e0cbb1736e0c60ad69fbd45c49be38705dbd478cf7516f379f76a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet beim Laden den buildgeprüften preserve_split_source-Modifikator, aktiviert ihn und restauriert ihn beim Entladen. Protokolliert, dass dieselbe Inventory-Subtraktion auch andere Aufrufer betrifft. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet beim Laden den buildgeprüften preserve_split_source-Modifikator, aktiviert ihn und restauriert ihn beim Entladen. Protokolliert, dass dieselbe Inventory-Subtraktion auch andere Aufrufer betrifft. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-infinite-item-split/src/mod.lua:5 set_enabled`<br>`reference-mods-0.9/sf-infinite-item-split/src/mod.lua:25 on_load`<br>`reference-mods-0.9/sf-infinite-item-split/src/mod.lua:28 bind_modifier`<br>`reference-mods-0.9/sf-infinite-item-split/src/mod.lua:39 on_unload` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.functions.bind_modifier · runtime.gameplay.patch · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Buildprofil-Modifikator Ausgabe: Effektstatus |
| 7. Seiteneffekte und Ressourcen | Native Funktionsmodifikation aktivieren/restaurieren |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.functions.bind_modifier · runtime.gameplay.patch · shroudforge.log. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Bind-/Enable-/Unload-Gerüst weitgehend gleich in sf-infinite-item-split, sf-infinite-item-use und sf-no-resource-cost, unterschiedliche native Modifier und Fehlermeldungen. |
| 12. Nutzungsstatus | Runtime-Referenzmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | runtime_functions.rs:644 bundled_mods_bind_profile_modifiers_and_restore_on_unload. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Inventarteilungsregel über native Modifikation. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-infinite-item-split/src/mod.lua:5 set_enabled · reference-mods-0.9/sf-infinite-item-split/src/mod.lua:25 on_load · reference-mods-0.9/sf-infinite-item-split/src/mod.lua:28 bind_modifier · reference-mods-0.9/sf-infinite-item-split/src/mod.lua:39 on_unload

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-infinite-item-split/src/mod.lua`.

### [reference-mods-0.9/sf-infinite-item-use/src/mod.lua](../../reference-mods-0.9/sf-infinite-item-use/src/mod.lua)

Vollständig gelesen, 33 Zeilen. SHA-256: `04fed23a9b83eee7dbec4fa212545ddeb00cbf4e7a1bec83a3273bf58ba68244`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet und aktiviert den buildgeprüften clear_item_use_argument-Modifikator und restauriert ihn beim Entladen. Verweigert die Aktivierung bei nicht verfügbarem Profil. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet und aktiviert den buildgeprüften clear_item_use_argument-Modifikator und restauriert ihn beim Entladen. Verweigert die Aktivierung bei nicht verfügbarem Profil. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-infinite-item-use/src/mod.lua:5 set_enabled`<br>`reference-mods-0.9/sf-infinite-item-use/src/mod.lua:19 on_load`<br>`reference-mods-0.9/sf-infinite-item-use/src/mod.lua:30 on_unload` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.functions.bind_modifier · runtime.gameplay.patch · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Buildprofil-Modifikator Ausgabe: Effektstatus |
| 7. Seiteneffekte und Ressourcen | Native Funktionsmodifikation aktivieren/restaurieren |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.functions.bind_modifier · runtime.gameplay.patch · shroudforge.log. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Bind-/Enable-/Unload-Gerüst weitgehend gleich in sf-infinite-item-split, sf-infinite-item-use und sf-no-resource-cost, unterschiedliche native Modifier und Fehlermeldungen. |
| 12. Nutzungsstatus | Runtime-Referenzmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | runtime_functions.rs:644 bundled_mods_bind_profile_modifiers_and_restore_on_unload. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Verbrauchsregel über native Modifikation. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-infinite-item-use/src/mod.lua:5 set_enabled · reference-mods-0.9/sf-infinite-item-use/src/mod.lua:19 on_load · reference-mods-0.9/sf-infinite-item-use/src/mod.lua:30 on_unload

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-infinite-item-use/src/mod.lua`.

### [reference-mods-0.9/sf-no-fall-damage/src/mod.lua](../../reference-mods-0.9/sf-no-fall-damage/src/mod.lua)

Vollständig gelesen, 173 Zeilen. SHA-256: `cc31aec6ef1868fb9c64bd270b36ba5d1be9f02c600d8dafd407485d6886ee25`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Setzt FallDamage-Startdistanz in den tatsächlich vom Spieler verwendeten geladenen Templates auf die Lethal-Distanz. Hält eigene Baselines pro Ressource/Sitzung, liest Änderungen zurück und restauriert nur bei unverändertem Besitz. Stoppt bei unsicherem Ergebnis oder externer Änderung. Bietet UI-Toggle. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Setzt FallDamage-Startdistanz in den tatsächlich vom Spieler verwendeten geladenen Templates auf die Lethal-Distanz. Hält eigene Baselines pro Ressource/Sitzung, liest Änderungen zurück und restauriert nur bei unverändertem Besitz. Stoppt bei unsicherem Ergebnis oder externer Änderung. Bietet UI-Toggle. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-no-fall-damage/src/mod.lua:26 fall_config`<br>`reference-mods-0.9/sf-no-fall-damage/src/mod.lua:56 write`<br>`reference-mods-0.9/sf-no-fall-damage/src/mod.lua:67 restore`<br>`reference-mods-0.9/sf-no-fall-damage/src/mod.lua:100 update`<br>`reference-mods-0.9/sf-no-fall-damage/src/mod.lua:154 toggleFallDamagePatch`<br>`reference-mods-0.9/sf-no-fall-damage/src/mod.lua:164 Lifecycle` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.ecs · runtime.world · game.assets.write_loaded_field/get_loaded_resource · shroudforge.ui |
| 6. Eingaben und Ausgaben | Eingaben: Spielertemplates · Toggle-Aktion · Welt-Sitzung Ausgabe: Effekt-/Fehlerstatus |
| 7. Seiteneffekte und Ressourcen | geladene Template-Skalare schreiben/restaurieren |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.ecs · runtime.world · game.assets.write_loaded_field/get_loaded_resource · shroudforge.ui. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Runtime-Referenzmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | runtime_functions.rs:798 referenziert echte Moddatei im Test. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Fallschadensregel mit rücksetzbarer Ressourcenänderung. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-no-fall-damage/src/mod.lua:26 fall_config · reference-mods-0.9/sf-no-fall-damage/src/mod.lua:56 write · reference-mods-0.9/sf-no-fall-damage/src/mod.lua:67 restore · reference-mods-0.9/sf-no-fall-damage/src/mod.lua:100 update · reference-mods-0.9/sf-no-fall-damage/src/mod.lua:154 toggleFallDamagePatch · reference-mods-0.9/sf-no-fall-damage/src/mod.lua:164 Lifecycle

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-no-fall-damage/src/mod.lua`.

### [reference-mods-0.9/sf-no-resource-cost/src/mod.lua](../../reference-mods-0.9/sf-no-resource-cost/src/mod.lua)

Vollständig gelesen, 33 Zeilen. SHA-256: `825c524dc2d276d23c6d4c5df00a5f9aaecbb9baffb6f04641a011aec042ef79`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bindet/aktiviert zero_resource_argument für Rezeptkosten über den buildgeprüften nativen Modifikator und restauriert ihn beim Entladen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bindet/aktiviert zero_resource_argument für Rezeptkosten über den buildgeprüften nativen Modifikator und restauriert ihn beim Entladen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-no-resource-cost/src/mod.lua:5 set_enabled`<br>`reference-mods-0.9/sf-no-resource-cost/src/mod.lua:19 on_load`<br>`reference-mods-0.9/sf-no-resource-cost/src/mod.lua:30 on_unload` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | runtime.functions.bind_modifier · runtime.gameplay.patch · shroudforge.log |
| 6. Eingaben und Ausgaben | Eingaben: Buildprofil-Modifikator Ausgabe: Effektstatus |
| 7. Seiteneffekte und Ressourcen | Native Funktionsmodifikation aktivieren/restaurieren |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: runtime.functions.bind_modifier · runtime.gameplay.patch · shroudforge.log. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Bind-/Enable-/Unload-Gerüst weitgehend gleich in sf-infinite-item-split, sf-infinite-item-use und sf-no-resource-cost, unterschiedliche native Modifier und Fehlermeldungen. |
| 12. Nutzungsstatus | Runtime-Referenzmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | runtime_functions.rs:644 bundled_mods_bind_profile_modifiers_and_restore_on_unload. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Rezeptkostenregel über native Modifikation. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-no-resource-cost/src/mod.lua:5 set_enabled · reference-mods-0.9/sf-no-resource-cost/src/mod.lua:19 on_load · reference-mods-0.9/sf-no-resource-cost/src/mod.lua:30 on_unload

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-no-resource-cost/src/mod.lua`.

### [reference-mods-0.9/sf-production-time/src/mod.lua](../../reference-mods-0.9/sf-production-time/src/mod.lua)

Vollständig gelesen, 31 Zeilen. SHA-256: `94503f8661f249ec91be0f8bce1222b8f84b4e4b98402584f39d53502ac6d2a8`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Validiert eine absolute Produktionsdauer in Sekunden und alle RecipeRegistry-Werte, berechnet Nanosekunden und schreibt erst nach erfolgreicher Gesamtprüfung alle betroffenen Ressourcen. Bereits gewünschte Dauer bleibt unverändert. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Validiert eine absolute Produktionsdauer in Sekunden und alle RecipeRegistry-Werte, berechnet Nanosekunden und schreibt erst nach erfolgreicher Gesamtprüfung alle betroffenen Ressourcen. Bereits gewünschte Dauer bleibt unverändert. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-production-time/src/mod.lua:2 Capability`<br>`reference-mods-0.9/sf-production-time/src/mod.lua:3 seconds`<br>`reference-mods-0.9/sf-production-time/src/mod.lua:15 Validierungsschleife`<br>`reference-mods-0.9/sf-production-time/src/mod.lua:30 Commit` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | loader.features.patch · shroudforge.settings · game.types/assets |
| 6. Eingaben und Ausgaben | Eingaben: seconds · RecipeRegistryResource-Rezepte Ausgabe: absolute geänderte craftingDuration-Werte · Logzähler |
| 7. Seiteneffekte und Ressourcen | Assetänderungen für Pregame-Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: loader.features.patch · shroudforge.settings · game.types/assets. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Patch-Referenzmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | src/loader/api/tests/production_time.rs:33 und :173 referenzieren Modquelle/Package. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Produktionszeiten im vorbereiteten Datenbestand. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-production-time/src/mod.lua:2 Capability · reference-mods-0.9/sf-production-time/src/mod.lua:3 seconds · reference-mods-0.9/sf-production-time/src/mod.lua:15 Validierungsschleife · reference-mods-0.9/sf-production-time/src/mod.lua:30 Commit

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-production-time/src/mod.lua`.

### [reference-mods-0.9/sf-unlock-blueprints/src/mod.lua](../../reference-mods-0.9/sf-unlock-blueprints/src/mod.lua)

Vollständig gelesen, 77 Zeilen. SHA-256: `99788a2c4504584c30c1f5b54b823a7e3c4f7434876f03c3173f79518bdd39d2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Löst den Flame-Hinweis-Knowledge-ID aus einer benannten Engine-Query/Action-Beziehung auf und setzt Rezept-Knowledgebedingungen auf diesen SimpleBool-Vergleich. Prüft Mehrdeutigkeit und fehlende Enginebeziehung. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Löst den Flame-Hinweis-Knowledge-ID aus einer benannten Engine-Query/Action-Beziehung auf und setzt Rezept-Knowledgebedingungen auf diesen SimpleBool-Vergleich. Prüft Mehrdeutigkeit und fehlende Enginebeziehung. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:13 query_type`<br>`reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:16 Enginebeziehung`<br>`reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:39 Rezepte`<br>`reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:64 Ressourcencommit` |
| 4. Aufrufer und Verbraucher | Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 5. Verwendete Komponenten | loader.features.patch · game.types/assets |
| 6. Eingaben und Ausgaben | Eingaben: GameKnowledgeQueryResourceDb · RecipeRegistryResource Ausgabe: angepasste Rezeptanforderungen · Logzähler |
| 7. Seiteneffekte und Ressourcen | Assetänderungen für Pregame-Patchpass |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: loader.features.patch · game.types/assets. Aufruferbezug: Zugehöriges mod.json deklariert Capability. Loader runner/state.rs:154 liest src/mod.lua, api/src/lib.rs:636-638 bindet Runtime-Callbacks, sofern Runtime-Mod. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Patch-Referenzmod laut Manifest. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | runtime_functions.rs:961 referenziert echte Moddatei. workflow/pregame.rs:567 referenziert einen anderen Pfad ../../mods/sf-unlock-blueprints, dessen Existenz/Absicht ist nicht aus diesem Scope geklärt. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Rezeptfreischaltung im vorbereiteten Datenbestand. Neue Einordnung: Modpakete als Produkte. Besitzer: Gameplay-Mods. |

**Belege:** reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:13 query_type · reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:16 Enginebeziehung · reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:39 Rezepte · reference-mods-0.9/sf-unlock-blueprints/src/mod.lua:64 Ressourcencommit

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/sf-unlock-blueprints/src/mod.lua`.

### [reference-mods-0.9/world-editor/src/mod.lua](../../reference-mods-0.9/world-editor/src/mod.lua)

Vollständig gelesen, 3321 Zeilen. SHA-256: `75dea5eb4c84b46fb03d2e2f1421d4fbe94976247c1fc46f264eda6ef229b440`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Implementiert die eigentliche Weltbearbeitung auf Client und headless Server. Erfasst und rotiert Voxel/Props, validiert/exportiert/importiert SFBP-V7-Blueprints, verwaltet Bibliotheksdateien, baut Vorschauen, platziert/ersetzt Props und Terrain und führt besitz-/sitzungsgeprüftes Undo/Teilrollback aus. Steuert F3-F8 und UI-Aktionen, veröffentlicht editor-state.txt und wählt anhand nativer Weltkontextart lokalen Writer oder serverseitige P2P-Ausführung. Verwaltet Servertransaktionen, Undo-Token und bestätigte Abandon-Freigabe. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Implementiert die eigentliche Weltbearbeitung auf Client und headless Server. Erfasst und rotiert Voxel/Props, validiert/exportiert/importiert SFBP-V7-Blueprints, verwaltet Bibliotheksdateien, baut Vorschauen, platziert/ersetzt Props und Terrain und führt besitz-/sitzungsgeprüftes Undo/Teilrollback aus. Steuert F3-F8 und UI-Aktionen, veröffentlicht editor-state.txt und wählt anhand nativer Weltkontextart lokalen Writer oder serverseitige P2P-Ausführung. Verwaltet Servertransaktionen, Undo-Token und bestätigte Abandon-Freigabe. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `mod.lua:73 execution_backend`<br>`mod.lua:107 process_update`<br>`mod.lua:214 publish_editor_state`<br>`mod.lua:297 rotate_vector_quarters`<br>`mod.lua:343 rotated_blueprint`<br>`mod.lua:536 resolve_placeable_items`<br>`mod.lua:746 capture_region_props`<br>`mod.lua:831 update_pending_queries`<br>`mod.lua:908 update_world_session`<br>`mod.lua:1007 finish_capture`<br>`mod.lua:1089 copy_voxels`<br>`mod.lua:1207 encode_blueprint`<br>`mod.lua:1301 load_blueprint_content`<br>`mod.lua:1488 refresh_blueprint_library`<br>`mod.lua:1533/1599/1659 Bibliotheksdateioperationen`<br>`mod.lua:1706 paste_voxels`<br>`mod.lua:2021 undo_voxels`<br>`mod.lua:2254 rollback_partial_paste`<br>`mod.lua:2655/2674 Tokenpersistierung`<br>`mod.lua:2793 Serverpaste`<br>`mod.lua:2858 server_begin_remote_abandon`<br>`mod.lua:2932 create_p2p_bridge`<br>`mod.lua:3038/3091 Serverjobs`<br>`mod.lua:3179-3209 UI-Aktionen`<br>`mod.lua:3211 Lifecycle` |
| 4. Aufrufer und Verbraucher | mod.json deklariert runtime und export. · Loader runner/state.rs:154 lädt src/mod.lua und Runtime bindet Lifecycle. · world-editor-ui/src/main.rs:1412 schreibt Aktionen für modId world-editor, die hier :3179-3209 registriert sind. · src/loader/api/tests/world_editor_p2p.rs lädt echte Quelle. |
| 5. Verwendete Komponenten | runtime.world.voxel/entity/cursor/context_kind/session_id · runtime.network · runtime.features/status · game.assets geladene ItemInfos mit KFC-Lesefallback · shroudforge.settings/input/ui/log · io.export/read_export/export_list/copy/rename/delete · p2p.lua |
| 6. Eingaben und Ausgaben | Eingaben: Modsettings · Cursor-/Tastatur-/UI-Aktionen · Live-Voxel und Props/ItemInfo-Rezepte · SFBP-V7-Dateien · autorisierte P2P-Blueprint-/Undo-/Abandon-Anfragen Ausgabe: veränderte Live-Welt · SFBP-V7-Dateien und Bibliotheksaktionen · Editor-/Prozesszustand als Textdatei · P2P-Ergebnisse und Undo-Token |
| 7. Seiteneffekte und Ressourcen | Live-Entity-Spawn/Destroy/Scale und Voxelwrites · Exportdateien schreiben/kopieren/umbenennen/löschen · Steam-Modnachrichten · Logs |
| 8. Art der Verantwortung | Geschäftslogik für Weltoperationen und Transaktionen, Blueprintformat-/Dateiintegration, UI- und Mehrspielersteuerung |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Nur innerhalb des bestehenden Referenzpakets nach tatsächlichen Teilaufgaben gliedern. Keine Aufnahme in mods/ und keine Änderung des Releaseumfangs. Befunde: Fachlich trennbare Verantwortung in 3.321 Zeilen: Blueprintcodec/Rotation/Bibliothek, lokale Weltoperation mit Undojournal, UI-/Cursor-/Prozesszustände sowie P2P-Servertransaktionen. Eine Aufteilung muss gemeinsames Journal und Session-/Besitzinvarianten erhalten. · Im replace-Propzweig wird :1864 #props ausgewertet, bevor :1865 local props = captured_props deklariert. Die vorherige Zeile bindet daher globales props, während im vollständigen Modul kein solcher Globalwert gesetzt wird. Der Branch kann bereits vor Ersatz-/Weltwrites an nil-Länge abbrechen. Nicht ausgeführt. · Bestätigte Live-Writes werden ausdrücklich von Save-Persistenz getrennt, etwa :2011 und :2251. Audit behauptet keine getestete Speicherpersistenz. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Weltbearbeitung und Blueprintdaten. Technische Kopplung: runtime.world.voxel/entity/cursor/context_kind/session_id · runtime.network · runtime.features/status · game.assets geladene ItemInfos mit KFC-Lesefallback · shroudforge.settings/input/ui/log · io.export/read_export/export_list/copy/rename/delete · p2p.lua. Aufruferbezug: mod.json deklariert runtime und export. · Loader runner/state.rs:154 lädt src/mod.lua und Runtime bindet Lifecycle. · world-editor-ui/src/main.rs:1412 schreibt Aktionen für modId world-editor, die hier :3179-3209 registriert sind. · src/loader/api/tests/world_editor_p2p.rs lädt echte Quelle. |
| 11. Duplikation | Live-Blueprintbibliothek wird außerdem im Rust-UI-Host aus denselben Exportdateien gelesen. Das ist eine reale Dateivertragskopplung, keine automatische Löschentscheidung. |
| 12. Nutzungsstatus | Ladepfad durch Manifest und echte Tests nachgewiesen. Reale Installation/Engineausführung im Audit nicht geprüft. |
| 13. Risiko bei Änderung oder Verschiebung | Sehr hoch bei funktionaler Zerlegung: Undo-/Teilrollback und Serverlock, kein Überschreiben fremder Änderungen, Sessiongenerationen, SFBP-V7-Dateiformat, editor-state.txt, UI-Aktionsnamen, p2p require-Auflösung und persistierte Server-Undo-Token müssen konsistent bleiben. |
| 14. Vorhandener Testschutz | src/loader/api/tests/world_editor_p2p.rs:7-17 lädt p2p.lua/mod.lua und Lua-Fixture. · Weitere echte Quellenreferenz im gleichen Test:64. · Tests nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Weltbearbeitung, Blueprintdaten und transaktionale Wiederherstellung. Neue Einordnung: Weltbearbeitung und Blueprintdaten. Besitzer: Welteditor-Mod. |

**Belege:** mod.lua:73 execution_backend · mod.lua:107 process_update · mod.lua:214 publish_editor_state · mod.lua:297 rotate_vector_quarters · mod.lua:343 rotated_blueprint · mod.lua:536 resolve_placeable_items · mod.lua:746 capture_region_props · mod.lua:831 update_pending_queries · mod.lua:908 update_world_session · mod.lua:1007 finish_capture · mod.lua:1089 copy_voxels · mod.lua:1207 encode_blueprint · mod.lua:1301 load_blueprint_content · mod.lua:1488 refresh_blueprint_library · mod.lua:1533/1599/1659 Bibliotheksdateioperationen · mod.lua:1706 paste_voxels · mod.lua:2021 undo_voxels · mod.lua:2254 rollback_partial_paste · mod.lua:2655/2674 Tokenpersistierung · mod.lua:2793 Serverpaste · mod.lua:2858 server_begin_remote_abandon · mod.lua:2932 create_p2p_bridge · mod.lua:3038/3091 Serverjobs · mod.lua:3179-3209 UI-Aktionen · mod.lua:3211 Lifecycle

**Evidenzgrenze:** Vollständig gelesen, Verhalten und Aufrufer nachgewiesen. Laufzeitwirkung/Save-Persistenz nicht ausgeführt.

**Maßnahme:** aufteilen. Nur innerhalb des bestehenden Referenzpakets nach tatsächlichen Teilaufgaben gliedern. Keine Aufnahme in mods/ und keine Änderung des Releaseumfangs.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/world-editor/src/mod.lua`, `reference-mods-0.9/world-editor/src/blueprint.lua`, `reference-mods-0.9/world-editor/src/library.lua`, `reference-mods-0.9/world-editor/src/preview.lua`, `reference-mods-0.9/world-editor/src/operations.lua`, `reference-mods-0.9/world-editor/src/history.lua`.

### [reference-mods-0.9/world-editor/src/p2p.lua](../../reference-mods-0.9/world-editor/src/p2p.lua)

Vollständig gelesen, 454 Zeilen. SHA-256: `f5a87e102a7bda9c78e33df709e6a9ff0dc7d05e279c3f6dff598bbb4f8388f2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Kapselt World-Editor-Nachrichten für verlässlichen Steam-Modtransport. Zerlegt SFBP-Text in 384-KiB-Blöcke bis32MiB, validiert Metadaten/Adler32/Sequenzen und autorisierte Peers, serialisiert Serverarbeit und spätere Abandon-Aufträge, cached128 Ergebnisse und behandelt Client-Stall/Timeout sowie idempotente Freigabewiederholung. Führt Weltoperationen nur über übergebene Callbacks aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Kapselt World-Editor-Nachrichten für verlässlichen Steam-Modtransport. Zerlegt SFBP-Text in 384-KiB-Blöcke bis32MiB, validiert Metadaten/Adler32/Sequenzen und autorisierte Peers, serialisiert Serverarbeit und spätere Abandon-Aufträge, cached128 Ergebnisse und behandelt Client-Stall/Timeout sowie idempotente Freigabewiederholung. Führt Weltoperationen nur über übergebene Callbacks aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `p2p.lua:3 Factory`<br>`p2p.lua:37 adler32`<br>`p2p.lua:94 dispatch_abandon`<br>`p2p.lua:134 finish_upload`<br>`p2p.lua:164 handle_server_message`<br>`p2p.lua:286 start_paste`<br>`p2p.lua:318 start_undo`<br>`p2p.lua:327 start_abandon`<br>`p2p.lua:365 tick` |
| 4. Aufrufer und Verbraucher | mod.lua:2934 require('p2p')(network_api(), callbacks) · mod.lua:2972 start_paste, :3014 start_undo, :2909 start_abandon, :3254 tick · src/loader/api/tests/world_editor_p2p.rs:9 und :27 lädt echte Quelle |
| 5. Verwendete Komponenten | übergebene api.network.send_mod/receive_mod/status · api.world.session_id · callbacks on_paste/on_undo/on_abandon/is_allowed_peer · Lua Standardbibliothek |
| 6. Eingaben und Ausgaben | Eingaben: Blueprinttext und Platzierungsmetadaten · Steam-Nachrichten B/C/U/A/S/P/R · delta_seconds · Callbackresultate Ausgabe: reliable Steam-Modnachrichten · Callbackaufrufe und Fortschritt/Resultate |
| 7. Seiteneffekte und Ressourcen | Netzwerk senden/empfangen · prozessinterne Upload-/Resultat-/Transaktionspuffer · Logs |
| 8. Art der Verantwortung | Protokoll-/Transportintegration und Transaktionskoordinierung |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Fokussiertes fachliches Protokoll, keine direkten Weltwrites. Änderungen an Cache-/Timeout-/Abandonlogik beeinflussen Wiederanlauf und Serverfreigabe. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modpakete als Produkte. Technische Kopplung: übergebene api.network.send_mod/receive_mod/status · api.world.session_id · callbacks on_paste/on_undo/on_abandon/is_allowed_peer · Lua Standardbibliothek. Aufruferbezug: mod.lua:2934 require('p2p')(network_api(), callbacks) · mod.lua:2972 start_paste, :3014 start_undo, :2909 start_abandon, :3254 tick · src/loader/api/tests/world_editor_p2p.rs:9 und :27 lädt echte Quelle |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Aktiv über Welteditor require und Tickaufruf, Tests laden echte Quelle. |
| 13. Risiko bei Änderung oder Verschiebung | require('p2p') relativ zur Packagequelle und alle Paket-/Callbackfelder erhalten. Reihenfolge/Größenlimits, autorisierter Peerbezug, Ergebnisdeduplizierung und Freigabe bei unbekanntem Ausgang beibehalten. |
| 14. Vorhandener Testschutz | src/loader/api/tests/world_editor_p2p.rs:9,:27 und zugehörige world_editor_p2p.lua-Fixture, nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Mehrspielerprotokoll für Blueprinttransaktionen. Neue Einordnung: Modpakete als Produkte. Besitzer: Welteditor-Mod. |

**Belege:** p2p.lua:3 Factory · p2p.lua:37 adler32 · p2p.lua:94 dispatch_abandon · p2p.lua:134 finish_upload · p2p.lua:164 handle_server_message · p2p.lua:286 start_paste · p2p.lua:318 start_undo · p2p.lua:327 start_abandon · p2p.lua:365 tick

**Evidenzgrenze:** Vollständig gelesen, Import und reale Quellenreferenzen in Tests nachgewiesen. Live-Steamausführung nicht geprüft.

**Maßnahme:** beibehalten. Bestehenden Paketort erhalten. mods/ enthält ausschließlich die beiden vorhandenen SF-Mods. EML-, Referenz- und Diagnosebestand wird nicht dorthin verschoben.

**Vorgeschlagene Zielpfade:** `reference-mods-0.9/world-editor/src/p2p.lua`.

### [templates/mod/src/mod.lua](../../templates/mod/src/mod.lua)

Vollständig gelesen, 26 Zeilen. SHA-256: `469489aa16d3239f0c588b99f1641077be2b32497042c37b1cb8a568556aa760`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Demonstriert eine Runtime-Mod mit settings-gesteuerter Begrüßung, Logschweregrad, UI-Aktion logGreeting und optionaler Begrüßung beim Laden. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Demonstriert eine Runtime-Mod mit settings-gesteuerter Begrüßung, Logschweregrad, UI-Aktion logGreeting und optionaler Begrüßung beim Laden. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `templates/mod/src/mod.lua:3 setting`<br>`templates/mod/src/mod.lua:7 log_greeting`<br>`templates/mod/src/mod.lua:18 on_action`<br>`templates/mod/src/mod.lua:20 Lifecycle` |
| 4. Aufrufer und Verbraucher | Vorlagenkopie für Modautoren, kein direkter Produktaufruf nachgewiesen. |
| 5. Verwendete Komponenten | runtime.require · shroudforge.settings/log/ui |
| 6. Eingaben und Ausgaben | Eingaben: greeting, severity, showOnLoad · logGreeting-Aktion Ausgabe: Begrüßungslog |
| 7. Seiteneffekte und Ressourcen | Loaderlog schreiben |
| 8. Art der Verantwortung | Spielmodlogik und öffentliche API-Integration |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit über den beschriebenen Zweck hinaus belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Modentwicklung. Technische Kopplung: runtime.require · shroudforge.settings/log/ui. Aufruferbezug: Vorlagenkopie für Modautoren, kein direkter Produktaufruf nachgewiesen. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Vorlage, tatsächliche Installation nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Packagewurzel, mod.json, optionale Erweiterung/Settings und fester src/mod.lua-Einstieg müssen gemeinsam bleiben. Öffentliche API-/Callback-/Dateiverträge erhalten. |
| 14. Vorhandener Testschutz | Kein Test nachgewiesen. · Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Runtime-Modvorlage. Neue Einordnung: Modentwicklung. Besitzer: Modvorlage. |

**Belege:** templates/mod/src/mod.lua:3 setting · templates/mod/src/mod.lua:7 log_greeting · templates/mod/src/mod.lua:18 on_action · templates/mod/src/mod.lua:20 Lifecycle

**Evidenzgrenze:** Verhalten aus vollständiger Dateilektüre nachgewiesen. Einsatz außerhalb des Repositorys und nicht angegebene Tests ungeklärt.

**Maßnahme:** beibehalten. Eigenständiges Autorenpaket. Nicht mit ausführendem Loadercode vermischen.

**Vorgeschlagene Zielpfade:** `templates/mod/src/mod.lua`.

## Website und Gestaltungsvorschau

20 Dateien, 3.561 Zeilen.

### [docs/sf/modloader-typography-preview.html](../../docs/sf/modloader-typography-preview.html)

Vollständig gelesen, 210 Zeilen. SHA-256: `e379b8321577a84eefcdcb609ef6859ff11b80fb0b5c65807d6bdb8410489cab`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Bietet eine eigenständige deutsche Typografie-/Layoutvorschau mit repräsentativen Modloaderkarten, Nachrichten, Logzeilen und Dialog. Ein Breitenregler ändert das Vorschaufenster und prüft per scrollWidth/clientWidth den horizontalen Überlauf ausgewählter Bereiche. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Bietet eine eigenständige deutsche Typografie-/Layoutvorschau mit repräsentativen Modloaderkarten, Nachrichten, Logzeilen und Dialog. Ein Breitenregler ändert das Vorschaufenster und prüft per scrollWidth/clientWidth den horizontalen Überlauf ausgewählter Bereiche. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `docs/sf/modloader-typography-preview.html:8 Schriftgrößenvariablen`<br>`docs/sf/modloader-typography-preview.html:123 Toolbar`<br>`docs/sf/modloader-typography-preview.html:146 Beispieldarstellung`<br>`docs/sf/modloader-typography-preview.html:193 updatePreview`<br>`docs/sf/modloader-typography-preview.html:197 Überlaufprüfung` |
| 4. Aufrufer und Verbraucher | Manueller Browserabruf. Produkt-/Buildaufruf nicht nachgewiesen. |
| 5. Verwendete Komponenten | Browser DOM/CSS/JavaScript |
| 6. Eingaben und Ausgaben | Eingaben: Breitenregler und Resize Ausgabe: visuelle Vorschau und Überlaufmeldung |
| 7. Seiteneffekte und Ressourcen | DOM-/CSS-Änderung |
| 8. Art der Verantwortung | Design-/Entwicklungswerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Entwicklerwissen. Technische Kopplung: Browser DOM/CSS/JavaScript. Aufruferbezug: Manueller Browserabruf. Produkt-/Buildaufruf nicht nachgewiesen. |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Eigenständiges manuelles Vorschauartefakt, Produktverwendung nicht nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Relative CSS-/Skript-/Linkpfade beziehungsweise eigenständige Vorschaufunktion erhalten. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Interaktive Typografie- und Layoutvorschau. Neue Einordnung: Entwicklerwissen. Besitzer: Dokumentation. |

**Belege:** docs/sf/modloader-typography-preview.html:8 Schriftgrößenvariablen · docs/sf/modloader-typography-preview.html:123 Toolbar · docs/sf/modloader-typography-preview.html:146 Beispieldarstellung · docs/sf/modloader-typography-preview.html:193 updatePreview · docs/sf/modloader-typography-preview.html:197 Überlaufprüfung

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Dokumentation nach Codeänderungen aktualisieren. Alte Entwürfe sind kein Verhaltensbeleg.

**Vorgeschlagene Zielpfade:** `docs/sf/modloader-typography-preview.html`.

### [website/app.js](../../website/app.js)

Vollständig gelesen, 1134 Zeilen. SHA-256: `ff4f2f82286380c73caa75900f2eec2b04c278c50d911adddd6a7ac77b8b2cc4`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Betreibt die gesamte Dokumentationswebsite im Browser: Routing, zweisprachige Navigation, Markdownrenderer, Manifesteditor, eigene Validierung/Schemahilfe, Modloader-Vorschau, Iconupload/JSON-/ZIP-Download sowie API-/Typ-/Ressourcensuche. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Betreibt die gesamte Dokumentationswebsite im Browser: Routing, zweisprachige Navigation, Markdownrenderer, Manifesteditor, eigene Validierung/Schemahilfe, Modloader-Vorschau, Iconupload/JSON-/ZIP-Download sowie API-/Typ-/Ressourcensuche. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `36: function apiGuideFor(item) {`<br>`45: function groupDocsForNavigation(pages) {`<br>`58: function apiDescription(item, guide) {`<br>`83: function rebuildManifestStudio() {`<br>`163: function setPage(page, updateHash = true) {`<br>`176: async function loadMarkdownPage(page) {`<br>`192: function renderMarkdown(markdown, page) {`<br>`252: function renderMarkdownInline(value, page) {`<br>`272: function resolveMarkdownUrl(href, page) {`<br>`397: function changeEditor(mode) {`<br>`410: function updatePreview() {`<br>`426: function validateManifests(manifest, extended) {`<br>`500: function renderSchemaReference() {`<br>`534: function renderExtendedItems() {`<br>`545: function parseCurrent() {`<br>`548: function syncEditorFromExtended() {`<br>`555: function setBuilderTab(mode) {`<br>`558: function readManifestIntoBuilder() {`<br>`571: function renderDependencyList(dependencies) {`<br>`576: function showUploadedIcon(file, dataUrl) {`<br>`615: function crc32(bytes) {`<br>`623: function write16(view, offset, value) { view.setUint16(offset, value, true); }`<br>`624: function write32(view, offset, value) { view.setUint32(offset, value, true); }`<br>`625: async function makeModZip() {`<br>`701: function updateExtended(mutator) {`<br>`711: function syncTargetPicker(extended) {`<br>`746: function renderBadgeBuilder() {`<br>`826: function renderPreview(manifest, extended) {`<br>`859: function renderSetting(key, setting) {`<br>`902: async function copyText(text, button) {`<br>`910: function downloadJson(text, filename) {`<br>`926: function addCopyControls() {`<br>`957: async function loadCatalog() {`<br>`979: async function checkJson(response) { if (!response.ok) throw new Error(\`${response.status} ${response.url}\`); return response.json(); }`<br>`980: function setCatalogStatus(message, stateName = "") {`<br>`989: function showCatalogFailure(message, error, retry) {`<br>`999: function updateCatalogVersion() {`<br>`1006: async function loadGameProfile() {`<br>`1023: async function loadGameData(name) {`<br>`1052: function setCatalog(name) {`<br>`1059: function renderApi(query = "") {`<br>`1087: function renderApiResult(item) {`<br>`1122: function renderTypes(query = "") {`<br>`1128: function renderResources(query = "") {` |
| 4. Aufrufer und Verbraucher | website/de/index.html und website/en/index.html laden app.js als ES-Modul |
| 5. Verwendete Komponenten | DOM, localStorage, Clipboard/FileReader/Blob, content.de/en.js, erzeugter pages/index.js, statische data/api.json und Spielschemasnapshots |
| 6. Eingaben und Ausgaben | Eingaben: Sprachauswahl/URLhash, JSON-Editorwerte, Uploadicon, Suchabfragen und statische Dokumentationsdaten Ausgabe: DOM-Ansichten, Preview und herunterladbare JSON-/ZIP-/Icondateien |
| 7. Seiteneffekte und Ressourcen | ändert DOM/URLhash/localStorage, lädt statische Dateien per fetch, schreibt Clipboard und startet Browserdownloads |
| 8. Art der Verantwortung | Mehrere: Präsentation, Autorenwerkzeug, Validierung und ZIP-Export |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Routing, Editorvalidierung, Referenzsuche und Autoren-ZIP sind verschiedene Funktionen. Autoren-ZIP ist kein installierter Runtime-Mod. Befunde: Viele fachlich geschlossene Aufgaben in einer Datei. Validierungsregeln, Linkbadges, Schemahilfetabellen und Controlrendering werden neben den Loaderquellen handgepflegt. targets wird hier ausdrücklich akzeptiert, ein früherer gegenteiliger Planbefund ist für diesen Stand falsch. makeModZip erzeugt Manifest/Icon-Paket, kein Lua-Entrypoint. Das ist Autorenexport, kein ZIP-Modbetrieb. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Modautoreneditor. Technische Kopplung: DOM, localStorage, Clipboard/FileReader/Blob, content.de/en.js, erzeugter pages/index.js, statische data/api.json und Spielschemasnapshots. Aufruferbezug: website/de/index.html und website/en/index.html laden app.js als ES-Modul |
| 11. Duplikation | validateManifests wiederholt den Manifest-/Settingsvertrag aus package. Linkbadge-IDs und Rendersemantik sind zusätzlich zur realen UI gepflegt. Echte Auswahltypen/Optionsvalidierung unterscheiden sich, keine Gleichheit behauptet. |
| 12. Nutzungsstatus | Als Browser-Einstieg statisch referenziert. Nicht im Browser ausgeführt. |
| 13. Risiko bei Änderung oder Verschiebung | DOM-IDs, zweisprachige Contentmodule, relative fetch-/Quellcodepfade und Editorzustand erhalten. ZIP-Download nur aus dem Autoreneditorzuordnung ableiten. |
| 14. Vorhandener Testschutz | Direkte automatisierte Browser-Verhaltensprüfung nicht nachgewiesen. Websiteprüfung des erzeugten Datenbestands ist davon zu unterscheiden. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Laufzeit, Autoreneditor und Referenzbrowser. Neue Einordnung: Dokumentationsoberfläche und Modautoreneditor. Besitzer: Website. |

**Belege:** rebuildManifestStudio, setPage, renderMarkdown, validateManifests, renderSchemaReference, makeModZip, renderPreview, renderSetting, loadCatalog, renderApi/Types/Resources

**Evidenzgrenze:** Datei vollständig in lückenlosen Zeichenabschnitten0-89487 gelesen. Inhalt nachgewiesen, Browserverhalten statisch abgeleitet.

**Maßnahme:** aufteilen. Routing, Editorvalidierung, Referenzsuche und Autoren-ZIP sind verschiedene Funktionen. Autoren-ZIP ist kein installierter Runtime-Mod.

**Vorgeschlagene Zielpfade:** `website/app.js`, `website/navigation.js`, `website/markdown.js`, `website/mod-editor.js`, `website/manifest-validation.js`, `website/reference-search.js`, `website/package-download.js`.

### [website/content.de.js](../../website/content/de.js)

Vollständig gelesen, 109 Zeilen. SHA-256: `4453ae5544a6f151f3f60c2a6e88c80237ab2bf521d81f8c8729be9a41e2281a`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Setzt den deutschen Website-Inhaltskatalog aus importierten Anleitungen und eigenen HTML-Templates zusammen. Enthält Start-/Spieler-/Moddingseiten, Manifestanleitung, Formular- und JSON-Editoroberflächen für Modmanifeste, Modvorschau sowie API-Suchoberfläche mit lokalisierten Bezeichnungen. Interaktive Abläufe werden anhand dieser DOM-IDs in app.js angeschlossen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Setzt den deutschen Website-Inhaltskatalog aus importierten Anleitungen und eigenen HTML-Templates zusammen. Enthält Start-/Spieler-/Moddingseiten, Manifestanleitung, Formular- und JSON-Editoroberflächen für Modmanifeste, Modvorschau sowie API-Suchoberfläche mit lokalisierten Bezeichnungen. Interaktive Abläufe werden anhand dieser DOM-IDs in app.js angeschlossen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/content.de.js:1-3 importierte Anleitungen`<br>`website/content.de.js:5 Default-Export`<br>`website/content.de.js:13 Startseite`<br>`website/content.de.js:47 Manifestanleitung`<br>`website/content.de.js:57 Manifesteditor-/Vorschautemplate`<br>`website/content.de.js:96 API-Oberfläche`<br>`website/content.de.js:108 Editorbezeichnungen` |
| 4. Aufrufer und Verbraucher | website/app.js:1 importiert de |
| 5. Verwendete Komponenten | website/server-guides.de.js · website/platform-guides.de.js · website/support-guides.de.js · DOM-IDs und HTML-Klassen für Website-App/Styles |
| 6. Eingaben und Ausgaben | Eingaben: Importierte lokalisierte Anleitungen. Statisch verfasste Texte und HTML-Templates. Ausgabe: Default-Export mit Oberflächenbezeichnungen, Navigation, HTML-Seiten und Editor-/API-Texten |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Netzwerk-, Prozess- oder Dateisystemwirkungen. Import erzeugt Inhaltsobjekt. Spätere DOM-Darstellung erfolgt im Aufrufer. |
| 8. Art der Verantwortung | Lokalisierter Dokumentationsinhalt und UI-Template |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Gemeinsame HTML-Struktur von übersetzten Inhalten trennen, ohne ein neues Websiteframework einzuführen. Befunde: Ein Inhaltsmodul verbindet allgemeine Übersetzungen und Anleitungen mit umfangreichen interaktiven Editor-DOM-Templates. Formular-IDs und Vorschau-Markup koppeln die Inhalte an app.js. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lokalisierte Inhalte und Editorvorlagen. Technische Kopplung: website/server-guides.de.js · website/platform-guides.de.js · website/support-guides.de.js · DOM-IDs und HTML-Klassen für Website-App/Styles. Aufruferbezug: website/app.js:1 importiert de |
| 11. Duplikation | website/content.de.js und website/content.en.js enthalten weitgehend gleiche große HTML-Strukturen, Formulare und IDs in zwei Sprachen. Es handelt sich um strukturelle Doppelpflege der Templates. |
| 12. Nutzungsstatus | Import durch zentrale Website-App nachgewiesen. DOM-Verträge sind an den exportierten HTML-IDs sichtbar. |
| 13. Risiko bei Änderung oder Verschiebung | Import-/Exportstruktur, Sprachschlüssel, Navigationsziele, Formular-/Vorschau-IDs und Markup-Verträge zu app.js gemeinsam berücksichtigen. Relative Schema-Links in Beispielen sind Dokumentationsinhalt und begründen keine Konfigurationsmigration. |
| 14. Vorhandener Testschutz | Keine direkten Tests der lokalisierten DOM-Templates nachgewiesen. website/tools/validate.mjs prüft API-/Snapshotdaten und veröffentlichte Schemas, diese Templates nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lokalisierter Website-Inhalt und Editoroberfläche. Neue Einordnung: Lokalisierte Inhalte und Editorvorlagen. Besitzer: Website. |

**Belege:** website/content.de.js:1-3 importierte Anleitungen · website/content.de.js:5 Default-Export · website/content.de.js:13 Startseite · website/content.de.js:47 Manifestanleitung · website/content.de.js:57 Manifesteditor-/Vorschautemplate · website/content.de.js:96 API-Oberfläche · website/content.de.js:108 Editorbezeichnungen

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** aufteilen. Gemeinsame HTML-Struktur von übersetzten Inhalten trennen, ohne ein neues Websiteframework einzuführen.

**Vorgeschlagene Zielpfade:** `website/content/de.js`, `website/editor-template.js`.

### [website/content.en.js](../../website/content/en.js)

Vollständig gelesen, 107 Zeilen. SHA-256: `f3ac876f75aa67a0924bae6be7f7b6fb9d1daf3eb1e15c4af6394f4bc6c4f80d`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Setzt den englischen Website-Inhaltskatalog aus importierten Anleitungen und eigenen HTML-Templates zusammen. Enthält Start-/Spieler-/Moddingseiten, Manifestanleitung, Formular- und JSON-Editoroberflächen für Modmanifeste, Modvorschau sowie API-Suchoberfläche mit lokalisierten Bezeichnungen. Interaktive Abläufe werden anhand dieser DOM-IDs in app.js angeschlossen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Setzt den englischen Website-Inhaltskatalog aus importierten Anleitungen und eigenen HTML-Templates zusammen. Enthält Start-/Spieler-/Moddingseiten, Manifestanleitung, Formular- und JSON-Editoroberflächen für Modmanifeste, Modvorschau sowie API-Suchoberfläche mit lokalisierten Bezeichnungen. Interaktive Abläufe werden anhand dieser DOM-IDs in app.js angeschlossen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/content.en.js:1-3 importierte Anleitungen`<br>`website/content.en.js:5 Default-Export`<br>`website/content.en.js:13 Startseite`<br>`website/content.en.js:45 Manifestanleitung`<br>`website/content.en.js:55 Manifesteditor-/Vorschautemplate`<br>`website/content.en.js:94 API-Oberfläche`<br>`website/content.en.js:106 Editorbezeichnungen` |
| 4. Aufrufer und Verbraucher | website/app.js:2 importiert en |
| 5. Verwendete Komponenten | website/server-guides.en.js · website/platform-guides.en.js · website/support-guides.en.js · DOM-IDs und HTML-Klassen für Website-App/Styles |
| 6. Eingaben und Ausgaben | Eingaben: Importierte lokalisierte Anleitungen. Statisch verfasste Texte und HTML-Templates. Ausgabe: Default-Export mit Oberflächenbezeichnungen, Navigation, HTML-Seiten und Editor-/API-Texten |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Netzwerk-, Prozess- oder Dateisystemwirkungen. Import erzeugt Inhaltsobjekt. Spätere DOM-Darstellung erfolgt im Aufrufer. |
| 8. Art der Verantwortung | Lokalisierter Dokumentationsinhalt und UI-Template |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Beide Sprachdateien teilen die Editorstruktur, Texte bleiben getrennt. Befunde: Ein Inhaltsmodul verbindet allgemeine Übersetzungen und Anleitungen mit umfangreichen interaktiven Editor-DOM-Templates. Formular-IDs und Vorschau-Markup koppeln die Inhalte an app.js. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Lokalisierte Inhalte und Editorvorlagen. Technische Kopplung: website/server-guides.en.js · website/platform-guides.en.js · website/support-guides.en.js · DOM-IDs und HTML-Klassen für Website-App/Styles. Aufruferbezug: website/app.js:2 importiert en |
| 11. Duplikation | website/content.en.js und website/content.de.js enthalten weitgehend gleiche große HTML-Strukturen, Formulare und IDs in zwei Sprachen. Es handelt sich um strukturelle Doppelpflege der Templates. |
| 12. Nutzungsstatus | Import durch zentrale Website-App nachgewiesen. DOM-Verträge sind an den exportierten HTML-IDs sichtbar. |
| 13. Risiko bei Änderung oder Verschiebung | Import-/Exportstruktur, Sprachschlüssel, Navigationsziele, Formular-/Vorschau-IDs und Markup-Verträge zu app.js gemeinsam berücksichtigen. Relative Schema-Links in Beispielen sind Dokumentationsinhalt und begründen keine Konfigurationsmigration. |
| 14. Vorhandener Testschutz | Keine direkten Tests der lokalisierten DOM-Templates nachgewiesen. website/tools/validate.mjs prüft API-/Snapshotdaten und veröffentlichte Schemas, diese Templates nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Lokalisierter Website-Inhalt und Editoroberfläche. Neue Einordnung: Lokalisierte Inhalte und Editorvorlagen. Besitzer: Website. |

**Belege:** website/content.en.js:1-3 importierte Anleitungen · website/content.en.js:5 Default-Export · website/content.en.js:13 Startseite · website/content.en.js:45 Manifestanleitung · website/content.en.js:55 Manifesteditor-/Vorschautemplate · website/content.en.js:94 API-Oberfläche · website/content.en.js:106 Editorbezeichnungen

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** aufteilen. Beide Sprachdateien teilen die Editorstruktur, Texte bleiben getrennt.

**Vorgeschlagene Zielpfade:** `website/content/en.js`, `website/editor-template.js`.

### [website/de/index.html](../../website/de/index.html)

Vollständig gelesen, 16 Zeilen. SHA-256: `c695d2a2300becd14c9dce3823f40b35379dd323b6f0be2354bb793218049758`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt den deutschen Website-Einstieg mit SEO-Metadaten, Icon, gemeinsamem Stylesheet, Ladeskelett und app.js bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt den deutschen Website-Einstieg mit SEO-Metadaten, Icon, gemeinsamem Stylesheet, Ladeskelett und app.js bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/de/index.html:2 Locale`<br>`website/de/index.html:7 Meta`<br>`website/de/index.html:13 app`<br>`website/de/index.html:14 Moduleinstieg` |
| 4. Aufrufer und Verbraucher | website/index.html:10 oder direkter Browserabruf |
| 5. Verwendete Komponenten | ../app.js · ../styles.css · Marken-SVG |
| 6. Eingaben und Ausgaben | Eingaben: Browserabruf Ausgabe: deutsches Startdokument |
| 7. Seiteneffekte und Ressourcen | app.js-Start |
| 8. Art der Verantwortung | Präsentationseinstieg |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ../app.js · ../styles.css · Marken-SVG. Aufruferbezug: website/index.html:10 oder direkter Browserabruf |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Relative CSS-/Skript-/Linkpfade beziehungsweise eigenständige Vorschaufunktion erhalten. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Deutscher Website-Einstieg. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/de/index.html:2 Locale · website/de/index.html:7 Meta · website/de/index.html:13 app · website/de/index.html:14 Moduleinstieg

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/de/index.html`.

### [website/en/index.html](../../website/en/index.html)

Vollständig gelesen, 16 Zeilen. SHA-256: `3d51926146aa11d1024a88213ddaac3eb16cbeb1eb0639fcb6363d52e9dc77a0`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Stellt den englischen Website-Einstieg mit SEO-Metadaten, Icon, gemeinsamem Stylesheet, Ladeskelett und app.js bereit. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Stellt den englischen Website-Einstieg mit SEO-Metadaten, Icon, gemeinsamem Stylesheet, Ladeskelett und app.js bereit. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/en/index.html:2 Locale`<br>`website/en/index.html:7 Meta`<br>`website/en/index.html:13 app`<br>`website/en/index.html:14 Moduleinstieg` |
| 4. Aufrufer und Verbraucher | website/index.html:10 oder direkter Browserabruf |
| 5. Verwendete Komponenten | ../app.js · ../styles.css · Marken-SVG |
| 6. Eingaben und Ausgaben | Eingaben: Browserabruf Ausgabe: englisches Startdokument |
| 7. Seiteneffekte und Ressourcen | app.js-Start |
| 8. Art der Verantwortung | Präsentationseinstieg |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ../app.js · ../styles.css · Marken-SVG. Aufruferbezug: website/index.html:10 oder direkter Browserabruf |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Relative CSS-/Skript-/Linkpfade beziehungsweise eigenständige Vorschaufunktion erhalten. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Englischer Website-Einstieg. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/en/index.html:2 Locale · website/en/index.html:7 Meta · website/en/index.html:13 app · website/en/index.html:14 Moduleinstieg

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/en/index.html`.

### [website/index.html](../../website/index.html)

Vollständig gelesen, 15 Zeilen. SHA-256: `b0f15d9ce286894c7b9d6610d36ee133ab5168605df2e82694a07c3732fb65f3`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Wählt gespeicherte oder browserbasierte Sprache und leitet auf de/ oder en/ weiter, mit englischem Noscript-Fallback und manuellen Sprachlinks. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Wählt gespeicherte oder browserbasierte Sprache und leitet auf de/ oder en/ weiter, mit englischem Noscript-Fallback und manuellen Sprachlinks. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/index.html:9 preferred`<br>`website/index.html:10 location.replace`<br>`website/index.html:12 Noscript` |
| 4. Aufrufer und Verbraucher | Statischer Webserver/GitHub-Pages-Root |
| 5. Verwendete Komponenten | localStorage sf-language · navigator.language · location |
| 6. Eingaben und Ausgaben | Eingaben: Sprachpräferenz Ausgabe: Weiterleitung auf Sprachstartseite |
| 7. Seiteneffekte und Ressourcen | Browsernavigation |
| 8. Art der Verantwortung | Präsentationseinstieg |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: localStorage sf-language · navigator.language · location. Aufruferbezug: Statischer Webserver/GitHub-Pages-Root |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Relative CSS-/Skript-/Linkpfade beziehungsweise eigenständige Vorschaufunktion erhalten. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Sprachweiterleitung. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/index.html:9 preferred · website/index.html:10 location.replace · website/index.html:12 Noscript

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/index.html`.

### [website/platform-guides.de.js](../../website/platform-guides.de.js)

Vollständig gelesen, 15 Zeilen. SHA-256: `b0fd92a0f638dfbc7928f9461464399da0756506ffc0e909bc03fcb7a8636f38`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liefert deutsche HTML-Texte und Diagramme für die Seiten workings und runtime. Erklärt die Rollen von Modloader, EML-Patches, Lua-Mods und nativer KFC-Runtime, Startphasen, Fähigkeiten und geprüften Buildprofilen. Die Datei enthält Dokumentation, keine Implementierung dieser Funktionen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liefert deutsche HTML-Texte und Diagramme für die Seiten workings und runtime. Erklärt die Rollen von Modloader, EML-Patches, Lua-Mods und nativer KFC-Runtime, Startphasen, Fähigkeiten und geprüften Buildprofilen. Die Datei enthält Dokumentation, keine Implementierung dieser Funktionen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/platform-guides.de.js:1 Default-Export`<br>`website/platform-guides.de.js:2 workings`<br>`website/platform-guides.de.js:8 runtime` |
| 4. Aufrufer und Verbraucher | website/content.de.js:2 importiert platformGuides · website/content.de.js:25 und 26 übernimmt workings/runtime |
| 5. Verwendete Komponenten | ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und data-view-Navigation der Website |
| 6. Eingaben und Ausgaben | Eingaben: Keine Laufzeitparameter. Statisch verfasste HTML-Texte. Ausgabe: Default-Export mit workings- und runtime-HTML |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Nebenwirkungen. Import liefert statische Strings, Browserrenderer setzt sie später als Inhalt ein. |
| 8. Art der Verantwortung | Lokalisierter Dokumentationsinhalt |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Dokumentierte Fähigkeiten sind durch diese Texte allein nicht als Laufzeitverhalten bewiesen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und data-view-Navigation der Website. Aufruferbezug: website/content.de.js:2 importiert platformGuides · website/content.de.js:25 und 26 übernimmt workings/runtime |
| 11. Duplikation | website/platform-guides.de.js und website/platform-guides.en.js besitzen dieselben Inhaltsbereiche und weitgehend dieselbe HTML-Struktur mit übersetzten Texten. |
| 12. Nutzungsstatus | Import und Übernahme in Website-Inhalt nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Sprachpaar, Exportnamen workings/runtime, data-view-Ziele und HTML-Klassen müssen beim Verschieben zusammen erhalten bleiben. |
| 14. Vorhandener Testschutz | Keine direkten Inhaltstests nachgewiesen. website/tools/validate.mjs prüft diese HTML-Texte nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Erklärtexte für Modplattform und Laufzeit. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/platform-guides.de.js:1 Default-Export · website/platform-guides.de.js:2 workings · website/platform-guides.de.js:8 runtime

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/platform-guides.de.js`.

### [website/platform-guides.en.js](../../website/platform-guides.en.js)

Vollständig gelesen, 15 Zeilen. SHA-256: `8702d09199b8e683569c2697daefbe5a92c32651cab84d3876b763632c79822e`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liefert englische HTML-Texte und Diagramme für die Seiten workings und runtime. Erklärt die Rollen von Modloader, EML-Patches, Lua-Mods und nativer KFC-Runtime, Startphasen, Fähigkeiten und geprüften Buildprofilen. Die Datei enthält Dokumentation, keine Implementierung dieser Funktionen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liefert englische HTML-Texte und Diagramme für die Seiten workings und runtime. Erklärt die Rollen von Modloader, EML-Patches, Lua-Mods und nativer KFC-Runtime, Startphasen, Fähigkeiten und geprüften Buildprofilen. Die Datei enthält Dokumentation, keine Implementierung dieser Funktionen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/platform-guides.en.js:1 Default-Export`<br>`website/platform-guides.en.js:2 workings`<br>`website/platform-guides.en.js:8 runtime` |
| 4. Aufrufer und Verbraucher | website/content.en.js:2 importiert platformGuides · website/content.en.js:23 und 24 übernimmt workings/runtime |
| 5. Verwendete Komponenten | ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und data-view-Navigation der Website |
| 6. Eingaben und Ausgaben | Eingaben: Keine Laufzeitparameter. Statisch verfasste HTML-Texte. Ausgabe: Default-Export mit workings- und runtime-HTML |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Nebenwirkungen. Import liefert statische Strings, Browserrenderer setzt sie später als Inhalt ein. |
| 8. Art der Verantwortung | Lokalisierter Dokumentationsinhalt |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Dokumentierte Fähigkeiten sind durch diese Texte allein nicht als Laufzeitverhalten bewiesen. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und data-view-Navigation der Website. Aufruferbezug: website/content.en.js:2 importiert platformGuides · website/content.en.js:23 und 24 übernimmt workings/runtime |
| 11. Duplikation | website/platform-guides.en.js und website/platform-guides.de.js besitzen dieselben Inhaltsbereiche und weitgehend dieselbe HTML-Struktur mit übersetzten Texten. |
| 12. Nutzungsstatus | Import und Übernahme in Website-Inhalt nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Sprachpaar, Exportnamen workings/runtime, data-view-Ziele und HTML-Klassen müssen beim Verschieben zusammen erhalten bleiben. |
| 14. Vorhandener Testschutz | Keine direkten Inhaltstests nachgewiesen. website/tools/validate.mjs prüft diese HTML-Texte nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Erklärtexte für Modplattform und Laufzeit. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/platform-guides.en.js:1 Default-Export · website/platform-guides.en.js:2 workings · website/platform-guides.en.js:8 runtime

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/platform-guides.en.js`.

### [website/server-guides.de.js](../../website/server-guides.de.js)

Vollständig gelesen, 40 Zeilen. SHA-256: `6a6c937bbd336a0e324b12475fca783a105d0cada04933481e7014110af496cb`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liefert deutsche Navigation, API-Bezeichnung und HTML-Anleitungen zu dedizierten Servern, benötigten Release-Dateien, Windows und Docker. Die enthaltenen Shell-/Compose-Beispiele sind Dokumentation und werden durch das Modul nicht ausgeführt. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liefert deutsche Navigation, API-Bezeichnung und HTML-Anleitungen zu dedizierten Servern, benötigten Release-Dateien, Windows und Docker. Die enthaltenen Shell-/Compose-Beispiele sind Dokumentation und werden durch das Modul nicht ausgeführt. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/server-guides.de.js:1 Default-Export`<br>`website/server-guides.de.js:2 navGroups`<br>`website/server-guides.de.js:23 Serveranleitungen`<br>`website/server-guides.de.js:36 Dockeranleitung` |
| 4. Aufrufer und Verbraucher | website/content.de.js:1 importiert serverGuides · website/content.de.js:24 übernimmt serverGuides |
| 5. Verwendete Komponenten | ES-Modulimport durch lokalisierten Inhaltskatalog · Website-Navigationsschlüssel und HTML-Klassen |
| 6. Eingaben und Ausgaben | Eingaben: Keine Laufzeitparameter. Statische Navigationsdaten und Anleitungstexte. Ausgabe: Default-Export mit navGroups, apiLabel und Server-/Windows-/Docker-Seiten |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Nebenwirkungen. Die Datei führt die abgebildeten Befehle nicht aus. |
| 8. Art der Verantwortung | Lokalisierter Dokumentations- und Navigationsinhalt |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Ein Export verbindet globale Website-Navigation mit fachlichen Serveranleitungen. Die Verbindung ist durch navGroups und die Anleitungsfelder im selben Export belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ES-Modulimport durch lokalisierten Inhaltskatalog · Website-Navigationsschlüssel und HTML-Klassen. Aufruferbezug: website/content.de.js:1 importiert serverGuides · website/content.de.js:24 übernimmt serverGuides |
| 11. Duplikation | website/server-guides.de.js und website/server-guides.en.js bilden dieselben Navigations-/Anleitungsstrukturen in zwei Sprachen ab. |
| 12. Nutzungsstatus | Import und Spread in Website-Inhalt nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Navigationsschlüssel, Datei-/Downloadnamen, data-view-Ziele und Beispiele einschließlich notwendiger Befehlszeichen erhalten. Inhaltlicher Umzug darf keine Befehle ausführen. |
| 14. Vorhandener Testschutz | Keine direkten Inhalts- oder Befehlsbeispieltests nachgewiesen. website/tools/validate.mjs prüft diese Texte nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Navigation und Serveranleitungen. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/server-guides.de.js:1 Default-Export · website/server-guides.de.js:2 navGroups · website/server-guides.de.js:23 Serveranleitungen · website/server-guides.de.js:36 Dockeranleitung

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/server-guides.de.js`.

### [website/server-guides.en.js](../../website/server-guides.en.js)

Vollständig gelesen, 40 Zeilen. SHA-256: `ae3cf121c6a692cb78be2370ad2360b04390df4d94dca054525a1e7510723f88`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liefert englische Navigation, API-Bezeichnung und HTML-Anleitungen zu dedizierten Servern, benötigten Release-Dateien, Windows und Docker. Die enthaltenen Shell-/Compose-Beispiele sind Dokumentation und werden durch das Modul nicht ausgeführt. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liefert englische Navigation, API-Bezeichnung und HTML-Anleitungen zu dedizierten Servern, benötigten Release-Dateien, Windows und Docker. Die enthaltenen Shell-/Compose-Beispiele sind Dokumentation und werden durch das Modul nicht ausgeführt. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/server-guides.en.js:1 Default-Export`<br>`website/server-guides.en.js:2 navGroups`<br>`website/server-guides.en.js:23 Serveranleitungen`<br>`website/server-guides.en.js:36 Dockeranleitung` |
| 4. Aufrufer und Verbraucher | website/content.en.js:1 importiert serverGuides · website/content.en.js:22 übernimmt serverGuides |
| 5. Verwendete Komponenten | ES-Modulimport durch lokalisierten Inhaltskatalog · Website-Navigationsschlüssel und HTML-Klassen |
| 6. Eingaben und Ausgaben | Eingaben: Keine Laufzeitparameter. Statische Navigationsdaten und Anleitungstexte. Ausgabe: Default-Export mit navGroups, apiLabel und Server-/Windows-/Docker-Seiten |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Nebenwirkungen. Die Datei führt die abgebildeten Befehle nicht aus. |
| 8. Art der Verantwortung | Lokalisierter Dokumentations- und Navigationsinhalt |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Ein Export verbindet globale Website-Navigation mit fachlichen Serveranleitungen. Die Verbindung ist durch navGroups und die Anleitungsfelder im selben Export belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ES-Modulimport durch lokalisierten Inhaltskatalog · Website-Navigationsschlüssel und HTML-Klassen. Aufruferbezug: website/content.en.js:1 importiert serverGuides · website/content.en.js:22 übernimmt serverGuides |
| 11. Duplikation | website/server-guides.en.js und website/server-guides.de.js bilden dieselben Navigations-/Anleitungsstrukturen in zwei Sprachen ab. |
| 12. Nutzungsstatus | Import und Spread in Website-Inhalt nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Navigationsschlüssel, Datei-/Downloadnamen, data-view-Ziele und Beispiele einschließlich notwendiger Befehlszeichen erhalten. Inhaltlicher Umzug darf keine Befehle ausführen. |
| 14. Vorhandener Testschutz | Keine direkten Inhalts- oder Befehlsbeispieltests nachgewiesen. website/tools/validate.mjs prüft diese Texte nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Navigation und Serveranleitungen. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/server-guides.en.js:1 Default-Export · website/server-guides.en.js:2 navGroups · website/server-guides.en.js:23 Serveranleitungen · website/server-guides.en.js:36 Dockeranleitung

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/server-guides.en.js`.

### [website/styles.css](../../website/styles.css)

Vollständig gelesen, 240 Zeilen. SHA-256: `6521c43a5ca2695fd71986e43016f090056b04b95117af158644c5c212aa655c`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Gestaltet die gesamte Dokumentationswebsite mit Designvariablen, Navigation, responsive Layouts, Anleitungsdiagrammen, API-Suche/-Ergebnissen, Markdownartikeln und dem interaktiven Manifesteditor samt Modvorschau. Spätere Regelblöcke überlagern frühere Layoutdefinitionen, unter anderem für API-Ergebnisse und die mitscrollende Seitennavigation. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Gestaltet die gesamte Dokumentationswebsite mit Designvariablen, Navigation, responsive Layouts, Anleitungsdiagrammen, API-Suche/-Ergebnissen, Markdownartikeln und dem interaktiven Manifesteditor samt Modvorschau. Spätere Regelblöcke überlagern frühere Layoutdefinitionen, unter anderem für API-Ergebnisse und die mitscrollende Seitennavigation. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/styles.css:1 Designvariablen und Grundelemente`<br>`website/styles.css:8 Manifeststudio und Vorschau`<br>`website/styles.css:55 gestufte Editoranordnung`<br>`website/styles.css:116 Schematabelle`<br>`website/styles.css:138 Sprachumschalter`<br>`website/styles.css:150 Markdownartikel`<br>`website/styles.css:198 spätere API-Ergebnisdefinition`<br>`website/styles.css:225-240 abschließendes Navigationslayout` |
| 4. Aufrufer und Verbraucher | website/de/index.html:10 Stylesheet-Link · website/en/index.html:10 Stylesheet-Link |
| 5. Verwendete Komponenten | HTML-Klassen/IDs aus website/content.*.js und den Anleitungsmodulen · DOM-Struktur aus website/app.js · Browser-CSS einschließlich :has und Media Queries |
| 6. Eingaben und Ausgaben | Eingaben: DOM-Struktur und Klassen/Attribute · Viewportbreite und Benutzerpräferenzen wie reduzierte Bewegung Ausgabe: Berechnete Darstellung und responsive Anordnung der Website |
| 7. Seiteneffekte und Ressourcen | Beeinflusst das Browserlayout. Keine Datei-, Netzwerk- oder Prozessoperationen im Stylesheet. |
| 8. Art der Verantwortung | Darstellung |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Die Datei bündelt allgemeines Seitendesign und spezialisierte Manifesteditor-/API-/Markdown-Ansichten. Mehrere nachträgliche Regelblöcke verändern frühere Selektoren, wodurch die Reihenfolge für das Verhalten wesentlich ist. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: HTML-Klassen/IDs aus website/content.*.js und den Anleitungsmodulen · DOM-Struktur aus website/app.js · Browser-CSS einschließlich :has und Media Queries. Aufruferbezug: website/de/index.html:10 Stylesheet-Link · website/en/index.html:10 Stylesheet-Link |
| 11. Duplikation | .api-result summary wird früh für Ergebniszeilen und erneut in Zeile131 sowie198 definiert. .sidebar und .main-content besitzen frühe Grundlayouts sowie abschließende Neudefinitionen ab226/239. Dies ist belegte überlappende Regelpflege, keine nachgewiesene unabhängige Implementierung. |
| 12. Nutzungsstatus | Einbindung durch beide Sprach-Einstiegsseiten nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Reihenfolge der CSS-Kaskade, Variablennamen, responsive Breakpoints und Selektorverträge erhalten. Eine Aufteilung ohne Berücksichtigung der späten Overrides kann die Darstellung verändern. |
| 14. Vorhandener Testschutz | Keine direkten CSS-/Screenshottests nachgewiesen. Keine Darstellung im Browser geprüft. website/tools/validate.mjs deckt das Stylesheet nicht ab. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Darstellung und responsive Oberfläche. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/styles.css:1 Designvariablen und Grundelemente · website/styles.css:8 Manifeststudio und Vorschau · website/styles.css:55 gestufte Editoranordnung · website/styles.css:116 Schematabelle · website/styles.css:138 Sprachumschalter · website/styles.css:150 Markdownartikel · website/styles.css:198 spätere API-Ergebnisdefinition · website/styles.css:225-240 abschließendes Navigationslayout

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/styles.css`.

### [website/support-guides.de.js](../../website/support-guides.de.js)

Vollständig gelesen, 8 Zeilen. SHA-256: `7bd9c2c71f31ea411b470c5a6068338833023ac0e5d0ce2ccec84463465be320`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liefert deutschen community-HTML-Inhalt für die Supportseite. Ordnet Mod-, Spiel- und Modloaderprobleme zu, erklärt die Prüfung beider Installationen sowie aussagekräftige Versions-/Logangaben und verlinkt vorhandene GitHub-Vorlagen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liefert deutschen community-HTML-Inhalt für die Supportseite. Ordnet Mod-, Spiel- und Modloaderprobleme zu, erklärt die Prüfung beider Installationen sowie aussagekräftige Versions-/Logangaben und verlinkt vorhandene GitHub-Vorlagen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/support-guides.de.js:1 Default-Export`<br>`website/support-guides.de.js:2 community`<br>`website/support-guides.de.js:5 Berichtshinweise und Vorlagenlinks` |
| 4. Aufrufer und Verbraucher | website/content.de.js:3 importiert supportGuides · website/content.de.js:95 übernimmt community |
| 5. Verwendete Komponenten | ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und externe GitHub-/Nexuslinks |
| 6. Eingaben und Ausgaben | Eingaben: Keine Laufzeitparameter. Statischer Anleitungstext. Ausgabe: Default-Export mit community-HTML |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Nebenwirkungen und keine Nachrichtenversendung. |
| 8. Art der Verantwortung | Lokalisierter Dokumentationsinhalt |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und externe GitHub-/Nexuslinks. Aufruferbezug: website/content.de.js:3 importiert supportGuides · website/content.de.js:95 übernimmt community |
| 11. Duplikation | website/support-guides.de.js und website/support-guides.en.js verwenden dieselbe Supportstruktur und Links mit übersetzten Texten. |
| 12. Nutzungsstatus | Import und Übernahme des community-Felds nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Exportname community, Linkziele und HTML-Klassen mit dem Inhalt erhalten. |
| 14. Vorhandener Testschutz | Keine direkten Inhaltstests nachgewiesen. website/tools/validate.mjs prüft diese Texte nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Supportanleitung. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/support-guides.de.js:1 Default-Export · website/support-guides.de.js:2 community · website/support-guides.de.js:5 Berichtshinweise und Vorlagenlinks

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/support-guides.de.js`.

### [website/support-guides.en.js](../../website/support-guides.en.js)

Vollständig gelesen, 8 Zeilen. SHA-256: `6e8ac97b6d5daba88043604cea7eea02e4628ff94053618505bfa11ca6acddaa`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Liefert englischen community-HTML-Inhalt für die Supportseite. Ordnet Mod-, Spiel- und Modloaderprobleme zu, erklärt die Prüfung beider Installationen sowie aussagekräftige Versions-/Logangaben und verlinkt vorhandene GitHub-Vorlagen. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Liefert englischen community-HTML-Inhalt für die Supportseite. Ordnet Mod-, Spiel- und Modloaderprobleme zu, erklärt die Prüfung beider Installationen sowie aussagekräftige Versions-/Logangaben und verlinkt vorhandene GitHub-Vorlagen. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/support-guides.en.js:1 Default-Export`<br>`website/support-guides.en.js:2 community`<br>`website/support-guides.en.js:5 Berichtshinweise und Vorlagenlinks` |
| 4. Aufrufer und Verbraucher | website/content.en.js:3 importiert supportGuides · website/content.en.js:93 übernimmt community |
| 5. Verwendete Komponenten | ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und externe GitHub-/Nexuslinks |
| 6. Eingaben und Ausgaben | Eingaben: Keine Laufzeitparameter. Statischer Anleitungstext. Ausgabe: Default-Export mit community-HTML |
| 7. Seiteneffekte und Ressourcen | Keine unmittelbaren Nebenwirkungen und keine Nachrichtenversendung. |
| 8. Art der Verantwortung | Lokalisierter Dokumentationsinhalt |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationsoberfläche und Inhalte. Technische Kopplung: ES-Modulimport durch lokalisierten Inhaltskatalog · HTML-Klassen und externe GitHub-/Nexuslinks. Aufruferbezug: website/content.en.js:3 importiert supportGuides · website/content.en.js:93 übernimmt community |
| 11. Duplikation | website/support-guides.en.js und website/support-guides.de.js verwenden dieselbe Supportstruktur und Links mit übersetzten Texten. |
| 12. Nutzungsstatus | Import und Übernahme des community-Felds nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Exportname community, Linkziele und HTML-Klassen mit dem Inhalt erhalten. |
| 14. Vorhandener Testschutz | Keine direkten Inhaltstests nachgewiesen. website/tools/validate.mjs prüft diese Texte nicht. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Supportanleitung. Neue Einordnung: Dokumentationsoberfläche und Inhalte. Besitzer: Website. |

**Belege:** website/support-guides.en.js:1 Default-Export · website/support-guides.en.js:2 community · website/support-guides.en.js:5 Berichtshinweise und Vorlagenlinks

**Evidenzgrenze:** Datei vollständig gelesen. Statische Rolle durch Exporte, Selektoren und angegebene Verwendungsstellen belegt. Keine Laufzeitprüfung ausgeführt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/support-guides.en.js`.

### [website/tools/build-pages.mjs](../../website/tools/build-pages.mjs)

Vollständig gelesen, 40 Zeilen. SHA-256: `7223e7cb0e183a8458c0600f8f7a3bb469633d21d04b175a83543880d5654eb2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Inventarisiert rekursiv page.json-Artikelmetadaten, verlangt deutsche und englische Markdowntexte, validiert IDs/Eindeutigkeit, sortiert die Artikel und schreibt das generierte pages/index.js. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Inventarisiert rekursiv page.json-Artikelmetadaten, verlangt deutsche und englische Markdowntexte, validiert IDs/Eindeutigkeit, sortiert die Artikel und schreibt das generierte pages/index.js. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/tools/build-pages.mjs:8 pages`<br>`website/tools/build-pages.mjs:24 ID-Prüfung`<br>`website/tools/build-pages.mjs:31 writeFileSync`<br>`website/tools/build-pages.mjs:34 walk` |
| 4. Aufrufer und Verbraucher | root package.json:5 site:build |
| 5. Verwendete Komponenten | node:fs/path · pages/**/page.json und de/en.md |
| 6. Eingaben und Ausgaben | Eingaben: Artikelmetadaten/Dateibaum Ausgabe: pages/index.js |
| 7. Seiteneffekte und Ressourcen | generierte Datei schreiben |
| 8. Art der Verantwortung | Build- und Dokumentationswerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationswerkzeuge. Technische Kopplung: node:fs/path · pages/**/page.json und de/en.md. Aufruferbezug: root package.json:5 site:build |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Eingabe-/Ausgabepfade und root package.json-Aufrufe müssen zusammen angepasst werden. Generator nicht ohne Auftrag ausführen. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Artikelindex erzeugen. Neue Einordnung: Dokumentationswerkzeuge. Besitzer: Website. |

**Belege:** website/tools/build-pages.mjs:8 pages · website/tools/build-pages.mjs:24 ID-Prüfung · website/tools/build-pages.mjs:31 writeFileSync · website/tools/build-pages.mjs:34 walk

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/tools/build-pages.mjs`.

### [website/tools/build-runtime-ecs-reference.mjs](../../website/tools/build-runtime-ecs-reference.mjs)

Vollständig gelesen, 147 Zeilen. SHA-256: `0ca9a5c4581e657f8eb25274fafdfea54828326705934a4f7849dc48367bda5f`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Filtert keen::ecs-Typen aus dem aktuellen reflektierten Snapshot, folgt inner_type-Ketten, klassifiziert Komponenten/Events/Structs und schreibt JSON-Katalog plus kompakte Markdownreferenz. Übernimmt den Capturezeitstempel aus profile.game_version. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Filtert keen::ecs-Typen aus dem aktuellen reflektierten Snapshot, folgt inner_type-Ketten, klassifiziert Komponenten/Events/Structs und schreibt JSON-Katalog plus kompakte Markdownreferenz. Übernimmt den Capturezeitstempel aus profile.game_version. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/tools/build-runtime-ecs-reference.mjs:19 entries`<br>`website/tools/build-runtime-ecs-reference.mjs:43 summary`<br>`website/tools/build-runtime-ecs-reference.mjs:57/61 Dateien`<br>`website/tools/build-runtime-ecs-reference.mjs:71 innerTypeChain`<br>`website/tools/build-runtime-ecs-reference.mjs:83 classify`<br>`website/tools/build-runtime-ecs-reference.mjs:98 markdown` |
| 4. Aufrufer und Verbraucher | root package.json:7 runtime:types |
| 5. Verwendete Komponenten | node:fs/path · data/current.json · snapshot profile/types.json |
| 6. Eingaben und Ausgaben | Eingaben: Typensnapshot und Profil Ausgabe: runtime-ecs-types.json · runtime-ecs-types.md |
| 7. Seiteneffekte und Ressourcen | generierte Referenzdateien schreiben |
| 8. Art der Verantwortung | Build- und Dokumentationswerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationswerkzeuge. Technische Kopplung: node:fs/path · data/current.json · snapshot profile/types.json. Aufruferbezug: root package.json:7 runtime:types |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Eingabe-/Ausgabepfade und root package.json-Aufrufe müssen zusammen angepasst werden. Generator nicht ohne Auftrag ausführen. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: ECS-Datenreferenz erzeugen. Neue Einordnung: Dokumentationswerkzeuge. Besitzer: Website. |

**Belege:** website/tools/build-runtime-ecs-reference.mjs:19 entries · website/tools/build-runtime-ecs-reference.mjs:43 summary · website/tools/build-runtime-ecs-reference.mjs:57/61 Dateien · website/tools/build-runtime-ecs-reference.mjs:71 innerTypeChain · website/tools/build-runtime-ecs-reference.mjs:83 classify · website/tools/build-runtime-ecs-reference.mjs:98 markdown

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/tools/build-runtime-ecs-reference.mjs`.

### [website/tools/build.mjs](../../website/tools/build.mjs)

Vollständig gelesen, 451 Zeilen. SHA-256: `ae51f50343789eac47e98f3f31be19cf2b741def840eb6b2c20e608653c2b4e8`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Erzeugt den Website-API-Katalog aus Lua-Annotations-/Stubdateien. Kopiert Modschemas, wählt neuesten Datensnapshot, extrahiert Klassenfelder/Funktionsparameter/Rückgaben/Beispiele/Fehler und mappt interne Klassennamen auf öffentliche Pfade. Ergänzt zahlreiche englische/deutsche Integer- und Parametererläuterungen aus im Generator eingebetteten Regeln. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Erzeugt den Website-API-Katalog aus Lua-Annotations-/Stubdateien. Kopiert Modschemas, wählt neuesten Datensnapshot, extrahiert Klassenfelder/Funktionsparameter/Rückgaben/Beispiele/Fehler und mappt interne Klassennamen auf öffentliche Pfade. Ergänzt zahlreiche englische/deutsche Integer- und Parametererläuterungen aus im Generator eingebetteten Regeln. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/tools/build.mjs:11 Schemakopie`<br>`website/tools/build.mjs:17 Lua-Dateien`<br>`website/tools/build.mjs:47 Snapshotwahl`<br>`website/tools/build.mjs:57 Parser`<br>`website/tools/build.mjs:131 API schreiben`<br>`website/tools/build.mjs:134 parseReturnAnnotation`<br>`website/tools/build.mjs:166 parseDocumentationSections`<br>`website/tools/build.mjs:206 addIntegerHelp`<br>`website/tools/build.mjs:333 addMissingParameterHelp`<br>`website/tools/build.mjs:419 publicName`<br>`website/tools/build.mjs:439 namespaceOf` |
| 4. Aufrufer und Verbraucher | root package.json:5 site:build |
| 5. Verwendete Komponenten | node:fs/path · VERSION · api/**/*.lua · registry-Schemas · website/data Profile |
| 6. Eingaben und Ausgaben | Eingaben: Lua-Definitionen/Annotationen · Snapshotprofile Ausgabe: data/api.json · data/current.json · schemas/manifest und extended.mod |
| 7. Seiteneffekte und Ressourcen | generierte Daten/Schemas schreiben |
| 8. Art der Verantwortung | Build-/Dokumentationslogik mit redaktioneller API-Anreicherung |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Technischer Annotationparser und umfangreiche manuell gepflegte mehrsprachige API-Erklärungen stehen zusammen. Der Outputname 'implemented symbols' ist keine automatische Prüfung der Rust-Implementierung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationswerkzeuge. Technische Kopplung: node:fs/path · VERSION · api/**/*.lua · registry-Schemas · website/data Profile. Aufruferbezug: root package.json:5 site:build |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Eingabe-/Ausgabepfade und root package.json-Aufrufe müssen zusammen angepasst werden. Generator nicht ohne Auftrag ausführen. |
| 14. Vorhandener Testschutz | website/tools/validate.mjs prüft erzeugte Daten/Schemakopien, nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: API-Dokumentation und Veröffentlichungsdaten erzeugen. Neue Einordnung: Dokumentationswerkzeuge. Besitzer: Website. |

**Belege:** website/tools/build.mjs:11 Schemakopie · website/tools/build.mjs:17 Lua-Dateien · website/tools/build.mjs:47 Snapshotwahl · website/tools/build.mjs:57 Parser · website/tools/build.mjs:131 API schreiben · website/tools/build.mjs:134 parseReturnAnnotation · website/tools/build.mjs:166 parseDocumentationSections · website/tools/build.mjs:206 addIntegerHelp · website/tools/build.mjs:333 addMissingParameterHelp · website/tools/build.mjs:419 publicName · website/tools/build.mjs:439 namespaceOf

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/tools/build.mjs`.

### [website/tools/sanitize-snapshots.mjs](../../website/tools/sanitize-snapshots.mjs)

Vollständig gelesen, 18 Zeilen. SHA-256: `5f40da758f1f165b54ac8ff3229f17ed3a93dcf427792612694c92158fc6b7c2`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Entfernt vier interne Hashfelder aus allen öffentlichen types.json-Snapshots und schreibt die Typen zurück. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Entfernt vier interne Hashfelder aus allen öffentlichen types.json-Snapshots und schreibt die Typen zurück. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/tools/sanitize-snapshots.mjs:5 Snapshot-Schleife`<br>`website/tools/sanitize-snapshots.mjs:11-14 Löschungen`<br>`website/tools/sanitize-snapshots.mjs:16 writeFileSync` |
| 4. Aufrufer und Verbraucher | root package.json:6 site:sanitize |
| 5. Verwendete Komponenten | node:fs/path · website/data/*/types.json |
| 6. Eingaben und Ausgaben | Eingaben: Typensnapshots Ausgabe: bereinigte types.json |
| 7. Seiteneffekte und Ressourcen | Dateien überschreiben |
| 8. Art der Verantwortung | Daten-/Publikationswerkzeug |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationswerkzeuge. Technische Kopplung: node:fs/path · website/data/*/types.json. Aufruferbezug: root package.json:6 site:sanitize |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Eingabe-/Ausgabepfade und root package.json-Aufrufe müssen zusammen angepasst werden. Generator nicht ohne Auftrag ausführen. |
| 14. Vorhandener Testschutz | Keine direkten Tests nachgewiesen. Keine Tests oder Builds ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Öffentliche Snapshotdaten bereinigen. Neue Einordnung: Dokumentationswerkzeuge. Besitzer: Website. |

**Belege:** website/tools/sanitize-snapshots.mjs:5 Snapshot-Schleife · website/tools/sanitize-snapshots.mjs:11-14 Löschungen · website/tools/sanitize-snapshots.mjs:16 writeFileSync

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/tools/sanitize-snapshots.mjs`.

### [website/tools/seed-reference-pages.mjs](../../website/tools/seed-reference-pages.mjs)

Vollständig gelesen, 878 Zeilen. SHA-256: `77dda9599038f8e414dfe4385c149638495c6ab7e8b8a1577d2b14fa1d29c045`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Manuell aufrufbarer Node-Seeder für zweisprachige Modding-Referenzartikel. Liest die Manifest- und Erweiterungsschemas, Loaderkonfigurationsschema und Standardwerte sowie die eingecheckten Mods. Erzeugt pro Artikel page.json, de.md und en.md unter website/pages/modding. Enthält umfangreiche feste deutsch-englische Lua-/Manifest-/EML-/Runtime-Anleitungen, Übersetzungen und separat gepflegte Listen für Link-IDs, Fähigkeiten und zwölf Steuerelemente. Generiert daraus zusätzlich Schemafeld-, Loaderpräferenz-, Mod- und Mod-Einstellungsseiten. Schreibt standardmäßig ausschließlich fehlende Dateien, mit --overwrite auch bestehende. Führt keine Mod- oder Runtimefunktion aus. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Manuell aufrufbarer Node-Seeder für zweisprachige Modding-Referenzartikel. Liest die Manifest- und Erweiterungsschemas, Loaderkonfigurationsschema und Standardwerte sowie die eingecheckten Mods. Erzeugt pro Artikel page.json, de.md und en.md unter website/pages/modding. Enthält umfangreiche feste deutsch-englische Lua-/Manifest-/EML-/Runtime-Anleitungen, Übersetzungen und separat gepflegte Listen für Link-IDs, Fähigkeiten und zwölf Steuerelemente. Generiert daraus zusätzlich Schemafeld-, Loaderpräferenz-, Mod- und Mod-Einstellungsseiten. Schreibt standardmäßig ausschließlich fehlende Dateien, mit --overwrite auch bestehende. Führt keine Mod- oder Runtimefunktion aus. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `addPage:18-35 erstellt Metadaten und schreibt die drei Artikeldateien.`<br>`writeSeedFile:37-39 implementiert fehlende-Datei- beziehungsweise Überschreibverhalten.`<br>`constraints:211-260 projiziert eine begrenzte Auswahl von JSON-Schemawerten und übersetzt ausgewählte Beschreibungen.`<br>`sampleForSchema:262-277 und exampleForField:279-320 bauen anschauliche Werte und verschachtelte Manifestausschnitte.`<br>`schemaField:322-346 erzeugt lokalisierte Schemafeldartikel, Konflikt-IDs und Quelllinks.`<br>`schemaLeaves:508-516 und Präferenzschleife:525-584 erzeugen konfigurative Blattseiten.`<br>`Mod-Einstellungs-/Modübersichtsschleifen:712-872 verbinden reale Paketdefinitionen mit festen Erklärungen und gruppierten Links.` |
| 4. Aufrufer und Verbraucher | website/README.md:60-70 dokumentiert die manuelle Initialerzeugung und --overwrite. · Kein automatischer Aufrufer in package.json oder .github nachgewiesen. package.json site:build führt build.mjs und build-pages.mjs aus, site:check ruft den Seeder nicht auf. · Nachgelagerter Verbraucher ist website/tools/build-pages.mjs:8-31. Es indiziert die erzeugten page.json und überprüft die Existenz beider Sprachdateien. · website/pages/index.js enthält die erzeugten IDs und Pfade. Die Markdownartikel sind laut website/README.md:62-65 die gepflegte leserseitige Quelle. |
| 5. Verwendete Komponenten | Node node:fs und node:path, import.meta.dirname · src/loader/package/src/registry/manifest.schema.json · src/loader/package/src/registry/extended.mod.schema.json · src/loader/package/src/config/loader.schema.json und loader.default.json · mods/*/mod.json und optional extended.mod.json · Vorhandensein von mods/*/README.md für Linkaufnahme · Feste Github-Quellpfade, Hashrouten #doc-*, bekannte UI-Link-IDs und API-Symbolnamen in Texten |
| 6. Eingaben und Ausgaben | Eingaben: Repositorypfade relativ zu import.meta.dirname und vier synchron geparste JSON-Verträge. · CLI-Schalter --overwrite und bestehende Dateien auf dem Dateisystem. · Sortierte unmittelbare Mod-Unterordner, deren Manifesttexte, Einstellungen, Gruppen, Optionen, Ziele und Metadaten. · Vollständig eingecheckte zweisprachige Textpayloads einschließlich Beispiele, Links und handgepflegter Übersetzungen. Ausgabe: Je Artikel ein Metadatenobjekt mit stabiler ID, lokalisierten Titeln/Zusammenfassungen/Navigationsgruppen, Sortierung und optional API-Zuordnungen. · website/pages/modding/**/{page.json,de.md,en.md}. · Konsolenmeldung über pages.length. Diese Zahl zählt vorbereitete Artikelobjekte, auch wenn vorhandene Dateien übersprungen wurden. · Kein Navigationsindex. Dieser entsteht erst durch build-pages.mjs. |
| 7. Seiteneffekte und Ressourcen | Synchrones Lesen von Schemas, Defaults und Moddefinitionen. · Rekursives Anlegen der Artikelordner und synchrones Schreiben fehlender Dateien. · --overwrite ersetzt bestehende Artikelmetadaten und beide Markdownsprachen. Ohne den Schalter wird jede vorhandene Datei einzeln beibehalten. · Fehler beim Lesen, JSON-Parsen oder Schreiben brechen das Skript ohne eigene Fehlerbehandlung ab. Bereits erzeugte Dateien bleiben bestehen. |
| 8. Art der Verantwortung | mehrere |
| 9. Abgrenzung und Auffälligkeiten | Aufteilungskandidat. Schemaableitung und lange redaktionelle Texte trennen. Seeder bleibt ein gezieltes Pflegewerkzeug, kein Runtimebestandteil. Befunde: Belegte Verantwortungsmischung aus Dateierzeugung, Schema- und Paketprojektion, umfangreichem redaktionellem Inhalt, Übersetzungen und Produktlisten. Jeder dieser Bereiche kann sich aus unterschiedlichen Gründen ändern. · Statisch und an vorhandenen Ausgaben belegter Fehler: modules ist bei Zeile 446 bereits loaderSchema.properties.modules.properties. Zeile 522 übergibt diese properties-Map als schemaLeaves({properties:{modules}},['modules']). schemaLeaves erkennt sie ohne properties als ein Blatt und erzeugt modules.modules statt einzelner Modulfelder. Die reale Loaderstruktur hat modules.properties.debugConsole usw. in loader.schema.json:689-694. Die eingecheckte Seite nennt dadurch Typ array, Default nicht festgelegt und den falschen JSON-Pfad /modules/modules. · Belegter defekter Quelllink: Die Aktionsfeldschleife:374 verwendet extended-mod.schema.json, während der tatsächliche Vertrag extended.mod.schema.json heißt. Die erzeugten action-fields-Seiten übernehmen den falschen Github-Pfad. · Der Seeder implementiert nur einen Teil von JSON Schema. constraints projiziert weder sämtliche Kombinatoren noch alle Typregeln. sampleForSchema kann beispielhafte statt vollständig vertragsvalidierte Werte liefern. Eine vollständige Beispielvalidierung ist hier nicht implementiert. · Kein pauschaler Erhaltungsbedarf aus dem Vorhandensein dieses Seeders. Der dokumentierte Pflegeweg behandelt bestehendes Markdown als Quelle und den Seeder als Initialerzeugung. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationspflege. Technische Kopplung: Node node:fs und node:path, import.meta.dirname · src/loader/package/src/registry/manifest.schema.json · src/loader/package/src/registry/extended.mod.schema.json · src/loader/package/src/config/loader.schema.json und loader.default.json · mods/*/mod.json und optional extended.mod.json · Vorhandensein von mods/*/README.md für Linkaufnahme · Feste Github-Quellpfade, Hashrouten #doc-*, bekannte UI-Link-IDs und API-Symbolnamen in Texten. Aufruferbezug: website/README.md:60-70 dokumentiert die manuelle Initialerzeugung und --overwrite. · Kein automatischer Aufrufer in package.json oder .github nachgewiesen. package.json site:build führt build.mjs und build-pages.mjs aus, site:check ruft den Seeder nicht auf. · Nachgelagerter Verbraucher ist website/tools/build-pages.mjs:8-31. Es indiziert die erzeugten page.json und überprüft die Existenz beider Sprachdateien. · website/pages/index.js enthält die erzeugten IDs und Pfade. Die Markdownartikel sind laut website/README.md:62-65 die gepflegte leserseitige Quelle. |
| 11. Duplikation | Links sind in linkIds:392 separat aufgelistet, obwohl die Artikel selbst src/loader/modules/modloader-ui/ui/src/links/order.json als Registerquelle verlinken. Fähigkeiten und Controls sind ebenso feste Tabellen neben gelesenen Schemas. · Schema-/Settingregeln, Standardwerte und Modtexte werden in lesbare Markdownkopien übertragen. Das ist für Initialerzeugung beabsichtigt. Der Default schreibt vorhandene Seiten nicht neu, wodurch spätere Quelländerungen nicht automatisch übernommen werden. · Zweisprachige bodyDe/bodyEn enthalten strukturell parallele redaktionelle Anleitungen. Eine funktionale Dublette eines weiteren Seeders ist nicht nachgewiesen. |
| 12. Nutzungsstatus | Dokumentiertes manuelles Hilfswerkzeug mit eingecheckten erzeugten Seiten. Kein Bestandteil der automatisch verdrahteten Website-Buildskripte nachgewiesen. Das Vorhandensein der Ausgaben belegt Inhaltsverwendung, keine regelmäßige Ausführung des Seeders. |
| 13. Risiko bei Änderung oder Verschiebung | Relativ zu import.meta.dirname berechnete Repositorywurzel, feste Paket-/Mods-/Websitepfade und Quelllinks müssen beim Verschieben angepasst werden. · Stabile Seiten-IDs und Ordnerpfade werden durch vorhandene #doc-Links, page.json und pages/index.js verwendet. Änderungen können Navigation und Querverweise brechen. · --overwrite kann redaktionell gepflegtes Markdown ersetzen. Eine Verschiebung oder Aufteilung muss das einzelne-Datei-Erhaltungsverhalten und die bewusste Initialerzeugung nachvollziehbar berücksichtigen. · Feste Übersetzungen hängen von exakt übereinstimmenden Modbeschreibungen, Labels und Verzeichnisnamen ab. |
| 14. Vorhandener Testschutz | Direkte automatisierte Tests des Seeders, seiner Beispielkonformität, Präferenzpfade und Quelllinks nicht nachgewiesen. · website/tools/build-pages.mjs:8-29 prüft nachgelagert vorhandene de.md/en.md, gültige IDsyntax und eindeutige IDs. Es validiert weder Artikelaussagen noch Schemaquelllinks oder Konfigurationspfade. · website/tools/validate.mjs prüft veröffentlichte Manifest-Schema-Gleichheit, Version/Profilzahlen, API-Symbole und verbotene Tokens in index.html/APIJSON. Die Seeder-Markdowntexte und Modulpräferenzpfade gehören nicht zu diesen Prüfungen. · Keine Tests oder Buildskripte während dieser Analyse ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Dokumentationsinhalte und einmalige Referenzseitenerzeugung. Neue Einordnung: Dokumentationspflege. Besitzer: Website. |

**Belege:** Volllektüre aller 127.703 UTF-16-Zeichen in lückenlosen Intervallen [0,18000), [18000,36000), [36000,54000), [54000,72000), [72000,90000), [90000,108000), [108000,126000), [126000,127703), einschließlich sämtlicher statischer Strings. Datei hat 128.222 Bytes und 878 Zeilen. · Schema-/Defaultpfade und CLI-Schalter:4-11. Dateierzeugung:18-39. · Feste Grundanleitungen:45-172, Feldbeschreibungen:174-209, Fähigkeiten:401-412, Steuerelemente:414-433. · Schemafeldprojektion und tatsächliche Loops:322-374. Handgepflegte Linkliste:392-399. · Loaderpräferenzen:445-584. Modübersetzungen/Erklärungen:586-706. Mod-Einstellungsseiten:709-804. Modübersichten:806-872. · Zusätzlich vollständig gelesen: package.json, website/README.md, website/tools/validate.mjs und website/tools/build-pages.mjs. · Verifizierte bestehende Ausgabe website/pages/modding/modloader-settings/modules-modules/{page.json,en.md} zeigt den nicht existierenden Pfad /modules/modules. · Existenzprüfung belegt extended.mod.schema.json vorhanden und extended-mod.schema.json fehlt. Eingecheckte action-fields-Markdownseiten enthalten den falschen Quelllink.

**Evidenzgrenze:** Vollständig selbst gelesen, SHA256 und Zeilenzahl überprüft. Verwendung durch dokumentierten manuellen Einstieg und vorhandene generierte Seiten belegt. Ergänzende Caller-/Ausgabensuche und gezielte Schemaausschnitte belegen die genannten Fehler. Kein Laufzeit- oder Ausführungstest.

**Maßnahme:** aufteilen. Schemaableitung und lange redaktionelle Texte trennen. Seeder bleibt ein gezieltes Pflegewerkzeug, kein Runtimebestandteil.

**Vorgeschlagene Zielpfade:** `website/tools/seed-reference-pages.mjs`, `website/content/reference/de.json`, `website/content/reference/en.json`.

### [website/tools/validate.mjs](../../website/tools/validate.mjs)

Vollständig gelesen, 54 Zeilen. SHA-256: `f8a8c30b4f7a06a06e5293787f0102d7dfbdd966984381505884d1462e3a789b`.

| Prüffrage | Ergebnis |
| --- | --- |
| 1. Tatsächliche Funktion | Validiert veröffentlichte Schemakopien gegen ihre Quellbytes, APIversion gegen VERSION, Snapshotzähler und das Fehlen interner Hashfelder. Verlangt ausgewählte API-Symbole und prüft erlaubte Namensräume sowie einige ausgeschlossene Texte. |
| 2. Gelöstes Problem | Die Implementierung deckt folgenden konkreten Bedarf ab: Validiert veröffentlichte Schemakopien gegen ihre Quellbytes, APIversion gegen VERSION, Snapshotzähler und das Fehlen interner Hashfelder. Verlangt ausgewählte API-Symbole und prüft erlaubte Namensräume sowie einige ausgeschlossene Texte. |
| 3. Wichtige Klassen, Funktionen und Schnittstellen | `website/tools/validate.mjs:14 Schemaidentität`<br>`website/tools/validate.mjs:20 Versions-/Zählerprüfung`<br>`website/tools/validate.mjs:25 Hashverbot`<br>`website/tools/validate.mjs:30 Pflichtsymbole`<br>`website/tools/validate.mjs:45 Namensräume`<br>`website/tools/validate.mjs:50 verbotene Texte` |
| 4. Aufrufer und Verbraucher | root package.json:8 site:validate |
| 5. Verwendete Komponenten | node:fs/path · VERSION · API-/Snapshotjson · registry-Schemas und website/schemas |
| 6. Eingaben und Ausgaben | Eingaben: erzeugte Website-Referenzdaten und Quellschemas Ausgabe: Erfolgsmeldung oder Error |
| 7. Seiteneffekte und Ressourcen | Dateien lesen · stdout |
| 8. Art der Verantwortung | Qualitätssicherung der Dokumentationsartefakte |
| 9. Abgrenzung und Auffälligkeiten | Befunde der vollständigen Lektüre: Keine zusätzliche unabhängige Verantwortlichkeit belegt. |
| 10. Fachliche und technische Abhängigkeiten | Fachlicher Zusammenhang: Dokumentationswerkzeuge. Technische Kopplung: node:fs/path · VERSION · API-/Snapshotjson · registry-Schemas und website/schemas. Aufruferbezug: root package.json:8 site:validate |
| 11. Duplikation | Keine zusätzliche Angabe im Leseprotokoll. |
| 12. Nutzungsstatus | Build-/Browseraufruf durch angegebene Referenzen nachgewiesen. |
| 13. Risiko bei Änderung oder Verschiebung | Repositoryrelative Eingabe-/Ausgabepfade und root package.json-Aufrufe müssen zusammen angepasst werden. Generator nicht ohne Auftrag ausführen. |
| 14. Vorhandener Testschutz | Datei ist selbst Konsistenzprüfung, nicht ausgeführt. |
| 15. Aus dem Code abgeleitete Kategorie | Gelesene Verantwortung: Website-Referenzkonsistenz prüfen. Neue Einordnung: Dokumentationswerkzeuge. Besitzer: Website. |

**Belege:** website/tools/validate.mjs:14 Schemaidentität · website/tools/validate.mjs:20 Versions-/Zählerprüfung · website/tools/validate.mjs:25 Hashverbot · website/tools/validate.mjs:30 Pflichtsymbole · website/tools/validate.mjs:45 Namensräume · website/tools/validate.mjs:50 verbotene Texte

**Evidenzgrenze:** Datei vollständig gelesen, beschriebene Wirkung durch Code nachgewiesen. Generierte Ergebnisse nicht neu erzeugt.

**Maßnahme:** beibehalten. Website bleibt eigene Veröffentlichung. Generierte Daten werden über ihren Erzeuger gepflegt, Inhalte nicht als implementiertes Verhalten gewertet.

**Vorgeschlagene Zielpfade:** `website/tools/validate.mjs`.

## Getrennt gelesene Abhängigkeitsintegration

Die folgende Datei zählt ausdrücklich nicht zum Nenner 287. Das übrige externe Parserprojekt wurde inventarisiert, nicht vollständig inhaltlich geprüft.

### [src/parser/kfc-parser/crates/mod-loader-bridge/src/lib.rs](../../kfc-parser/crates/mod-loader-bridge/src/lib.rs)

Vollständig gelesen, 432 Zeilen. SHA-256: `e5fc7179a38f05962b4035cff72174307a4b9e8f0219222cc2f9ca2de6282a90`.

**actual_role:** Wählt EML oder ShroudForge für Proxy-DLLs, prüft lokale Release-Dateiliste, übergibt an winmm und startet bei Fehler unabhängige Wiederherstellung. Protokolliert frühe Fehler vor Loggerstart.

**category:** Proxy-Backendwahl, Installationsprüfung und Startbrücke

**dependencies:** Windows Loader/Threads/Events, serde_json, modloader-config.json und version.json

**callers:** dbghelp-/dinput8-Proxys im KFC-Submodul und shroudforge-updater::missing_required_files. Cargo metadata führt Crate als Workspace-Mitglied.

**evidence:** selected_backend, missing_required_files, try_start, queue_recovery, recover_installation, write_diagnostic

**concerns:** Datei im Parser-Submodul enthält ShroudForge-Start-/Recovery- und Logginglogik. Dateiprüfung kontrolliert Pfade und Existenz, keine Inhaltsprüfsummen.

**suggested_group:** Windows-Proxyintegration mit Backendwahl und früher Installationsprüfung

**inputs:** aktueller Prozessroot, modloader-Backend, managedPaths, Logpolicy

**outputs:** Backend/Handoff/fehlende Dateien und Diagnoseresultat

**side_effects:** liest Config und version.json, lädt winmm, startet Recoveryworker/-EXE, schreibt pending-Logs und kann Spielprozess beenden

**responsibility_kind:** Mehrere: Integration, Installationsprüfung, Frühlogging

**usage_status:** Statisch im Produktbuild und im Workspace nachgewiesen. Kein Live-Lauf.

**duplicate_evidence:** Level-/Pfad-/Logformatinterpretation auch im Bootstrap und Rustlogger vorhanden. Frühstartgrenze ist technische Bedingung, kein Beleg identischer Implementierung.

**move_risk:** Beide Proxys sowie Updater umstellen, Backendwahl und DllMain-Loaderlockgrenze erhalten, eigenständige EML-Nutzung nicht beschädigen

**protecting_tests:** Keine eingebetteten Tests. Kein isolierter automatisierter Nachweis der gesamten Handoffsequenz im Audit gefunden.

**evidence_status:** Quellinhalt und Cargo-Mitgliedschaft nachgewiesen, Startverhalten statisch abgeleitet

**symbols:** 13: pub enum Backend { · 20: pub fn selected_backend() -> Backend { · 44: pub enum Handoff { · 53: pub fn missing_required_files(root: &Path) -> Vec<String> { · 105: pub fn try_start() -> Handoff { · 113: fn try_start_shroudforge() -> Handoff { · 152: fn try_start_shroudforge() -> Handoff { · 156: fn failed_handoff(bootstrap: Option<PathBuf>, error: String) -> Handoff { · 176: fn queue_recovery(message: String) -> Handoff { · 239: unsafe extern "system" fn recover_installation(context: *mut std::ffi::c_void) -> u32 { · 295: pub fn write_diagnostic(level: char, source: &str, message: &str) -> bool { · 358: fn game_root() -> Option<PathBuf> { · 363: fn allows(root: &Path, level: char) -> bool { · 393: fn configured_directory(root: &Path, key: &str, default: &str) -> PathBuf { · 407: fn pending_path(root: &Path) -> PathBuf { · 425: fn append_line(path: &Path, line: &[u8]) -> std::io::Result<()> {
