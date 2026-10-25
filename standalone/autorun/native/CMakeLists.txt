cmake_minimum_required(VERSION 3.25)
project(enshrouded_autorun LANGUAGES CXX)

if(NOT WIN32 OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
    message(FATAL_ERROR "Autorun's native plugin requires Windows x64")
endif()

add_library(enshrouded-autorun SHARED
    autorun_plugin.cpp
    engine-hooks/allocate_near_code.cpp
    engine-hooks/scan_executable_signature.cpp
    engine-hooks/write_executable_bytes.cpp)
target_compile_features(enshrouded-autorun PRIVATE cxx_std_20)
target_include_directories(enshrouded-autorun PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}"
    "${CMAKE_CURRENT_LIST_DIR}/json/single_include")
target_compile_definitions(enshrouded-autorun PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
target_link_libraries(enshrouded-autorun PRIVATE bcrypt)
set_property(TARGET enshrouded-autorun PROPERTY
    MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
set_target_properties(enshrouded-autorun PROPERTIES OUTPUT_NAME "autorun" PREFIX "")

enable_testing()
add_executable(input-observation-test input_observation_test.cpp)
target_compile_features(input-observation-test PRIVATE cxx_std_20)
add_test(NAME input-observation COMMAND input-observation-test)
