cmake_minimum_required(VERSION 3.25)
project(wand_durability LANGUAGES CXX)
enable_testing()
add_executable(attack-cost-test attack_cost_test.cpp)
target_compile_features(attack-cost-test PRIVATE cxx_std_20)
add_test(NAME attack-cost COMMAND attack-cost-test)
add_executable(export-hook-code export_hook_code.cpp)
target_compile_features(export-hook-code PRIVATE cxx_std_20)

if(WIN32 AND CMAKE_SIZEOF_VOID_P EQUAL 8)
    add_library(wand-durability SHARED wand_durability_plugin.cpp
        engine-hooks/allocate_near_code.cpp
        engine-hooks/scan_executable_signature.cpp
        engine-hooks/write_executable_bytes.cpp)
    target_compile_features(wand-durability PRIVATE cxx_std_20)
    target_include_directories(wand-durability PRIVATE "${CMAKE_CURRENT_LIST_DIR}" json/single_include)
    target_compile_definitions(wand-durability PRIVATE WIN32_LEAN_AND_MEAN NOMINMAX)
    target_link_libraries(wand-durability PRIVATE bcrypt)
    set_property(TARGET wand-durability PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
    set_target_properties(wand-durability PROPERTIES PREFIX "")
endif()
