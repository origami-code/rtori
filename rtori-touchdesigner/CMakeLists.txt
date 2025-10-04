cmake_minimum_required(VERSION 3.23...3.30)
set(CMAKE_LINK_LIBRARIES_ONLY_TARGETS ON)

project(rtori-touchdesigner
    DESCRIPTION "Real-Time Origami Toolkit Touchdesigner Integration"
    LANGUAGES CXX
)

set(RTORI_TD_TARGETED_VERSIONS "2023;2025" CACHE STRING "Targeted versions of TouchDesigner")
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")

if((NOT cxx_std_20 IN_LIST CMAKE_CXX_COMPILE_FEATURES) AND(NOT TARGET fmt))
    # We test the compiler, as we provide the fmt library (https://github.com/fmtlib/fmt) if it does not support std::format
    include(FetchContent)
    FetchContent_Declare(
        fmt
        GIT_REPOSITORY github.com/fmtlib/fmt
        GIT_TAG 11.1.3
    )
    FetchContent_MakeAvailable(fmt)
endif()

if(APPLE)
    include(ext/cmake-macos/cmake-macos.cmake)
endif()

if(NOT TARGET rtori_core::CXXHeaders)
    message(FATAL_ERROR "rtori_core::CXXHeaders not defined. If running a standalone build, use the CMakeLists in the parent directory.")
endif()

add_subdirectory(src)

# Minimum supported of Windows 10
# This means that every CPU should have SSE4.1
#    - https://learn.microsoft.com/en-gb/windows-hardware/design/minimum/supported/windows-10-ltsb-1507-supported-intel-processors
# Thus builds for windows should target that