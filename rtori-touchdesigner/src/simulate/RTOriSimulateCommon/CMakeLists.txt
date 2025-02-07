list(APPEND RTOriSimulateCommon_Impl
    src/Simulator.cpp
)

list(APPEND RTOriSimulateCommon_Headers
    include/rtori/td/InfoCHOP.hpp
    include/rtori/td/Interests.hpp
    include/rtori/td/Simulator.hpp
    include/rtori/td/SimulateOP.hpp
)

add_library(RTOriSimulateCommon SHARED ${RTOriSimulateCommon_Impl})

# To export the interface
target_compile_definitions(RTOriSimulateCommon PRIVATE RTORI_TD_BUILD_SHARED=1)

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
    RTOriCommon
    TouchDesigner_Common
)

install(
    TARGETS RTOriSimulateCommon
    RUNTIME # For Windows
    DESTINATION "." # To the root
    LIBRARY # On macOS we use rpath
    DESTINATION "lib"
)