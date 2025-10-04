
# Extract the toolchain architecture that is currently considered.
# And error out in case of multi-build or unknown architectures in the generators which are multi-arch.
# As well as guess what is the current architecture otherwise, in terms of rust architectures.
function(rtori_guess_cmake_toolchain_abi_arch)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""
        "OUTPUT_VARIABLE"
        ""
    )

    # We special case the multi-arch generators (MSVC & XCode)
    if(XCODE)
        list(LENGTH CMAKE_OSX_ARCHITECTURES arch_count)

        if(${arch_count} GREATER 1)
            message(FATAL_ERROR "Only a single architecture can be targeted.")
        elseif(${arch_count} EQUAL 0)
            message(WARNING "On XCode but no CMAKE_OSX_ARCHITECTURES set, selecting current processor.")
        else()
            if(CMAKE_OSX_ARCHITECTURES STREQUAL "arm64")
                set(arch "aarch64")
            elseif(CMAKE_OSX_ARCHITECTURES STREQUAL "x86_64")
                set(arch "x86_64")
            else()
                message(WARNING "Unknown architecture ${CMAKE_OSX_ARCHITECTURES}")
            endif()

            set(expected_triple "${arch}-apple-darwin")
        endif()
    elseif(CMAKE_GENERATOR MATCHES "Visual Studio")
        if(CMAKE_VS_PLATFORM_NAME)
            if(CMAKE_VS_PLATFORM_NAME STREQUAL "x64")
                set(arch "x86_64")
            elseif(CMAKE_VS_PLATFORM_NAME STREQUAL "win32")
                set(arch "x86")
            elseif(CMAKE_VS_PLATFORM_NAME STREQUAL "arm64")
                set(arch "aarch64")
            elseif(CMAKE_VS_PLATFORM_NAME STREQUAL "arm64ec")
                set(arch "arm64ec")
            else()
                message(WARNING "using visual studio, CMAKE_VS_PLATFORM_NAME detected but unknown platform: ${CMAKE_VS_PLATFORM_NAME}")
            endif()
        else()
            if(CMAKE_SYSTEM_PROCESSOR STREQUAL "AMD64")
                set(arch "x86_64")
            elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "ARM64")
                set(arch "aarch64")
            endif()
        endif()

        if(NOT arch)
            message(WARNING "using MSVC, yet targeting an unknown platform ${CMAKE_SYSTEM_PROCESSOR}")
        else()
            set(expected_triple "${arch}-pc-windows-msvc")
        endif()
    else()
        # We assume we're targeting a single architecture here
        # So we're going by CMAKE_SYSTEM_PROCESSOR

        if((CMAKE_SYSTEM_PROCESSOR STREQUAL "AMD64")
            OR(CMAKE_SYSTEM_PROCESSOR STREQUAL "x86_64"))
            set(arch "x86_64")
        elseif((CMAKE_SYSTEM_PROCESSOR STREQUAL "i686")
            OR(CMAKE_SYSTEM_PROCESSOR STREQUAL "x86")
            OR(CMAKE_SYSTEM_PROCESSOR STREQUAL "win32")
        )
            set(arch "i686")
        elseif((CMAKE_SYSTEM_PROCESSOR STREQUAL "ARM64") OR(CMAKE_SYSTEM_PROCESSOR STREQUAL "arm64"))
            set(arch "aarch64")
        elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "arm64ec")
            set(arch "arm64ec")
        elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "wasm32")
            set(arch "wasm32")
        elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "powerpc")
            set(arch "powerpc")
        else()
            message(WARNING "couldn't match on the sytem processor, defaulting to the value of CMAKE_SYSTEM_PROCESSOR: ${CMAKE_SYSTEM_PROCESSOR}")
            set(arch "${CMAKE_SYSTEM_PROCESSOR}")
        endif()
    endif()

    set(${arg_OUTPUT_VARIABLE} ${arch} PARENT_SCOPE)
endfunction()

# Tries to find the os, in rust naming, that is being targeted
# See https://cmake.org/cmake/help/latest/variable/CMAKE_SYSTEM_NAME.html
# And https://crates.io/crates/platforms
function(rtori_guess_cmake_toolchain_abi_os)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""
        "OUTPUT_VARIABLE"
        ""
    )

    if((CMAKE_SYSTEM_NAME MATCHES "Windows") OR(CMAKE_SYSTEM_NAME MATCHES MSYS))
        set(os "windows")
    elseif(CMAKE_SYSTEM_NAME MATCHES "Linux")
        set(os "linux")
    elseif(CMAKE_SYSTEM_NAME MATCHES "Darwin")
        set(os "macos")
    elseif(CMAKE_SYSTEM_NAME MATCHES "Fuchsia")
        set(os "fuchsia")
    elseif(CMAKE_SYSTEM_NAME MATCHES "Android")
        set(os "android")
    elseif(CMAKE_SYSTEM_NAME MATCHES "iOS")
        set(os "ios")
    elseif(CMAKE_SYSTEM_NAME MATCHES "Emscripten")
        set(os "emscripten")
    elseif(CMAKE_SYSTEM_NAME MATCHES "WASI")
        set(os "wasi")
    elseif(CMAKE_SYSTEM_NAME MATCHES "Generic")
        set(os "none")
    else()
        message(WARNING "Unknown CMAKE_SYSTEM_NAME ${CMAKE_SYSTEM_NAME}, trying to match as is")
        set(os "${CMAKE_SYSTEM_NAME}")
    endif()

    set(${arg_OUTPUT_VARIABLE} ${os} PARENT_SCOPE)
endfunction()

# Tries to guess the environment (for ex. MSVC vs GNU on windows, musl vs gnu on linux, ...)
# Needs to have access to the previously guessed architecture & os 
function(rtori_guess_cmake_toolchain_abi_env)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""
        "ARCH;OS;OUTPUT_VARIABLE"
        ""
    )

    # Checks to make sure the library matches the target ABI
    if(arg_OS STREQUAL "Windows")
        if((CMAKE_C_SIMULATE_ID STREQUAL "MSVC") OR(CMAKE_C_COMPILER_ID STREQUAL "MSVC"))
            set(env "msvc")
        else()
            set(env "gnu")
        endif()
    elseif(arg_OS STREQUAL "Linux")
        # We cannot discriminate between GNU & MUSL it seems
        set(env "gnu")
    else()
        set(env "none")
    endif()

    set(${arg_OUTPUT_VARIABLE} ${env} PARENT_SCOPE)
endfunction()

# Guess the current system/toolchain ABI targeted by this invocation of cmake,
# using the naming convention used by rust (close to clang/llvm).
# See rtori_guess_cmake_toolchain_abi_arch, rtori_guess_cmake_toolchain_abi_os, rtori_guess_cmake_toolchain_abi_env
# See https://crates.io/crates/platforms for details on the convention
function(rtori_guess_cmake_toolchain_abi)
    include(CMakeParseArguments)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""
        "ARCH;OS;ENV"
        ""
    )
    
    rtori_guess_cmake_toolchain_abi_arch(OUTPUT_VARIABLE arch)
    rtori_guess_cmake_toolchain_abi_os(OUTPUT_VARIABLE os)
    rtori_guess_cmake_toolchain_abi_env(ARCH ${arch} OS ${os} OUTPUT_VARIABLE env)

    set(${arg_ARCH} ${arch} PARENT_SCOPE)
    set(${arg_OS} ${os} PARENT_SCOPE)
    set(${arg_ENV} ${env} PARENT_SCOPE)
endfunction()

# Given the rust triple components (arch, os, env) with which rtori-core was built,
# ensure that it is compatible with the toolchain ABI targeted by cmake
# as returned by [rtori_core_guess_cmake_toolchain_abi]
# See https://crates.io/crates/platforms for details on the convention
function(rtori_check_abi)
    include(CMakeParseArguments)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""    
        "ARCH;OS;ENV;TRIPLE"
        ""
    )

    set(built_arch ${arg_ARCH})
    set(built_os ${arg_OS})
    set(built_env ${arg_ENV})
    set(built_triple ${arg_TRIPLE})

    rtori_guess_cmake_toolchain_abi(
        ARCH detected_arch
        OS detected_os
        ENV detected_env
    )

    if(
        NOT(detected_arch STREQUAL built_arch)
    )
        message(WARNING "Unsupported target architecture: building for ${detected_arch} but rtori-core built for ${built_arch} (specifically ${built_triple}). Continuing but this might lead to compile or linking errors, or worse.")
    endif()

    if(
        NOT(detected_os STREQUAL built_os)
    )
        message(WARNING "Unsupported target os: building for ${detected_os} but rtori-core built for ${built_os} (specifically ${built_triple}). Continuing but this might lead to compile or linking errors, or worse.")
    endif()

    if(
        NOT(detected_env STREQUAL built_env)
    )
        message(WARNING "Unsupported target env: building for ${detected_env} but rtori-core built for ${built_env} (specifically ${built_triple}). Continuing but this might lead to compile or linking errors, or worse.")
    endif()

    
endfunction()