#include "profile_commands.h"

namespace profile_development {
int import_ecs_capture_command(int argc, char** argv) {
    std::ifstream input(argv[2]);
    if (!input) { std::cerr << "cannot-open-capture\n"; return 2; }
    const auto image_path = option(argc, argv, "--image-report");
    const auto reflection_path = option(argc, argv, "--reflection");
    if (!image_path || !reflection_path) { std::cerr << "import-ecs-capture requires --image-report and --reflection\n"; return 2; }
    json image, reflection;
    try {
        image = json::parse(std::ifstream(*image_path));
        reflection = json::parse(std::ifstream(*reflection_path));
    } catch (const std::exception& exception) { std::cerr << "invalid image or reflection report: " << exception.what() << '\n'; return 2; }
    std::unordered_map<std::string, std::vector<std::uint32_t>> reflection_sizes;
    const bool parser_registry = reflection.contains("types") && reflection.at("types").is_array();
    const auto& reflection_entries = parser_registry ? reflection.at("types") : reflection.at("entries");
    for (const auto& entry : reflection_entries) {
        if (!entry.contains("size")) continue;
        const auto name_key = parser_registry ? "qualifiedName" : "qualified_name";
        if (!entry.contains(name_key)) continue;
        if (!parser_registry && entry.value("kind", "") != "component") continue;
        const auto name = entry.at(name_key).get<std::string>();
        if (!name.starts_with("keen::ecs::")) continue;
        reflection_sizes[name].push_back(entry.at("size").get<std::uint32_t>());
    }
    if (reflection_sizes.empty()) {
        std::cerr << "reflection input has no keen::ecs types; expected kfc-parser reflection_data.json or normalized entries catalog\n";
        return 3;
    }
    json components = json::array(), unresolved = json::array();
    std::unordered_set<std::string> names;
    std::unordered_set<unsigned> indices;
    std::string line;
    while (std::getline(input, line)) {
        std::istringstream row(line);
        std::string index_text, size_text, name;
        if (!std::getline(row, index_text, '\t') || !std::getline(row, size_text, '\t') || !std::getline(row, name)) continue;
        unsigned index{}, size{};
        const auto parsed_index = std::from_chars(index_text.data(), index_text.data() + index_text.size(), index);
        const auto parsed_size = std::from_chars(size_text.data(), size_text.data() + size_text.size(), size);
        if (parsed_index.ec != std::errc{} || parsed_size.ec != std::errc{} ||
            !name.starts_with("keen::ecs::") || index >= 1024 || !size || size > 65535) continue;
        if (!names.insert(name).second || !indices.insert(index).second) {
            unresolved.push_back({{"name", name}, {"index", index}, {"size", size}, {"reason", "duplicate-live-name-or-index"}});
            continue;
        }
        const auto found = reflection_sizes.find(name);
        if (found == reflection_sizes.end() || found->second.size() != 1 || found->second.front() != size) {
            unresolved.push_back({{"name", name}, {"index", index}, {"liveSize", size},
                {"reason", found == reflection_sizes.end() ? "missing-reflection-component" : found->second.size() != 1 ? "ambiguous-reflection-name" : "reflection-size-mismatch"}});
            continue;
        }
        components.push_back({{"name", name}, {"index", index}, {"size", size}});
    }
    if (components.empty()) { std::cerr << "no component rows found; provide output from kfc-runtime-capture-ecs.exe\n"; return 3; }
    const auto exe_name = fs::path(image.at("file").get<std::string>()).filename().string();
    const std::string target = exe_name == "enshrouded_server.exe" ? "enshrouded_server.exe" : "enshrouded.exe";
    const auto version_text = reflection.value("game_version", std::string{});
    const auto version_separator = version_text.find('|');
    const auto game_version = version_text.substr(0, version_separator);
    const std::string id_prefix = target == "enshrouded.exe" ? "enshrouded-client-" : "enshrouded-server-";
    const auto requested_id = option(argc, argv, "--id");
    const auto id = requested_id ? requested_id->string() :
        id_prefix + (!game_version.empty() ? game_version : std::to_string(image.at("timestamp").get<std::uint32_t>()));
    const json catalog = {{"schemaVersion", 1}, {"id", id}, {"target", target},
        {"image", {{"timestamp", image.at("timestamp")}, {"size", image.at("imageSize")}, {"sha256", image.at("sha256")}}},
        {"components", std::move(components)}, {"unresolved", std::move(unresolved)},
        {"source", {{"liveCapture", fs::absolute(argv[2]).string()}, {"reflection", fs::absolute(*reflection_path).string()}}}};
    if (const auto out = option(argc, argv, "--out")) {
        if (!write_json(*out, catalog)) return 2;
        std::cout << "live ECS catalog: " << catalog["components"].size() << " rows -> " << fs::absolute(*out).string() << '\n';
    } else std::cout << catalog.dump(2) << '\n';
    return 0;
}
}
