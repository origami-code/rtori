set(TouchDesigner_Versions "2023;2025" CACHE INTERNAL "Supported touchdesigner versions" FORCE)
set(TouchDesigner_DefaultVersion 2023 CACHE INTERNAL "Default touchdesigner version" FORCE)

function(TouchDesigner_declareInterface)
    cmake_parse_arguments(PARSE_ARGV 0 arg "LINK_COMMON;REQUIRED" "NAME;PATH;VERSION" "")
    
    set(full_path ${arg_VERSION}/${arg_PATH})
    if (NOT (EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/${full_path}))
        if(arg_REQUIRED)
            message(FATAL_ERROR "${arg_NAME} required but path given does not exist (checked <${full_path}> from version ${arg_VERSION} and subpath ${arg_PATH})")
        endif()
        return()
    endif()

    set(target_name "TouchDesigner_${arg_VERSION}_${arg_NAME}")
    add_library(${target_name} INTERFACE)
    target_sources(${target_name}
        INTERFACE
        FILE_SET HEADERS
        TYPE HEADERS
        BASE_DIRS ${arg_VERSION}/${arg_SUBDIR}
        FILES
        ${full_path}
    )
    set_target_properties(${target_name} PROPERTIES
        TOUCHDESIGNER_VERSION ${arg_VERSION}
        TOUCHDESIGNER_API_COMPONENT ${arg_NAME}
    )
    if (arg_LINK_COMMON)
        target_link_libraries(${target_name} INTERFACE TouchDesigner_${arg_VERSION}_Common)
    endif()
endfunction()

foreach(version ${TouchDesigner_Versions})
    TouchDesigner_declareInterface(
        VERSION ${version}
        NAME Common
        PATH common/CPlusPlus_Common.h
        REQUIRED
    )
    TouchDesigner_declareInterface(
        VERSION ${version}
        NAME SOP
        PATH sop/SOP_CPlusPlusBase.h
        LINK_COMMON
    )
    TouchDesigner_declareInterface(
        VERSION ${version}
        NAME POP
        PATH pop/POP_CPlusPlusBase.h
        LINK_COMMON
    )
    TouchDesigner_declareInterface(
        VERSION ${version}
        NAME DAT
        PATH dat/DAT_CPlusPlusBase.h
        LINK_COMMON
    )
    TouchDesigner_declareInterface(
        VERSION ${version}
        NAME CHOP
        PATH chop/CHOP_CPlusPlusBase.h
        LINK_COMMON
    )
    TouchDesigner_declareInterface(
        VERSION ${version}
        NAME TOP
        PATH top/TOP_CPlusPlusBase.h
        LINK_COMMON
    )
endforeach()

add_library(TouchDesigner::Common ALIAS TouchDesigner_${TouchDesigner_DefaultVersion}_Common)
if (TARGET TouchDesigner_${TouchDesigner_DefaultVersion}_SOP)
    add_library(TouchDesigner::SOP ALIAS TouchDesigner_${TouchDesigner_DefaultVersion}_SOP)
endif()
if (TARGET TouchDesigner_${TouchDesigner_DefaultVersion}_POP)
    add_library(TouchDesigner::POP ALIAS TouchDesigner_${TouchDesigner_DefaultVersion}_POP)
endif()
if (TARGET TouchDesigner_${TouchDesigner_DefaultVersion}_DAT)
    add_library(TouchDesigner::DAT ALIAS TouchDesigner_${TouchDesigner_DefaultVersion}_DAT)
endif()
if (TARGET TouchDesigner_${TouchDesigner_DefaultVersion}_CHOP)
    add_library(TouchDesigner::CHOP ALIAS TouchDesigner_${TouchDesigner_DefaultVersion}_CHOP)
endif()
if (TARGET TouchDesigner_${TouchDesigner_DefaultVersion}_TOP)
    add_library(TouchDesigner::TOP ALIAS TouchDesigner_${TouchDesigner_DefaultVersion}_TOP)
endif()