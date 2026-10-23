#pragma once
#include <string>

namespace NativeMainMenu {
// Process-owned native UI binding. Menu discovery does not depend on an ECS profile.
void Initialize() noexcept;
void Shutdown() noexcept;
std::string Status();
}
