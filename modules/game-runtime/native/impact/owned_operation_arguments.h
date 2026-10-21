#pragma once
#include <windows.h>
#include <array>
#include <cstddef>
#include <cstdint>

// Owned VM arguments for the separately admitted original Impact callbacks.
// These layouts do not authorize arbitrary operations or borrowed Query frames.
namespace ImpactRuntime::OwnedVm {
struct Variable {
    std::uint32_t name{}, config{}, type{};
    std::uint16_t size{}, offset{};
    std::uint64_t debug_name{};
    bool operator==(const Variable&) const = default;
};
static_assert(sizeof(Variable) == 24 && offsetof(Variable, offset) == 14);

struct Arguments {
    const std::uint32_t* indices{};
    std::uint64_t count{};
    bool operator==(const Arguments&) const = default;
};
struct Context {
    const Variable* variables{};
    std::uint64_t variable_count{};
    const std::uint32_t* data{};
    std::uint64_t data_extent{};
    std::array<std::uintptr_t,2> reserved{};
    const void* execution_context{};
    bool operator==(const Context&) const = default;
};
static_assert(sizeof(Context) == 56 && offsetof(Context, data) == 16 &&
    offsetof(Context, execution_context) == 48);

inline bool Invoke(std::uintptr_t callback, void* result, const Arguments* arguments,
    const Context* context, void*& returned) {
    using Callback = void*(__fastcall*)(void*, const void*, const void*);
    __try {
        returned = reinterpret_cast<Callback>(callback)(result, arguments, context);
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
}
