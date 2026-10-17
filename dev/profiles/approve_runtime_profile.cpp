#include "profile_commands.h"

namespace profile_development {
int approve_profile_command(int argc, char** argv) {
    const auto scan_path = option(argc, argv, "--function-scan");
    const auto components_path = option(argc, argv, "--components");
    if (!scan_path || !components_path) { std::cerr << "approve-profile requires --function-scan and --components\n"; return 2; }
    try {
        auto profile = json::parse(std::ifstream(argv[2]));
        const auto scan = json::parse(std::ifstream(*scan_path));
        const auto catalog = json::parse(std::ifstream(*components_path));
        if (!profile.contains("provenance") || profile.at("provenance").value("status", "") != "draft-requires-live-review") {
            std::cerr << "input profile is not a generated draft\n"; return 2;
        }
        const auto capture_path = fs::path(profile.at("provenance").value("componentCapture", std::string{}));
        if (capture_path.empty() || !fs::exists(capture_path)) {
            std::cerr << "original live ECS capture report is missing; regenerate the profile draft\n"; return 2;
        }
        const auto live_capture = json::parse(std::ifstream(capture_path));
        if (profile.at("image").at("sha256") != scan.at("sha256") ||
            profile.at("image").at("timestamp") != scan.at("timestamp") ||
            profile.at("image").at("size") != scan.at("imageSize") ||
            profile.at("image").at("sha256") != catalog.at("image").at("sha256") ||
            profile.at("image").at("timestamp") != catalog.at("image").at("timestamp") ||
            profile.at("image").at("size") != catalog.at("image").at("size") ||
            profile.at("id") != catalog.at("id") || profile.at("target") != catalog.at("target")) {
            std::cerr << "profile, function scan, and component catalog must match by filename and executable SHA-256\n"; return 2;
        }
        if (!scan.value("allFunctionCandidatesUnique", false)) {
            std::cerr << "function scan contains missing, ambiguous, wrong-RVA, or byte-mismatched candidates\n"; return 3;
        }
        if (catalog.contains("unresolved") && !catalog.at("unresolved").empty()) {
            std::cerr << "component capture contains unresolved reflection joins\n"; return 3;
        }
        if (live_capture.at("image").at("sha256") != profile.at("image").at("sha256") ||
            live_capture.at("id") != profile.at("id") || live_capture.at("target") != profile.at("target") ||
            live_capture.at("components") != catalog.at("components") ||
            (live_capture.contains("unresolved") && !live_capture.at("unresolved").empty())) {
            std::cerr << "source live capture differs from the normalized component catalog or selected executable\n"; return 3;
        }
        if (profile.at("provenance").value("unresolvedComponentCount", std::size_t(-1)) != 0) {
            std::cerr << "profile provenance records unresolved component mappings\n"; return 3;
        }
        std::string validation_error;
        const auto& components = catalog.at("components");
        if (!validate_profile(profile, validation_error, &components)) {
            std::cerr << "profile structure invalid: " << validation_error << '\n'; return 2;
        }
        std::cout << "Profile: " << profile.at("id").get<std::string>() << '\n'
                  << "Image SHA-256: " << profile.at("image").at("sha256").get<std::string>() << '\n'
                  << "Function scan: " << fs::absolute(*scan_path).string() << '\n'
                  << "Component catalog: " << fs::absolute(*components_path).string() << '\n';
        std::string answer;
        std::cout << "Have you verified all layout offsets against a live session of this exact build? Type VERIFY: ";
        std::getline(std::cin, answer);
        if (answer != "VERIFY") { std::cerr << "profile remains a draft\n"; return 3; }
        std::cout << "Have you reviewed hook/function semantics, calling conventions, and side effects? Type VERIFY: ";
        std::getline(std::cin, answer);
        if (answer != "VERIFY") { std::cerr << "profile remains a draft\n"; return 3; }
        FILETIME now{};
        GetSystemTimeAsFileTime(&now);
        ULARGE_INTEGER ticks{};
        ticks.LowPart = now.dwLowDateTime;
        ticks.HighPart = now.dwHighDateTime;
        auto& provenance = profile["provenance"];
        provenance["status"] = "approved";
        provenance["runtimeLayoutValidated"] = true;
        provenance["componentMappingsValidated"] = true;
        provenance["functionSemanticsValidated"] = true;
        std::array<char, 256> username{};
        const auto username_size = GetEnvironmentVariableA("USERNAME", username.data(), static_cast<DWORD>(username.size()));
        provenance["reviewer"] = username_size > 0 && username_size < username.size()
            ? std::string(username.data(), username_size) : "local-developer";
        provenance["reviewedAtFileTimeUtc"] = ticks.QuadPart;
        provenance["reviewEvidence"] = json::array({fs::absolute(*scan_path).string(), fs::absolute(capture_path).string(), fs::absolute(*components_path).string()});
        if (!write_json(argv[2], profile)) return 2;
        std::cout << "profile approved and saved: " << fs::absolute(argv[2]).string() << '\n';
        return 0;
    } catch (const std::exception& exception) { std::cerr << "profile approval failed: " << exception.what() << '\n'; return 2; }
}
}
