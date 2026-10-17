#pragma once
#include "pe_image.h"
namespace profile_development {
std::vector<int> parse_pattern(std::string_view text);
std::vector<std::uint32_t> find_pattern(const PeImage& pe, const std::vector<int>& pattern);
bool rva_bytes_equal(const PeImage& pe, std::uint32_t rva, const std::vector<std::uint8_t>& expected);
}
