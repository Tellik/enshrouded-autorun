#include "ecs/ecs_state.h"



using namespace EcsImplementation;

static_assert(sizeof(KfcRuntimePropRecord) == 88);

static_assert(sizeof(KfcRuntimePropRecipe) == 32);

static_assert(offsetof(KfcRuntimePropRecord, position) == 8);

static_assert(offsetof(KfcRuntimePropRecord, orientation) == 32);

static_assert(offsetof(KfcRuntimePropRecord, scale) == 48);

static_assert(offsetof(KfcRuntimePropRecord, template_uuid) == 64);

static_assert(offsetof(KfcRuntimePropRecord, entity_id) == 80);

namespace EcsImplementation {
OperationCounters operation_counters;

std::atomic<std::uint64_t> active_query_cursor{}, active_query_total{}, active_query_matches{};

bool compact_entity_pointers(const std::uintptr_t* slots, std::size_t count,
                            const std::uint64_t* occupancy, std::size_t occupancy_count,
                            std::vector<EntityPointer>& pointers){
    if ((!slots && count) || (occupancy && occupancy_count != (count + 63) / 64) ||
        (!occupancy && occupancy_count)) return false;
    pointers.clear();
    if (occupancy) {
        std::size_t occupied{};
        for (std::size_t word = 0; word < occupancy_count; ++word)
            occupied += std::popcount(occupancy[word]);
        pointers.reserve(occupied);
    } else {
        pointers.reserve(count);
    }
    for (std::size_t index = 0; index < count; ++index) {
        const bool active = !occupancy ||
            (occupancy[index / 64] & (std::uint64_t{1} << (index % 64)));
        if (active && slots[index]) pointers.push_back({slots[index], index});
    }
    return true;
}

std::mutex state_mutex;

// Entity operations and lifecycle invalidation share this outer lock. Discovery
// scans run outside it. Lock order is operation, discovery, then state.
std::recursive_mutex operation_mutex;

std::unordered_map<std::string, ComponentType> types;

std::unordered_map<std::string, std::uint32_t> configured_types;

ComponentRegistry::Snapshot component_registry;

bool component_registry_ready{};

bool component_registry_world_bound{};

bool component_registry_seen{};

std::string component_registry_error{"awaiting entity manager"};

std::uint64_t component_registry_last_read{};

std::uintptr_t component_registry_manager{};

std::string component_registry_json;

std::unordered_map<std::uint32_t, std::unordered_set<std::uint16_t>> observed_indices_by_size;

std::unordered_set<std::uintptr_t> discovered_layouts;

std::unordered_map<std::uintptr_t, std::vector<ComponentSlot>> discovered_layout_components;

std::unordered_map<std::string, TemplateLayoutSample> template_layout_samples;

std::uint64_t template_layout_samples_dropped{};

std::vector<std::uintptr_t> discovery_entities;

std::size_t discovery_cursor{};

std::uintptr_t discovery_table{};

std::uint64_t discovery_last_restart{};

std::uint64_t discovery_scanned_entities{};

std::uint64_t discovery_layout_count{};

std::uint64_t discovery_restarts{};

bool discovery_complete{};

std::mutex discovery_mutex;

ComponentRegistry::Discovery registry_discovery;

ResolvedLayout live_layout{};

bool layout_ready{};

bool read_only_binding{};

EngineContext::EcsView read_only_view;

std::uintptr_t live_table{};

std::unordered_map<std::uint32_t, HandleRecord> handles;

std::unordered_map<std::uint64_t, std::uint32_t> reverse_handles;

std::unordered_map<std::string, QueryScanState> query_scans;

std::unordered_map<std::string, QueryCacheEntry> query_cache;

std::unordered_map<std::uint32_t, ResolveScanState> resolve_scans;

std::unordered_map<std::uint32_t, ResolveCacheEntry> resolve_cache;

std::uint32_t next_handle{1};

std::uint64_t layout_epoch{1};

// Protected by state_mutex. A confirmed new native binding epoch is the only
// reset. Busy guards and ordinary expected-value conflicts never set this.
std::uint64_t write_fault_epoch{};

const char* write_fault_reason{};

bool write_fault_active_locked(){ return write_fault_epoch == layout_epoch && write_fault_reason; }

std::mutex write_mutex;

std::mutex prop_recipe_mutex;

std::unordered_map<std::uint32_t, RegisteredPropRecipe> prop_recipes;

bool prop_recipe_catalog_ready{};

std::mutex prop_query_call_mutex;

PropQueryResultCache prop_query_result_cache;

std::atomic<bool> stop_requested{};

std::uintptr_t image_base{};

std::size_t image_size{};

bool readable(std::uintptr_t address, std::size_t size, ReadRegionCache* regions){
    if (regions) return regions->check(address, size);
    if (!address || !size || address > UINTPTR_MAX - size) return false;
    MEMORY_BASIC_INFORMATION memory{};
    if (!VirtualQuery(reinterpret_cast<const void*>(address), &memory, sizeof(memory))) return false;
    const auto end = reinterpret_cast<std::uintptr_t>(memory.BaseAddress) + memory.RegionSize;
    return memory.State == MEM_COMMIT && !(memory.Protect & (PAGE_GUARD | PAGE_NOACCESS)) &&
        address + size <= end;
}

// Runtime reads originate from the live ECS entity table in this process.
// ReadProcessMemory plus VirtualQuery for every scalar made the incremental
// scan advance only one or two entities per game-thread callback. Keep the
// same fault containment, but copy directly under SEH; callers still validate
// externally supplied ranges where required and invalid/stale pointers fail
// closed on an access violation.
bool guarded_copy(void* destination, const void* source, std::size_t size){
    __try {
        std::memcpy(destination, source, size);
        return true;
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return false;
    }
}

bool read_bytes(std::uintptr_t address, void* value, std::size_t size){
    return address && value && size && address <= UINTPTR_MAX - size &&
        guarded_copy(value, reinterpret_cast<const void*>(address), size);
}

bool layout_snapshot(ResolvedLayout& result){
    std::scoped_lock lock(state_mutex);
    if (!layout_ready) return false;
    result = live_layout;
    return true;
}

bool entity_pointers(const ResolvedLayout& layout, std::vector<EntityPointer>& pointers){
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
        !read(layout.table_address, table) || !table) {
        std::scoped_lock lock(state_mutex);
        layout_ready = false;
        ++layout_epoch;
        handles.clear();
        reverse_handles.clear();
        return false;
    }
    std::vector<std::uintptr_t> slots(static_cast<std::size_t>(count));
    if (!read_bytes(table, slots.data(), slots.size() * sizeof(slots[0]))) return false;
    pointers.clear();
    if (layout.occupancy_address) {
        std::uintptr_t occupancy{};
        if (!read(layout.occupancy_address, occupancy) || !occupancy) return false;
        std::vector<std::uint64_t> bits((slots.size() + 63) / 64);
        if (!read_bytes(occupancy, bits.data(), bits.size() * sizeof(bits[0]))) return false;
        if (!compact_entity_pointers(slots.data(), slots.size(), bits.data(), bits.size(), pointers)) return false;
        std::uintptr_t occupancy_again{}, table_again{};
        std::uint64_t count_again{};
        if (!read(layout.occupancy_address, occupancy_again) || occupancy_again != occupancy ||
            !read(layout.table_address, table_again) || table_again != table ||
            !read(layout.count_address, count_again) || count_again != count) return false;
    } else {
        if (!compact_entity_pointers(slots.data(), slots.size(), nullptr, 0, pointers)) return false;
    }
    return true;
}

// The original table contains non-null pointers in erased buckets. Match the
// original ID hash lookup instead of treating these stale allocations as live
// entities. Profiles without this independently validated table layout retain
// their existing dispatcher-backed lookup contract.
bool entity_registered(const EntityView& entity, const ResolvedLayout& layout){
    if (!layout.occupancy_address && !layout.keys_address) return true;
    std::uint64_t count{};
    std::uintptr_t table{}, occupancy{}, keys{};
    if (!layout.occupancy_address || !layout.keys_address ||
        !read(layout.count_address, count) || !count || count > (1u << 20) || (count & (count - 1)) ||
        !read(layout.table_address, table) || !table ||
        !read(layout.occupancy_address, occupancy) || !occupancy ||
        !read(layout.keys_address, keys) || !keys) return false;
    auto hash = entity.id;
    hash = ((hash >> 16) ^ hash) * 0x45d9f3bu;
    hash = ((hash >> 16) ^ hash) * 0x45d9f3bu;
    hash = (hash >> 16) ^ hash;
    auto slot = static_cast<std::size_t>(hash & (count - 1));
    for (std::size_t visited = 0; visited < count; ++visited, slot = (slot + 1) & (count - 1)) {
        std::uint64_t bits{};
        std::uint32_t key{};
        if (!read(occupancy + (slot / 64) * 8, bits) || !(bits & (std::uint64_t{1} << (slot % 64)))) return false;
        if (!read(keys + slot * sizeof(key), key)) return false;
        if (key != entity.id) continue;
        std::uintptr_t current{};
        std::uint64_t again{};
        std::uint64_t count_again{};
        std::uintptr_t table_again{}, occupancy_again{}, keys_again{};
        std::uint32_t key_again{}, id_again{}, generation_again{};
        return read(table + slot * sizeof(current), current) && current == entity.pointer &&
            read(occupancy + (slot / 64) * 8, again) && (again & (std::uint64_t{1} << (slot % 64))) &&
            read(keys + slot * sizeof(key_again), key_again) && key_again == entity.id &&
            read(entity.pointer + layout.entity_id, id_again) && id_again == entity.id &&
            read(entity.pointer + layout.entity_generation, generation_again) && generation_again == entity.generation &&
            read(layout.count_address, count_again) && count_again == count &&
            read(layout.table_address, table_again) && table_again == table &&
            read(layout.occupancy_address, occupancy_again) && occupancy_again == occupancy &&
            read(layout.keys_address, keys_again) && keys_again == keys;
    }
    return false;
}

bool entity_view(std::uintptr_t pointer, const ResolvedLayout& layout, EntityView& entity){
    entity.pointer = pointer;
    if (!pointer || !read(pointer + layout.entity_layout, entity.layout) || !entity.layout ||
        !read(pointer + layout.entity_storage, entity.storage) || !entity.storage ||
        !read(pointer + layout.entity_row, entity.row) || entity.row > (1u << 24) ||
        !read(pointer + layout.entity_id, entity.id) || !entity.id ||
        !read(pointer + layout.entity_generation, entity.generation)) return false;
    read(pointer + layout.entity_definition, entity.definition);
    return entity_registered(entity, layout);
}

std::uint64_t identity_key(std::uint32_t id, std::uint32_t generation){
    return static_cast<std::uint64_t>(generation) << 32 | id;
}

std::uint32_t handle_for(const EntityView& entity, std::size_t table_index){
    std::scoped_lock lock(state_mutex);
    const auto table_slot = read_only_binding && table_index != SIZE_MAX ?
        read_only_view.entity_table + table_index * sizeof(std::uintptr_t) : std::uintptr_t{};
    const auto key = identity_key(entity.id, entity.generation);
    if (const auto found = reverse_handles.find(key); found != reverse_handles.end()) {
        auto& record = handles.at(found->second);
        if (record.pointer != entity.pointer) record.table_slot = 0;
        record.pointer = entity.pointer;
        if (table_slot) record.table_slot = table_slot;
        return found->second;
    }
    // Never recycle an opaque handle during this process, including world changes.
    if (!next_handle) return 0;
    const auto handle = next_handle++;
    handles.emplace(handle, HandleRecord{entity.id, entity.generation, layout_epoch, entity.pointer, table_slot});
    reverse_handles.emplace(key, handle);
    return handle;
}

bool SnapshotEntityIds(std::vector<std::uint32_t>& ids){
    ResolvedLayout layout{};
    std::vector<EntityPointer> pointers;
    if (!layout_snapshot(layout) || !entity_pointers(layout, pointers) || pointers.size() > (1u << 20)) return false;
    ids.clear();
    ids.reserve(pointers.size());
    for (const auto& pointer : pointers) {
        EntityView entity{};
        if (entity_view(pointer.pointer, layout, entity)) ids.push_back(entity.id);
    }
    return true;
}

bool entity_for_handle(std::uint32_t handle, const ResolvedLayout& layout, EntityView& entity){
    HandleRecord record{};
    bool direct{};
    std::uintptr_t bound_table{};
    {
        std::scoped_lock lock(state_mutex);
        const auto found = handles.find(handle);
        if (found == handles.end() || found->second.epoch != layout_epoch) return false;
        record = found->second;
        direct = read_only_binding;
        bound_table = read_only_view.entity_table;
    }
    if (direct) {
        std::uint64_t count{};
        std::uintptr_t table{}, pointer{};
        if (!read(layout.count_address, count) || !count || count > (1u << 20) ||
            !read(layout.table_address, table) || table != bound_table) return false;
        const auto end = table + count * sizeof(std::uintptr_t);
        if (record.table_slot < table || record.table_slot >= end ||
            !read(record.table_slot, pointer) || pointer != record.pointer) {
            // Discover the slot once, then validate it on every read. A readable
            // object with an old ID is insufficient after entity removal.
            std::array<std::uintptr_t, 256> batch{};
            record.table_slot = 0;
            for (std::size_t offset = 0; offset < count && !record.table_slot; offset += batch.size()) {
                const auto size = (std::min)(batch.size(), static_cast<std::size_t>(count) - offset);
                if (!read_bytes(table + offset * sizeof(pointer), batch.data(), size * sizeof(pointer))) return false;
                for (std::size_t index = 0; index < size; ++index) if (batch[index] == record.pointer) {
                    record.table_slot = table + (offset + index) * sizeof(pointer);
                    break;
                }
            }
            if (!record.table_slot || !read(record.table_slot, pointer) || pointer != record.pointer) return false;
            std::scoped_lock lock(state_mutex);
            const auto found = handles.find(handle);
            if (found == handles.end() || found->second.epoch != record.epoch || found->second.pointer != record.pointer) return false;
            found->second.table_slot = record.table_slot;
        }
    }
    // Re-read the live identity and layout; never cache component addresses.
    // A moved/deleted entity is rediscovered by the next query, not a full-world
    // scan for each individual component read and write.
    return entity_view(record.pointer, layout, entity) && entity.id == record.id &&
        entity.generation == record.generation;
}

std::shared_ptr<PropQueryOperation> pending_prop_query;

std::uint64_t current_epoch(){
    std::scoped_lock lock(state_mutex);
    return layout_epoch;
}

bool invoke_read_operation(GameThreadDispatcher::Operation operation, const std::shared_ptr<void>& context,
                           std::uint32_t timeout_ms){
    bool direct{};
    { std::scoped_lock lock(state_mutex); direct = read_only_binding; }
    if (!direct) return GameThreadDispatcher::Invoke(operation, context, timeout_ms);
    std::scoped_lock operation_lock(operation_mutex);
    EngineContext::EcsView expected, before, after;
    std::uint64_t epoch{};
    {
        std::scoped_lock lock(state_mutex);
        if (!read_only_binding || !layout_ready || !component_registry_ready || !component_registry_world_bound) return false;
        expected = read_only_view;
        epoch = layout_epoch;
    }
    if (!EngineContext::ResolveEcs(before) || before != expected) return false;
    operation(context.get());
    return EngineContext::ResolveEcs(after) && after == before && current_epoch() == epoch;
}

void resolve_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<ResolveOperation*>(opaque);
    ResolvedLayout layout{};
    if (!layout_snapshot(layout)) return;
    const auto epoch = current_epoch();
    const auto now_ms = GetTickCount64();
    for (auto iterator = resolve_scans.begin(); iterator != resolve_scans.end();) {
        if (now_ms - iterator->second.touched_ms > 2000) iterator = resolve_scans.erase(iterator);
        else ++iterator;
    }
    for (auto iterator = resolve_cache.begin(); iterator != resolve_cache.end();) {
        if (now_ms - iterator->second.completed_ms > 2000) iterator = resolve_cache.erase(iterator);
        else ++iterator;
    }
    if (auto cached = resolve_cache.find(operation.entity_id); cached != resolve_cache.end()) {
        if (cached->second.epoch == epoch && now_ms - cached->second.completed_ms <= 50) {
            operation.result = cached->second.handle;
            return;
        }
        resolve_cache.erase(cached);
    }
    auto scan = resolve_scans.find(operation.entity_id);
    if (scan == resolve_scans.end() || scan->second.epoch != epoch) {
        ResolveScanState fresh{};
        fresh.epoch = epoch;
        fresh.touched_ms = now_ms;
        if (!entity_pointers(layout, fresh.pointers)) return;
        scan = resolve_scans.insert_or_assign(operation.entity_id, std::move(fresh)).first;
    }
    auto& progress = scan->second;
    progress.touched_ms = now_ms;
    const auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(1);
    std::size_t visited = 0;
    for (; progress.cursor < progress.pointers.size() && visited < 256; ++progress.cursor, ++visited) {
        EntityView candidate{};
        if (entity_view(progress.pointers[progress.cursor].pointer, layout, candidate) &&
            candidate.id == operation.entity_id) {
            operation.result = handle_for(candidate, progress.pointers[progress.cursor].table_index);
            resolve_cache.insert_or_assign(operation.entity_id,
                ResolveCacheEntry{epoch, now_ms, operation.result});
            resolve_scans.erase(scan);
            return;
        }
        if (std::chrono::steady_clock::now() >= deadline) {
            ++progress.cursor;
            break;
        }
    }
    if (progress.cursor >= progress.pointers.size()) {
        resolve_cache.insert_or_assign(operation.entity_id, ResolveCacheEntry{epoch, now_ms, 0});
        resolve_scans.erase(scan);
    }
}

void template_uuid_on_game_thread(void* opaque){
    std::scoped_lock operation_lock(operation_mutex);
    auto& operation = *static_cast<TemplateUuidOperation*>(opaque);
    ResolvedLayout layout{};
    EntityView before{}, after{};
    std::array<std::uint8_t, 16> again{};
    if (!layout_snapshot(layout) || !layout.entity_definition_uuid_validated ||
        !entity_for_handle(operation.handle, layout, before) || !before.definition ||
        before.definition > UINTPTR_MAX - layout.entity_definition_uuid ||
        before.definition + layout.entity_definition_uuid > UINTPTR_MAX - operation.uuid.size()) return;
    const auto address = before.definition + layout.entity_definition_uuid;
    if (!read_bytes(address, operation.uuid.data(), operation.uuid.size()) ||
        std::all_of(operation.uuid.begin(), operation.uuid.end(), [](auto byte) { return byte == 0; }) ||
        !read_bytes(address, again.data(), again.size()) || again != operation.uuid ||
        !entity_for_handle(operation.handle, layout, after) || after != before) return;
    operation.result = true;
}

std::atomic<const char*> last_write_reason{"not-attempted"};
}

namespace EcsRuntime {
struct SessionGuard::State {
    std::unique_lock<std::mutex> lock{state_mutex};
    EngineContext::EcsView view;
    std::uint64_t epoch{};
};

SessionGuard::SessionGuard() = default;

SessionGuard::~SessionGuard() = default;

bool SessionGuard::TryAcquire(std::uint64_t expected_session) {
    if (state_ || !expected_session) return false;
    auto state = std::make_unique<State>();
    if (stop_requested.load(std::memory_order_acquire) || !read_only_binding || !layout_ready ||
        !component_registry_ready || !component_registry_world_bound || expected_session != layout_epoch + 1 ||
        read_only_view.role != EngineContext::EcsRole::LocalAuthority ||
        component_registry.owner != read_only_view.registry_owner ||
        component_registry_manager != read_only_view.manager) return false;
    EngineContext::EcsView current;
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!EngineContext::ResolveEcs(current) || current != read_only_view ||
        live_layout.count_address != current.manager + current.layout.manager_count ||
        live_layout.table_address != current.manager + current.layout.manager_table ||
        !read(live_layout.count_address, count) || !count || count > (1u << 20) ||
        !read(live_layout.table_address, table) || !table || table != live_table ||
        table != current.entity_table) return false;
    state->view = current;
    state->epoch = layout_epoch;
    state_ = std::move(state);
    return true;
}

const EngineContext::EcsView& SessionGuard::View() const { return state_->view; }

bool SessionGuard::WritesAllowed() const { return state_ && !write_fault_active_locked(); }

void SessionGuard::MarkWriteUncertain(const char* reason) {
    if (!state_ || !reason) return;
    write_fault_epoch = state_->epoch;
    write_fault_reason = reason;
    last_write_reason.store(reason, std::memory_order_relaxed);
}

bool EntityIdentity(std::uint32_t handle, std::uint32_t* entity_id) {
    if (!handle || !entity_id) return false;
    std::scoped_lock operation_lock(operation_mutex);
    bool direct{};
    EngineContext::EcsView expected, before, after;
    {
        std::scoped_lock lock(state_mutex);
        direct = read_only_binding;
        expected = read_only_view;
    }
    if (direct && (!EngineContext::ResolveEcs(before) || before != expected)) return false;
    ResolvedLayout layout{};
    EntityView entity{};
    if (!layout_snapshot(layout) || !entity_for_handle(handle, layout, entity)) return false;
    if (direct && (!EngineContext::ResolveEcs(after) || after != before)) return false;
    *entity_id = entity.id;
    return entity.id != 0;
}

bool SnapshotEntityIds(std::vector<std::uint32_t>& ids) {
    return ::SnapshotEntityIds(ids);
}

bool Initialize() {
    const bool execution_profile = KfcRuntimeCompatibility::EnshroudedClient::Load();
    image_base = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    if (!image_base) return false;
    const auto dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(image_base);
    if (dos->e_magic != IMAGE_DOS_SIGNATURE) return false;
    const auto nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(image_base + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE) return false;
    image_size = nt->OptionalHeader.SizeOfImage;
    stop_requested.store(false, std::memory_order_release);
    if (execution_profile) GameThreadDispatcher::Initialize();
    // Metadata discovery remains available when a build lacks proven native
    // execution adapters. Original-owner entity reads do not authorize writes.
    return true;
}

void Tick() {
    if (stop_requested.load(std::memory_order_acquire)) return;
    auto manager = GameThreadDispatcher::EntityManager();
    EngineContext::EcsView owners;
    const bool direct_read = (!manager || !GameThreadDispatcher::Ready()) && EngineContext::ResolveEcs(owners);
    if (direct_read) manager = owners.manager;
    read_component_registry(manager);
    if (!manager) {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (layout_ready) {
            layout_ready = false;
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
        return;
    }
    const auto count_address = manager + (direct_read ? owners.layout.manager_count : KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count);
    const auto table_address = manager + (direct_read ? owners.layout.manager_table : KfcRuntimeCompatibility::EnshroudedClient::entity_manager_table);
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!read(count_address, count) || !count || count > (1u << 20) ||
        !read(table_address, table) || !readable(table, sizeof(std::uintptr_t))) {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (layout_ready) {
            layout_ready = false;
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
        return;
    }
    {
        std::scoped_lock operation_lock(operation_mutex);
        std::scoped_lock lock(state_mutex);
        if (!layout_ready || live_table != table || live_layout.count_address != count_address ||
            live_layout.table_address != table_address || read_only_binding != direct_read ||
            (direct_read && read_only_view != owners)) {
            live_layout = {};
            live_table = table;
            read_only_binding = direct_read;
            read_only_view = direct_read ? owners : EngineContext::EcsView{};
            live_layout.count_address = count_address;
            live_layout.table_address = table_address;
            live_layout.occupancy_address = direct_read && owners.layout.manager_occupancy ? manager + owners.layout.manager_occupancy : 0;
            live_layout.keys_address = direct_read && owners.layout.manager_keys ? manager + owners.layout.manager_keys : 0;
            live_layout.entity_id = direct_read ? owners.layout.entity_id : KfcRuntimeCompatibility::EnshroudedClient::entity_id;
            live_layout.entity_generation = direct_read ? owners.layout.entity_generation : KfcRuntimeCompatibility::EnshroudedClient::entity_generation;
            live_layout.entity_layout = direct_read ? owners.layout.entity_layout : KfcRuntimeCompatibility::EnshroudedClient::entity_layout;
            live_layout.entity_storage = direct_read ? owners.layout.entity_storage : KfcRuntimeCompatibility::EnshroudedClient::entity_storage;
            live_layout.entity_definition = direct_read ? owners.layout.entity_definition : KfcRuntimeCompatibility::EnshroudedClient::entity_definition;
            live_layout.entity_definition_uuid = direct_read ? owners.layout.entity_definition_uuid : 0;
            live_layout.entity_definition_uuid_validated = direct_read && owners.layout.entity_definition_uuid_validated;
            live_layout.entity_row = direct_read ? owners.layout.entity_row : KfcRuntimeCompatibility::EnshroudedClient::entity_row;
            live_layout.component_bits = direct_read ? owners.layout.component_bits : KfcRuntimeCompatibility::EnshroudedClient::component_bits;
            live_layout.component_offsets = direct_read ? owners.layout.component_offsets : KfcRuntimeCompatibility::EnshroudedClient::component_offsets;
            live_layout.component_strides = direct_read ? owners.layout.component_strides : KfcRuntimeCompatibility::EnshroudedClient::component_strides;
            layout_ready = true;
            ++layout_epoch;
            handles.clear();
            reverse_handles.clear();
        }
    }
    if (!direct_read) component_discovery_tick(manager);
}

void Shutdown() {
    stop_requested.store(true, std::memory_order_release);
    GameThreadDispatcher::Shutdown();
    std::scoped_lock operation_lock(operation_mutex);
    reset_component_discovery();
    std::scoped_lock lock(state_mutex);
    types.clear();
    configured_types.clear();
    component_registry = {};
    component_registry_ready = false;
    component_registry_world_bound = false;
    component_registry_seen = false;
    component_registry_json.clear();
    component_registry_error = "runtime stopped";
    live_layout = {};
    layout_ready = false;
    read_only_binding = false;
    read_only_view = {};
    handles.clear();
    reverse_handles.clear();
    query_scans.clear();
    query_cache.clear();
    resolve_scans.clear();
    resolve_cache.clear();
}
}

extern "C" std::uint64_t __cdecl KfcRuntimeWorldSessionId() {
    std::scoped_lock lock(state_mutex);
    if (read_only_binding) {
        EngineContext::EcsView current;
        if (!EngineContext::ResolveEcs(current) || current != read_only_view) return 0;
    }
    const auto manager = read_only_binding ? component_registry_manager : GameThreadDispatcher::EntityManager();
    std::uint64_t count{};
    std::uintptr_t table{};
    if (!manager || !layout_ready ||
        live_layout.count_address != manager + (read_only_binding ? read_only_view.layout.manager_count : KfcRuntimeCompatibility::EnshroudedClient::entity_manager_count) ||
        !read(live_layout.count_address, count) || !count || count > (1u << 20) ||
        !read(live_layout.table_address, table) || !table || table != live_table) return 0;
    // Conservative identity: rebuilding ECS layouts also invalidates old journals.
    return layout_epoch + 1;
}

extern "C" bool __cdecl KfcRuntimeEcsReady() {
    std::scoped_lock lock(state_mutex);
    if (!layout_ready || types.empty()) return false;
    if (!read_only_binding) return GameThreadDispatcher::Ready() &&
        (!component_registry_ready || component_registry_world_bound);
    EngineContext::EcsView current;
    return component_registry_ready && component_registry_world_bound &&
        EngineContext::ResolveEcs(current) && current == read_only_view;
}

extern "C" std::uint32_t __cdecl KfcRuntimeEcsResolve(std::uint32_t entity_id) {
    operation_counters.resolves.fetch_add(1, std::memory_order_relaxed);
    if (!entity_id || !KfcRuntimeEcsReady()) {
        operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
        return 0;
    }
    auto operation = std::make_shared<ResolveOperation>();
    operation->entity_id = entity_id;
    if (!invoke_read_operation(resolve_on_game_thread, operation)) {
        operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
        return 0;
    }
    if (operation->result) operation_counters.resolve_successes.fetch_add(1, std::memory_order_relaxed);
    else operation_counters.resolve_failures.fetch_add(1, std::memory_order_relaxed);
    return operation->result;
}

extern "C" bool __cdecl KfcRuntimeEcsTemplateUuid(std::uint32_t handle, std::uint8_t* uuid) {
    if (!handle || !uuid || !KfcRuntimeEcsReady()) return false;
    auto operation = std::make_shared<TemplateUuidOperation>();
    operation->handle = handle;
    if (!invoke_read_operation(template_uuid_on_game_thread, operation) || !operation->result) return false;
    std::memcpy(uuid, operation->uuid.data(), operation->uuid.size());
    return true;
}

extern "C" bool __cdecl KfcRuntimeEcsEntityIdentity(std::uint32_t handle, std::uint32_t* entity_id) {
    return EcsRuntime::EntityIdentity(handle, entity_id);
}
