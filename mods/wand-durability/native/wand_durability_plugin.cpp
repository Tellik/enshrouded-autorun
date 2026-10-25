#include "attack_cost.h"
#include "durability_hook_code.h"
#include "engine-hooks/allocate_near_code.h"
#include "engine-hooks/encode_relative_jump.h"
#include "engine-hooks/scan_executable_signature.h"
#include "engine-hooks/write_executable_bytes.h"
#include <nlohmann/json.hpp>
#include <windows.h>
#include <bcrypt.h>
#include <array>
#include <atomic>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <string>
#include <string_view>
#include <vector>

namespace {
constexpr std::uint8_t Original[]{0x41,0xb8,0xff,0xff,0xff,0xff};
struct ImageProfile {
    const char* role;
    const char* sha256;
    std::uint32_t timestamp, size;
    std::uintptr_t hook_rva, skip_rva;
    const char* signature;
};
constexpr ImageProfile Profiles[]{
    {"client", "d64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882",
        1792507660u, 44163072u, 0x225450, 0x225461,
        "41 B8 FF FF FF FF 48 8B CB 44 8B 08 E8 DF A1 18 00 48 8D 3D"},
    {"dedicated-server", "d7baa768eb00aa895d9f5a1cd8747ccf66858c7b542f50983092b2e3261588ee",
        1792507668u, 31264768u, 0x430a0, 0x430b1,
        "41 B8 FF FF FF FF 48 8B CB 44 8B 08 E8 2F 85 11 00 48 8D 3D"},
};
HMODULE plugin_module{};
std::atomic<std::uint64_t> session{}, deadline{}, observed{}, charged{}, suppressed{}, unrecognized{}, invalid{};
std::atomic<std::uint32_t> last_sequence{};
std::atomic_flag costs_busy = ATOMIC_FLAG_INIT;
WandDurability::AttackCosts costs;

bool read_attack(const void* actor, const void* durability, std::uint32_t engine_actor,
                 WandDurability::Attack& attack) {
    if (!actor || !durability) return false;
    // These reads occur synchronously inside the original system, with its
    // original component binding and lifetime. No engine pointer is followed
    // later. Bad/changed data preserves the game's original charge.
    __try {
        const auto* bytes = static_cast<const std::uint8_t*>(actor);
        attack.actor = reinterpret_cast<std::uintptr_t>(actor);
        attack.durability = reinterpret_cast<std::uintptr_t>(durability);
        std::memcpy(&attack.sequence, bytes, 4);
        std::memcpy(&attack.trigger_time, bytes + 8, 8);
        std::memcpy(&attack.actor_entity, bytes + 0x10, 4);
        std::memcpy(&attack.call_index, bytes + 0x14, 1);
        std::memcpy(&attack.weapon_entity, bytes + 0x3c, 4);
        return engine_actor != 0 && attack.actor_entity == engine_actor;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool __cdecl suppress_cost(const void* actor, const void* durability, std::uint32_t engine_actor) {
    observed.fetch_add(1, std::memory_order_relaxed);
    const auto current_session = session.load(std::memory_order_acquire);
    if (!current_session || GetTickCount64() > deadline.load(std::memory_order_acquire)) return false;
    WandDurability::Attack attack{};
    attack.session = current_session;
    const bool valid = read_attack(actor, durability, engine_actor, attack);
    last_sequence.store(attack.sequence, std::memory_order_relaxed);
    if (!valid) { invalid.fetch_add(1, std::memory_order_relaxed); return false; }
    if (!WandDurability::is_wand_sequence(attack.sequence)) {
        unrecognized.fetch_add(1, std::memory_order_relaxed);
        return false;
    }
    // Never block the game thread or call the runtime's locking APIs here.
    if (costs_busy.test_and_set(std::memory_order_acquire)) return false;
    if (session.load(std::memory_order_acquire) != current_session) {
        costs_busy.clear(std::memory_order_release);
        return false;
    }
    const bool skip = costs.suppress(attack);
    costs_busy.clear(std::memory_order_release);
    (skip ? suppressed : charged).fetch_add(1, std::memory_order_relaxed);
    return skip;
}

bool game_image_matches(const ImageProfile*& profile) {
    profile = nullptr;
    auto* base = reinterpret_cast<std::uint8_t*>(GetModuleHandleW(nullptr));
    if (!base) return false;
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(base);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew <= 0) return false;
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE) return false;
    for (const auto& candidate : Profiles)
        if (nt->FileHeader.TimeDateStamp == candidate.timestamp &&
            nt->OptionalHeader.SizeOfImage == candidate.size) profile = &candidate;
    if (!profile) return false;
    wchar_t path[32768]{};
    const auto length = GetModuleFileNameW(nullptr, path, static_cast<DWORD>(std::size(path)));
    if (!length || length >= std::size(path)) return false;
    HANDLE file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE,
        nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (file == INVALID_HANDLE_VALUE) return false;
    BCRYPT_ALG_HANDLE algorithm{};
    BCRYPT_HASH_HANDLE hash{};
    bool okay = BCryptOpenAlgorithmProvider(&algorithm, BCRYPT_SHA256_ALGORITHM, nullptr, 0) >= 0;
    DWORD object_length{}, result_length{};
    if (okay) okay = BCryptGetProperty(algorithm, BCRYPT_OBJECT_LENGTH,
        reinterpret_cast<PUCHAR>(&object_length), sizeof(object_length), &result_length, 0) >= 0;
    std::vector<UCHAR> object(object_length);
    if (okay) okay = BCryptCreateHash(algorithm, &hash, object.data(), object_length, nullptr, 0, 0) >= 0;
    std::array<UCHAR, 8192> buffer{};
    while (okay) {
        DWORD count{};
        if (!ReadFile(file, buffer.data(), static_cast<DWORD>(buffer.size()), &count, nullptr)) {
            okay = false;
            break;
        }
        if (!count) break;
        okay = BCryptHashData(hash, buffer.data(), count, 0) >= 0;
    }
    std::array<UCHAR, 32> digest{};
    if (okay) okay = BCryptFinishHash(hash, digest.data(), static_cast<ULONG>(digest.size()), 0) >= 0;
    if (hash) BCryptDestroyHash(hash);
    if (algorithm) BCryptCloseAlgorithmProvider(algorithm, 0);
    CloseHandle(file);
    if (!okay) return false;
    constexpr char hex[] = "0123456789abcdef";
    char text[65]{};
    for (std::size_t i = 0; i < digest.size(); ++i) {
        text[i * 2] = hex[digest[i] >> 4];
        text[i * 2 + 1] = hex[digest[i] & 15];
    }
    if (std::string_view(text, 64) != profile->sha256) {
        profile = nullptr; return false;
    }
    return true;
}



const ImageProfile* install_hook() {
    auto* base = reinterpret_cast<std::uint8_t*>(GetModuleHandleW(nullptr));
    const ImageProfile* profile{};
    if (!base || !game_image_matches(profile)) return nullptr;
    const auto target = reinterpret_cast<std::uintptr_t>(base) + profile->hook_rva;
    const auto match = EngineHooks::FindExecutableSignature(base, profile->signature, 2);
    if (match.count != 1 || match.address != target ||
        std::memcmp(reinterpret_cast<void*>(target), Original, sizeof(Original))) return nullptr;
    HMODULE pinned{};
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
        reinterpret_cast<LPCWSTR>(&install_hook), &pinned)) return nullptr;
    auto hook = WandDurability::make_hook(reinterpret_cast<std::uintptr_t>(&suppress_cost));
    auto& code = hook.bytes;
    auto* payload = static_cast<std::uint8_t*>(EngineHooks::AllocateNearCode(target, code.size()));
    if (!payload) return nullptr;
    std::int32_t charge{}, skip{}, detour{};
    const auto address = reinterpret_cast<std::uintptr_t>(payload);
    if (!EngineHooks::EncodeRelativeJump(address + hook.charge_jump, target + sizeof(Original), charge) ||
        !EngineHooks::EncodeRelativeJump(address + hook.skip_jump, reinterpret_cast<std::uintptr_t>(base) + profile->skip_rva, skip) ||
        !EngineHooks::EncodeRelativeJump(target, address, detour)) {
        VirtualFree(payload, 0, MEM_RELEASE); return nullptr;
    }
    std::memcpy(code.data() + hook.charge_jump + 1, &charge, 4);
    std::memcpy(code.data() + hook.skip_jump + 1, &skip, 4);
    std::memcpy(payload, code.data(), code.size());
    DWORD previous{};
    if (!VirtualProtect(payload, code.size(), PAGE_EXECUTE_READ, &previous)) {
        VirtualFree(payload, 0, MEM_RELEASE); return nullptr;
    }
    FlushInstructionCache(GetCurrentProcess(), payload, code.size());
    std::uint8_t replacement[sizeof(Original)]{0xe9,0,0,0,0,0x90};
    std::memcpy(replacement + 1, &detour, 4);
    if (!EngineHooks::ReplaceExecutableBytes(target, Original, replacement, sizeof(Original))) {
        VirtualFree(payload, 0, MEM_RELEASE); return nullptr;
    }
    return profile;
}

std::filesystem::path plugin_dir() {
    wchar_t path[32768]{};
    auto length = GetModuleFileNameW(plugin_module, path, static_cast<DWORD>(std::size(path)));
    return length && length < std::size(path) ? std::filesystem::path(path).parent_path().parent_path()
                                           : std::filesystem::path{};
}

void log(const std::filesystem::path& root, const std::string& message) {
    std::ofstream stream(root / "wand-durability.log", std::ios::app);
    stream << GetTickCount64() << ' ' << message << '\n';
}

bool enabled(const std::filesystem::path& root) {
    try {
        auto path = root / "extended.mod.json";
        if (std::filesystem::file_size(path) > 65536) return false;
        std::ifstream stream(path);
        auto settings = nlohmann::json::parse(stream, nullptr, false);
        return settings.is_object() && settings.contains("enabled") &&
            settings["enabled"].is_boolean() && settings["enabled"].get<bool>();
    } catch (...) { return false; }
}

DWORD WINAPI worker(void*) {
    const auto root = plugin_dir();
    if (root.empty()) return 0;
    log(root, "Wand Durability 0.2.0 experimental plugin starting");
    auto* runtime = GetModuleHandleW(L"kfc-runtime.dll");
    for (unsigned i = 0; !runtime && i < 300; ++i) {
        Sleep(100); runtime = GetModuleHandleW(L"kfc-runtime.dll");
    }
    if (!runtime) { log(root, "Stock ShroudForge runtime unavailable; inactive"); return 0; }
    using AbiFn = std::uint32_t (__cdecl*)();
    using SessionFn = std::uint64_t (__cdecl*)();
    const auto abi = reinterpret_cast<AbiFn>(GetProcAddress(runtime, "KfcRuntimeAbi"));
    const auto world_session = reinterpret_cast<SessionFn>(GetProcAddress(runtime, "KfcRuntimeWorldSessionId"));
    const auto world_kind = reinterpret_cast<AbiFn>(GetProcAddress(runtime, "KfcRuntimeWorldContextKind"));
    if (!abi || abi() != 12 || !world_session || !world_kind) {
        log(root, "Required stock ABI-12 world identity API unavailable; inactive"); return 0;
    }
    const auto* profile = install_hook();
    if (!profile) { log(root, "Unsupported image or changed hook bytes; inactive"); return 0; }
    log(root, std::string("Exact-build ") + profile->role +
        " durability hook installed; waiting for enabled authoritative world");
    std::uint64_t last_session{}, last_report{}, last_observed{};
    std::uint32_t last_kind = ~0u;
    bool was_enabled = false;
    while (true) {
        const auto now = GetTickCount64();
        const bool active = enabled(root);
        const auto kind = world_kind();
        const auto current = active && kind == 1 ? world_session() : 0;
        // A setting transition resets the journal even if the world stays.
        if (active != was_enabled || current != last_session) {
            session.store(0, std::memory_order_release);
            while (costs_busy.test_and_set(std::memory_order_acquire)) Sleep(1);
            costs = {};
            costs_busy.clear(std::memory_order_release);
        }
        deadline.store(now + 1000, std::memory_order_release);
        session.store(current, std::memory_order_release);
        if (active != was_enabled || current != last_session || kind != last_kind) {
            log(root, "enabled=" + std::to_string(active) + " session=" + std::to_string(current) +
                " world_kind=" + std::to_string(kind) +
                (kind == 2 ? " (remote world: install on the host/server)" : ""));
            was_enabled = active; last_session = current; last_kind = kind;
        }
        const auto hits = observed.load();
        if (now >= last_report + 2000 && hits != last_observed) {
            log(root, "projectile_cost_calls=" + std::to_string(hits) + " wand_charges=" +
                std::to_string(charged.load()) + " bonus_costs_skipped=" + std::to_string(suppressed.load()) +
                " other_sequences=" + std::to_string(unrecognized.load()) +
                " invalid_state=" + std::to_string(invalid.load()) +
                " last_sequence=" + std::to_string(last_sequence.load()));
            last_report = now; last_observed = hits;
        }
        Sleep(250);
    }
}
}

BOOL APIENTRY DllMain(HMODULE module, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        plugin_module = module;
        DisableThreadLibraryCalls(module);
        if (HANDLE thread = CreateThread(nullptr, 0, worker, nullptr, 0, nullptr)) CloseHandle(thread);
    } else if (reason == DLL_PROCESS_DETACH) session.store(0, std::memory_order_release);
    return TRUE;
}
