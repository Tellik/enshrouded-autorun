#pragma once
#include <windows.h>
#include <bcrypt.h>
#include <tlhelp32.h>
#include <nlohmann/json.hpp>
#include "../../../modules/game-runtime/native/abi/runtime_api.h"

#include <algorithm>
#include <array>
#include <charconv>
#include <cctype>
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <iterator>
#include <sstream>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_set>
#include <vector>

#include <cstring>
#include <unordered_map>

namespace fs = std::filesystem;
using json = nlohmann::json;

namespace profile_development {
struct PeImage {
    std::vector<std::uint8_t> bytes;
    std::uint32_t timestamp{};
    std::uint32_t image_size{};
    std::uint32_t checksum{};
    std::uint16_t machine{};
    struct Section { std::string name; std::uint32_t rva{}, virtual_size{}, raw_offset{}, raw_size{}, characteristics{}; };
    std::vector<Section> sections;
    struct RuntimeFunction { std::uint32_t begin{}, end{}, unwind{}; };
    std::vector<RuntimeFunction> runtime_functions;
};

std::optional<PeImage> read_pe(const fs::path& path, std::string& error);
std::string sha256(const fs::path& path);
json image_report(const fs::path& path, const PeImage& pe);
}
