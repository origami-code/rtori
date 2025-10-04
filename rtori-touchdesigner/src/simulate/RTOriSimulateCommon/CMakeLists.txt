list(APPEND RTOriSimulateCommon_Impl
    src/Simulator.cpp
)

list(APPEND RTOriSimulateCommon_Headers
    include/rtori/td/InfoCHOP.hpp
    include/rtori/td/Interests.hpp
    include/rtori/td/Simulator.hpp
    include/rtori/td/SimulateOP.hpp
)

foreach (td_version ${RTORI_TD_TARGETED_VERSIONS})
    set(target_name RTOriSimulateCommon_td${td_version})
        
    add_library(${target_name} SHARED ${RTOriSimulateCommon_Impl})
    prepare_shared_library(TARGET ${target_name})

    # To export the interface
    target_compile_definitions(${target_name} PRIVATE RTORI_TD_BUILD_SHARED=1)

    target_sources(${target_name} PUBLIC
        FILE_SET HEADERS
        TYPE HEADERS
        BASE_DIRS include
        FILES ${RTOriSimulateCommon_Headers}
    )

    target_compile_features(${target_name} PUBLIC cxx_std_20)
    target_link_libraries(
        ${target_name}

        PUBLIC
        RTOriCommon
        TouchDesigner::${td_version}::Common
    )

    if(APPLE)
        code_sign_macos(
            RTOriSimulateCommon_sign
            PATH "$<TARGET_FILE:${target_name}>"
            IDENTITY ${MACOS_CODE_SIGNING_IDENTITY}
        )
    endif()

    install(
        TARGETS ${target_name}
        RUNTIME # For Windows
        DESTINATION "." # To the root
        LIBRARY # On macOS we use rpath
        DESTINATION "lib"
    )
endforeach()