#include "profile_cli.h"
namespace profile_development {
json function_candidates(const PeImage& pe) {
    json candidates = json::array();
    // Conservative x64 entry-point heuristics. These are leads for a developer
    // to inspect, never approved function bindings or safe hook locations.
    for (const auto& section : pe.sections) {
        if (!(section.characteristics & IMAGE_SCN_MEM_EXECUTE) || section.raw_offset >= pe.bytes.size()) continue;
        const auto size = (std::min)(static_cast<std::size_t>(section.raw_size), pe.bytes.size() - section.raw_offset);
        std::uint32_t last_rva{};
        for (std::size_t offset = 0; offset + 8 <= size; ++offset) {
            const auto* code = pe.bytes.data() + section.raw_offset + offset;
            const bool push_nonvolatile = (code[0] == 0x53 || code[0] == 0x55 || code[0] == 0x56 || code[0] == 0x57 ||
                ((code[0] & 0xf8) == 0x40 && (code[1] == 0x53 || code[1] == 0x55 || code[1] == 0x56 || code[1] == 0x57)));
            const bool stack_frame = code[0] == 0x48 && code[1] == 0x83 && code[2] == 0xec && code[3] >= 0x20;
            const bool large_stack_frame = code[0] == 0x48 && code[1] == 0x81 && code[2] == 0xec;
            if (!push_nonvolatile && !stack_frame && !large_stack_frame) continue;
            const auto rva = section.rva + static_cast<std::uint32_t>(offset);
            if (rva - last_rva < 8) continue;
            last_rva = rva;
            std::string prefix;
            constexpr char digits[] = "0123456789ABCDEF";
            for (std::size_t i = 0; i < 16 && offset + i < size; ++i) {
                if (!prefix.empty()) prefix.push_back(' ');
                const auto byte = code[i];
                prefix.push_back(digits[byte >> 4]); prefix.push_back(digits[byte & 15]);
            }
            candidates.push_back({{"rva", rva}, {"section", section.name}, {"heuristic",
                stack_frame ? "sub-rsp-imm8" : large_stack_frame ? "sub-rsp-imm32" : "push-nonvolatile-register"},
                {"prefix", std::move(prefix)}, {"status", "unverified-candidate"}});
        }
    }
    return candidates;
}

json discover_function_candidates(const fs::path& exe, const PeImage& pe, bool selected_build) {
    auto report = image_report(exe, pe);
    report["method"] = selected_build ? "x64-exception-table-and-prologue-heuristics" : "x64-prologue-heuristics";
    report["warning"] = selected_build
        ? "Function ranges are recovered from PE unwind metadata; names and semantics require investigation."
        : "The PE exception directory enumerates compiler-described function ranges. Prologue candidates are heuristic and neither source names nor semantics are inferred.";
    json ranges = json::array();
    for (const auto& function : pe.runtime_functions)
        ranges.push_back({{"beginRva", function.begin}, {"endRva", function.end},
            {"size", function.end - function.begin}, {"unwindInfoRva", function.unwind}});
    report["runtimeFunctions"] = std::move(ranges);
    report["candidates"] = function_candidates(pe);
    return report;
}

int discover_command(int argc, char** argv) {
    const fs::path exe = argv[2];
    std::string error;
    auto pe = read_pe(exe, error);
    if (!pe) { std::cerr << error << '\n'; return 2; }
    auto report = discover_function_candidates(exe, *pe);
    if (const auto out = option(argc, argv, "--out")) {
        if (!write_json(*out, report)) return 2;
        std::cout << "function candidates: " << report["candidates"].size() << " -> " << fs::absolute(*out).string() << '\n';
    } else std::cout << report.dump(2) << '\n';
    return 0;
}
}
