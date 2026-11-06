cmake_minimum_required(VERSION 3.13)
include($ENV{PICO_SDK_PATH}/external/pico_sdk_import.cmake)
project(orion_dummy C CXX ASM)
pico_sdk_init()

add_executable(orion_dummy orion_dummy.c)
target_include_directories(orion_dummy PRIVATE ${CMAKE_CURRENT_LIST_DIR})
target_link_libraries(orion_dummy pico_stdlib tinyusb_device tinyusb_board)
pico_add_extra_outputs(orion_dummy)
