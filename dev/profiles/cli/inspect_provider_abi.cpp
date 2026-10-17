#include "profile_cli.h"

namespace profile_development {
int inspect_provider_command(int argc, char** argv) {
    const fs::path path = fs::absolute(argv[2]);
    HMODULE module = LoadLibraryExW(path.c_str(), nullptr, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
    if (!module) { std::cerr << "cannot-load-provider: " << GetLastError() << '\n'; return 2; }
    using AbiFunction = unsigned (__cdecl*)();
    const auto abi_function = reinterpret_cast<AbiFunction>(GetProcAddress(module, "KfcRuntimeAbi"));
    json exports = json::array();
    constexpr std::string_view names[]{
        "KfcRuntimeAbi", "KfcRuntimeInitialize", "KfcRuntimeTick", "KfcRuntimeShutdown",
        "KfcRuntimeStatus", "KfcRuntimeDiagnostics", "KfcRuntimeEcsConfigure",
        "KfcRuntimeEcsReady", "KfcRuntimeEcsCanWrite", "KfcRuntimeEcsDescribe",
        "KfcRuntimeEcsQuery", "KfcRuntimeEcsResolve", "KfcRuntimeEcsRead", "KfcRuntimeEcsWrite",
        "KfcRuntimeEcsCompareExchange", "KfcRuntimeEcsRegistry", "KfcRuntimeEcsResolveType",
        "KfcRuntimeFunctions", "KfcRuntimeFunctionCode",
        "KfcRuntimePatchAvailable", "KfcRuntimePatchSetEnabled",
        "KfcRuntimeWorldOperationAvailable", "KfcRuntimeWorldContextActive",
        "KfcRuntimeWorldEntityContextReady", "KfcRuntimeWorldCursorRead",
        "KfcRuntimeWorldVoxelRead", "KfcRuntimeWorldVoxelWrite",
        "KfcRuntimeWorldEntitySpawn", "KfcRuntimeWorldEntityPlace", "KfcRuntimeWorldEntityDestroy",
        "KfcRuntimeWorldEntityFinishBuilding"
    };
    bool complete = true;
    for (const auto name : names) {
        const bool present = GetProcAddress(module, std::string(name).c_str()) != nullptr;
        exports.push_back({{"name", name}, {"present", present}});
        complete = complete && present;
    }
    const auto abi = abi_function ? abi_function() : 0;
    FreeLibrary(module);
    const json report = {{"schemaVersion", 1}, {"provider", path.string()}, {"providerAbi", abi},
        {"expectedAbi", KFC_RUNTIME_ABI_VERSION}, {"expectedExportsPresent", complete}, {"exports", std::move(exports)}};
    if (const auto out = option(argc, argv, "--out")) {
        if (!write_json(*out, report)) return 2;
        std::cout << "provider inventory: " << fs::absolute(*out).string() << '\n';
    } else std::cout << report.dump(2) << '\n';
    return complete && abi == KFC_RUNTIME_ABI_VERSION ? 0 : 3;
}
}
