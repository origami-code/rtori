cmake_minimum_required(VERSION 3.21...3.30)

project(rtori-touchdesigner
    DESCRIPTION "Real-Time Origami Toolkit Touchdesigner Integration"
    LANGUAGES CXX
)

if(NOT TARGET rtori_core)
    add_subdirectory("../rtori-core-ext" ${CMAKE_CURRENT_BINARY_DIR}/rtori_core)
endif()

add_subdirectory(plugins)