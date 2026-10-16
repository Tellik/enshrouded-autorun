#include "ecs_runtime.h"
#include "world_runtime.h"
#include "function_inventory.h"
#include "engine_context.h"
#include "runtime.h"
#include "resource_runtime.h"
#include <algorithm>
#include <atomic>
#include <cstring>
#include <string>
#include <nlohmann/json.hpp>
namespace { std::atomic<bool> provider_initialized{}; }
extern "C" {
KFC_EXPORT unsigned __cdecl KfcRuntimeAbi() { return KFC_RUNTIME_ABI_VERSION; }
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
KFC_EXPORT bool __cdecl KfcRuntimeInitialize() {
    try {
        const auto initialized = EcsRuntime::Initialize();
        ResourceRuntime::Initialize();
        provider_initialized.store(true, std::memory_order_release);
        return initialized;
    } catch (...) {
        provider_initialized.store(false, std::memory_order_release);
        ResourceRuntime::Shutdown();
        EcsRuntime::Shutdown();
        return false;
    }
}
KFC_EXPORT void __cdecl KfcRuntimeTick() {
    try { EcsRuntime::Tick(); } catch (...) { EcsRuntime::Shutdown(); }
}
KFC_EXPORT void __cdecl KfcRuntimeShutdown() {
    provider_initialized.store(false, std::memory_order_release);
    KfcRuntimeNetworkServiceShutdown();
    ResourceRuntime::Shutdown();
    EcsRuntime::Shutdown();
}
KFC_EXPORT void __cdecl KfcRuntimeDiagnostics(char* buffer, std::size_t capacity) {
    if (!buffer || !capacity) return;
    const auto write_error = [buffer, capacity](const char* message, std::size_t required = 0) {
        const auto text = std::string("{\"error\":\"") + message +
            (required ? "\",\"requiredBytes\":" + std::to_string(required) : "\"") + "}";
        const auto size = (std::min)(text.size(), capacity - 1);
        std::memcpy(buffer, text.data(), size); buffer[size] = 0;
    };
    try {
        auto diagnostics = nlohmann::json::parse(EcsRuntime::Diagnostics());
        diagnostics["resourceService"] = nlohmann::json::parse(ResourceRuntime::Status());
        const auto text = diagnostics.dump();
        if (text.size() >= capacity) { write_error("report-too-large", text.size() + 1); return; }
        std::memcpy(buffer, text.data(), text.size()); buffer[text.size()] = 0;
    } catch (...) { write_error("diagnostics-exception"); }
}
KFC_EXPORT void __cdecl KfcRuntimeStatus(char* buffer, std::size_t capacity) {
    if (!buffer || !capacity) return;
    try {
        const auto text = EcsRuntime::Status() + " resource_service=" + ResourceRuntime::Status();
        const auto size = (std::min)(text.size(), capacity - 1);
        std::memcpy(buffer, text.data(), size); buffer[size] = 0;
    } catch (...) { buffer[0] = 0; }
}
}
