cmake_minimum_required(VERSION 3.23..3.30)


set(CMAKE_EXPERIMENTAL_FIND_CPS_PACKAGES e82e467b-f997-4464-8ace-b00808fff261)
project(rtori)


find_package(rtori_core REQUIRED PATHS ${CMAKE_CURRENT_SOURCE_DIR} NO_DEFAULT_PATH)
add_subdirectory("rtori-core-ext")
add_subdirectory("rtori-touchdesigner")