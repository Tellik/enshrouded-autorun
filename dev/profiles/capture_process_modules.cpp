#include "profile_commands.h"

namespace profile_development {
int capture_process_command(int argc, char** argv) {
    DWORD pid{};
    const auto text = std::string_view(argv[2]);
    const auto parsed = std::from_chars(text.data(), text.data() + text.size(), pid);
    if (parsed.ec != std::errc{} || parsed.ptr != text.data() + text.size() || !pid) {
        std::cerr << "invalid-pid\n"; return 2;
    }
    HANDLE process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
    if (!process) { std::cerr << "cannot-open-process: " << GetLastError() << '\n'; return 2; }
    std::wstring path(32768, L'\0');
    DWORD path_size = static_cast<DWORD>(path.size());
    FILETIME now{};
    GetSystemTimeAsFileTime(&now);
    ULARGE_INTEGER ticks{};
    ticks.LowPart = now.dwLowDateTime;
    ticks.HighPart = now.dwHighDateTime;
    json report = {{"schemaVersion", 1}, {"pid", pid}, {"capturedAtFileTimeUtc", ticks.QuadPart}};
    if (QueryFullProcessImageNameW(process, 0, path.data(), &path_size)) {
        path.resize(path_size);
        const fs::path image_path(path);
        report["processPath"] = image_path.string();
        std::string error;
        if (auto pe = read_pe(image_path, error)) report["image"] = image_report(image_path, *pe);
        else report["imageError"] = error;
    }
    CloseHandle(process);
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid);
    json modules = json::array();
    if (snapshot != INVALID_HANDLE_VALUE) {
        MODULEENTRY32W module{sizeof(module)};
        if (Module32FirstW(snapshot, &module)) do {
            modules.push_back({{"name", fs::path(module.szModule).string()},
                {"path", fs::path(module.szExePath).string()}, {"baseAddress", reinterpret_cast<std::uintptr_t>(module.modBaseAddr)},
                {"size", module.modBaseSize}});
        } while (Module32NextW(snapshot, &module));
        CloseHandle(snapshot);
    }
    report["modules"] = std::move(modules);
    if (const auto out = option(argc, argv, "--out")) {
        if (!write_json(*out, report)) return 2;
        std::cout << "process capture: " << fs::absolute(*out).string() << '\n';
    } else std::cout << report.dump(2) << '\n';
    return 0;
}
}
