#include "impact/resolve_registered_operation.h"
#include "impact/owned_operation_arguments.h"
#include "abi/runtime_api.h"
#include "ecs/entity_session.h"
#include <windows.h>
#include <array>
#include <cstring>

namespace {
using ImpactRuntime::OwnedVm::Variable;
using ImpactRuntime::OwnedVm::Arguments;
using ImpactRuntime::OwnedVm::Context;

bool readable_output(std::uint8_t* output){
    __try { const volatile auto current = *output; (void)current; return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}

bool publish(std::uint8_t* output, std::uint8_t value){
    __try { *output = value; return true; }
    __except (EXCEPTION_EXECUTE_HANDLER) { return false; }
}
}

extern "C" std::uint32_t __cdecl KfcRuntimeImpactCompareEntityIds(
    std::uint64_t expected_session, std::uint32_t left, std::uint32_t right, std::uint8_t* equal){
    if (!expected_session || !equal || !readable_output(equal)) return 2;
    try {
        ImpactRuntime::RegisteredOperation operation;
        const auto resolution = ImpactRuntime::ResolveRegisteredOperation(expected_session, "isSameEntity", operation);
        if (resolution != ImpactRuntime::ResolveResult::Found) return static_cast<std::uint32_t>(resolution);
        if (operation.arguments != 2) return 5;

        // This one typed callback reads only these owned primitive buffers and
        // writes its 12-byte result. No game/Query/ECS pointer enters its context.
        std::array<Variable,3> variables{};
        variables[1].size = variables[2].size = sizeof(std::uint32_t);
        variables[2].offset = sizeof(std::uint32_t);
        const auto expected_variables = variables;
        std::array<std::uint32_t,2> data{left,right}, indices{1,2};
        const auto expected_data = data;
        const auto expected_indices = indices;
        Arguments arguments{indices.data(), indices.size()};
        Context context{variables.data(), variables.size(), data.data(), sizeof(data), {}, nullptr};
        const auto expected_arguments = arguments;
        const auto expected_context = context;
        alignas(8) std::array<std::uint8_t,44> storage;
        storage.fill(0xa5);
        auto* result = storage.data() + 16;
        void* returned{};
        if (KfcRuntimeWorldSessionId() != expected_session) return 4;
        if (!ImpactRuntime::OwnedVm::Invoke(operation.callback, result, &arguments, &context, returned)) return 6;
        std::uint32_t detail{};
        std::memcpy(&detail, result + 4, sizeof(detail));
        if (returned != result || result[0] || detail || result[8] > 1 ||
            result[8] != static_cast<std::uint8_t>(left == right) ||
            variables != expected_variables || data != expected_data || indices != expected_indices ||
            arguments != expected_arguments || context != expected_context) return 7;
        for (std::size_t i = 0; i < storage.size(); ++i) {
            const bool written = i == 16 || (i >= 20 && i < 24) || i == 24;
            if (!written && storage[i] != 0xa5) return 7;
        }
        // Reacquire only the session epoch after the callback has returned. No
        // original Scene gate or callback executes while publishing the host result.
        EcsRuntime::SessionGuard session;
        if (!session.TryAcquire(expected_session)) return 4;
        return publish(equal, result[8]) ? 1 : 2;
    } catch (...) { return 3; }
}
