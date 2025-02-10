
set(MACOS_BUNDLE_PREFIX "art.orilab.rtori.td" CACHE STRING "macOS bundle prefixing")
set(MACOS_BUNDLE_FULLNAME_SUFFIX " (rtori plugin for TouchDesigner)" CACHE STRING "macOS bundle full name suffix")
set(COPYRIGHT_HUMAN "Ars Electronica Futurelab 2024-2025" CACHE STRING "Copyright notice")
set(MACOS_MINIMUM_SYSTEM_VERSION "10.15" CACHE INTERNAL "minimum macOS version supported, should be in sync with TouchDesigner")
set(MACOS_CODE_SIGNING_IDENTITY "alexandre-fl-macos" CACHE STRING "code signing identity to use for signing the artefacts on macOS")

if (CMAKE_OSX_DEPLOYMENT_TARGET GREATER MACOS_MINIMUM_SYSTEM_VERSION)
    message(FATAL_ERROR "We should be targeting versipn ${MACOS_MINIMUM_SYSTEM_VERSION} (synced with TouchDesigner), but set deployment target is over : ${CMAKE_OSX_DEPLOYMENT_TARGET}")
endif()

# Hide inline functions
set(CMAKE_VISIBILITY_INLINES_HIDDEN on)

# Hide symbols from shared library exports unless explicitely exposed
set(CMAKE_SHARED_LIBRARY_ENABLE_EXPORTS On)

if(MSVC)
    add_compile_options("/W4") # TODO: Add /WX
endif()

function(get_target_property_or VARIABLE TARGET PROPERTY DEFAULT)
    get_target_property(found ${arg_TARGET} MACOS_IDENTIFIER)

    if(NOT found)
        set(found ${DEFAULT})
    endif()

    set(${VARIABLE} ${found} PARENT_SCOPE)
endfunction()

# Sets up a MODULE target to work as a TouchDesigner plugin
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
        message(FATAL_ERROR "TouchDesigner Plugins need to be configured as MODULE")
    endif()

    target_compile_definitions(${arg_TARGET} PRIVATE
        RTORI_TD_VERSION_MAJOR ${PROJECT_VERSION_MAJOR}
        RTORI_TD_VERSION_MINOR ${PROJECT_VERSION_MINOR}
        RTORI_TD_VERSION_PATCH ${PROJECT_VERSION_PATCH}
        RTORI_TD_VERSION_BUILD ${PROJECT_VERSION_TWEAK}
    )

    set_target_properties(
        ${arg_TARGET} PROPERTIES

        # Visibility Control
        CXX_VISIBILITY_PRESET hidden
    )

    # Configure file extensions
    if(APPLE)
        list(APPEND rpath
            # Right next to the plugin so
            "@loader_path/"

            # In the plugin bundle's Contents/Frameworks
            "@loader_path/../Frameworks"

            # Right outside the plugin bundle, side to side
            # To get out of the bundle we need  <../../../> as we're in <MyPlugin.plugin/Contents/MacOS/MyPlugin.so>
            "@loader_path/../../../lib"

            # In the frameworks from TouchDesigner
            "@executable_path/../Frameworks"

            # Globally installed
            # TODO
        )

        # First, we want a .so without prefix, but still with a suffix (that's personal preference)
        # Like this: RTOriMyOP.so
        set_target_properties(
            ${arg_TARGET}
            PROPERTIES
            SUFFIX ".so"
            PREFIX ""
            MACOSX_RPATH On
            INSTALL_RPATH "${rpath}"
            BUILD_WITH_INSTALL_RPATH On
        )

        get_target_property(target_name ${arg_TARGET} NAME)
        set(bundle_target "${target_name}_bundle")
        set(bundle_filename "${target_name}.plugin")
        set(bundle_path "${CMAKE_CURRENT_BINARY_DIR}/${bundle_filename}")
        set(dso_path "${bundle_path}/Contents/MacOS/${target_name}.so")
        file(
            MAKE_DIRECTORY "${bundle_path}/Contents/MacOS"
        )

        # Generate the plist
        # This is done at generation time, not configure time
        set(plist_path "${bundle_path}/Contents/Info.plist")
        add_macos_bundle_info(
            DESTINATION "${plist_path}"
            TARGET ${arg_TARGET}
            PROPERTIES
                CFBundleDisplayName "${target_name} (rtori plugin for TouchDesigner)"
                CFBundleIdentifier "art.orilab.rtori.td.${target_name}"
                CFBundleShortVersionString "0.0.0"
                CFBundleVersion "0.0.0.0"
                CFBundleLongVersionString "25-02-08"
                LSMinimumSystemVersion "${MACOS_MINIMUM_SYSTEM_VERSION}"
                NSHumanReadableCopyright "All rights reserved © Ars Electronica Futurelab 2024-2025"
        )

        # Place the executable at the right place
        add_custom_command(
            OUTPUT "${dso_path}"
            DEPENDS ${arg_TARGET}
            COMMAND ${CMAKE_COMMAND} -E copy_if_different "$<TARGET_FILE:${arg_TARGET}>" "${dso_path}"
        )
        
        # Sign it
        set(bundle_signature_target "${target_name}_sign")
        code_sign_macos(${bundle_signature_target}
            PATH "${bundle_path}"
            IDENTITY ${MACOS_CODE_SIGNING_IDENTITY}
            DEPENDS "${dso_path}" 
        )

        add_custom_target(
            ${bundle_target} ALL
            COMMENT
                "Crafting bundle for ${arg_TARGET}"
            DEPENDS
                "${dso_path}" "${plist_path}" ${arg_TARGET} ${bundle_signature_target}
        )

        install(
            DIRECTORY "${bundle_path}" DESTINATION "."
        )

    elseif(WIN32)
        # TODO: Resource file hackery

        # Configure destination
        # On windows, everything should go to the root of the install prefix
        install(
            TARGETS ${arg_TARGET}
            RUNTIME # For Windows
            DESTINATION "." # To the root
            LIBRARY # ???
            DESTINATION "."
        )
    else()
        message(FATAL_ERROR "Unsupported platform")
    endif()

    if(MSVC)
        install(FILES "$<TARGET_PDB_FILE:${arg_TARGET}>" DESTINATION "." COMPONENT Symbols)
    endif()
endfunction()

add_subdirectory(common)
add_subdirectory(simulate)
add_subdirectory(fold)

include(CPack)