list(APPEND RTOriSimulateCommon_Impl
    src/Simulator.cpp
)

list(APPEND RTOriSimulateCommon_Headers
    include/rtori/td/InfoCHOP.hpp
    include/rtori/td/Simulator.hpp
    include/rtori/td/SimulateOP.hpp
)

add_library(RTOriSimulateCommon SHARED ${RTOriSimulateCommon_Impl})

target_sources(RTOriSimulateCommon PUBLIC
    FILE_SET HEADERS
    TYPE HEADERS
    BASE_DIRS include
    FILES ${RTOriSimulateCommon_Headers}    
)

target_compile_features(RTOriSimulateCommon PUBLIC cxx_std_20)
target_link_libraries(
    RTOriSimulateCommon

    PUBLIC
    RTOriBridge
    TouchDesigner_Common
)

