#include "engine-context/engine_context_state.h"

namespace EngineContextImplementation {
bool resolve_registered_scene(const Layout& layout, std::uintptr_t image,
    RegisteredScene& output, const char** reason) {
    const auto fail = [&](const char* message) {
        if (reason) *reason = message;
        return false;
    };
    const auto& w = layout.write;
    if (!image || !w.capacity || !w.stride || !w.name_size || w.name_size > w.name.size())
        return fail("ecs-server-thread-layout-invalid");
    std::uint16_t count{};
    if (!field(image, w.registry_count, count) || count > w.capacity)
        return fail("ecs-thread-registry-count-invalid");
    std::uintptr_t entries{};
    if (!add(image, w.registry_entries, entries)) return fail("ecs-thread-registry-range-invalid");
    RegisteredScene selected;
    std::uintptr_t selected_row{}, selected_name{};
    unsigned matches{};
    for (std::size_t index = 0; index < count; ++index) {
        std::uintptr_t row{}, name{}, handle{};
        std::uint64_t length{};
        if (index > UINTPTR_MAX / w.stride || !add(entries, index * w.stride, row) ||
            !field(row, w.row_length, length) || !field(row, w.row_name, name) ||
            !field(row, w.row_handle, handle)) return fail("ecs-thread-registry-row-unreadable");
        if (length != w.name_size) continue;
        std::array<char, 64> text{};
        if (!read_span(name, text.data(), length)) return fail("ecs-thread-name-unreadable");
        if (std::memcmp(text.data(), w.name.data(), length)) continue;
        if (++matches > 1) return fail("ecs-server-thread-is-ambiguous");
        if (!handle || name < w.inline_name || name - w.inline_name < w.scene_thread)
            return fail("ecs-server-thread-range-invalid");
        selected.thread = name - w.inline_name;
        selected.scene = selected.thread - w.scene_thread;
        selected.handle = handle;
        selected_row = row;
        selected_name = name;
    }
    if (!matches) return fail("ecs-server-thread-unregistered");
    const auto valid_thread = [&]() {
        std::uintptr_t name{}, handle{}, callback{}, context{}, wanted_context{}, wanted_callback{}, inline_name{};
        std::uint64_t length{}, capacity{};
        std::uint32_t stop{};
        std::uint8_t state{};
        return add(selected.thread, w.inline_name, inline_name) &&
            field(selected.thread, w.name_pointer, name) && name == inline_name && name == selected_name &&
            field(selected.thread, w.name_length, length) && length == w.name_size &&
            field(selected.thread, w.name_capacity, capacity) && capacity == w.name_capacity_value &&
            field(selected.thread, w.thread_handle, handle) && handle == selected.handle &&
            add(image, w.callback, wanted_callback) &&
            field(selected.thread, w.thread_callback, callback) && callback == wanted_callback &&
            add(selected.scene, w.scene_context, wanted_context) &&
            field(selected.thread, w.thread_context, context) && context == wanted_context &&
            field(selected.thread, w.thread_stop, stop) && stop == 0 &&
            field(selected.scene, w.scene_state, state) && state == w.running_state;
    };
    if (!valid_thread()) return fail("ecs-server-thread-owner-invalid-or-stopping");
    std::uint16_t count_again{};
    std::uintptr_t name_again{}, handle_again{};
    std::uint64_t length_again{};
    std::array<char, 64> text_again{};
    if (!field(image, w.registry_count, count_again) || count_again != count ||
        !field(selected_row, w.row_name, name_again) || name_again != selected_name ||
        !field(selected_row, w.row_handle, handle_again) || handle_again != selected.handle ||
        !field(selected_row, w.row_length, length_again) || length_again != w.name_size ||
        !read_span(name_again, text_again.data(), length_again) ||
        std::memcmp(text_again.data(), w.name.data(), length_again) || !valid_thread())
        return fail("ecs-server-thread-changed-during-read");
    output = selected;
    if (reason) *reason = nullptr;
    return true;
}
}
