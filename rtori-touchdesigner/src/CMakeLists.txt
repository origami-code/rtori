
set(MACOS_BUNDLE_PREFIX "art.orilab.rtori.td" CACHE STRING "macOS bundle prefixing")
set(MACOS_BUNDLE_FULLNAME_SUFFIX " (rtori plugin for TouchDesigner)" CACHE STRING "macOS bundle full name suffix")
set(COPYRIGHT_HUMAN "Ars Electronica Futurelab 2024-2025" CACHE STRING "Copyright notice")

function(configure_info_plist)
    cmake_parse_arguments(PARSE_ARGV 0 args
        ""
        "TARGET"
        ""
    )

    if(NOT TARGET ${arg_TARGET})
        message(FATAL_ERROR "No TARGET given")
    endif()

    configure_file("${CMAKE_CURRENT_FUNCTION_LIST_DIR}/Info.plist.in" "${BINARY_DIR}/Info.plist" NEWLINE_STYLE UNIX)
endfunction()

function(get_target_property_or VARIABLE TARGET PROPERTY DEFAULT)
    get_target_property(found ${arg_TARGET} MACOS_IDENTIFIER)

    if(NOT found)
        set(found ${DEFAULT})
    endif()

    set(${VARIABLE} ${found} PARENT_SCOPE)
endfunction()

function(prepare_plugin_plist)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""
        "TARGET"
        ""
    )

    get_target_property(BINARY_DIR ${arg_TARGET} BINARY_DIR)

    get_target_property_or(identifier_suffix ${arg_TARGET} MACOS_IDENTIFIER_SUFFIX "${arg_TARGET}")

    get_target_property_or(IDENTIFIER ${arg_TARGET} MACOS_IDENTIFIER "${MACOS_BUNDLE_PREFIX}.${identifier_suffix}")
    get_target_property_or(NAME_SHORT ${arg_TARGET} MACOS_NAME_SHORT "${arg_TARGET}")
    get_target_property_or(NAME_FULL ${arg_TARGET} MACOS_NAME_FULL "${NAME_SHORT} ${MACOS_BUNDLE_FULLNAME_SUFFIX}")
    get_target_property_or(VERSION_TRIPLET ${arg_TARGET} MACOS_VERSION_TRIPLET "0.0.0")
    get_target_property_or(VERSION_TRIPLET_WITH_BUILD ${arg_TARGET} MACOS_VERSION_TRIPLET_WITH_BUILD "0.0.0.0")
    get_target_property_or(VERSION_HUMAN ${arg_TARGET} MACOS_VERSION_HUMAN "25-02-08")
    get_target_property_or(COPYRIGHT_HUMAN ${arg_TARGET} MACOS_COPYRIGHT_HUMAN "${COPYRIGHT_HUMAN}")
    get_target_property_or(EXECUTABLE_NAME ${arg_TARGET} MACOS_EXECUTABLE_NAME "plugin.so")
    get_target_property_or(ICON_FILE ${arg_TARGET} MACOS_ICON_FILE "icon.icns")
endfunction()

function(prepare_plugin)
    cmake_parse_arguments(PARSE_ARGV 0 arg
        ""
        "TARGET;NAME"
        ""
    )

    if(MSVC)
        get_target_property(source_dir ${arg_TARGET} SOURCE_DIR)
        target_compile_options(${arg_TARGET} PRIVATE "/d1trimfile:${source_dir}")
        target_link_options(${arg_TARGET} PRIVATE "/PDBALTPATH:%_PDB")
    endif()

    if(NOT TARGET ${arg_TARGET})
        message(FATAL_ERROR "No TARGET given")
    endif()

    get_target_property(target_property ${arg_TARGET} TYPE)

    if(NOT(${target_property} STREQUAL "MODULE_LIBRARY"))
        message(FATAL_ERROR "TouchDesigner Plugins need to be configures as MODULE")
    endif()

    target_compile_definitions(${arg_TARGET} PRIVATE
        RTORI_TD_VERSION_MAJOR ${PROJECT_VERSION_MAJOR}
        RTORI_TD_VERSION_MINOR ${PROJECT_VERSION_MINOR}
        RTORI_TD_VERSION_PATCH ${PROJECT_VERSION_PATCH}
        RTORI_TD_VERSION_BUILD ${PROJECT_VERSION_TWEAK}
    )

    # Configure file extensions
    if(APPLE)
        # Also see https://github.com/DBraun/TD-Faust/blob/main/faust2touchdesigner/CMakeLists.txt
        set_target_properties(
            ${arg_TARGET}
            BUNDLE On # Built as a CFBundle (as a plugin)
            BUNDLE_EXTENSION "plugin" # As requested by Derivative
            SUFFIX ".so"
        )

        if(arg_NAME)
            set_target_properties(${arg_TARGET} MACOS_NAME_SHORT ${arg_NAME})
        endif()

        prepare_plugin_plist(TARGET ${arg_TARGET})
    elseif(WIN32)
        # TODO: Resource file hackery
    endif()

    # Configure destination
    # On windows, everything should go to the root of the install prefix
    install(
        TARGETS ${arg_TARGET}
        RUNTIME # For Windows
        DESTINATION "." # To the root
        BUNDLE
        DESTINATION "."
        LIBRARY # ???
        DESTINATION "."
    )

    if(MSVC)
        install(FILES "$<TARGET_PDB_FILE:${arg_TARGET}>" DESTINATION ".")
    endif()
endfunction()

add_subdirectory(common)
add_subdirectory(simulate)
add_subdirectory(fold)