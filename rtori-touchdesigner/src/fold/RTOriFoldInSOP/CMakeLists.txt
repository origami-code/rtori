list(APPEND RTOriFoldInSOP_Sources
    src/RTOriFoldInSOP.cpp
    src/RTOriFoldInSOP.hpp
)

add_library(
    RTOriFoldInSOP
    MODULE
    ${RTOriFoldInSOP_Sources}
)

target_compile_features(RTOriFoldInSOP PRIVATE cxx_std_20)
target_link_libraries(
    RTOriFoldInSOP

    PUBLIC

    TouchDesigner_Common
    TouchDesigner_SOP

    RTOriCommon
)

prepare_plugin(TARGET RTOriFoldInSOP)