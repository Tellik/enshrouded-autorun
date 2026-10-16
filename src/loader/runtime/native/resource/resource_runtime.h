#pragma once

#include "runtime.h"
#include <cstddef>
#include <cstdint>
#include <string>

namespace ResourceRuntime {
bool Initialize();
void Shutdown();
bool Ready();
bool CanWrite();
bool ResourcesByType(std::uint32_t type_hash, KfcRuntimeResourceId* resources,
    std::size_t capacity, std::size_t* count);
bool Read(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    unsigned char* buffer, std::size_t capacity, std::size_t* actual);
// One aligned 32-bit scalar, with bitwise expected-value comparison. This is
// not a transaction over the resource or over multiple fields.
std::uint32_t CompareExchangeScalar(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width);
// expected is the complete resource_size snapshot, desired is one width=4
// scalar. Both the full input snapshot and descriptor/payload ownership are
// validated while the original Scene and resource lifetime guards remain held.
std::uint32_t CompareExchangeScalarForSession(const KfcRuntimeResourceId& resource, std::uint32_t static_hash,
    std::size_t resource_size, std::size_t offset, const void* expected, const void* desired, std::size_t width,
    std::uint64_t expected_session);
std::string Status();
}
