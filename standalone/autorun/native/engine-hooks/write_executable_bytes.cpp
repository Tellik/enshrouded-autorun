#include "engine-hooks/write_executable_bytes.h"
#include <windows.h>
#include <tlhelp32.h>
#include <cstring>
#include <vector>

namespace EngineHooks {
bool WriteExecutableBytes(std::uintptr_t address, const void* bytes, std::size_t size, const void* expected) {
    if (size > 32) return false;
    // Prepare handles before suspending: never allocate while an engine thread
    // might hold the process heap lock. Refuse to patch an occupied instruction.
    std::vector<HANDLE> threads;
    HANDLE snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if (snapshot == INVALID_HANDLE_VALUE) return false;
    THREADENTRY32 entry{sizeof(entry)};
    if (Thread32First(snapshot, &entry)) do {
        if (entry.th32OwnerProcessID != GetCurrentProcessId() || entry.th32ThreadID == GetCurrentThreadId()) continue;
        HANDLE thread = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION, FALSE, entry.th32ThreadID);
        if (!thread) {
            CloseHandle(snapshot);
            for (auto opened : threads) CloseHandle(opened);
            return false;
        }
        threads.push_back(thread);
    } while (Thread32Next(snapshot, &entry));
    CloseHandle(snapshot);
    std::size_t suspended{};
    bool safe = true;
    for (auto thread : threads) {
        if (SuspendThread(thread) == DWORD(-1)) { safe = false; break; }
        ++suspended;
        CONTEXT context{};
        context.ContextFlags = CONTEXT_CONTROL;
        if (!GetThreadContext(thread, &context) || (context.Rip >= address && context.Rip < address + size)) {
            safe = false; break;
        }
    }
    DWORD previous{};
    bool result{};
    if (safe && (!expected || std::memcmp(reinterpret_cast<void*>(address), expected, size) == 0) &&
        VirtualProtect(reinterpret_cast<void*>(address), size, PAGE_EXECUTE_READWRITE, &previous)) {
        std::uint8_t original[32]{};
        std::memcpy(original, reinterpret_cast<void*>(address), size);
        std::memcpy(reinterpret_cast<void*>(address), bytes, size);
        FlushInstructionCache(GetCurrentProcess(), reinterpret_cast<void*>(address), size);
        DWORD ignored{};
        result = VirtualProtect(reinterpret_cast<void*>(address), size, previous, &ignored) != FALSE;
        if (!result) {
            std::memcpy(reinterpret_cast<void*>(address), original, size);
            FlushInstructionCache(GetCurrentProcess(), reinterpret_cast<void*>(address), size);
            VirtualProtect(reinterpret_cast<void*>(address), size, previous, &ignored);
        }
    }
    for (std::size_t index = 0; index < suspended; ++index) ResumeThread(threads[index]);
    for (auto thread : threads) CloseHandle(thread);
    return result;
}

bool ReplaceExecutableBytes(std::uintptr_t address, const void* expected, const void* replacement, std::size_t size) {
    if (!address || !expected || !replacement || !size || size > 32) return false;
    for (unsigned attempt = 0; attempt < 8; ++attempt) {
        if (WriteExecutableBytes(address, replacement, size, expected)) return true;
        // A different patch or changed build is not a transient thread race.
        if (std::memcmp(reinterpret_cast<void*>(address), expected, size)) return false;
        Sleep(2);
    }
    return false;
}
}
