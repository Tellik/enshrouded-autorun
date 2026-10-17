cmake_minimum_required(VERSION 3.24)
if(NOT WIN32 OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(STATUS "Native game runtime is only included in Windows x64 builds")
    return()
endif()

enable_language(RC)

# Each approved build is a complete, single-file compatibility profile.
file(GLOB_RECURSE profile_json_files CONFIGURE_DEPENDS
    "${CMAKE_CURRENT_SOURCE_DIR}/../profiles/*.json")
set(runtime_profiles)
set(resource_read_profiles)
foreach(profile_file IN LISTS profile_json_files)
    file(READ "${profile_file}" profile_contents)
    string(JSON profile_provenance_type ERROR_VARIABLE profile_provenance_error
        TYPE "${profile_contents}" provenance)
    if(profile_provenance_error MATCHES "not found")
        # Existing execution profiles predate provenance metadata. Keep their
        # established runtime validation behavior while requiring provenance
        # for newly introduced read-only resource profiles.
        list(APPEND runtime_profiles "${profile_file}")
    elseif(NOT profile_provenance_error STREQUAL "NOTFOUND" OR
           NOT profile_provenance_type STREQUAL "OBJECT")
        message(FATAL_ERROR "Profile provenance must be a JSON object: ${profile_file}")
    else()
        string(JSON profile_status ERROR_VARIABLE profile_status_error
            GET "${profile_contents}" provenance status)
        if(NOT profile_status_error STREQUAL "NOTFOUND")
            message(FATAL_ERROR "Profile provenance status is required: ${profile_file}")
        elseif(profile_status STREQUAL "approved")
        list(APPEND runtime_profiles "${profile_file}")
        elseif(profile_status STREQUAL "read-validated")
            string(JSON resource_read_validated ERROR_VARIABLE resource_status_error
                GET "${profile_contents}" provenance resourceReadValidated)
            string(JSON resource_layout_type ERROR_VARIABLE resource_layout_error
                TYPE "${profile_contents}" resourceLayout)
            if(NOT resource_status_error STREQUAL "NOTFOUND" OR
               NOT resource_read_validated OR
               NOT resource_layout_error STREQUAL "NOTFOUND" OR
               NOT resource_layout_type STREQUAL "OBJECT")
                message(FATAL_ERROR "Read-validated profiles must contain a validated resourceLayout: ${profile_file}")
            endif()
            list(APPEND resource_read_profiles "${profile_file}")
        else()
            message(FATAL_ERROR "Unsupported profile provenance status: ${profile_file}")
        endif()
    endif()
endforeach()
if(NOT runtime_profiles)
    message(FATAL_ERROR "No production profiles found under profiles/")
endif()

set(generated_dir "${CMAKE_CURRENT_BINARY_DIR}/generated")
file(MAKE_DIRECTORY "${generated_dir}")
set(profile_resource_rc "${generated_dir}/embedded_profiles.rc")
set(profile_resource_header "${generated_dir}/embedded_profile_ids.h")
set(profile_rc_content "#include <windows.h>\n")
set(profile_header_content "#pragma once\nnamespace KfcRuntimeEmbedded {\ninline constexpr int profile_resource_ids[] = {")
set(resource_read_header_content "inline constexpr int resource_read_profile_ids[] = {")

set(profile_resource_id 101)
foreach(profile IN LISTS runtime_profiles)
    file(TO_CMAKE_PATH "${profile}" profile_rc_path)
    string(APPEND profile_rc_content "${profile_resource_id} RCDATA \"${profile_rc_path}\"\n")
    string(APPEND profile_header_content "${profile_resource_id},")
    math(EXPR profile_resource_id "${profile_resource_id} + 1")
endforeach()
if(resource_read_profiles)
    foreach(profile IN LISTS resource_read_profiles)
        file(TO_CMAKE_PATH "${profile}" profile_rc_path)
        string(APPEND profile_rc_content "${profile_resource_id} RCDATA \"${profile_rc_path}\"\n")
        string(APPEND resource_read_header_content "${profile_resource_id},")
        math(EXPR profile_resource_id "${profile_resource_id} + 1")
    endforeach()
else()
    string(APPEND resource_read_header_content "0,")
endif()
string(APPEND profile_header_content "};\n${resource_read_header_content}};\n}")

file(WRITE "${profile_resource_rc}" "${profile_rc_content}")
file(WRITE "${profile_resource_header}" "${profile_header_content}")
set_source_files_properties("${profile_resource_rc}" PROPERTIES
    GENERATED TRUE
    OBJECT_DEPENDS "${runtime_profiles};${resource_read_profiles}")

add_library(kfc-runtime SHARED
    ecs/entities.cpp
    ecs/components.cpp
    ecs/props.cpp
    ecs/queries.cpp
    ecs/registry/component_discovery.cpp
    patches/patch_exports.cpp
    engine-context/owners.cpp
    engine-context/scene_gate.cpp
    engine-context/resource_lifetime.cpp
    loaded-resources/profile_layout.cpp
    loaded-resources/resource_index.cpp
    loaded-resources/item_registry.cpp
    loaded-resources/scalar_write.cpp
    engine-context/session.cpp
    player-input/cursor.cpp
    voxels/cells.cpp
    building/removal.cpp
    building/placement.cpp
    building/spawn.cpp
    voxels/grids.cpp
    named-services/registry.cpp
    named-services/plain_data.cpp
    host/lifecycle.cpp
    code-inventory/function_exports.cpp
    host/diagnostics.cpp
    execution/dispatcher.cpp
    patches/patch_runtime.cpp
    profiles/profile_selection.cpp
    player-input/building.cpp
    ../../modules/transport/native/steam_network.cpp
    "${profile_resource_rc}")
target_include_directories(kfc-runtime PRIVATE
    "${generated_dir}"
    "${CMAKE_CURRENT_SOURCE_DIR}"
    "${CMAKE_CURRENT_SOURCE_DIR}/ecs"
    "${CMAKE_CURRENT_SOURCE_DIR}/execution"
    "${CMAKE_CURRENT_SOURCE_DIR}/player-input"
    "${CMAKE_CURRENT_SOURCE_DIR}/patches"
    "${CMAKE_CURRENT_SOURCE_DIR}/engine-context"
    "${CMAKE_CURRENT_SOURCE_DIR}/loaded-resources"
    "${CMAKE_CURRENT_SOURCE_DIR}/json/single_include")
target_compile_features(kfc-runtime PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime PRIVATE
    KFC_RUNTIME_BUILD WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(kfc-runtime PRIVATE bcrypt)
set_property(TARGET kfc-runtime PROPERTY MSVC_RUNTIME_LIBRARY
    "MultiThreaded$<$<CONFIG:Debug>:Debug>")
foreach(configuration DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
    set_target_properties(kfc-runtime PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY_${configuration} "${CMAKE_BINARY_DIR}/bin")
endforeach()
if(MSVC)
    target_compile_options(kfc-runtime PRIVATE /W4 /EHsc)
endif()

# Development tools never run from the game or from the modloader.
add_executable(kfc-runtime-test-resource EXCLUDE_FROM_ALL
    ../tests/resource_runtime_test.cpp engine-context/owners.cpp engine-context/resource_lifetime.cpp engine-context/scene_gate.cpp)
target_include_directories(kfc-runtime-test-resource PRIVATE
    "${generated_dir}" "${CMAKE_CURRENT_SOURCE_DIR}"
    "${CMAKE_CURRENT_SOURCE_DIR}/loaded-resources" "${CMAKE_CURRENT_SOURCE_DIR}/engine-context"
    "${CMAKE_CURRENT_SOURCE_DIR}/json/single_include")
target_compile_features(kfc-runtime-test-resource PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-test-resource PRIVATE KFC_RUNTIME_BUILD WIN32_LEAN_AND_MEAN NOMINMAX)
set_property(TARGET kfc-runtime-test-resource PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
get_target_property(prop_test_sources kfc-runtime SOURCES)
list(REMOVE_ITEM prop_test_sources
    ecs/entities.cpp
    ecs/components.cpp
    ecs/props.cpp
    ecs/queries.cpp
    ecs/registry/component_discovery.cpp
    patches/patch_exports.cpp
)
add_executable(kfc-runtime-test-prop-query EXCLUDE_FROM_ALL
    ../tests/prop_query_test.cpp ${prop_test_sources})
add_executable(kfc-runtime-test-ecs-read EXCLUDE_FROM_ALL
    ../tests/ecs_read_test.cpp ${prop_test_sources})
add_executable(kfc-runtime-test-service EXCLUDE_FROM_ALL
    ../tests/service_runtime_test.cpp ${prop_test_sources})
get_target_property(prop_test_includes kfc-runtime INCLUDE_DIRECTORIES)
target_include_directories(kfc-runtime-test-prop-query PRIVATE ${prop_test_includes})
target_include_directories(kfc-runtime-test-ecs-read PRIVATE ${prop_test_includes})
target_include_directories(kfc-runtime-test-service PRIVATE ${prop_test_includes})
target_compile_features(kfc-runtime-test-service PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-test-service PRIVATE KFC_RUNTIME_BUILD WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(kfc-runtime-test-service PRIVATE bcrypt)
set_property(TARGET kfc-runtime-test-service PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
target_compile_features(kfc-runtime-test-ecs-read PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-test-ecs-read PRIVATE KFC_RUNTIME_BUILD WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(kfc-runtime-test-ecs-read PRIVATE bcrypt)
set_property(TARGET kfc-runtime-test-ecs-read PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
target_compile_features(kfc-runtime-test-prop-query PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-test-prop-query PRIVATE KFC_RUNTIME_BUILD WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(kfc-runtime-test-prop-query PRIVATE bcrypt)
set_property(TARGET kfc-runtime-test-prop-query PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set(profile_development_directory "${CMAKE_CURRENT_SOURCE_DIR}/../../dev/profiles")
add_executable(kfc-runtime-dev EXCLUDE_FROM_ALL
    "${profile_development_directory}/profile_commands.cpp"
    "${profile_development_directory}/command_arguments.cpp"
    "${profile_development_directory}/write_profile_report.cpp"
    "${profile_development_directory}/pe_image.cpp"
    "${profile_development_directory}/function_signatures.cpp"
    "${profile_development_directory}/inspect_executable.cpp"
    "${profile_development_directory}/select_executable_build.cpp"
    "${profile_development_directory}/discover_function_candidates.cpp"
    "${profile_development_directory}/scan_function_signatures.cpp"
    "${profile_development_directory}/export_profile_functions.cpp"
    "${profile_development_directory}/capture_process_modules.cpp"
    "${profile_development_directory}/import_legacy_ecs_capture.cpp"
    "${profile_development_directory}/inspect_provider_abi.cpp"
    "${profile_development_directory}/validate_runtime_profile.cpp"
    "${profile_development_directory}/create_runtime_profile.cpp"
    "${profile_development_directory}/approve_runtime_profile.cpp")
add_executable(kfc-runtime-test-building-input EXCLUDE_FROM_ALL
    ../tests/building_input_test.cpp player-input/building.cpp)
target_compile_features(kfc-runtime-test-building-input PRIVATE cxx_std_20)
target_include_directories(kfc-runtime-test-building-input PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}" "${CMAKE_CURRENT_SOURCE_DIR}/player-input"
    "${CMAKE_CURRENT_SOURCE_DIR}/ecs" "${CMAKE_CURRENT_SOURCE_DIR}/engine-context"
    "${CMAKE_CURRENT_SOURCE_DIR}/json/single_include")
target_compile_definitions(kfc-runtime-test-building-input PRIVATE KFC_RUNTIME_BUILD NOMINMAX WIN32_LEAN_AND_MEAN)
target_compile_features(kfc-runtime-dev PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-dev PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX
    SHROUDFORGE_PROFILE_DIRECTORY="${profile_development_directory}")
target_include_directories(kfc-runtime-dev PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/json/single_include")
target_link_libraries(kfc-runtime-dev PRIVATE bcrypt)
if(MSVC)
    target_compile_options(kfc-runtime-dev PRIVATE /W4 /EHsc)
endif()

add_executable(kfc-runtime-capture-ecs EXCLUDE_FROM_ALL "${profile_development_directory}/capture_legacy_ecs_registry.cpp")
target_compile_features(kfc-runtime-capture-ecs PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-capture-ecs PRIVATE NOMINMAX)
if(MSVC)
    target_compile_options(kfc-runtime-capture-ecs PRIVATE /W4 /EHsc)
endif()

# Game-specific profile investigation tools. These ship only in the development package.
add_executable(kfc-runtime-inspect-component-metadata EXCLUDE_FROM_ALL
    "${profile_development_directory}/inspect_component_metadata.cpp")
add_executable(kfc-runtime-live-entity-manager-sample EXCLUDE_FROM_ALL
    "${profile_development_directory}/checks/sample_entity_manager_registers.cpp")
add_executable(kfc-runtime-live-entity-managers EXCLUDE_FROM_ALL
    "${profile_development_directory}/capture_entity_managers.cpp")
add_executable(kfc-runtime-live-type-references EXCLUDE_FROM_ALL
    "${profile_development_directory}/capture_type_references.cpp")
foreach(tool_target
    kfc-runtime-inspect-component-metadata kfc-runtime-live-entity-manager-sample
    kfc-runtime-live-entity-managers kfc-runtime-live-type-references)
    target_compile_features(${tool_target} PRIVATE cxx_std_20)
    set_property(TARGET ${tool_target} PROPERTY MSVC_RUNTIME_LIBRARY
        "MultiThreaded$<$<CONFIG:Debug>:Debug>")
endforeach()
add_executable(kfc-runtime-audit-live-component-registry EXCLUDE_FROM_ALL
    "${profile_development_directory}/audit_live_component_registry.cpp")
target_compile_features(kfc-runtime-audit-live-component-registry PRIVATE cxx_std_20)
target_include_directories(kfc-runtime-audit-live-component-registry PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/json/single_include")
set_property(TARGET kfc-runtime-audit-live-component-registry PROPERTY MSVC_RUNTIME_LIBRARY
    "MultiThreaded$<$<CONFIG:Debug>:Debug>")

add_executable(kfc-runtime-verify-component-registry EXCLUDE_FROM_ALL
    "${profile_development_directory}/checks/verify_component_registry.cpp")
target_compile_features(kfc-runtime-verify-component-registry PRIVATE cxx_std_20)
target_include_directories(kfc-runtime-verify-component-registry PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/json/single_include")

foreach(tool_target
    kfc-runtime-dev
    kfc-runtime-capture-ecs
    kfc-runtime-inspect-component-metadata
    kfc-runtime-live-entity-manager-sample
    kfc-runtime-live-entity-managers
    kfc-runtime-live-type-references
    kfc-runtime-audit-live-component-registry
    kfc-runtime-verify-component-registry)
    foreach(configuration DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
        set_target_properties(${tool_target} PROPERTIES
            RUNTIME_OUTPUT_DIRECTORY_${configuration} "${CMAKE_BINARY_DIR}/tools")
    endforeach()
endforeach()

get_target_property(world_context_test_sources kfc-runtime SOURCES)
list(REMOVE_ITEM world_context_test_sources
    engine-context/session.cpp
    player-input/cursor.cpp
    voxels/cells.cpp
    building/removal.cpp
    building/placement.cpp
    building/spawn.cpp
    voxels/grids.cpp
)
add_executable(kfc-runtime-test-world-context EXCLUDE_FROM_ALL
    ../tests/world_context_test.cpp ${world_context_test_sources})
target_include_directories(kfc-runtime-test-world-context PRIVATE ${prop_test_includes})
target_compile_features(kfc-runtime-test-world-context PRIVATE cxx_std_20)
target_compile_definitions(kfc-runtime-test-world-context PRIVATE KFC_RUNTIME_BUILD WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(kfc-runtime-test-world-context PRIVATE bcrypt)
set_property(TARGET kfc-runtime-test-world-context PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
