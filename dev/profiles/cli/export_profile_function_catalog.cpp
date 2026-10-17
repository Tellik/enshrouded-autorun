#include "profile_cli.h"

namespace profile_development {
std::string bytes_signature(const std::vector<std::uint8_t>& bytes) {
    constexpr char digits[] = "0123456789ABCDEF";
    std::string result;
    for (const auto byte : bytes) {
        if (!result.empty()) result.push_back(' ');
        result.push_back(digits[byte >> 4]); result.push_back(digits[byte & 15]);
    }
    return result;
}

int extract_profile_functions_command(int argc, char** argv) {
    const fs::path profile_path = argv[2];
    try {
        const auto profile = json::parse(std::ifstream(profile_path));
        json functions = json::array(), globals = json::array();
        const auto& hooks = profile.at("hooks");
        for (auto item = hooks.begin(); item != hooks.end(); ++item) {
            const auto id = "hook." + item.key();
            functions.push_back({{"id", id}, {"purpose", "Profile hook: " + item.key()},
                {"signature", item.value().at("signature")}, {"original", item.value().at("original")},
                {"callingConvention", "x64-windows; review from hook implementation"},
                {"thread", item.key() == "game_thread" ? "game-update-thread" : "engine-thread"},
                {"semanticStatus", "candidate"}, {"sideEffects", json::array({"hook-installation"})},
                {"evidence", json::array({"source profile hook entry"})}});
        }
        const auto& operations = profile.at("worldOperations");
        for (auto item = operations.begin(); item != operations.end(); ++item) {
            const auto& operation = item.value();
            if (operation.contains("functionRva") && operation.contains("guardRva") && !operation.at("guardBytes").empty()) {
                const auto raw = operation.at("guardBytes").get<std::vector<std::uint8_t>>();
                functions.push_back({{"id", item.key()}, {"purpose", operation.at("context")},
                    {"signature", bytes_signature(raw)}, {"original", raw}, {"expectedRva", operation.at("guardRva")},
                    {"callingConvention", operation.at("abi")}, {"thread", operation.at("thread")},
                    {"semanticStatus", "candidate"}, {"operationKind", "world-operation"},
                    {"sideEffects", json::array({"game-thread-call"})},
                    {"evidence", json::array({"source profile guard bytes and RVA"})}});
            } else if (operation.contains("globalRva")) {
                globals.push_back({{"id", item.key()}, {"rva", operation.at("globalRva")},
                    {"abi", operation.at("abi")}, {"thread", operation.at("thread")},
                    {"context", operation.at("context")}, {"semanticStatus", "candidate"}});
            }
        }
        const auto& patches = profile.at("runtimePatches");
        for (auto item = patches.begin(); item != patches.end(); ++item) {
            const auto& patch = item.value();
            const auto& function = patch.at("function");
            functions.push_back({{"id", item.key()}, {"purpose", "Guarded runtime patch: " + item.key()},
                {"signature", patch.at("signature")}, {"expectedRva", function.at("beginRva").get<std::uint32_t>() + function.at("targetOffset").get<std::uint32_t>()},
                {"callingConvention", "patch-site; review owning function ABI"}, {"thread", "game-thread"},
                {"semanticStatus", "candidate"}, {"operationKind", patch.at("kind")},
                {"sideEffects", json::array({"code-patch"})}, {"evidence", json::array({"source profile patch signature and function range"})}});
        }
        const auto& image = profile.at("image");
        const json catalog = {{"schemaVersion", 1}, {"id", profile.at("id")}, {"target", profile.at("target")},
            {"image", {{"timestamp", image.at("timestamp")}, {"size", image.at("size")}}},
            {"functions", std::move(functions)}, {"globals", std::move(globals)}};
        if (const auto out = option(argc, argv, "--out")) {
            if (!write_json(*out, catalog)) return 2;
            std::cout << "function catalog extracted from profile: " << fs::absolute(*out).string() << '\n';
        } else std::cout << catalog.dump(2) << '\n';
        return 0;
    } catch (const std::exception& exception) {
        std::cerr << "cannot extract function catalog: " << exception.what() << '\n';
        return 2;
    }
}
}
