#pragma once
#include <cstdint>
#include <nlohmann/json_fwd.hpp>
namespace KfcRuntimeCompatibility::EnshroudedClient {
void ReadGameProfile(const nlohmann::json& selected, bool exact, const std::uint8_t* base);
}
