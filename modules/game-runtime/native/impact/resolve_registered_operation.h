#pragma once
#include <cstdint>

namespace ImpactRuntime {
enum class ResolveResult : std::uint32_t {
    Found = 1,
    RegistryUnavailable = 3,
    SessionChanged = 4,
    UnsupportedContract = 5,
};

// Only a main-image code pointer and its declared arity leave the borrowed
// registry snapshot. This grants no generic right to invoke the operation.
struct RegisteredOperation {
    std::uintptr_t callback{};
    std::uint8_t arguments{};
};

// Resolves the exact original operation name in impact::ImpactSystem. Existing
// profile-backed DI/Scene ownership remains required. All Session/Scene/caller
// locks are released before this returns. No engine callback is invoked here.
// Failure leaves output unchanged.
ResolveResult ResolveRegisteredOperation(std::uint64_t expected_session,
    const char* exact_name, RegisteredOperation& output);
}
