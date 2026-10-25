#include "engine-hooks/scan_executable_signature.h"
#include <windows.h>
#include <algorithm>
#include <charconv>
#include <sstream>
#include <string>
#include <vector>

namespace EngineHooks {
std::vector<int> ParseExecutableSignature(std::string_view text) {
    std::vector<int> bytes;
    std::istringstream input{std::string(text)};
    std::string token;
    while (input >> token) {
        if (token == "?" || token == "??") { bytes.push_back(-1); continue; }
        unsigned value{};
        const auto parsed = std::from_chars(token.data(), token.data() + token.size(), value, 16);
        if (parsed.ec != std::errc{} || parsed.ptr != token.data() + token.size() || value > 255) return {};
        bytes.push_back(static_cast<int>(value));
    }
    return bytes;
}



SignatureMatch FindExecutableSignature(std::uint8_t* base, std::string_view text, std::size_t stop_after) {
    const auto signature = ParseExecutableSignature(text);
    if (signature.empty()) return {};
    const auto dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(base);
    const auto nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(base + dos->e_lfanew);
    const auto sections = IMAGE_FIRST_SECTION(nt);
    SignatureMatch result{};
    for (WORD i = 0; i < nt->FileHeader.NumberOfSections; ++i) {
        const auto& section = sections[i];
        if (!(section.Characteristics & IMAGE_SCN_MEM_EXECUTE)) continue;
        const auto length = (std::min)(static_cast<std::size_t>(section.Misc.VirtualSize),
            static_cast<std::size_t>(nt->OptionalHeader.SizeOfImage - section.VirtualAddress));
        if (length < signature.size()) continue;
        auto* begin = base + section.VirtualAddress;
        for (std::size_t offset = 0; offset <= length - signature.size(); ++offset) {
            bool matches = true;
            for (std::size_t j = 0; j < signature.size(); ++j)
                if (signature[j] >= 0 && begin[offset + j] != signature[j]) { matches = false; break; }
            if (!matches) continue;
            if (!result.count) result.address = reinterpret_cast<std::uintptr_t>(begin + offset);
            ++result.count;
            if (result.count >= stop_after) return result;
        }
    }
    return result;
}
}
