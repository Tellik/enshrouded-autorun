#include "profile_commands.h"
namespace profile_development {
int select_build_command(int argc, char** argv) {
    fs::path selected = fs::absolute(argv[2]);
    if (fs::is_directory(selected)) {
        const auto client = selected / "enshrouded.exe";
        const auto server = selected / "enshrouded_server.exe";
        if (fs::exists(client)) selected = client;
        else if (fs::exists(server)) selected = server;
        else { std::cerr << "no enshrouded.exe or enshrouded_server.exe in selected directory\n"; return 2; }
    }
    std::string error;
    const auto pe = read_pe(selected, error);
    if (!pe) { std::cerr << error << '\n'; return 2; }
    const auto hash = sha256(selected);
    if (hash.empty()) { std::cerr << "cannot-hash-selected-executable\n"; return 2; }
    const auto build_id = selected.stem().string() + "-" + std::to_string(pe->timestamp) + "-" + hash.substr(0, 12);
    const auto output = option(argc, argv, "--out-dir").value_or(fs::path(SHROUDFORGE_PROFILE_DIRECTORY) / "captures" / build_id);
    if (!write_json(output / "image-report.json", image_report(selected, *pe))) return 2;
    const auto function_report = discover_function_candidates(selected, *pe, true);
    if (!write_json(output / "function-candidates.json", function_report)) return 2;

    const auto catalog_directory = option(argc, argv, "--catalog-directory").value_or(fs::path(SHROUDFORGE_PROFILE_DIRECTORY) / "catalogs");
    std::optional<fs::path> catalog_path;
    if (fs::exists(catalog_directory)) for (const auto& entry : fs::recursive_directory_iterator(catalog_directory)) {
        if (entry.path().extension() != ".json" || entry.path().filename() == "schema.json") continue;
        try {
            const auto candidate = json::parse(std::ifstream(entry.path()));
            if (!candidate.contains("functions") || !candidate.at("functions").is_array()) continue;
            if (candidate.at("image").at("timestamp") == pe->timestamp && candidate.at("image").at("size") == pe->image_size) {
                if (catalog_path) { std::cerr << "multiple function catalogs match this PE identity\n"; return 3; }
                catalog_path = entry.path();
            }
        } catch (...) {}
    }
    std::cout << "selected build: " << selected.string() << '\n'
              << "build id: " << build_id << '\n'
              << "capture directory: " << fs::absolute(output).string() << '\n'
              << "runtime function ranges: " << pe->runtime_functions.size() << '\n'
              << "heuristic candidates: " << function_report["candidates"].size() << '\n';
    if (catalog_path) {
        const auto report_path = output / "function-scan.json";
        const auto catalog = json::parse(std::ifstream(*catalog_path));
        const auto scan = scan_function_signatures(selected, *pe, catalog, false);
        if (!write_json(report_path, scan)) return 2;
        std::cout << "function signature scan: " << report_path.string() << '\n';
    } else {
        std::cout << "no curated catalog for this build yet; inspect candidates and create a reviewed catalog before profile generation\n";
    }
    return 0;
}
}
