// Private data-ABI fixtures only. No process is opened and no game code runs.
// The DI resolver is isolated here, its real named-map contract is covered by
// named_service_access_test.cpp. This fixture exercises the actual queue reader.
#include "ecs/resolve_entity_command_queue.cpp"
#include <cstdio>
#include <cstdlib>
#include <source_location>

namespace {
unsigned checks{};
std::uintptr_t fixture_service{};
unsigned binding_calls{}, move_binding_on_call{};
void check(bool value, std::source_location location = std::source_location::current()) {
    ++checks;
    if (!value) {
        std::fprintf(stderr, "entity queue check %u failed at line %u\n", checks, location.line());
        std::exit(1);
    }
}
template<class T> void put(void* target, std::size_t offset, const T& value) {
    std::memcpy(static_cast<unsigned char*>(target) + offset, &value, sizeof(value));
}
void first_fixture_method() {}
void second_fixture_method() {}
constexpr std::array<void(*)(), 2> fixture_vtable{first_fixture_method, second_fixture_method};
}

namespace NamedServiceImplementation {
bool add(std::uintptr_t base, std::size_t offset, std::uintptr_t& result) {
    if (!base || offset > UINTPTR_MAX - base) return false;
    result = base + offset;
    return true;
}
bool resolve_binding(const BindingContract& binding, const EngineContext::EcsView&, std::uintptr_t& output) {
    ++binding_calls;
    if (binding.key != "ecs::Tasks" || !fixture_service) return false;
    output = fixture_service + (binding_calls == move_binding_on_call ? sizeof(std::uintptr_t) : 0);
    return true;
}
bool executable_callback(std::uintptr_t address) {
    return address == reinterpret_cast<std::uintptr_t>(first_fixture_method) ||
        address == reinterpret_cast<std::uintptr_t>(second_fixture_method);
}
bool writable(std::uintptr_t address, std::size_t size) {
    if (!address || !size || address > UINTPTR_MAX - size) return false;
    const auto end = address + size;
    while (address < end) {
        MEMORY_BASIC_INFORMATION info{};
        if (VirtualQuery(reinterpret_cast<const void*>(address), &info, sizeof(info)) != sizeof(info) ||
            info.State != MEM_COMMIT || info.Type != MEM_PRIVATE || info.Protect != PAGE_READWRITE) return false;
        const auto base = reinterpret_cast<std::uintptr_t>(info.BaseAddress);
        if (base > UINTPTR_MAX - info.RegionSize || base + info.RegionSize <= address) return false;
        address = (std::min)(end, base + info.RegionSize);
    }
    return true;
}
}

int main() {
    constexpr std::size_t manager_size = 0xebc100, capacity = 0x100000;
    auto* manager = static_cast<unsigned char*>(VirtualAlloc(nullptr, manager_size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
    auto* task_records = static_cast<unsigned char*>(VirtualAlloc(nullptr, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
    auto* queues = static_cast<EcsRuntime::EntityCommandQueueHeader*>(VirtualAlloc(nullptr, 0x1000, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
    auto* buffers = static_cast<unsigned char*>(VirtualAlloc(nullptr, capacity * 2, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE));
    check(manager && task_records && queues && buffers);
    const auto manager_address = reinterpret_cast<std::uintptr_t>(manager);
    const auto tasks_address = reinterpret_cast<std::uintptr_t>(task_records);
    const auto queues_address = reinterpret_cast<std::uintptr_t>(queues);
    const auto tracker = manager_address + 0xebc030;
    const auto counter = manager_address + 0x498;
    std::uintptr_t service_payload[2]{manager_address + 0x538, manager_address + 0x538};
    fixture_service = reinterpret_cast<std::uintptr_t>(service_payload);
    std::uintptr_t registry{};
    EngineContext::EcsView owner;
    owner.role = EngineContext::EcsRole::LocalAuthority;
    owner.manager = manager_address;
    owner.registry_owner = reinterpret_cast<std::uintptr_t>(&registry);
    owner.owners.image = reinterpret_cast<std::uintptr_t>(GetModuleHandleW(nullptr));
    put(manager, 0, owner.registry_owner);
    put(manager, 0x528, tasks_address);
    put(manager, 0x530, std::uint64_t{2});
    put(manager, 0xeb9f08, queues_address);
    put(manager, 0xeb9f10, std::uint64_t{2});
    for (std::size_t index = 0; index < 2; ++index) {
        EcsRuntime::EntityCommandQueueTaskLinks links{manager_address, queues_address + index * 64, tracker};
        put(task_records, index * 0xe0, links);
        queues[index].vtable = reinterpret_cast<std::uintptr_t>(fixture_vtable.data());
        queues[index].allocator = manager_address + 0x100;
        queues[index].buffer = reinterpret_cast<std::uintptr_t>(buffers + index * capacity);
        queues[index].capacity = capacity;
        queues[index].system_id = static_cast<std::uint16_t>(20 + index);
        queues[index].entity_id_counter = counter;
    }
    NamedServiceImplementation::BindingContract binding;
    binding.key = "ecs::Tasks";
    EcsRuntime::EntityCommandQueueSelection original, current;
    check(EcsRuntime::ResolveEntityCommandQueue(binding, owner, original));
    check(original.selected_index == 0 && original.queue_address == queues_address && original.queue_count == 2);
    check(original.header == queues[0] && original.task_links[1].queue == queues_address + 64);
    check(EcsRuntime::RereadEntityCommandQueue(binding, owner, original, current));

    auto expect_failure = [&] {
        current.queue_address = 0x1234;
        check(!EcsRuntime::ResolveEntityCommandQueue(binding, owner, current));
        check(current.queue_address == 0x1234);
    };
    owner.role = EngineContext::EcsRole::ClientReplica; expect_failure();
    owner.role = EngineContext::EcsRole::LocalAuthority;
    put(manager, 0, std::uintptr_t{1}); expect_failure(); put(manager, 0, owner.registry_owner);
    service_payload[0]++; expect_failure(); service_payload[0]--;
    put(manager, 0x530, std::uint64_t{3}); expect_failure(); put(manager, 0x530, std::uint64_t{2});
    put(manager, 0x530, std::uint64_t{65}); put(manager, 0xeb9f10, std::uint64_t{65}); expect_failure();
    put(manager, 0x530, std::uint64_t{2}); put(manager, 0xeb9f10, std::uint64_t{2});
    put(manager, 0xebbfc0, std::uint32_t{1}); expect_failure(); put(manager, 0xebbfc0, std::uint32_t{});
    put(task_records, 0xe0, std::uintptr_t{1}); expect_failure(); put(task_records, 0xe0, manager_address);
    put(task_records, 0xe0 + 8, queues_address); expect_failure(); put(task_records, 0xe0 + 8, queues_address + 64);
    put(task_records, 0xe0 + 16, tracker + 8); expect_failure(); put(task_records, 0xe0 + 16, tracker);
    queues[1].entity_id_counter++; expect_failure(); queues[1].entity_id_counter--;
    queues[1].capacity--; expect_failure(); queues[1].capacity++;
    queues[1].allocator++; expect_failure(); queues[1].allocator--;
    queues[1].byte_cursor = capacity + 1; expect_failure(); queues[1].byte_cursor = 0;
    queues[1].command_count = 1; expect_failure(); queues[1].command_count = 0;
    queues[1].buffer = queues[0].buffer; expect_failure(); queues[1].buffer += capacity;
    queues[1].execution_mode = 1; expect_failure(); queues[1].execution_mode = 0;

    // A moved named payload during the complete second binding read is rejected.
    binding_calls = 0; move_binding_on_call = 2; expect_failure(); move_binding_on_call = 0;
    check(EcsRuntime::ResolveEntityCommandQueue(binding, owner, current));
    queues[0].byte_cursor = 16; queues[0].command_count = 1;
    check(EcsRuntime::RereadEntityCommandQueue(binding, owner, original, current));
    check(current.queue_address == original.queue_address && current.header.byte_cursor == 16 && current.header.command_count == 1);
    check(EcsRuntime::ResolveEntityCommandQueue(binding, owner, current) && current.selected_index == 1);
    queues[0].system_id++;
    check(!EcsRuntime::RereadEntityCommandQueue(binding, owner, original, current));
    queues[0].system_id--;
    queues[1].byte_cursor = 16; queues[1].command_count = 1;
    check(!EcsRuntime::RereadEntityCommandQueue(binding, owner, original, current));
    queues[1].byte_cursor = 0; queues[1].command_count = 0;
    queues[0].byte_cursor = 0; queues[0].command_count = 0;
    const auto expected_buffer = queues[0].buffer;
    queues[0].buffer += 64;
    check(!EcsRuntime::RereadEntityCommandQueue(binding, owner, original, current));
    queues[0].buffer = expected_buffer;
    DWORD old_protection{};
    check(VirtualProtect(buffers, 4096, PAGE_READONLY, &old_protection) != 0);
    expect_failure();
    check(VirtualProtect(buffers, 4096, old_protection, &old_protection) != 0);
    check(EcsRuntime::ResolveEntityCommandQueue(binding, owner, current));
    VirtualFree(buffers, 0, MEM_RELEASE);
    VirtualFree(queues, 0, MEM_RELEASE);
    VirtualFree(task_records, 0, MEM_RELEASE);
    VirtualFree(manager, 0, MEM_RELEASE);
    std::printf("entity command queue checks passed: %u\n", checks);
}
