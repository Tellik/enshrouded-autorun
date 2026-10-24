cmake_minimum_required(VERSION 3.25)
project(enshrouded_autorun LANGUAGES CXX)

if(NOT WIN32 OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "Autorun's native plugin requires Windows x64")
endif()

set(RUNTIME_NATIVE "${CMAKE_CURRENT_LIST_DIR}/../../../modules/game-runtime/native")
add_library(enshrouded-autorun SHARED
    autorun_plugin.cpp
    "${RUNTIME_NATIVE}/engine-hooks/allocate_near_code.cpp"
    "${RUNTIME_NATIVE}/engine-hooks/scan_executable_signature.cpp"
    "${RUNTIME_NATIVE}/engine-hooks/write_executable_bytes.cpp")
target_compile_features(enshrouded-autorun PRIVATE cxx_std_20)
target_include_directories(enshrouded-autorun PRIVATE
    "${RUNTIME_NATIVE}"
    "${RUNTIME_NATIVE}/json/single_include")
target_compile_definitions(enshrouded-autorun PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(enshrouded-autorun PRIVATE bcrypt)
set_property(TARGET enshrouded-autorun PROPERTY
    MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set_target_properties(enshrouded-autorun PROPERTIES OUTPUT_NAME "autorun" PREFIX "")
