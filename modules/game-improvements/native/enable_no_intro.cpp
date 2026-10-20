#include "enable_no_intro.h"

#include "../../entrypoints/windows-bootstrap/logging_config.h"

#include <windows.h>
#include <winternl.h>

#include <cstring>
#include <limits>
#include <string_view>

namespace {
constexpr std::wstring_view skip_intro_option = L"+com_skipIntroVideo";

bool enabled_in_configuration(const std::filesystem::path& root) {
    try {
        const auto config = ShroudforgeConfig::Read(root);
        const auto& module = config.at("modules").at("gameImprovements");
        return module.value("enabled", true) &&
            module.at("noIntro").value("enabled", false);
    } catch (...) {
        return false;
    }
}

bool is_space(wchar_t value) {
    return value == L' ' || value == L'\t';
}

std::size_t option_position(std::wstring_view command_line) {
    for (std::size_t position = 0;
         (position = command_line.find(skip_intro_option, position)) != std::wstring_view::npos;
         ++position) {
        const bool starts_argument = position == 0 || is_space(command_line[position - 1]);
        const auto end = position + skip_intro_option.size();
        const bool ends_argument = end == command_line.size() || is_space(command_line[end]);
        if (starts_argument && ends_argument) return position;
    }
    return std::wstring_view::npos;
}

bool set_process_command_line(std::wstring_view command_line) {
    if (command_line.size() > (std::numeric_limits<USHORT>::max() / sizeof(wchar_t)) - 1) {
        return false;
    }

    auto* teb = NtCurrentTeb();
    auto* parameters = teb && teb->ProcessEnvironmentBlock
        ? teb->ProcessEnvironmentBlock->ProcessParameters
        : nullptr;
    if (!parameters) return false;
    if (parameters->CommandLine.Buffer != GetCommandLineW()) return false;

    const auto command_bytes = command_line.size() * sizeof(wchar_t);
    const auto allocation_bytes = command_bytes + sizeof(wchar_t);
    auto* replacement = static_cast<wchar_t*>(
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, allocation_bytes));
    if (!replacement) return false;
    std::memcpy(replacement, command_line.data(), command_bytes);

    const auto original = parameters->CommandLine;
    parameters->CommandLine.Buffer = replacement;
    parameters->CommandLine.Length = static_cast<USHORT>(command_bytes);
    parameters->CommandLine.MaximumLength = static_cast<USHORT>(allocation_bytes);

    const auto* visible_command_line = GetCommandLineW();
    if (!visible_command_line || std::wstring_view(visible_command_line) != command_line) {
        parameters->CommandLine = original;
        HeapFree(GetProcessHeap(), 0, replacement);
        return false;
    }
    return true;
}

bool configure_skip_intro() {
    const auto* current = GetCommandLineW();
    auto* teb = NtCurrentTeb();
    auto* parameters = teb && teb->ProcessEnvironmentBlock
        ? teb->ProcessEnvironmentBlock->ProcessParameters
        : nullptr;
    if (!current || !parameters || !parameters->CommandLine.Buffer ||
        parameters->CommandLine.Length > parameters->CommandLine.MaximumLength ||
        parameters->CommandLine.Length % sizeof(wchar_t) != 0) {
        return false;
    }

    std::wstring command_line(current);
    const auto option = option_position(command_line);
    if (option == std::wstring::npos) {
        command_line.append(L" ");
        command_line.append(skip_intro_option);
        command_line.append(L" 1");
    } else {
        auto value_start = option + skip_intro_option.size();
        const auto value_separator = value_start;
        while (value_start < command_line.size() && is_space(command_line[value_start])) ++value_start;
        auto value_end = value_start;
        while (value_end < command_line.size() && !is_space(command_line[value_end])) ++value_end;
        if (value_start == value_end) {
            command_line.insert(value_start, value_start == value_separator ? L" 1" : L"1");
        } else {
            command_line.replace(value_start, value_end - value_start, L"1");
        }
    }

    return set_process_command_line(command_line);
}
}

namespace GameImprovements {
std::optional<std::string> EnableNoIntro(const std::filesystem::path& root) {
    if (!enabled_in_configuration(root)) return std::nullopt;
    if (!configure_skip_intro()) {
        return "Game Improvements: could not add the Enshrouded intro-skip option. Game startup will continue unchanged";
    }
    return std::nullopt;
}
}
