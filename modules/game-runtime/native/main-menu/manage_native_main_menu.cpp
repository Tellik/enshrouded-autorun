#include "main-menu/native_main_menu.h"
#include "main-menu/resolve_main_menu.h"
#include "engine-hooks/allocate_near_code.h"
#include "engine-hooks/encode_relative_jump.h"
#include "engine-hooks/write_executable_bytes.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <atomic>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <mutex>
#include <vector>
#include <nlohmann/json.hpp>

namespace NativeMainMenu {
namespace {
// Enshrouded's immediate menu-entry argument: UTF-8 span, its engine identity,
// then eight option bytes. Preserve every option from the Settings entry.
struct MenuDescriptor {
    const char* text;
    std::uint64_t length;
    std::uint64_t identity;
    std::array<std::uint8_t, 8> options;
};
static_assert(sizeof(MenuDescriptor) == 32);
static_assert(offsetof(MenuDescriptor, options) == 24);
using DrawEntry = bool(__fastcall*)(void*, const MenuDescriptor*);

constexpr std::uint64_t label_identity(const char* text) {
    // FNV-1a is the engine's widget-label identity, never an executable gate.
    std::uint64_t value = 14695981039346656037ull;
    while (*text) { value ^= static_cast<unsigned char>(*text++); value *= 1099511628211ull; }
    return value;
}

struct InstalledCall {
    MenuCall call;
    std::array<std::uint8_t, 5> replacement;
};
std::mutex lifecycle_mutex;
std::atomic<DrawEntry> original_entry{};
std::atomic<bool> accepting{};
std::atomic<std::uint64_t> draws{}, clicks{};
std::vector<InstalledCall> installed;
std::size_t menu_calls{};
std::string state{"not-initialized"}, reason;

// Published callbacks, thunks and event handles stay alive until process exit.
// A UI thread can retain a return address even after its CALL is restored.
HANDLE enabled_event{}, open_event{};

bool enabled() noexcept {
    return enabled_event && WaitForSingleObject(enabled_event, 0) == WAIT_OBJECT_0;
}

bool __fastcall draw_with_mods(void* selection, const MenuDescriptor* descriptor) {
    const auto draw = original_entry.load(std::memory_order_acquire);
    if (!draw) return false;
    if (accepting.load(std::memory_order_acquire) && enabled()) {
        auto mods = *descriptor;
        mods.text = "Mods";
        mods.length = 4;
        mods.identity = label_identity("Mods");
        draws.fetch_add(1, std::memory_order_relaxed);
        // Stay on the engine's UI thread for rendering. The external Modloader
        // owns its event loop, just as it does for the configured shortcut and
        // shroudforge.ui.toggle_modloader(). Never enter Lua from this thread.
        if (draw(selection, &mods) && accepting.load(std::memory_order_acquire)) {
            if (SetEvent(open_event)) clicks.fetch_add(1, std::memory_order_relaxed);
        }
    }
    return draw(selection, descriptor);
}

bool ensure_events() {
    const auto name = [](const wchar_t* kind) {
        return std::wstring(L"Local\\ShroudForge.ModMenu.") + kind + L"." +
            std::to_wstring(GetCurrentProcessId());
    };
    if (!enabled_event) enabled_event = CreateEventW(nullptr, TRUE, FALSE, name(L"Enabled").c_str());
    if (!open_event) open_event = CreateEventW(nullptr, FALSE, FALSE, name(L"Open").c_str());
    return enabled_event && open_event;
}

bool restore_calls() {
    bool restored = true;
    for (auto iterator = installed.rbegin(); iterator != installed.rend(); ++iterator) {
        if (!iterator->call.address) continue;
        if (EngineHooks::ReplaceExecutableBytes(iterator->call.address,
                iterator->replacement.data(), iterator->call.original.data(), iterator->call.original.size()))
            iterator->call.address = 0;
        else restored = false;
    }
    std::erase_if(installed, [](const InstalledCall& call) { return !call.call.address; });
    return restored;
}
}

void Initialize() noexcept {
    try {
        std::lock_guard lock(lifecycle_mutex);
        if (accepting.load(std::memory_order_acquire)) return;
        if (!installed.empty()) {
            state = "unavailable"; reason = "previous-menu-calls-could-not-be-restored"; return;
        }
        state = "unavailable";
        reason.clear();
        MenuBinding binding;
        if (!ResolveMainMenu(binding)) { reason = binding.error; return; }
        if (!binding.original_entry || binding.insertion_calls.empty()) {
            reason = "menu-binding-incomplete"; return;
        }
        menu_calls = binding.menu_calls;
        if (!ensure_events()) { reason = "menu-event-creation-failed"; return; }
        HMODULE pinned{};
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
                reinterpret_cast<LPCWSTR>(&draw_with_mods), &pinned)) {
            reason = "menu-provider-pin-failed"; return;
        }
        // One absolute indirect jump is reachable from all selected native CALLs.
        std::array<std::uint8_t, 14> thunk{0xff, 0x25, 0, 0, 0, 0};
        const auto callback = reinterpret_cast<std::uintptr_t>(&draw_with_mods);
        std::memcpy(thunk.data() + 6, &callback, sizeof(callback));
        auto allocation = EngineHooks::AllocateNearCode(binding.insertion_calls.front().address, thunk.size());
        if (!allocation) { reason = "menu-near-allocation-failed"; return; }
        std::memcpy(allocation, thunk.data(), thunk.size());
        DWORD previous{};
        if (!VirtualProtect(allocation, thunk.size(), PAGE_EXECUTE_READ, &previous)) {
            VirtualFree(allocation, 0, MEM_RELEASE); reason = "menu-thunk-protection-failed"; return;
        }
        FlushInstructionCache(GetCurrentProcess(), allocation, thunk.size());
        std::vector<InstalledCall> prepared;
        for (const auto& call : binding.insertion_calls) {
            InstalledCall next{call, {0xe8}};
            std::int32_t relative{};
            if (!EngineHooks::EncodeRelativeJump(call.address,
                    reinterpret_cast<std::uintptr_t>(allocation), relative)) {
                VirtualFree(allocation, 0, MEM_RELEASE); reason = "menu-call-out-of-range"; return;
            }
            std::memcpy(next.replacement.data() + 1, &relative, sizeof(relative));
            prepared.push_back(next);
        }
        if (!OriginalCode::EvidenceUnchanged(binding.evidence)) {
            VirtualFree(allocation, 0, MEM_RELEASE); reason = "menu-code-changed-before-install"; return;
        }
        installed.reserve(prepared.size());
        original_entry.store(reinterpret_cast<DrawEntry>(binding.original_entry), std::memory_order_release);
        bool published = false;
        for (const auto& call : prepared) {
            if (!EngineHooks::ReplaceExecutableBytes(call.call.address, call.call.original.data(),
                    call.replacement.data(), call.replacement.size())) {
                restore_calls();
                if (!published) VirtualFree(allocation, 0, MEM_RELEASE);
                reason = "menu-call-installation-rejected"; return;
            }
            published = true;
            installed.push_back(call);
        }
        state = "installed";
        accepting.store(true, std::memory_order_release);
    } catch (...) {
        accepting.store(false, std::memory_order_release);
        // If bookkeeping allocation failed, keep the published callback mapped
        // and forwarding to the original engine function.
        try { std::lock_guard lock(lifecycle_mutex); restore_calls(); state = "unavailable"; reason = "menu-initialization-exception"; }
        catch (...) {}
    }
}

void Shutdown() noexcept {
    accepting.store(false, std::memory_order_release);
    try {
        std::lock_guard lock(lifecycle_mutex);
        const bool restored = restore_calls();
        state = restored ? "stopped" : "stopped-with-forwarding-callback";
        if (!restored) reason = "menu-call-restoration-rejected";
    } catch (...) {}
}

std::string Status() {
    std::lock_guard lock(lifecycle_mutex);
    return nlohmann::json({{"state", state}, {"reason", reason}, {"enabled", enabled()},
        {"menuCalls", menu_calls}, {"insertionCalls", installed.size()},
        {"draws", draws.load(std::memory_order_relaxed)},
        {"clicks", clicks.load(std::memory_order_relaxed)}}).dump();
}
}
