set(RTOriSimulateSOP_Version_Major 0)
set(RTOriSimulateSOP_Version_Minor 0)

list(APPEND RTOriSimulateSOP_Sources
    src/RTOriSimulateSOP.cpp
    src/RTOriSimulateSOP.hpp
)

add_library(
    RTOriSimulateSOP
    SHARED
    ${RTOriSimulateSOP_Sources}
)
target_compile_features(RTOriSimulateSOP PRIVATE cxx_std_20)
target_link_libraries(
    RTOriSimulateSOP
    PUBLIC
    TouchDesigner_Common
    TouchDesigner_SOP
)

target_compile_definitions(
    RTOriSimulateSOP
    PRIVATE
    RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MAJOR=${RTOriSimulateSOP_Version_Major}
    RTORI_TOUCHDESIGNER_SIMULATE_SOP_VERSION_MINOR=${RTOriSimulateSOP_Version_Minor}
)

if(CMAKE_SYSTEM_NAME MATCHES "apple")
    target_link_libraries(RTOriSimulateSOP PRIVATE
        "-bundle"
    )
    set_target_properties(RTOriSimulateSOP PROPERTIES
        SUFFIX ".dylib"
    )
endif()