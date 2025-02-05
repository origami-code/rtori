cmake_minimum_required(VERSION 3.23...3.30)
set(CMAKE_LINK_LIBRARIES_ONLY_TARGETS ON)

project(rtori-touchdesigner
    DESCRIPTION "Real-Time Origami Toolkit Touchdesigner Integration"
    LANGUAGES CXX
)

if(NOT TARGET rtori_core)
    add_subdirectory("../rtori-core-ext" ${CMAKE_CURRENT_BINARY_DIR}/rtori_core)
endif()

add_subdirectory(src)

# Minimum supported of Windows 10
# This means that every CPU should have SSE4.1
#    - https://learn.microsoft.com/en-gb/windows-hardware/design/minimum/supported/windows-10-ltsb-1507-supported-intel-processors
# Thus builds for windows should target that