list(APPEND RTOriSimulateShared_Impl
    src/SimulationThread.cpp
    src/Solver.cpp
    src/Solver.hpp
    src/Context.cpp
)

list(APPEND RTOriSimulateShared_Headers
    include/rtori/td/Input.hpp
    include/rtori/td/Output.hpp
    include/rtori/td/SimulationThread.hpp
    include/rtori/td/Context.hpp
)

add_library(RTOriSimulateShared ${RTOriSimulateShared_Impl})

target_sources(RTOriSimulateShared PUBLIC
    FILE_SET HEADERS
    TYPE HEADERS
    BASE_DIRS include
    FILES ${RTOriSimulateShared_Headers}    
)

target_compile_features(RTOriSimulateShared PUBLIC cxx_std_20)
target_link_libraries(
    RTOriSimulateShared

    PUBLIC
    rtori_core
)