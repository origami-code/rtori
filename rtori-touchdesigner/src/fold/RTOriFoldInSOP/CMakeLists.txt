list(APPEND RTOriFoldInSOP_Sources
    src/RTOriFoldInSOP.cpp
    src/RTOriFoldInSOP.hpp
)
foreach (td_version ${RTORI_TD_TARGETED_VERSIONS})
    set(target_name RTOriFoldInSOP_${td_version})
    add_library(
        ${target_name}
        MODULE
        ${RTOriFoldInSOP_Sources}
    )

    target_compile_features(${target_name} PRIVATE cxx_std_20)
    target_link_libraries(
        ${target_name}

        PUBLIC

        TouchDesigner_${td_version}_Common
        TouchDesigner_${td_version}_SOP

        RTOriCommon
    )

    prepare_plugin(TARGET ${target_name} TD_VERSION ${td_version})
endforeach()