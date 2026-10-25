#pragma once
#include <cstddef>
#include <cstdint>
namespace EngineHooks {
void* AllocateNearCode(std::uintptr_t target, std::size_t size);
}
