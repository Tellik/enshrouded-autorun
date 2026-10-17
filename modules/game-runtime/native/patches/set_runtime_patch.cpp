#include "abi/runtime_api.h"
#include "engine-hooks/scan_executable_signature.h"
#include "engine-hooks/allocate_near_code.h"
#include "engine-hooks/encode_relative_jump.h"
#include "engine-hooks/write_executable_bytes.h"
#include "patches/set_runtime_patch.h"

#include "game-thread/run_game_thread_operations.h"
#include "profiles/game_profile.h"

#include <windows.h>
#include <nlohmann/json.hpp>
#include <algorithm>
#include <charconv>
#include <cstring>
#include <limits>
#include <optional>
#include <sstream>
#include <string_view>
#include <vector>

namespace {
struct PatchState {
    const KfcRuntimeCompatibility::EnshroudedClient::RuntimePatch* profile{};
    std::uintptr_t target{};
    std::uint32_t function_begin_rva{}, function_end_rva{};
    std::size_t target_offset{};
    std::size_t signature_matches{};
    std::vector<std::uint8_t> original, replacement;
    void* trampoline{};
    std::string status{"unresolved"};
    bool enabled{};
};
std::vector<PatchState> patches;

std::optional<std::pair<std::uint32_t, std::uint32_t>> function_range(
    std::uintptr_t image_base, std::uintptr_t target) {
    if (!image_base || target < image_base) return std::nullopt;
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(image_base);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew <= 0) return std::nullopt;
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(image_base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE || nt->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC)
        return std::nullopt;
    const auto directory = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_EXCEPTION];
    if (!directory.VirtualAddress || directory.Size < 12 ||
        directory.VirtualAddress > nt->OptionalHeader.SizeOfImage ||
        directory.Size > nt->OptionalHeader.SizeOfImage - directory.VirtualAddress)
        return std::nullopt;
    struct FunctionEntry { std::uint32_t begin, end, unwind; };
    const auto* entries = reinterpret_cast<const FunctionEntry*>(image_base + directory.VirtualAddress);
    const auto count = directory.Size / sizeof(FunctionEntry);
    const auto target_rva = target - image_base;
    for (std::size_t index = 0; index < count; ++index) {
        if (entries[index].begin <= target_rva && target_rva < entries[index].end)
            return std::pair{entries[index].begin, entries[index].end};
    }
    return std::nullopt;
}

bool resolve(PatchState& state, std::uint8_t* base) {
    const auto& spec = *state.profile;
    const auto parsed_signature = EngineHooks::ParseExecutableSignature(spec.signature);
    if (parsed_signature.size() < spec.overwrite || (spec.kind == "detour" && spec.overwrite < 5)) {
        state.status = "invalid-signature-or-overwrite-length";
        return false;
    }
    const auto match = EngineHooks::FindExecutableSignature(base, spec.signature);
    state.signature_matches = match.count;
    if (match.count != 1 || !match.address) {
        state.status = match.count == 0 ? "signature-missing" : "signature-ambiguous";
        return false;
    }
    state.target = match.address;
    const auto image_base = reinterpret_cast<std::uintptr_t>(base);
    const auto actual_range = function_range(image_base, state.target);
    if (!actual_range) {
        state.status = "function-boundary-unavailable";
        return false;
    }
    state.function_begin_rva = actual_range->first;
    state.function_end_rva = actual_range->second;
    state.target_offset = static_cast<std::size_t>((state.target - image_base) - actual_range->first);
    const bool association_matches = state.function_begin_rva == spec.function_begin_rva &&
        state.function_end_rva == spec.function_end_rva && state.target_offset == spec.target_offset;
    if (!association_matches && KfcRuntimeCompatibility::EnshroudedClient::exact_build_match) {
        state.status = "function-association-mismatch";
        return false;
    }
    if (!association_matches) state.status = "verified-build-diff-function-association";
    state.original.assign(reinterpret_cast<const std::uint8_t*>(match.address),
        reinterpret_cast<const std::uint8_t*>(match.address) + spec.overwrite);
    if (spec.kind == "bytes") {
        state.replacement = spec.payload;
        if (association_matches) state.status = "verified";
        return true;
    }

    std::vector<std::uint8_t> trampoline = spec.payload;
    for (const auto& reference : spec.inline_references) {
        const auto displacement = static_cast<std::int32_t>(reference.data) - static_cast<std::int32_t>(reference.next);
        std::memcpy(trampoline.data() + reference.displacement, &displacement, sizeof(displacement));
    }
    std::int32_t return_delta{};
    const auto trampoline_size = trampoline.size() + 16;
    auto* allocation = static_cast<std::uint8_t*>(EngineHooks::AllocateNearCode(state.target, trampoline_size));
    if (!allocation) { state.status = "trampoline-allocation-failed"; return false; }
    state.trampoline = allocation;
    if (!EngineHooks::EncodeRelativeJump(reinterpret_cast<std::uintptr_t>(allocation) + spec.return_rel32_offset - 1,
        state.target + spec.overwrite, return_delta)) { state.status = "return-jump-out-of-range"; return false; }
    std::memcpy(trampoline.data() + spec.return_rel32_offset, &return_delta, sizeof(return_delta));
    std::memcpy(allocation, trampoline.data(), trampoline.size());
    FlushInstructionCache(GetCurrentProcess(), allocation, trampoline.size());
    state.replacement.assign(spec.overwrite, 0x90);
    state.replacement[0] = 0xe9;
    std::int32_t detour{};
    if (!EngineHooks::EncodeRelativeJump(state.target, reinterpret_cast<std::uintptr_t>(allocation), detour)) { state.status = "detour-out-of-range"; return false; }
    std::memcpy(state.replacement.data() + 1, &detour, sizeof(detour));
    if (association_matches) state.status = "verified";
    return true;
}
}

namespace PatchRuntime {
bool Initialize() {
    patches.clear();
    const auto base = reinterpret_cast<std::uint8_t*>(GetModuleHandleW(nullptr));
    if (!base) return false;
    for (const auto& profile : KfcRuntimeCompatibility::EnshroudedClient::runtime_patches) {
        PatchState state{};
        state.profile = &profile;
        resolve(state, base);
        patches.push_back(std::move(state));
    }
    return !patches.empty();
}

void Shutdown() {
    for (auto& patch : patches) {
        if (patch.enabled && EngineHooks::ReplaceExecutableBytes(patch.target, patch.replacement.data(), patch.original.data(), patch.original.size()))
            patch.enabled = false;
    }
}

bool Available(const char* name) {
    if (!name) return false;
    for (const auto& patch : patches)
        if (patch.profile->name == name) return patch.target && patch.status.starts_with("verified");
    return false;
}

bool AnyAvailable() {
    for (const auto& patch : patches)
        if (patch.target && patch.status.starts_with("verified")) return true;
    return false;
}

bool SetEnabled(const char* name, bool enabled, std::uint32_t* outcome) {
    if (outcome) *outcome = 1;
    if (!name || !outcome) return false;
    for (auto& patch : patches) {
        if (patch.profile->name != name) continue;
        if (!patch.target || !patch.status.starts_with("verified")) return false;
        if (patch.enabled == enabled) { *outcome = 0; return true; }
        const auto* expected = enabled ? patch.original.data() : patch.replacement.data();
        const auto* replacement = enabled ? patch.replacement.data() : patch.original.data();
        if (!EngineHooks::ReplaceExecutableBytes(patch.target, expected, replacement, patch.original.size())) {
            patch.status = "live-bytes-changed-or-thread-unsafe";
            return false;
        }
        patch.enabled = enabled;
        *outcome = 0;
        return true;
    }
    return false;
}

std::string Diagnostics() {
    nlohmann::json rows = nlohmann::json::array();
    const auto image_base = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    for (const auto& patch : patches) {
        const auto& spec = *patch.profile;
        const bool association_verified = patch.target && patch.function_begin_rva == spec.function_begin_rva &&
            patch.function_end_rva == spec.function_end_rva && patch.target_offset == spec.target_offset;
        rows.push_back({{"name", patch.profile->name}, {"status", patch.status},
            {"resolved", patch.target != 0}, {"enabled", patch.enabled},
            {"signatureMatches", patch.signature_matches},
            {"targetRva", patch.target && patch.target >= image_base ?
                nlohmann::json(patch.target - image_base) : nlohmann::json(nullptr)},
            {"functionRva", patch.target ? nlohmann::json{{"begin", patch.function_begin_rva},
                {"end", patch.function_end_rva}} : nlohmann::json(nullptr)},
            {"functionAssociation", {{"id", spec.function_id},
                {"expectedBeginRva", spec.function_begin_rva}, {"expectedEndRva", spec.function_end_rva},
                {"expectedTargetOffset", spec.target_offset}, {"actualBeginRva", patch.target ?
                    nlohmann::json(patch.function_begin_rva) : nlohmann::json(nullptr)},
                {"actualEndRva", patch.target ? nlohmann::json(patch.function_end_rva) : nlohmann::json(nullptr)},
                {"actualTargetOffset", patch.target ? nlohmann::json(patch.target_offset) : nlohmann::json(nullptr)},
                {"verified", association_verified}, {"buildDiffWarning", !association_verified && patch.status.starts_with("verified")}}},
            {"overwriteBytes", patch.original.size()}});
    }
    return rows.dump();
}
}

extern "C" KFC_RUNTIME_API bool KFC_RUNTIME_CALL KfcRuntimePatchAvailable(const char* name) {
    if (name && std::strcmp(name, "runtime.gameplay.patch") == 0) return PatchRuntime::AnyAvailable();
    return PatchRuntime::Available(name);
}

extern "C" KFC_RUNTIME_API bool KFC_RUNTIME_CALL KfcRuntimePatchSetEnabled(const char* name, bool enabled, std::uint32_t* outcome) {
    return PatchRuntime::SetEnabled(name, enabled, outcome);
}
