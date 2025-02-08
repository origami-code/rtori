list(APPEND RTOriSimulateSOP_Sources
    src/RTOriSimulateSOP.cpp
    src/RTOriSimulateSOP.hpp
)

add_library(
    RTOriSimulateSOP
    MODULE
    ${RTOriSimulateSOP_Sources}
)
target_compile_features(RTOriSimulateSOP PRIVATE cxx_std_20)
target_link_libraries(
    RTOriSimulateSOP

    PUBLIC

    TouchDesigner_Common
    TouchDesigner_SOP
    RTOriSimulateCommon
    RTOriCommon # Temporary, during transition to using only RTOriSimulateCommon
)

prepare_plugin(TARGET RTOriSimulateSOP)