#pragma once
#include <cstddef>
#include <cstdint>
#include <string_view>
#include <vector>
namespace EngineHooks {
std::vector<int> ParseExecutableSignature(std::string_view text);
struct SignatureMatch { std::uintptr_t address{}; std::size_t count{}; };
SignatureMatch FindExecutableSignature(std::uint8_t* base, std::string_view text,
                                       std::size_t stop_after = SIZE_MAX);
}
