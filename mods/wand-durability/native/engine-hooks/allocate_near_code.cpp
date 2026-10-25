#include "engine-hooks/allocate_near_code.h"
#include <windows.h>
#include <algorithm>

namespace EngineHooks {
void* AllocateNearCode(std::uintptr_t target, std::size_t size) {
    SYSTEM_INFO system{};
    GetSystemInfo(&system);
    const auto granularity = static_cast<std::uintptr_t>(system.dwAllocationGranularity);
    constexpr std::uintptr_t range = 0x7fff0000;
    const auto minimum = (std::max)(reinterpret_cast<std::uintptr_t>(system.lpMinimumApplicationAddress),
        target > range ? target - range : 0);
    const auto maximum = (std::min)(reinterpret_cast<std::uintptr_t>(system.lpMaximumApplicationAddress),
        target <= UINTPTR_MAX - range ? target + range : target);
    for (auto cursor = minimum; cursor < maximum;) {
        MEMORY_BASIC_INFORMATION memory{};
        if (!VirtualQuery(reinterpret_cast<void*>(cursor), &memory, sizeof(memory))) break;
        const auto region = reinterpret_cast<std::uintptr_t>(memory.BaseAddress);
        const auto end = region + memory.RegionSize;
        if (memory.State == MEM_FREE) {
            const auto candidate = (region + granularity - 1) & ~(granularity - 1);
            if (candidate >= minimum && candidate + size <= end && candidate + size <= maximum) {
                if (auto allocation = VirtualAlloc(reinterpret_cast<void*>(candidate), size,
                    MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE)) return allocation;
            }
        }
        if (end <= cursor) break;
        cursor = end;
    }
    return nullptr;
}
}
