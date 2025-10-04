
function(TouchDesigner_declareInterface)
    cmake_parse_arguments(
        PARSE_ARGV 0
        arg
        "REQUIRED"
        "TARGET;PATH"
        ""
    )
    
    if (NOT (EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/${arg_PATH}"))
        if(arg_REQUIRED)
            message(FATAL_ERROR "${arg_NAME} required but path given does not exist (checked <${arg_PATH}> relative to current source dir <${CMAKE_CURRENT_SOURCE_DIR}>)")
        endif()
        return()
    endif()

    cmake_path(GET arg_PATH PARENT_PATH parent_dir)

    add_library(${arg_TARGET} INTERFACE)
    message(STATUS "Adding ${arg_TARGET} with path ${arg_PATH} (base dir ${parent_dir})")
    target_sources(${arg_TARGET}
        INTERFACE
        FILE_SET HEADERS
        TYPE HEADERS
        BASE_DIRS ${parent_dir}
        FILES
        ${arg_PATH}
    )
endfunction()

function(TouchDesigner_declareOperatorInterface)
    cmake_parse_arguments(
        PARSE_ARGV 0
        arg
        "REQUIRED"
        "OP;PREFIX"
        ""
    )

    string(TOLOWER ${arg_OP} op_path_component)
    string(TOUPPER ${arg_OP} op_name)
    
    set(target_name "${arg_PREFIX}${arg_OP}")
    TouchDesigner_declareInterface(
        TARGET ${target_name}
        NAME "${op_name}"
        PATH "${op_path_component}/${op_name}_CPlusPlusBase.h"
    )
    if (NOT TARGET ${target_name})
        if(arg_REQUIRED)
            message(FATAL_ERROR "${arg_OP} required but not present in tree")
        endif()
        return()
    endif()

    set(common_target_name ${arg_PREFIX}Common)
    target_link_libraries(${target_name} INTERFACE ${common_target_name})
    get_target_property(td_version ${common_target_name} TOUCHDESIGNER_VERSION)
    get_target_property(td_branch ${common_target_name} TOUCHDESIGNER_BRANCH)
    get_target_property(td_python_version ${common_target_name} TOUCHDESIGNER_PYTHON_VERSION)

    set_target_properties(${target_name} PROPERTIES
        TOUCHDESIGNER_OPERATOR_KIND ${arg_OP}

        # Derived
        TOUCHDESIGNER_VERSION ${td_version}
        TOUCHDESIGNER_BRANCH ${td_branch}
        TOUCHDESIGNER_PYTHON_VERSION ${td_python_version}
    )
endfunction()

set(TouchDesigner_Versions "" CACHE STRING "Supported touchdesigner versions" FORCE)
set(TouchDesigner_Branches "" CACHE STRING "Supported touchdesigner branches" FORCE)
set(TouchDesigner_Versions "" CACHE STRING "Supported touchdesigner versions" FORCE)

function(TouchDesigner_declareAPI)
    cmake_parse_arguments(PARSE_ARGV 0 arg "" "VERSION;BRANCH;PYTHON_VERSION" "")

    if (NOT arg_VERSION)
        cmake_path(GET CMAKE_CURRENT_SOURCE_DIR FILENAME version)
    else()
        set(version ${arg_VERSION})
    endif()

    # Add it to the common list of versions
    set(tmp_list $CACHE{TouchDesigner_Versions})
    list(APPEND tmp_list ${version})
    set(TouchDesigner_Versions "${tmp_list}" CACHE STRING "Supported touchdesigner versions" FORCE)

    # Add it to the common list of branches
    if (NOT (arg_BRANCH IN_LIST CACHE{TouchDesigner_Branches}))
        set(tmp_list $CACHE{TouchDesigner_Branches})
        list(APPEND tmp_list ${arg_BRANCH})
        set(TouchDesigner_Branches "${tmp_list}" CACHE STRING "Supported touchdesigner branches" FORCE)

        set(TouchDesigner_Branch_${arg_BRANCH}_Versions "" CACHE STRING "Supported TouchDesigner versions for branch ${arg_BRANCH}" FORCE)
    endif()

    # Add it to the version list for the given branch
    set(tmp_list $CACHE{TouchDesigner_Branch_${arg_BRANCH}_Versions})
    list(APPEND tmp_list ${version})
    set(TouchDesigner_Branch_${arg_BRANCH}_Versions "${tmp_list}" CACHE STRING "Supported TouchDesigner versions for branch ${arg_BRANCH}" FORCE)

    # The prefix then comes from the version
    set(prefix "TouchDesigner_${version}_")

    # The common holds the source of truth for the td version, branch & python version
    TouchDesigner_declareInterface(
        TARGET ${prefix}Common
        PATH common/CPlusPlus_Common.h
        REQUIRED
    )

    set_target_properties(${prefix}Common PROPERTIES
        TOUCHDESIGNER_VERSION ${version}
        TOUCHDESIGNER_BRANCH ${arg_BRANCH}
        TOUCHDESIGNER_PYTHON_VERSION ${arg_PYTHON_VERSION}
    )
    add_library(TouchDesigner::${version}::Common ALIAS ${prefix}Common)


    # Each component is then derived from their name
    set(operator_kinds "SOP;TOP;CHOP;DAT;POP")
    foreach (kind ${operator_kinds})
        TouchDesigner_declareOperatorInterface(OP ${kind} PREFIX ${prefix})

        set(target_name  ${prefix}${kind})
        if (TARGET ${target_name})
            add_library(TouchDesigner::${version}::${kind} ALIAS ${target_name})
        endif()
    endforeach()
endfunction()

add_subdirectory(2020.28110)
add_subdirectory(2021.16960)
add_subdirectory(2022.35320)
add_subdirectory(2023.12480)
add_subdirectory(2025.30960)

set(TouchDesigner_DefaultVersion 2023.12480 CACHE INTERNAL "Default touchdesigner version" FORCE)

function(TouchDesigner_getBranchForVersion)
    cmake_parse_arguments(PARSE_ARGV 0 arg "" "VERSION;OUTPUT_VARIABLE" "")
    
    set(target_name "TouchDesigner_${version}_Common")
    get_target_property(branch ${target_name} TOUCHDESIGNER_BRANCH)

    set(${arg_OUTPUT_VARIABLE} ${branch} PARENT_SCOPE)
endfunction()

foreach (branch ${TouchDesigner_Branches})
    foreach (version ${TouchDesigner_Branch_${branch}_Versions})
        if (NOT highest)
            set(highest ${version})
        endif()
        if (NOT lowest)
            set(lowest ${version})
        endif()
        
        string(COMPARE GREATER "${version}" "${highest}" is_higher)
        if (is_higher)
            set(highest ${version})
        endif()

        string(COMPARE LESS "${version}" "${lowest}" is_lower)
        if (is_lower)
            set(lowest ${version})
        endif()
    endforeach()
    

    set(TouchDesigner_Branch_${branch}_Highest ${highest})
    set(TouchDesigner_Branch_${branch}_Lowest ${lowest})

    unset(highest)
    unset(lowest)
endforeach()

foreach(version ${TouchDesigner_Versions})
    TouchDesigner_getBranchForVersion(VERSION ${version} OUTPUT_VARIABLE branch)

    if ("${version}" STREQUAL "${TouchDesigner_Branch_${branch}_Highest}")
        add_library(TouchDesigner::Branch::${branch}::Common ALIAS TouchDesigner_${version}_Common)
        if (TARGET TouchDesigner_${version}_SOP)
            add_library(TouchDesigner::${branch}::SOP ALIAS TouchDesigner_${version}_SOP)
        endif()
        if (TARGET TouchDesigner_${version}_POP)
            add_library(TouchDesigner::Branch::${branch}::POP ALIAS TouchDesigner_${version}_POP)
        endif()
        if (TARGET TouchDesigner_${version}_DAT)
            add_library(TouchDesigner::Branch::${branch}::DAT ALIAS TouchDesigner_${version}_DAT)
        endif()
        if (TARGET TouchDesigner_${version}_CHOP)
            add_library(TouchDesigner::Branch::${branch}::CHOP ALIAS TouchDesigner_${version}_CHOP)
        endif()
        if (TARGET TouchDesigner_${version}_TOP)
            add_library(TouchDesigner::Branch::${branch}::TOP ALIAS TouchDesigner_${version}_TOP)
        endif()
    else()
        message(FATAL_ERROR "branch: ${branch}, highest: ${TouchDesigners_${branch}_Highest}")
    endif()
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