list(APPEND RTOriSimulateSOP_Sources
    src/RTOriSimulateSOP.cpp
    src/RTOriSimulateSOP.hpp
)

add_library(
    RTOriSimulateSOP
    SHARED
    ${RTOriSimulateSOP_Sources}
)
target_link_libraries(
    RTOriSimulateSOP
    PUBLIC
    TouchDesigner_Common
    TouchDesigner_SOP
)

if(CMAKE_SYSTEM_NAME MATCHES "apple")
    target_link_libraries(RTOriSimulateSOP PRIVATE
        "-bundle"
    )
    set_target_properties(RTOriSimulateSOP PROPERTIES
        SUFFIX ".dylib"
    )
endif()