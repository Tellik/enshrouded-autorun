#pragma once
#include "abi/runtime.h"
#include <cstdint>
namespace EcsRuntime {
bool ResolvePropEntityId(std::uint32_t entity_id, KfcRuntimePropRecord* prop);
}
