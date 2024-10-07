# 3.23 for FILE_SET HEADERS support
cmake_minimum_required(VERSION 3.23...3.30)

project(rtori-core
    DESCRIPTION "Real-Time Origami Toolkit"
)

set(Rust_TOOLCHAIN "nightly" CACHE STRING "Requires nightly rust")
include(FetchContent)

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

target_sources(rtori_core INTERFACE FILE_SET HEADERS TYPE HEADERS BASE_DIRS ${CMAKE_CURRENT_SOURCE_DIR}/include FILES ${CMAKE_CURRENT_SOURCE_DIR}/include/rtori/attributes.h)