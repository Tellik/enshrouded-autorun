#include "player-input/autorun_movement.h"
#include "player-input/world_cursor_state.h"
#include "engine-hooks/allocate_near_code.h"
#include "engine-hooks/encode_relative_jump.h"
#include "engine-hooks/scan_executable_signature.h"
#include "engine-hooks/write_executable_bytes.h"
#include "profiles/game_profile.h"
#include "abi/runtime_api.h"
#include <windows.h>
#include <atomic>
#include <cmath>
#include <cstring>
#include <iterator>
#include <vector>

namespace {
// Exact Enshrouded 26082781 ClientPlayerInputData layout, corroborated by
// live main-client snapshots. This hook is never enabled on another image.
constexpr std::uintptr_t InputHookRva = 0x243b5c;
constexpr std::size_t MovementX = 0x2b8, MovementY = 0x2bc;
constexpr std::uint8_t Original[]{0x0f, 0xb6, 0xc8, 0x84, 0xc0};
constexpr auto Signature = "0F B6 C8 84 C0 74 5A 83 F9 01 74 20";
constexpr auto ImageSha256 = "d64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882";
constexpr bool MovementWritesEnabled = true;
constexpr std::uint64_t RequestLifetimeMs = 500;
constexpr std::uint64_t ActivityTimeoutMs = 1000;
std::atomic<bool> installed{}, active{};
std::atomic<std::uint64_t> request_session{}, request_deadline{}, last_hook_hit{}, last_local_hit{};
std::atomic<std::uint64_t> hook_hits{}, local_hits{}, writes{};
std::atomic<float> observed_x{}, observed_y{};

bool elapsed_more_than(std::uint64_t now, std::uint64_t previous, std::uint64_t limit){
    return now >= previous && now - previous > limit;
}

bool foreground_game(){
    const auto window = GetForegroundWindow();
    if (!window) return false;
    DWORD process{};
    GetWindowThreadProcessId(window, &process);
    return process == GetCurrentProcessId();
}

bool read_axes(void* input, float& x, float& y){
    __try {
        auto* bytes = static_cast<std::uint8_t*>(input);
        std::memcpy(&x, bytes + MovementX, sizeof(x));
        std::memcpy(&y, bytes + MovementY, sizeof(y));
        if (!std::isfinite(x) || !std::isfinite(y) ||
            x < -1.01f || x > 1.01f || y < -1.01f || y > 1.01f) return false;
        return true;
    } __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool apply_forward(void* input, float x, float y){
    __try {
        auto* bytes = static_cast<std::uint8_t*>(input);
        // Add forward to ordinary input, then keep the stick at unit magnitude.
        // This preserves left/right steering and the game's default speed.
        const float forward = y + 1.0f;
        const float magnitude = std::sqrt(x * x + forward * forward);
        if (!std::isfinite(magnitude) || magnitude < 0.5f) return false;
        x /= magnitude;
        y = forward / magnitude;
        std::memcpy(bytes + MovementX, &x, sizeof(x));
        std::memcpy(bytes + MovementY, &y, sizeof(y));
        return true;
    } __except(EXCEPTION_EXECUTE_HANDLER) { return false; }
}

void __cdecl on_movement_input(void* input){
    if (!installed.load(std::memory_order_acquire) || !active.load(std::memory_order_acquire)) return;
    const auto now = GetTickCount64();
    if (now > request_deadline.load(std::memory_order_acquire) ||
        elapsed_more_than(now, last_local_hit.load(std::memory_order_acquire), ActivityTimeoutMs)) {
        active.store(false, std::memory_order_release);
        return;
    }
    hook_hits.fetch_add(1, std::memory_order_relaxed);
    last_hook_hit.store(now, std::memory_order_release);
    const auto session = request_session.load(std::memory_order_acquire);
    if (!session || KfcRuntimeWorldSessionId() != session || !foreground_game()) {
        active.store(false, std::memory_order_release);
        return;
    }
    if (!WorldRuntime::Implementation::MatchesPublishedLocalInput(input, session)) return;
    local_hits.fetch_add(1, std::memory_order_relaxed);
    last_local_hit.store(now, std::memory_order_release);
    float x{}, y{};
    if (!read_axes(input, x, y)) { active.store(false, std::memory_order_release); return; }
    observed_x.store(x, std::memory_order_relaxed);
    observed_y.store(y, std::memory_order_relaxed);
    if (y < -0.1f) { active.store(false, std::memory_order_release); return; }
    if constexpr (MovementWritesEnabled) {
        if (apply_forward(input, x, y)) writes.fetch_add(1, std::memory_order_relaxed);
        else active.store(false, std::memory_order_release);
    }
}

std::vector<std::uint8_t> callback_code(){
    // Same register/XMM and stack preservation contract as the existing
    // ShroudForge cursor hook. RSI is nonvolatile and holds ClientPlayerInput.
    std::vector<std::uint8_t> code{0x9c,0x50,0x51,0x52,0x41,0x50,0x41,0x51,0x41,0x52,0x41,0x53,
        0x48,0x81,0xec,0x80,0,0,0};
    for (unsigned index = 0; index < 6; ++index) {
        const std::uint8_t store[]{0xf3,0x0f,0x7f,static_cast<std::uint8_t>(0x44+index*8),0x24,
            static_cast<std::uint8_t>(0x20+index*16)};
        code.insert(code.end(), std::begin(store), std::end(store));
    }
    code.insert(code.end(), {0xfc,0x48,0x89,0xf1,0x48,0xb8}); // cld; rcx=rsi; mov rax,callback
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
}

namespace AutoRunMovement {
bool Install(){
    if (installed.load(std::memory_order_acquire)) return true;
    using namespace KfcRuntimeCompatibility::EnshroudedClient;
    if (image_target != "enshrouded.exe" || image_sha256 != ImageSha256 ||
        image_timestamp != 1792507660u || image_size != 44163072u) return false;
    auto* base = reinterpret_cast<std::uint8_t*>(GetModuleHandleW(nullptr));
    if (!base) return false;
    const auto match = EngineHooks::FindExecutableSignature(base, Signature, 2);
    const auto target = reinterpret_cast<std::uintptr_t>(base) + InputHookRva;
    if (match.count != 1 || match.address != target ||
        std::memcmp(reinterpret_cast<void*>(target), Original, sizeof(Original))) return false;
    HMODULE pinned{};
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
        reinterpret_cast<LPCWSTR>(&Install), &pinned)) return false;
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

void Stop(){ active.store(false, std::memory_order_release); }
bool Available(){ return installed.load(std::memory_order_acquire); }
bool Set(bool enabled, std::uint64_t expected_session){
    if (!enabled) { Stop(); return true; }
    if (!Available() || !expected_session || KfcRuntimeWorldSessionId() != expected_session ||
        !foreground_game()) return false;
    const auto now = GetTickCount64();
    if (active.load(std::memory_order_acquire) &&
        request_session.load(std::memory_order_acquire) == expected_session &&
        now <= request_deadline.load(std::memory_order_acquire)) {
        request_deadline.store(now + RequestLifetimeMs, std::memory_order_release);
        return true;
    }
    Stop();
    if (!WorldRuntime::Implementation::LocalCursorInputAvailable()) return false;
    hook_hits.store(0, std::memory_order_relaxed);
    local_hits.store(0, std::memory_order_relaxed);
    writes.store(0, std::memory_order_relaxed);
    observed_x.store(0.0f, std::memory_order_relaxed);
    observed_y.store(0.0f, std::memory_order_relaxed);
    request_session.store(expected_session, std::memory_order_release);
    last_hook_hit.store(now, std::memory_order_release);
    last_local_hit.store(now, std::memory_order_release);
    request_deadline.store(now + RequestLifetimeMs, std::memory_order_release);
    active.store(true, std::memory_order_release);
    return true;
}
void Status(std::uint64_t& hits, std::uint64_t& local, std::uint64_t& applied, bool& enabled,
            float& last_x, float& last_y){
    const auto now = GetTickCount64();
    if (active.load(std::memory_order_acquire) &&
        (!foreground_game() || KfcRuntimeWorldSessionId() != request_session.load(std::memory_order_acquire) ||
         now > request_deadline.load(std::memory_order_acquire) ||
         elapsed_more_than(now, last_hook_hit.load(std::memory_order_acquire), ActivityTimeoutMs) ||
         elapsed_more_than(now, last_local_hit.load(std::memory_order_acquire), ActivityTimeoutMs))) Stop();
    hits = hook_hits.load(std::memory_order_relaxed);
    local = local_hits.load(std::memory_order_relaxed);
    applied = writes.load(std::memory_order_relaxed);
    enabled = active.load(std::memory_order_acquire);
    last_x = observed_x.load(std::memory_order_relaxed);
    last_y = observed_y.load(std::memory_order_relaxed);
}
}

extern "C" {
KFC_RUNTIME_API bool KFC_RUNTIME_CALL KfcRuntimeWorldAutoRunAvailable(){
    return AutoRunMovement::Available();
}
KFC_RUNTIME_API bool KFC_RUNTIME_CALL KfcRuntimeWorldAutoRunSet(bool enabled, std::uint64_t expected_session){
    return AutoRunMovement::Set(enabled, expected_session);
}
KFC_RUNTIME_API bool KFC_RUNTIME_CALL KfcRuntimeWorldAutoRunStatus(
    std::uint64_t* hits, std::uint64_t* local, std::uint64_t* applied, bool* enabled,
    float* last_x, float* last_y){
    if (!hits || !local || !applied || !enabled || !last_x || !last_y) return false;
    AutoRunMovement::Status(*hits, *local, *applied, *enabled, *last_x, *last_y);
    return true;
}
}
