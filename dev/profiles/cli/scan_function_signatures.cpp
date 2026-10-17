#include "profile_cli.h"
namespace profile_development {
json scan_function_signatures(const fs::path& exe, const PeImage& pe, const json& catalog, bool include_expected_rva) {
    json functions = json::array();
    bool all_unique = true;
    for (const auto& entry : catalog.at("functions")) {
        const auto id = entry.at("id").get<std::string>();
        const auto signature = entry.at("signature").get<std::string>();
        const auto pattern = parse_pattern(signature);
        const auto matches = find_pattern(pe, pattern);
        json rvas = json::array();
        for (const auto rva : matches) rvas.push_back(rva);
        const bool one_match = matches.size() == 1;
        const bool expected_rva_matches = !entry.contains("expectedRva") ||
            (one_match && matches.front() == entry.at("expectedRva").get<std::uint32_t>());
        const auto original = entry.value("original", std::vector<std::uint8_t>{});
        const bool original_matches = original.empty() || (one_match && rva_bytes_equal(pe, matches.front(), original));
        const bool unique = one_match && expected_rva_matches && original_matches;
        all_unique = all_unique && unique;
        functions.push_back({{"id", id}, {"signature", signature}, {"matches", std::move(rvas)},
            {"matchCount", matches.size()}, {"expectedRva", entry.value("expectedRva", 0u)},
            {"original", original},
            {"expectedRvaMatches", expected_rva_matches}, {"originalBytesMatch", original_matches},
            {"status", unique ? "unique-candidate" : matches.empty() ? "not-found" : matches.size() > 1 ? "ambiguous" : !expected_rva_matches ? "wrong-rva" : "original-bytes-mismatch"},
            {"callingConvention", entry.value("callingConvention", "unknown")},
            {"semanticStatus", entry.value("semanticStatus", "unverified")}});
    }
    if (!include_expected_rva) for (auto& row : functions) row.erase("expectedRva");
    auto report = image_report(exe, pe);
    report["functions"] = std::move(functions);
    report["allFunctionCandidatesUnique"] = all_unique;
    json runtime_functions = json::array();
    for (const auto& function : pe.runtime_functions)
        runtime_functions.push_back({{"beginRva", function.begin}, {"endRva", function.end}});
    report["runtimeFunctions"] = std::move(runtime_functions);
    return report;
}

int scan_command(int argc, char** argv) {
    const fs::path exe = argv[2], catalog_path = argv[3];
    std::string error;
    auto pe = read_pe(exe, error);
    if (!pe) { std::cerr << error << '\n'; return 2; }
    json catalog;
    try { catalog = json::parse(std::ifstream(catalog_path)); }
    catch (const std::exception& exception) { std::cerr << "invalid function catalog: " << exception.what() << '\n'; return 2; }
    auto report = scan_function_signatures(exe, *pe, catalog);
    const auto all_unique = report.at("allFunctionCandidatesUnique").get<bool>();
    if (const auto out = option(argc, argv, "--out")) {
        if (!write_json(*out, report)) return 2;
        std::cout << "function scan: " << fs::absolute(*out).string() << '\n';
    } else std::cout << report.dump(2) << '\n';
    return all_unique ? 0 : 3;
}
}
