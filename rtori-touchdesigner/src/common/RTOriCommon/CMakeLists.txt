
set(RTOriCommon_AllocatorImpl
    "$<IF:$<BOOL:$<TARGET_PROPERTY:ALLOC_USE_WINDOWS_HEAP>>,src/allocator/AllocatorWinHeap.cpp,src/allocator/AllocatorDefault.cpp>"
)

list(APPEND RTOriCommon_Impl
    src/SimulationThread.cpp
    src/Solver.cpp
    src/Solver.hpp
    src/Context.cpp
    ${RTOriCommon_AllocatorImpl}
)

list(APPEND RTOriCommon_Headers
    include/rtori/td/Input.hpp
    include/rtori/td/Output.hpp
    include/rtori/td/SimulationThread.hpp
    include/rtori/td/Context.hpp
)

add_library(RTOriCommon SHARED ${RTOriCommon_Impl})
prepare_shared_library(TARGET RTOriCommon)
target_compile_definitions(RTOriCommon PRIVATE RTORI_TD_BUILD_SHARED=1)

target_sources(RTOriCommon PUBLIC
    FILE_SET HEADERS
    TYPE HEADERS
    BASE_DIRS include
    FILES ${RTOriCommon_Headers}
)

target_compile_features(RTOriCommon PUBLIC cxx_std_20)
target_link_libraries(
    RTOriCommon

    PUBLIC
    rtori_core::Shared
)

if(MSVC)
    set_target_properties(
        RTOriCommon PROPERTIES

        # Custom property
        ALLOC_USE_WINDOWS_HEAP On
    )
endif()

if(APPLE)
    code_sign_macos(
        RTOriCommon_sign
        PATH "$<TARGET_FILE:RTOriCommon>"
        IDENTITY ${MACOS_CODE_SIGNING_IDENTITY}
    )
endif()

install(
    TARGETS RTOriCommon
    RUNTIME # For Windows
    DESTINATION "." # To the root
    LIBRARY # On macOS we use rpath
    DESTINATION "lib"
)
