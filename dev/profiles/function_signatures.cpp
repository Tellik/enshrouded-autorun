#include "function_signatures.h"

namespace profile_development {
std::vector<int> parse_pattern(std::string_view text) {
    std::vector<int> result;
    std::size_t cursor{};
    while (cursor < text.size()) {
        while (cursor < text.size() && std::isspace(static_cast<unsigned char>(text[cursor]))) ++cursor;
        if (cursor == text.size()) break;
        const auto end = text.find_first_of(" \t\r\n", cursor);
        const auto token = text.substr(cursor, end == std::string_view::npos ? text.size() - cursor : end - cursor);
        if (token == "?" || token == "??") result.push_back(-1);
        else {
            unsigned byte{};
            const auto parsed = std::from_chars(token.data(), token.data() + token.size(), byte, 16);
            if (parsed.ec != std::errc{} || parsed.ptr != token.data() + token.size() || byte > 255) return {};
            result.push_back(static_cast<int>(byte));
        }
        cursor = end == std::string_view::npos ? text.size() : end;
    }
    return result;
}

std::vector<std::uint32_t> find_pattern(const PeImage& pe, const std::vector<int>& pattern) {
    std::vector<std::uint32_t> matches;
    if (pattern.empty()) return matches;
    for (const auto& section : pe.sections) {
        if (!(section.characteristics & IMAGE_SCN_MEM_EXECUTE) || section.raw_offset >= pe.bytes.size()) continue;
        const auto available = pe.bytes.size() - section.raw_offset;
        const auto size = (std::min)(static_cast<std::size_t>(section.raw_size), available);
        if (size < pattern.size()) continue;
        for (std::size_t offset = 0; offset <= size - pattern.size(); ++offset) {
            bool match = true;
            for (std::size_t i = 0; i < pattern.size(); ++i)
                if (pattern[i] >= 0 && pe.bytes[section.raw_offset + offset + i] != pattern[i]) { match = false; break; }
            if (match) matches.push_back(section.rva + static_cast<std::uint32_t>(offset));
        }
    }
    return matches;
}

bool rva_bytes_equal(const PeImage& pe, std::uint32_t rva, const std::vector<std::uint8_t>& expected) {
    for (const auto& section : pe.sections) {
        if (rva < section.rva) continue;
        const auto delta = static_cast<std::size_t>(rva - section.rva);
        if (delta > section.raw_size || expected.size() > section.raw_size - delta) continue;
        const auto raw = static_cast<std::size_t>(section.raw_offset) + delta;
        return raw <= pe.bytes.size() && expected.size() <= pe.bytes.size() - raw &&
            std::equal(expected.begin(), expected.end(), pe.bytes.begin() + raw);
    }
    return false;
}
}
