#pragma once
#include <windows.h>
#include <cstdint>
namespace KfcRuntimeCompatibility::EnshroudedClient {
bool IdentifyGameImage(const wchar_t* process_path, const std::uint8_t*& base,
                       const IMAGE_NT_HEADERS64*& nt);
}
