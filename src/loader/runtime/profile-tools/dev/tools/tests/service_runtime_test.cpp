// Real service exports, SessionGuard and SceneGuard against private objects.
// This executable never opens a game process or calls a gameplay function.
#include "ecs_runtime.cpp"
#include <cstdio>
#include <cstdlib>

namespace {
unsigned checks{};
void check(bool ok) {
    ++checks;
    if (!ok) { std::fprintf(stderr, "service check %u failed\n", checks); std::exit(1); }
}
template<class T> void put(void* target, std::size_t offset, const T& value) {
    std::memcpy(static_cast<unsigned char*>(target) + offset, &value, sizeof(value));
}
struct Owner { std::uintptr_t vtable{}, pointer{}, scene{}; } root;
struct Controller { std::uintptr_t vtable{}, client{}, main{}; };
struct Context { std::uintptr_t execution{}; };
struct Thread {
    std::uintptr_t name{};
    std::uint64_t length{10}, capacity{64};
    char text[64]{"GameServer"};
    std::uintptr_t handle{123}, context{};
    std::uint32_t stop{};
    std::uintptr_t callback{};
};
struct Scene {
    Thread thread;
    std::uintptr_t entries{};
    std::uint64_t count{1}, capacity{1};
    alignas(4) LONG gate{};
    std::uint8_t state{1};
    std::uint64_t loop{};
};
struct Entry {
    std::uintptr_t context{};
    std::uint64_t uuid[2]{11,22};
    std::uint8_t state{5};
    unsigned char padding[119]{};
};
struct ThreadRow { std::uintptr_t handle{}, name{}; std::uint64_t length{}; } thread_rows[1];
alignas(4) LONG thread_gate{};
std::uint16_t thread_count{1};
struct Execution { unsigned char registry[4096]{}, manager[400]{}; };
struct ServicePayload {
    std::uint8_t enabled{1}, sibling{7}, padding[2]{0x44,0x55};
    std::uint32_t count{84}, other{123}, tail{0xabcdef01};
    bool operator==(const ServicePayload&) const = default;
};
struct StringView { std::uintptr_t text{}; std::uint64_t length{}; };
std::uintptr_t root_vtable[1]{}, client_vtable[1]{}, controller_vtable[1]{};
unsigned char metadata[0x90]{}, descriptor[0xd0]{};
std::uintptr_t metadata_slot{};
const char service_name[]{"ecs::FixtureSettings"};
const char collision_name[]{"ecs::FixtureSettinGs"};
StringView mutable_names[1]{{reinterpret_cast<std::uintptr_t>(service_name), sizeof(service_name)-1}};
void fixture_callback() {}
void fixture_mutable_system() {}
void fixture_unused_tracker() {}
constexpr std::uint32_t type_hash = 0x11223344;
constexpr std::size_t record_stride = 0xe8, system_stride = 0x450, systems_offset = 0x300;
}

int main() {
    const auto base = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    const auto* dos = reinterpret_cast<const IMAGE_DOS_HEADER*>(base);
    const auto* nt = reinterpret_cast<const IMAGE_NT_HEADERS64*>(base + dos->e_lfanew);
    image_base = base; image_size = nt->OptionalHeader.SizeOfImage;
    KfcRuntimeCompatibility::EnshroudedClient::image_size = image_size;
    KfcRuntimeCompatibility::EnshroudedClient::image_timestamp = nt->FileHeader.TimeDateStamp;
    KfcRuntimeCompatibility::EnshroudedClient::image_sha256 = "private-service-fixture";
    KfcRuntimeCompatibility::EnshroudedClient::image_target = "private-service-test.exe";
    auto rva = [base](auto pointer) { return reinterpret_cast<std::uintptr_t>(pointer) - base; };
    Execution execution;
    Context context{reinterpret_cast<std::uintptr_t>(&execution)};
    Entry entry; entry.context = reinterpret_cast<std::uintptr_t>(&context);
    Scene scene;
    scene.entries = reinterpret_cast<std::uintptr_t>(&entry);
    scene.thread.name = reinterpret_cast<std::uintptr_t>(scene.thread.text);
    scene.thread.context = reinterpret_cast<std::uintptr_t>(&scene.loop);
    scene.thread.callback = reinterpret_cast<std::uintptr_t>(fixture_callback);
    thread_rows[0] = {scene.thread.handle,scene.thread.name,scene.thread.length};
    Controller controller{reinterpret_cast<std::uintptr_t>(controller_vtable),0,reinterpret_cast<std::uintptr_t>(&context)};
    Owner client{reinterpret_cast<std::uintptr_t>(client_vtable),reinterpret_cast<std::uintptr_t>(&controller),reinterpret_cast<std::uintptr_t>(&scene)};
    controller.client = reinterpret_cast<std::uintptr_t>(&client);
    root = {reinterpret_cast<std::uintptr_t>(root_vtable),reinterpret_cast<std::uintptr_t>(&client)};
    std::uintptr_t entity_table[1]{1};
    put(execution.manager,0,reinterpret_cast<std::uintptr_t>(execution.registry));
    put(execution.manager,344,std::uint64_t{1});
    put(execution.manager,392,reinterpret_cast<std::uintptr_t>(entity_table));
    put(execution.registry,16,std::uint64_t{1});
    metadata_slot = reinterpret_cast<std::uintptr_t>(metadata);
    put(metadata,0x40,std::uint32_t{sizeof(ServicePayload)}); put(metadata,0x50,type_hash);
    put(descriptor,0xb0,reinterpret_cast<std::uintptr_t>(mutable_names));
    put(descriptor,0xb8,std::uint64_t{1});
    auto* payload = static_cast<ServicePayload*>(VirtualAlloc(nullptr,4096,MEM_COMMIT|MEM_RESERVE,PAGE_READWRITE));
    check(payload != nullptr);
    *payload = ServicePayload{};
    unsigned char records[record_stride*2]{};
    const auto name_hash = ComponentRegistry::hash_name(service_name);
    put(records,0,reinterpret_cast<std::uintptr_t>(service_name));
    put(records,8,std::uint64_t{sizeof(service_name)-1}); put(records,0xa0,name_hash);
    put(records,0xa8,reinterpret_cast<std::uintptr_t>(payload)); put(records,0xb0,reinterpret_cast<std::uintptr_t>(payload));
    std::uint32_t keys[4]{}; std::uint16_t values[4]{};
    const auto slot = name_hash & 3;
    std::uint64_t occupancy = std::uint64_t{1} << slot;
    keys[slot] = name_hash;
    put(execution.registry,640,reinterpret_cast<std::uintptr_t>(records)); put(execution.registry,648,std::uint64_t{1});
    put(execution.registry,656,reinterpret_cast<std::uintptr_t>(&occupancy)); put(execution.registry,664,std::uint64_t{4});
    put(execution.registry,672,reinterpret_cast<std::uintptr_t>(keys)); put(execution.registry,680,reinterpret_cast<std::uintptr_t>(values));
    put(execution.registry,688,std::uint32_t{1}); put(execution.registry,692,std::uint16_t{1});
    auto* system = execution.registry + systems_offset;
    put(system,0,reinterpret_cast<std::uintptr_t>(descriptor)); put(system,0x28,reinterpret_cast<std::uintptr_t>(fixture_mutable_system));
    put(system,0x40,reinterpret_cast<std::uintptr_t>(payload)); put(system,0x240,reinterpret_cast<std::uintptr_t>(payload));
    put(system,0x440,std::uint8_t{1}); put(system,0x441,std::uint8_t{1});
    nlohmann::json profile{
        {"target","private-service-test.exe"},
        {"image",{{"sha256","private-service-fixture"},{"timestamp",nt->FileHeader.TimeDateStamp},{"size",image_size}}},
        {"provenance",{{"ecsReadValidated",true},{"ecsAuthorityReadValidated",true},{"ecsWriteGateValidated",true},
            {"ecsChangeTrackingValidated",true},{"servicePlainDataValidated",true}}},
        {"ownerLayout",{{"rootObjectRva",rva(&root)},{"rootVtableRva",rva(root_vtable)},
            {"clientPointerOffset",offsetof(Owner,pointer)},{"clientVtableRva",rva(client_vtable)},
            {"controllerPointerOffset",offsetof(Owner,pointer)},{"controllerVtableRva",rva(controller_vtable)},
            {"controllerClientBackPointerOffset",offsetof(Controller,client)},{"controllerMainOwnerOffset",offsetof(Controller,main)}}},
        {"ecsReadLayout",{{"mainExecutionOwnerOffset",offsetof(Context,execution)},
            {"executionRegistryOwnerOffset",offsetof(Execution,registry)},{"executionEntityManagerOffset",offsetof(Execution,manager)},
            {"entityManagerOwnerOffset",0},{"entityManagerCount",344},{"entityManagerTable",392},
            {"entityId",16},{"entityGeneration",20},{"entityLayout",24},{"entityStorage",32},{"entityDefinitionPointer",40},{"entityRow",48},
            {"componentBits",0},{"componentOffsets",132},{"componentStrides",2692},{"maximumEntities",8},
            {"clientSceneOwnerOffset",offsetof(Owner,scene)},{"sceneEntriesPointerOffset",offsetof(Scene,entries)},
            {"sceneEntriesCountOffset",offsetof(Scene,count)},{"sceneEntriesCapacityOffset",offsetof(Scene,capacity)},
            {"sceneEntryStride",sizeof(Entry)},{"sceneEntryContextOffset",offsetof(Entry,context)},{"sceneEntryUuidOffset",offsetof(Entry,uuid)},
            {"sceneEntryStateOffset",offsetof(Entry,state)},{"sceneEntryReadyState",5},{"maximumSceneEntries",8}}},
        {"ecsWriteLayout",{{"id","ecs-native-thread-scene-gate-v1"},{"enabled",true},{"phaseValidated",true},{"phasePolicy","untracked-components"},
            {"threadRegistryLockRva",rva(&thread_gate)},{"threadRegistryEntriesRva",rva(thread_rows)},{"threadRegistryCountRva",rva(&thread_count)},
            {"threadRegistryCapacity",1},{"threadRegistryEntryStride",sizeof(ThreadRow)},{"registryEntryHandleOffset",offsetof(ThreadRow,handle)},
            {"registryEntryNamePointerOffset",offsetof(ThreadRow,name)},{"registryEntryNameLengthOffset",offsetof(ThreadRow,length)},
            {"threadInlineNameOffset",offsetof(Thread,text)},{"sceneThreadOffset",offsetof(Scene,thread)},
            {"threadNamePointerOffset",offsetof(Thread,name)},{"threadNameLengthOffset",offsetof(Thread,length)},
            {"threadNameCapacityOffset",offsetof(Thread,capacity)},{"threadNameCapacity",64},{"threadHandleOffset",offsetof(Thread,handle)},
            {"threadContextOffset",offsetof(Thread,context)},{"threadStopOffset",offsetof(Thread,stop)},
            {"threadCallbackOffset",offsetof(Thread,callback)},{"threadCallbackRva",rva(fixture_callback)},{"threadName","GameServer"},
            {"sceneContextOffset",offsetof(Scene,loop)},{"sceneLockOffset",offsetof(Scene,gate)},{"sceneStateOffset",offsetof(Scene,state)},
            {"sceneRunningState",1},{"executionModeOffset",offsetof(Execution,manager)+8},{"executionAuthorityMode",0},{"maximumWriteBytes",64},
            {"changeTrackerRva",rva(fixture_unused_tracker)},{"changeTrackerContextOffset",400},{"changeTrackerManagerOffset",8},
            {"changeTrackerQueueBOffset",320},{"changeTrackerRegistryAOffset",400},{"changeTrackerRegistryBOffset",560},
            {"maximumTrackingQueueCapacity",8},{"changeTrackerBytes",std::string(66,'0')}}},
        {"serviceLayout",{{"id","engine-named-plain-services-v1"},{"source","private-service-fixture"},
            {"recordsPointerOffset",640},{"recordCountOffset",648},{"recordStride",record_stride},{"recordHashOffset",0xa0},
            {"recordNormalPointerOffset",0xa8},{"recordAlternatePointerOffset",0xb0},{"mapBitsOffset",656},{"mapCapacityOffset",664},
            {"mapKeysOffset",672},{"mapValuesOffset",680},{"mapCountOffset",688},{"systemsOffset",systems_offset},
            {"systemCountOffset",692},{"systemStride",system_stride},{"systemNormalBindingsOffset",0x40},
            {"systemAlternateBindingsOffset",0x240},{"systemNormalCountOffset",0x440},{"systemAlternateCountOffset",0x441},
            {"maximumRecords",2},{"maximumMapCapacity",4},{"maximumSystems",2},
            {"contracts",nlohmann::json::array({{{"key",service_name},{"qualifiedType",std::string("keen::")+service_name},
                {"qualifiedHash",type_hash},{"size",sizeof(ServicePayload)},{"metadataRva",rva(metadata)},
                {"registrySlotRva",rva(&metadata_slot)},{"mutableSystemDescriptorRva",rva(descriptor)},
                {"mutableSystemFunctionRva",rva(fixture_mutable_system)},{"plainData",true},{"scope","host-world-shared"}}})}}}
    };
    std::string error;
    check(EngineContext::Configure(profile,error));
    check(EngineContext::ResolveEcs(read_only_view));
    read_only_binding = layout_ready = component_registry_ready = component_registry_world_bound = true;
    component_registry.owner = read_only_view.registry_owner;
    component_registry_manager = read_only_view.manager;
    live_layout.count_address = read_only_view.manager+344;
    live_layout.table_address = read_only_view.manager+392;
    live_table = reinterpret_cast<std::uintptr_t>(entity_table);
    layout_epoch = 19;
    const std::uint64_t session = layout_epoch+1;
    ServicePayload snapshot{};
    auto load = [&](std::uint64_t token = UINT64_MAX) {
        if (token == UINT64_MAX) token = session;
        return KfcRuntimeServiceRead(service_name,type_hash,sizeof(snapshot),token,
            reinterpret_cast<unsigned char*>(&snapshot),sizeof(snapshot));
    };
    auto unchanged = [&] { check(scene.gate==0 && thread_gate==0); };
    check(load() && snapshot==*payload); unchanged();
    const auto baseline = *payload;
    auto desired = baseline;
    desired.enabled = 0; desired.count = 999;
    std::array<std::uint8_t,sizeof(ServicePayload)> mask{};
    mask[0] = 1;
    auto apply = [&](const ServicePayload& expected) {
        return KfcRuntimeServiceCompareExchange(service_name,type_hash,sizeof(snapshot),session,&expected,mask.data(),&desired);
    };
    check(apply(baseline)==1 && payload->enabled==0 && payload->count==baseline.count);
    check(payload->sibling==baseline.sibling && std::memcmp(payload->padding,baseline.padding,2)==0);
    check(load() && snapshot==*payload); unchanged();
    check(apply(baseline)==2 && payload->enabled==0); unchanged();
    check(apply(*payload)==1); // no-op still validates the full binding
    *payload = baseline;
    auto stale = baseline; ++stale.tail;
    check(apply(stale)==2 && *payload==baseline); unchanged();
    mask[0] = 2; check(apply(baseline)==0 && *payload==baseline); mask[0] = 1;
    check(!load(session-1) && !load(0)); unchanged();
    write_fault_epoch = layout_epoch; write_fault_reason = "private-service-fixture-fault";
    check(load() && apply(baseline)==0 && *payload==baseline);
    write_fault_epoch = 0; write_fault_reason = nullptr; unchanged();
    component_registry_ready = false; check(!load()); component_registry_ready = true;
    const auto role = read_only_view.role;
    read_only_view.role = EngineContext::EcsRole::ClientReplica; check(!load()); read_only_view.role = role;
    scene.gate = 1; check(!load()); scene.gate = 0; unchanged();
    scene.thread.stop = 1; check(!load()); scene.thread.stop = 0;
    ++entry.uuid[0]; check(!load()); --entry.uuid[0];
    put(records,0,reinterpret_cast<std::uintptr_t>(collision_name)); check(!load());
    put(records,0,reinterpret_cast<std::uintptr_t>(service_name));
    keys[slot]^=1; check(!load()); keys[slot]^=1;
    std::memcpy(records+record_stride,records,record_stride);
    put(execution.registry,648,std::uint64_t{2}); check(!load()); put(execution.registry,648,std::uint64_t{1});
    put(system,0x240,std::uintptr_t{1}); check(!load()); put(system,0x240,reinterpret_cast<std::uintptr_t>(payload));
    put(descriptor,0xb8,std::uint64_t{0}); check(!load()); put(descriptor,0xb8,std::uint64_t{1});
    put(descriptor,0x98,reinterpret_cast<std::uintptr_t>(mutable_names));
    put(descriptor,0xa0,std::uint64_t{1}); check(!load());
    put(descriptor,0x98,std::uintptr_t{}); put(descriptor,0xa0,std::uint64_t{});
    put(system,0x441,std::uint8_t{0}); check(!load()); put(system,0x441,std::uint8_t{1});
    std::memcpy(system+system_stride,system,system_stride);
    put(execution.registry,692,std::uint16_t{2}); check(!load()); put(execution.registry,692,std::uint16_t{1});
    metadata_slot = 1; check(!load()); metadata_slot = reinterpret_cast<std::uintptr_t>(metadata);
    put(metadata,0x50,type_hash+1); check(!load()); put(metadata,0x50,type_hash);
    put(records,0xa8,std::uintptr_t{1}); put(records,0xb0,std::uintptr_t{1});
    put(system,0x40,std::uintptr_t{1}); put(system,0x240,std::uintptr_t{1});
    check(!load() && apply(baseline)==0);
    put(records,0xa8,reinterpret_cast<std::uintptr_t>(payload)); put(records,0xb0,reinterpret_cast<std::uintptr_t>(payload));
    put(system,0x40,reinterpret_cast<std::uintptr_t>(payload)); put(system,0x240,reinterpret_cast<std::uintptr_t>(payload));
    DWORD old{};
    check(VirtualProtect(payload,4096,PAGE_READONLY,&old)!=FALSE);
    check(!load() && apply(baseline)==0);
    check(VirtualProtect(payload,4096,old,&old)!=FALSE && *payload==baseline);
    auto malformed = profile;
    malformed["serviceLayout"]["systemStride"] = 1;
    check(EngineContext::Configure(malformed,error) && !load());
    malformed = profile; malformed["serviceLayout"]["systemNormalBindingsOffset"] = system_stride-8;
    check(EngineContext::Configure(malformed,error) && !load());
    check(EngineContext::Configure(profile,error) && load());
    put(execution.registry,640,UINTPTR_MAX-16); check(!load());
    put(execution.registry,640,reinterpret_cast<std::uintptr_t>(records));
    check(KfcRuntimeServiceCompareExchange(service_name,type_hash,sizeof(snapshot),session,
        reinterpret_cast<void*>(1),mask.data(),&desired)==0);
    check(load() && *payload==baseline); unchanged();
    check(VirtualFree(payload,0,MEM_RELEASE)!=FALSE);
    std::printf("service runtime: %u checks passed\n",checks);
}
