#pragma once
#include "ecs/entity_session.h"
#include <cstddef>
#include <cstdint>
#include <string>
namespace WorldRuntime {
void ResetContext();
void OnPropUpdate(void* execution_view, void* actor_frame);
void OnActorPlacement(void* execution_view, void* actor_frame);
void OnBuildingDispatch(void* placement_context);
bool OperationAvailable(const char* name);
bool ActiveContextAvailable();
std::uint32_t ContextKind();
std::string ContextDiagnostics();
std::string ContextStatus();
bool EntityContextReady();
std::string EntityHookStatus();
}
