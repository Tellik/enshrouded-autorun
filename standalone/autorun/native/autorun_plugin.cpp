#include "engine-hooks/allocate_near_code.h"
#include "engine-hooks/encode_relative_jump.h"
#include "engine-hooks/scan_executable_signature.h"
#include "engine-hooks/write_executable_bytes.h"
#include <nlohmann/json.hpp>
#include <windows.h>
#include <bcrypt.h>
#include <algorithm>
#include <array>
#include <atomic>
#include <charconv>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <iterator>
#include <mutex>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace {
// Use only ShroudForge's existing package DLL loader and stable ABI-12 world
// identity functions. Movement injection is entirely owned by this plugin.
constexpr std::uintptr_t InputHookRva = 0x243b5c;
constexpr std::size_t MovementX = 0x2b8, MovementY = 0x2bc;
constexpr std::uint8_t Original[]{0x0f, 0xb6, 0xc8, 0x84, 0xc0};
constexpr auto Signature = "0F B6 C8 84 C0 74 5A 83 F9 01 74 20";
constexpr auto ImageSha256 = "d64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882";
constexpr std::uint32_t ImageTimestamp = 1792507660u;
constexpr std::uint32_t ImageSize = 44163072u;
constexpr std::uint64_t InputAgeMs = 1000, MinimumObservationMs = 500;
constexpr std::uint64_t MinimumObservationHits = 30;
constexpr std::uint64_t WorkerDeadlineMs = 250;

using SessionIdFn = std::uint64_t (__cdecl*)();
using ContextActiveFn = bool (__cdecl*)();
using AbiFn = std::uint32_t (__cdecl*)();
std::atomic<SessionIdFn> session_id{};
std::atomic<ContextActiveFn> context_active{};
std::atomic<bool> installed{}, active{};
std::atomic<std::uint64_t> active_session{}, deadline_ms{}, last_hook_ms{};
std::atomic<std::uint64_t> hook_hits{}, local_hits{}, writes{};
std::atomic<int> stop_reason{};
HMODULE plugin_module{};

enum StopReason { None, Toggle, Backward, Focus, Session, Expired, InputChanged, SettingsChanged };
const char* reason_name(int value) {
    switch (value) {
    case Toggle: return "toggle key";
    case Backward: return "backward input";
    case Focus: return "game lost focus";
    case Session: return "world session changed";
    case Expired: return "native deadline expired";
    case InputChanged: return "observed input changed";
    case SettingsChanged: return "settings changed or mod disabled";
    default: return "unknown";
    }
}

void stop(StopReason reason) {
    if (active.exchange(false, std::memory_order_acq_rel))
        stop_reason.store(reason, std::memory_order_release);
}

struct Observation {
    std::uint64_t session{}, first_ms{}, last_ms{}, hits{};
    const void* input{};
    bool conflicting{};
};
std::mutex observation_mutex;
Observation observation;

bool foreground_game() {
    const auto window = GetForegroundWindow();
    if (!window) return false;
    DWORD process{};
    GetWindowThreadProcessId(window, &process);
    return process == GetCurrentProcessId();
}

void observe_input(const void* input, std::uint64_t session, std::uint64_t now) {
    if (!input || !session) return;
    std::unique_lock lock(observation_mutex, std::try_to_lock);
    if (!lock.owns_lock()) return;
    if (observation.session != session) observation = {};
    observation.session = session;
    if (!observation.input) {
        observation.input = input;
        observation.first_ms = now;
    } else if (observation.input != input) {
        observation.conflicting = true;
        stop(InputChanged);
    }
    if (observation.input == input) {
        ++observation.hits;
        observation.last_ms = now;
    }
}

const void* stable_input(std::uint64_t session, std::uint64_t now) {
    std::unique_lock lock(observation_mutex, std::try_to_lock);
    if (!lock.owns_lock() || observation.session != session || !observation.input ||
        observation.conflicting || observation.hits < MinimumObservationHits ||
        now < observation.first_ms || now - observation.first_ms < MinimumObservationMs ||
        now < observation.last_ms || now - observation.last_ms > InputAgeMs) return nullptr;
    return observation.input;
}

bool read_axes(void* input, float& x, float& y) {
    __try {
        auto* bytes = static_cast<std::uint8_t*>(input);
        std::memcpy(&x, bytes + MovementX, sizeof(x));
        std::memcpy(&y, bytes + MovementY, sizeof(y));
        return std::isfinite(x) && std::isfinite(y) &&
            x >= -1.01f && x <= 1.01f && y >= -1.01f && y <= 1.01f;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool apply_forward(void* input, float x, float y) {
    __try {
        auto* bytes = static_cast<std::uint8_t*>(input);
        const float forward = y + 1.0f;
        const float magnitude = std::sqrt(x * x + forward * forward);
        if (!std::isfinite(magnitude) || magnitude < 0.5f) return false;
        x /= magnitude;
        y = forward / magnitude;
        std::memcpy(bytes + MovementX, &x, sizeof(x));
        std::memcpy(bytes + MovementY, &y, sizeof(y));
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

void __cdecl on_movement_input(void* input) {
    if (!installed.load(std::memory_order_acquire) || !input) return;
    const auto session_fn = session_id.load(std::memory_order_acquire);
    const auto context_fn = context_active.load(std::memory_order_acquire);
    if (!session_fn || !context_fn) return;
    const auto now = GetTickCount64();
    const auto session = session_fn();
    if (session && context_fn() && foreground_game()) observe_input(input, session, now);
    if (!active.load(std::memory_order_acquire)) return;
    hook_hits.fetch_add(1, std::memory_order_relaxed);
    last_hook_ms.store(now, std::memory_order_release);
    if (!session || session != active_session.load(std::memory_order_acquire) || !context_fn()) {
        stop(Session);
        return;
    }
    if (!foreground_game()) { stop(Focus); return; }
    if (now > deadline_ms.load(std::memory_order_acquire)) { stop(Expired); return; }
    if (stable_input(session, now) != input) return;
    local_hits.fetch_add(1, std::memory_order_relaxed);
    float x{}, y{};
    if (!read_axes(input, x, y)) { stop(InputChanged); return; }
    if (y < -0.1f) { stop(Backward); return; }
    if (apply_forward(input, x, y)) writes.fetch_add(1, std::memory_order_relaxed);
    else stop(InputChanged);
}

std::vector<std::uint8_t> callback_code() {
    // Same preserved-register contract as the proven ShroudForge client hook.
    // RSI holds the local producer's ClientPlayerInput on this exact image.
    std::vector<std::uint8_t> code{0x9c,0x50,0x51,0x52,0x41,0x50,0x41,0x51,0x41,0x52,0x41,0x53,
        0x48,0x81,0xec,0x80,0,0,0};
    for (unsigned index = 0; index < 6; ++index) {
        const std::uint8_t store[]{0xf3,0x0f,0x7f,static_cast<std::uint8_t>(0x44+index*8),0x24,
            static_cast<std::uint8_t>(0x20+index*16)};
        code.insert(code.end(), std::begin(store), std::end(store));
    }
    code.insert(code.end(), {0xfc,0x48,0x89,0xf1,0x48,0xb8});
    const auto address = reinterpret_cast<std::uintptr_t>(&on_movement_input);
    for (unsigned index = 0; index < 8; ++index)
        code.push_back(static_cast<std::uint8_t>(address >> (index * 8)));
    code.insert(code.end(), {0xff,0xd0});
    for (unsigned index = 0; index < 6; ++index) {
        const std::uint8_t restore[]{0xf3,0x0f,0x6f,static_cast<std::uint8_t>(0x44+index*8),0x24,
            static_cast<std::uint8_t>(0x20+index*16)};
        code.insert(code.end(), std::begin(restore), std::end(restore));
    }
    code.insert(code.end(), {0x48,0x81,0xc4,0x80,0,0,0,0x41,0x5b,0x41,0x5a,0x41,0x59,0x41,0x58,
        0x5a,0x59,0x58,0x9d});
    return code;
}

bool game_image_matches() {
    auto* base = reinterpret_cast<std::uint8_t*>(GetModuleHandleW(nullptr));
    if (!base) return false;
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(base);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew <= 0) return false;
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE || nt->FileHeader.TimeDateStamp != ImageTimestamp ||
        nt->OptionalHeader.SizeOfImage != ImageSize) return false;
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
    return std::string_view(text, 64) == ImageSha256;
}

bool install_hook() {
    auto* base = reinterpret_cast<std::uint8_t*>(GetModuleHandleW(nullptr));
    if (!base || !game_image_matches()) return false;
    const auto match = EngineHooks::FindExecutableSignature(base, Signature, 2);
    const auto target = reinterpret_cast<std::uintptr_t>(base) + InputHookRva;
    if (match.count != 1 || match.address != target ||
        std::memcmp(reinterpret_cast<void*>(target), Original, sizeof(Original))) return false;
    HMODULE pinned{};
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
        reinterpret_cast<LPCWSTR>(&install_hook), &pinned)) return false;
    auto code = callback_code();
    code.insert(code.end(), std::begin(Original), std::end(Original));
    const auto jump_offset = code.size();
    code.insert(code.end(), {0xe9,0,0,0,0});
    auto* payload = static_cast<std::uint8_t*>(EngineHooks::AllocateNearCode(target, code.size()));
    if (!payload) return false;
    std::int32_t back{}, detour{};
    if (!EngineHooks::EncodeRelativeJump(reinterpret_cast<std::uintptr_t>(payload) + jump_offset,
            target + sizeof(Original), back) ||
        !EngineHooks::EncodeRelativeJump(target, reinterpret_cast<std::uintptr_t>(payload), detour)) {
        VirtualFree(payload, 0, MEM_RELEASE);
        return false;
    }
    std::memcpy(code.data() + jump_offset + 1, &back, sizeof(back));
    std::memcpy(payload, code.data(), code.size());
    FlushInstructionCache(GetCurrentProcess(), payload, code.size());
    std::uint8_t replacement[sizeof(Original)]{0xe9};
    std::memcpy(replacement + 1, &detour, sizeof(detour));
    if (!EngineHooks::ReplaceExecutableBytes(target, Original, replacement, sizeof(Original))) {
        VirtualFree(payload, 0, MEM_RELEASE);
        return false;
    }
    installed.store(true, std::memory_order_release);
    return true;
}

struct Chord { int key{}; unsigned modifiers{}; bool operator==(const Chord&) const = default; };
struct Binding {
    std::array<Chord, 2> chords{};
    std::size_t count{};
    bool operator==(const Binding&) const = default;
};
struct Settings {
    Binding toggle, cancel;
    bool enabled{};
    bool operator==(const Settings&) const = default;
};

int key_from_scancode(unsigned code) {
    if (code >= 4 && code <= 29) return 'A' + static_cast<int>(code - 4);
    if (code >= 30 && code <= 38) return '1' + static_cast<int>(code - 30);
    if (code == 39) return '0';
    if (code >= 58 && code <= 69) return VK_F1 + static_cast<int>(code - 58);
    switch (code) {
    case 40: return VK_RETURN;
    case 41: return VK_ESCAPE;
    case 42: return VK_BACK;
    case 43: return VK_TAB;
    case 44: return VK_SPACE;
    case 73: return VK_INSERT;
    case 74: return VK_HOME;
    case 75: return VK_PRIOR;
    case 76: return VK_DELETE;
    case 77: return VK_END;
    case 78: return VK_NEXT;
    case 79: return VK_RIGHT;
    case 80: return VK_LEFT;
    case 81: return VK_DOWN;
    case 82: return VK_UP;
    default: return 0;
    }
}

std::optional<Chord> parse_chord(std::string_view text) {
    Chord result{};
    for (const auto [prefix, bit] : {std::pair{"Ctrl+", 1u}, {"Alt+", 2u},
             {"Shift+", 4u}, {"Meta+", 8u}}) {
        if (text.starts_with(prefix)) {
            result.modifiers |= bit;
            text.remove_prefix(std::strlen(prefix));
        }
    }
    bool mouse{};
    if (text.starts_with("SDL:")) text.remove_prefix(4);
    else if (text.starts_with("Mouse:")) { text.remove_prefix(6); mouse = true; }
    else return std::nullopt;
    unsigned number{};
    const auto parsed = std::from_chars(text.data(), text.data() + text.size(), number);
    if (parsed.ec != std::errc{} || parsed.ptr != text.data() + text.size()) return std::nullopt;
    if (!mouse) result.key = key_from_scancode(number);
    else {
        switch (number) {
        case 1: result.key = VK_LBUTTON; break;
        case 2: result.key = VK_MBUTTON; break;
        case 3: result.key = VK_RBUTTON; break;
        case 4: result.key = VK_XBUTTON1; break;
        case 5: result.key = VK_XBUTTON2; break;
        default: break;
        }
    }
    if (!result.key) return std::nullopt;
    return result;
}

std::optional<Binding> parse_binding(const std::string& value) {
    Binding result{};
    if (value.empty()) return result;
    std::string_view text(value);
    while (true) {
        if (result.count == result.chords.size()) return std::nullopt;
        const auto divider = text.find('|');
        const auto chord = parse_chord(text.substr(0, divider));
        if (!chord) return std::nullopt;
        result.chords[result.count++] = *chord;
        if (divider == std::string_view::npos) break;
        text.remove_prefix(divider + 1);
    }
    return result;
}

std::optional<Settings> read_settings(const std::filesystem::path& path) {
    std::ifstream file(path, std::ios::binary);
    if (!file) return std::nullopt;
    const auto json = nlohmann::json::parse(file, nullptr, false);
    if (json.is_discarded() || !json.is_object()) return std::nullopt;
    try {
        const auto& entries = json.at("settings");
        const auto toggle = parse_binding(entries.at("toggleAutorun").at("value").get<std::string>());
        const auto cancel = parse_binding(entries.at("cancelAutorun").at("value").get<std::string>());
        if (!toggle || !cancel || *toggle == *cancel) return std::nullopt;
        return Settings{*toggle, *cancel, json.at("enabled").get<bool>()};
    } catch (const nlohmann::json::exception&) { return std::nullopt; }
}

bool pressed(int key) { return (GetAsyncKeyState(key) & 0x8000) != 0; }
bool chord_down(const Chord& chord) {
    const unsigned held = (pressed(VK_CONTROL) ? 1u : 0u) |
        (pressed(VK_MENU) ? 2u : 0u) | (pressed(VK_SHIFT) ? 4u : 0u) |
        ((pressed(VK_LWIN) || pressed(VK_RWIN)) ? 8u : 0u);
    return held == chord.modifiers && pressed(chord.key);
}
bool binding_down(const Binding& binding) {
    for (std::size_t i = 0; i < binding.count; ++i)
        if (chord_down(binding.chords[i])) return true;
    return false;
}

std::filesystem::path plugin_dir() {
    wchar_t path[32768]{};
    const auto length = GetModuleFileNameW(plugin_module, path, static_cast<DWORD>(std::size(path)));
    if (!length || length >= std::size(path)) return {};
    return std::filesystem::path(path).parent_path().parent_path();
}

void log(const std::filesystem::path& root, const std::string& message) {
    OutputDebugStringA(("[autorun] " + message + "\n").c_str());
    if (root.empty()) return;
    std::ofstream stream(root / "autorun-plugin.log", std::ios::app);
    stream << GetTickCount64() << " " << message << '\n';
}

DWORD WINAPI worker(void*) {
    const auto root = plugin_dir();
    if (root.empty()) return 0;
    log(root, "Standalone plugin starting");
    auto* kfc = GetModuleHandleW(L"kfc-runtime.dll");
    for (unsigned i = 0; !kfc && i < 300; ++i) {
        Sleep(100);
        kfc = GetModuleHandleW(L"kfc-runtime.dll");
    }
    if (!kfc) { log(root, "Stock KFC runtime unavailable; plugin is inactive"); return 0; }
    const auto abi = reinterpret_cast<AbiFn>(GetProcAddress(kfc, "KfcRuntimeAbi"));
    const auto session = reinterpret_cast<SessionIdFn>(GetProcAddress(kfc, "KfcRuntimeWorldSessionId"));
    const auto context = reinterpret_cast<ContextActiveFn>(GetProcAddress(kfc, "KfcRuntimeWorldContextActive"));
    if (!abi || abi() != 12 || !session || !context) {
        log(root, "Required stock ABI-12 world API unavailable; plugin is inactive");
        return 0;
    }
    session_id.store(session, std::memory_order_release);
    context_active.store(context, std::memory_order_release);
    if (!install_hook()) { log(root, "Game image or hook did not match the supported build; plugin is inactive"); return 0; }
    log(root, "Exact-build input hook installed");
    std::optional<Settings> settings;
    std::uint64_t config_at{}, report_at{}, previous_session{};
    bool previous_toggle{}, previous_cancel{}, reported_invalid{};
    unsigned reports{};
    while (true) {
        const auto now = GetTickCount64();
        if (now >= config_at) {
            config_at = now + 250;
            const auto next = read_settings(root / "extended.mod.json");
            if (!next) {
                stop(SettingsChanged);
                if (!reported_invalid) log(root, "Invalid or unsupported hotkey settings; movement disabled");
                reported_invalid = true;
            } else {
                reported_invalid = false;
                if (!settings || *settings != *next) {
                    stop(SettingsChanged);
                    previous_toggle = binding_down(next->toggle);
                    previous_cancel = binding_down(next->cancel);
                    log(root, "Settings loaded or changed");
                }
            }
            settings = next;
        }
        const auto current = session();
        if (current != previous_session) {
            stop(Session);
            previous_session = current;
            if (settings) {
                previous_toggle = binding_down(settings->toggle);
                previous_cancel = binding_down(settings->cancel);
            }
        }
        const bool foreground = foreground_game();
        if (!foreground || !current || !context()) stop(foreground ? Session : Focus);
        if (active.load(std::memory_order_acquire) &&
            (now > last_hook_ms.load(std::memory_order_acquire) + InputAgeMs ||
             stable_input(current, now) == nullptr)) stop(Expired);
        if (settings && settings->enabled && foreground && current && context()) {
            const bool cancel = binding_down(settings->cancel);
            const bool toggle = binding_down(settings->toggle);
            if (cancel) stop(Backward);
            else if (toggle && !previous_toggle) {
                if (active.load(std::memory_order_acquire)) stop(Toggle);
                else if (stable_input(current, now)) {
                    hook_hits.store(0, std::memory_order_relaxed);
                    local_hits.store(0, std::memory_order_relaxed);
                    writes.store(0, std::memory_order_relaxed);
                    active_session.store(current, std::memory_order_release);
                    last_hook_ms.store(now, std::memory_order_release);
                    deadline_ms.store(now + WorkerDeadlineMs, std::memory_order_release);
                    active.store(true, std::memory_order_release);
                    reports = 0;
                    log(root, "ON");
                } else log(root, "Cannot start: stable local input not observed");
            }
            previous_toggle = toggle;
            previous_cancel = cancel;
        } else if (settings) {
            previous_toggle = binding_down(settings->toggle);
            previous_cancel = binding_down(settings->cancel);
        }
        if (active.load(std::memory_order_acquire))
            deadline_ms.store(now + WorkerDeadlineMs, std::memory_order_release);
        if (const auto reason = stop_reason.exchange(None, std::memory_order_acq_rel); reason) {
            log(root, std::string("OFF: ") + reason_name(reason) +
                " hits=" + std::to_string(hook_hits.load()) +
                " local=" + std::to_string(local_hits.load()) +
                " writes=" + std::to_string(writes.load()));
        }
        if (active.load(std::memory_order_acquire) && now >= report_at && reports < 30) {
            report_at = now + 1000;
            ++reports;
            log(root, "Native counts: hits=" + std::to_string(hook_hits.load()) +
                " local=" + std::to_string(local_hits.load()) +
                " writes=" + std::to_string(writes.load()));
        }
        Sleep(16);
    }
}
}

BOOL APIENTRY DllMain(HMODULE module, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) {
        plugin_module = module;
        DisableThreadLibraryCalls(module);
        if (HANDLE thread = CreateThread(nullptr, 0, worker, nullptr, 0, nullptr)) CloseHandle(thread);
    } else if (reason == DLL_PROCESS_DETACH) {
        active.store(false, std::memory_order_release);
    }
    return TRUE;
}
