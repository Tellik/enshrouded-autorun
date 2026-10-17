#include "host/provider_state.h"



using namespace ProviderImplementation;

extern "C" {
// Read-only, bounded code evidence; this export never invokes the address.
KFC_EXPORT std::size_t __cdecl KfcRuntimeFunctionCode(std::uint32_t rva, unsigned char* buffer, std::size_t capacity) {
    if (!buffer || !capacity || capacity > 512) return 0;
    const auto* image = reinterpret_cast<const unsigned char*>(GetModuleHandleW(nullptr));
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(image);
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(image + dos->e_lfanew);
    const auto* sections = IMAGE_FIRST_SECTION(nt);
    for (unsigned i = 0; i < nt->FileHeader.NumberOfSections; ++i) {
        const auto& section = sections[i];
        if (!(section.Characteristics & IMAGE_SCN_MEM_EXECUTE) || rva < section.VirtualAddress ||
            rva - section.VirtualAddress >= section.Misc.VirtualSize) continue;
        const auto size = (std::min)(capacity, static_cast<std::size_t>(section.Misc.VirtualSize - (rva - section.VirtualAddress)));
        SIZE_T actual{};
        if (!ReadProcessMemory(GetCurrentProcess(), image + rva, buffer, size, &actual) || actual != size) return 0;
        unsigned char again[512]{};
        if (!ReadProcessMemory(GetCurrentProcess(), image + rva, again, size, &actual) || actual != size ||
            std::memcmp(buffer, again, size)) return 0;
        return size;
    }
    return 0;
}

KFC_EXPORT std::size_t __cdecl KfcRuntimeFunctions(char* buffer, std::size_t capacity) {
    try {
        // Early callers must retry instead of permanently caching an inventory
        // before the exact read profile has supplied its scheduler evidence.
        if (!provider_initialized.load(std::memory_order_acquire)) return 0;
        static const auto catalog = FunctionInventory::Build(nullptr, 0, EngineContext::SystemDependencyLayout());
        const auto required = catalog.size() + 1;
        if (buffer && capacity >= required) std::memcpy(buffer, catalog.c_str(), required);
        return required;
    } catch (...) { return 0; }
}
}
