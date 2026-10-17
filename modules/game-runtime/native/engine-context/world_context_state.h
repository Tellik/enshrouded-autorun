#pragma once

#include "engine-context/world_context.h"
#include "ecs/component_access.h"
#include "ecs/snapshot_entity_ids.h"
#include "ecs/read_building_prop.h"
#include "profiles/game_profile.h"
#include "game-thread/run_game_thread_operations.h"
#include <windows.h>
#include <algorithm>
#include <array>
#include <cstring>
#include <memory>
#include <mutex>
#include <atomic>
#include <cmath>
#include <condition_variable>
#include <cstdint>
#include <vector>
#include <unordered_set>
#include <nlohmann/json.hpp>

// Private implementation contract. State is defined once, in the owning unit.

namespace WorldRuntime::Implementation {
extern std::atomic<std::uintptr_t> observed_actor_world;

extern std::atomic<std::uint64_t> observed_actor_world_ms;

extern std::atomic<std::uintptr_t> preferred_voxel_world;

extern std::atomic<std::uint64_t> actor_placement_hook_hits, actor_placement_request_hits;

extern std::atomic<std::uint64_t> building_dispatch_hook_hits, building_dispatch_request_hits;

extern std::atomic<std::uint64_t> entity_dispatch_completions;

extern std::atomic<std::uint64_t> prop_update_request_hits;

extern std::atomic<std::uintptr_t> observed_execution_view;

extern std::atomic<std::uint64_t> prop_update_hook_hits, cursor_hook_hits, context_resets;

extern std::atomic<std::uintptr_t> local_execution_root;

extern std::atomic<std::uintptr_t> client_read_world, client_read_manager;

extern std::atomic<std::uint64_t> client_read_ms, client_read_generation;

enum class ActorProbe { Never, MissingFrame, MissingLayout, MissingService, InvalidWorld, Valid };

extern std::atomic<ActorProbe> actor_probe;

bool read_memory(std::uintptr_t address, void* output, std::size_t size);

const KfcRuntimeCompatibility::EnshroudedClient::RuntimeOperation* find_operation(const char* name);

bool valid_world_context(std::uintptr_t world);

bool resolve_voxel_world(std::uintptr_t& world);

bool resolve_client_read_world(std::uintptr_t& world);

void observe_client_read_world(void* cursor_frame, void* execution_view);

void observe_actor_world(void* actor_frame);

bool actor_frame_matches_active_world(void* actor_frame);

}
