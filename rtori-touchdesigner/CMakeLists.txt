cmake_minimum_required(VERSION 3.23...3.30)
set(CMAKE_LINK_LIBRARIES_ONLY_TARGETS ON)

project(rtori-touchdesigner
    DESCRIPTION "Real-Time Origami Toolkit Touchdesigner Integration"
    LANGUAGES CXX
)

if(NOT TARGET rtori_core)
    add_subdirectory("../rtori-core-ext" ${CMAKE_CURRENT_BINARY_DIR}/rtori_core)
endif()

add_subdirectory(plugins)