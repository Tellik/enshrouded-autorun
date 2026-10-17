#pragma once
#include "function_signatures.h"
namespace profile_development {
void usage();
std::optional<fs::path> option(int argc, char** argv, std::string_view key);
bool write_json(const fs::path& path, const json& value);
json discover_function_candidates(const fs::path& exe, const PeImage& pe, bool selected_build = false);
json scan_function_signatures(const fs::path& exe, const PeImage& pe, const json& catalog, bool include_expected_rva = true);
bool validate_profile(const json& profile, std::string& error, const json* external_components = nullptr);
int inspect_command(int argc, char** argv);
int select_build_command(int argc, char** argv);
int scan_command(int argc, char** argv);
int discover_command(int argc, char** argv);
int extract_profile_functions_command(int argc, char** argv);
int capture_process_command(int argc, char** argv);
int import_ecs_capture_command(int argc, char** argv);
int inspect_provider_command(int argc, char** argv);
int validate_command(const fs::path& path, const std::optional<fs::path>& component_path = std::nullopt);
int approve_profile_command(int argc, char** argv);
int generate_command(int argc, char** argv);
}
