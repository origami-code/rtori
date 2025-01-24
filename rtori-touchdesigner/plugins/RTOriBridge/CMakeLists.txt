list(APPEND RTOriBridge_Impl
    src/SimulationThread.cpp
    src/Solver.cpp
    src/Solver.hpp
    src/Context.cpp
)

list(APPEND RTOriBridge_Headers
    include/rtori/td/Input.hpp
    include/rtori/td/Output.hpp
    include/rtori/td/SimulationThread.hpp
    include/rtori/td/Context.hpp
)

add_library(RTOriBridge ${RTOriBridge_Impl})

target_sources(RTOriBridge PUBLIC
    FILE_SET HEADERS
    TYPE HEADERS
    BASE_DIRS include
    FILES ${RTOriBridge_Headers}    
)

target_compile_features(RTOriBridge PUBLIC cxx_std_20)
target_link_libraries(
    RTOriBridge

    PUBLIC
    rtori_core
)