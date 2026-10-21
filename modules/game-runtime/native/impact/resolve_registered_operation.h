#pragma once
#include <cstdint>

namespace NamedServiceImplementation { struct BindingContract; }
namespace EngineContext { struct EcsView; }

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

// The caller prepares the impact::ImpactSystem binding before acquiring the
// original Scene/lifetime gates and validates owner under those gates. This
// bounded reader acquires no mutex, allocates no memory, queries no session and
// invokes no engine callback. Borrowed registry data never leaves this call.
// Failure leaves output unchanged. A resolved address grants no invocation ABI.
ResolveResult ResolveRegisteredOperationPinned(const NamedServiceImplementation::BindingContract& binding,
    const EngineContext::EcsView& owner, const char* exact_name, RegisteredOperation& output);

// Resolves the exact original operation name in impact::ImpactSystem. Existing
// profile-backed DI/Scene ownership remains required. All Session/Scene/caller
// locks are released before this returns. No engine callback is invoked here.
// Failure leaves output unchanged.
ResolveResult ResolveRegisteredOperation(std::uint64_t expected_session,
    const char* exact_name, RegisteredOperation& output);
}
