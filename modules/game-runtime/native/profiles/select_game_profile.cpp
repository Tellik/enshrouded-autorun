#include "profiles/game_profile.h"
#include "profiles/identify_game_image.h"
#include "profiles/read_game_profile.h"
#include "embedded_profile_ids.h"
#include <windows.h>
#include <filesystem>
#include <fstream>
#include <vector>
#include <nlohmann/json.hpp>

namespace KfcRuntimeCompatibility::EnshroudedClient {

bool Load() {
    try {
        HMODULE self{};
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            reinterpret_cast<LPCWSTR>(&Load), &self)) return false;
        wchar_t process_path[32768]{};
        if (!GetModuleFileNameW(nullptr, process_path, 32768)) return false;
        const std::uint8_t* base{};
        const IMAGE_NT_HEADERS64* nt{};
        if (!IdentifyGameImage(process_path, base, nt)) return false;
        const auto& process = image_target;
        const auto& process_hash = image_sha256;
        nlohmann::json exact_profile, fallback_profile;
        unsigned exact_count{}, fallback_count{};
        std::string requested_profile_id;
        const auto game_root = std::filesystem::path(process_path).parent_path();
        const auto loader_config_path = game_root / L"shroudforge" / L"config" / L"modloader-config.json";
        auto runtime_directory = game_root / L"shroudforge" / L"runtime";
        if (std::filesystem::exists(loader_config_path)) {
            std::ifstream config_file(loader_config_path);
            if (config_file) {
                const auto config = nlohmann::json::parse(config_file, nullptr, false);
                if (config.is_object() && config.contains("paths") && config["paths"].is_object() &&
                    config["paths"].contains("runtime") && config["paths"]["runtime"].is_string()) {
                    const auto configured = std::filesystem::path(config["paths"]["runtime"].get<std::string>());
                    runtime_directory = configured.is_absolute() ? configured : game_root / configured;
                }
                if (config.is_object() && config.contains("runtime") && config["runtime"].is_object() &&
                    config["runtime"].contains("profileId") && config["runtime"]["profileId"].is_string())
                    requested_profile_id = config["runtime"]["profileId"].get<std::string>();
            }
        }
        const auto profile_root = runtime_directory / L"profiles";
        auto load_external_profiles = [&](const std::string& id_filter) {
            std::vector<nlohmann::json> loaded_profiles;
            if (!std::filesystem::exists(profile_root)) return loaded_profiles;
            for (auto iterator = std::filesystem::recursive_directory_iterator(profile_root);
                iterator != std::filesystem::recursive_directory_iterator(); ++iterator) {
                const auto& entry = *iterator;
                if (entry.is_directory() && entry.path().filename() == L"mod-profiles") {
                    iterator.disable_recursion_pending();
                    continue;
                }
                if (!entry.is_regular_file() || entry.path().extension() != L".json") continue;
                std::ifstream profile_file(entry.path());
                if (!profile_file) continue;
                auto candidate = nlohmann::json::parse(profile_file, nullptr, false);
                if (!candidate.is_object() || !candidate.contains("ecsLayout") ||
                    !candidate.contains("hooks")) continue;
                const auto component_resolution = candidate.find("componentResolution");
                const bool live_components = component_resolution != candidate.end() &&
                    component_resolution->is_string() && *component_resolution == "live-registration";
                if (!candidate.contains("components") && !live_components) continue;
                if (!id_filter.empty() && candidate.value("id", std::string{}) != id_filter) continue;
                if (candidate.value("target", std::string{}) != process) continue;
                loaded_profiles.push_back(std::move(candidate));
            }
            return loaded_profiles;
        };
        std::vector<nlohmann::json> candidates;
        if (!requested_profile_id.empty()) {
            candidates = load_external_profiles(requested_profile_id);
            if (candidates.size() != 1) {
                status = candidates.empty() ? "manual-profile-not-found:" + requested_profile_id
                    : "manual-profile-id-ambiguous:" + requested_profile_id;
                return false;
            }
        } else {
            candidates = load_external_profiles({});
            if (candidates.empty()) {
                for (const auto resource_id : KfcRuntimeEmbedded::profile_resource_ids) {
                    const auto resource = FindResourceW(self, MAKEINTRESOURCEW(resource_id), MAKEINTRESOURCEW(10));
                    if (!resource) throw std::runtime_error("embedded compatibility profile is missing");
                    const auto loaded = LoadResource(self, resource);
                    if (!loaded) throw std::runtime_error("embedded compatibility profile could not be loaded");
                    const auto size = SizeofResource(self, resource);
                    const auto* data = static_cast<const char*>(LockResource(loaded));
                    if (!data || !size) throw std::runtime_error("embedded compatibility profile is empty");
                    auto candidate = nlohmann::json::parse(data, data + size);
                    candidates.push_back(std::move(candidate));
                }
            }
        }
        for (auto& candidate : candidates) {
            if (candidate.at("schemaVersion") != 1 || candidate.at("target") != process) continue;
            if (!candidate.contains("kfcFormat") || !candidate.at("kfcFormat").is_number_integer()) continue;
            const auto candidate_kfc_format = candidate.at("kfcFormat").get<int>();
            if (candidate_kfc_format != 3 && candidate_kfc_format != 4) continue;
            // Structural revalidation stays within the actual asset format.
            // KFC3 execution layouts cannot authorize a KFC4 process merely
            // because one of their instruction guards still happens to match.
            if (kfc_format && candidate_kfc_format != kfc_format) continue;
            // Generated profile drafts are developer artifacts. They must never
            // become active merely because the EXE identity happens to match.
            if (candidate.contains("provenance") &&
                candidate.at("provenance").value("status", std::string{}) != "approved") continue;
            const bool matches = candidate.at("image").at("timestamp") == nt->FileHeader.TimeDateStamp &&
                candidate.at("image").at("size") == nt->OptionalHeader.SizeOfImage &&
                (!candidate.at("image").contains("sha256") ||
                 (!process_hash.empty() && candidate.at("image").at("sha256").get<std::string>() == process_hash));
            if (candidate.contains("provenance")) {
                const auto& provenance = candidate.at("provenance");
                if (provenance.value("status", std::string{}) != "approved" ||
                    !provenance.value("runtimeLayoutValidated", false) ||
                    !provenance.value("componentMappingsValidated", false) ||
                    !provenance.value("functionSemanticsValidated", false) ||
                    !candidate.at("image").contains("sha256") ||
                    provenance.value("executableSha256", std::string{}) != candidate.at("image").at("sha256").get<std::string>() ||
                    provenance.value("unresolvedComponentCount", std::size_t(-1)) != 0) continue;
            }
            if (matches) {
                ++exact_count;
                exact_profile = std::move(candidate);
            } else if (!requested_profile_id.empty() || candidate.value("allowStructuralRevalidation", false)) {
                ++fallback_count;
                fallback_profile = std::move(candidate);
            }
        }
        // Exact image identity always takes precedence over every opt-in
        // structural fallback, regardless of file enumeration order.
        const bool exact = exact_count == 1;
        if (exact_count > 1) {
            status = "ambiguous-exact-profile"; return false;
        }
        if (exact_count == 0 && fallback_count != 1) {
            status = "missing-or-ambiguous-profile"; return false;
        }
        const auto& selected = exact ? exact_profile : fallback_profile;
        exact_build_match = exact;
        if (!kfc_format) kfc_format = static_cast<std::uint8_t>(selected.at("kfcFormat").get<unsigned>());
        // Identity is a lookup hint. Both hooks still require unique executable
        // matches and exact overwritten instructions before any patch is made.
        ReadGameProfile(selected, exact, base);
        return true;
    } catch (const std::exception& error) {
        status = std::string("profile-error:") + error.what();
        OutputDebugStringA(status.c_str());
        return false;
    }
}
}
