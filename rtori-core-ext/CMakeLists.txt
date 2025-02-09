# 3.23 for FILE_SET HEADERS support
cmake_minimum_required(VERSION 3.23...3.30)

project(rtori-core
    DESCRIPTION "Real-Time Origami Toolkit"
)

set(Rust_TOOLCHAIN "nightly" CACHE STRING "Requires nightly rust")
include(FetchContent)

if(XCODE)
    list(LENGTH CMAKE_OSX_ARCHITECTURES arch_count)
    if(${arch_count} GREATER 1)
        message(FATAL_ERROR "Cannot generate more than one architecture")
    elseif(${arch_count} EQUAL 0)
        message(WARNING "Generating for curren processor")
    else()
        if (CMAKE_OSX_ARCHITECTURES STREQUAL "arm64")
            set(rust_arch "aarch64")
        elseif(CMAKE_OSX_ARCHITECTURES STREQUAL "x86_64")
            set(rust_arch "x86_64")
        else()
            message(FATAL_ERROR "Unknown architecture ${CMAKE_OSX_ARCHITECTURES}")
        endif()

        set(Rust_CARGO_TARGET "${rust_arch}-apple-darwin")
    endif()

elseif(MSVC)
    if(CMAKE_VS_PLATFORM_NAME)
        if(CMAKE_VS_PLATFORM_NAME STREQUAL "x64")
            set(rust_arch "x86_64")
        elseif(CMAKE_VS_PLATFORM_NAME STREQUAL "win32")
            set(rust_arch "i686")
        elseif(CMAKE_VS_PLATFORM_NAME STREQUAL "arm64")
            set(rust_arch "aarch64")
        elseif(CMAKE_VS_PLATFORM_NAME STREQUAL "arm64ec")
            set(rust_arch "arm64ec")
        else()
            message(FATAL_ERROR "CMAKE_VS_PLATFORM_NAME detected but unknown platform: ${CMAKE_VS_PLATFORM_NAME}")
        endif()
    else()
        if(CMAKE_SYSTEM_PROCESSOR STREQUAL "AMD64")
            set(rust_arch "x86_64")
        elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "ARM64")
            set(rust_arch "aarch64")
        endif()
    endif()

    if(NOT rust_arch)
        message(FATAL_ERROR "Unknown platform")
    endif()

    set(Rust_CARGO_TARGET "${rust_arch}-pc-windows-msvc")
endif()

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG origin/master # Optionally specify a commit hash, version tag or branch here
)
FetchContent_MakeAvailable(Corrosion)

if(NOT Rust_IS_NIGHTLY)
    message(FATAL_ERROR "Needs nightly rust to build")
endif()

# Import targets defined in a package or workspace manifest `Cargo.toml` file
corrosion_import_crate(

    # We import from the workspace
    MANIFEST_PATH "../Cargo.toml"

    # But only that crate we're interested in
    CRATES "rtori-core-ext"

    # In DLL only
    # CRATE_TYPES cdylib
)

corrosion_experimental_cbindgen(
    TARGET "rtori_core"
    HEADER_NAME "rtori_core.h"
    FLAGS --config "${CMAKE_CURRENT_LIST_DIR}/cbindgen_c.toml" --lang c
)

corrosion_experimental_cbindgen(
    TARGET "rtori_core"
    HEADER_NAME "rtori_core.hpp"
    FLAGS --config "${CMAKE_CURRENT_LIST_DIR}/cbindgen_cpp.toml" --lang c++
)

target_sources(rtori_core INTERFACE FILE_SET HEADERS TYPE HEADERS BASE_DIRS include FILES include/rtori/attributes.h)