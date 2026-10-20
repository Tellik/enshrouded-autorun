#pragma once

#include <filesystem>
#include <optional>
#include <string>

namespace GameImprovements {
std::optional<std::string> EnableNoIntro(const std::filesystem::path& root);
}
