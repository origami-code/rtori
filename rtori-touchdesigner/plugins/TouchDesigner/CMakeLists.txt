add_library(TouchDesigner_Common INTERFACE)
target_sources(TouchDesigner_Common
    INTERFACE
    FILE_SET HEADERS
    TYPE HEADERS
    BASE_DIRS common
    FILES
    common/CPlusPlus_Common.h
)

add_library(TouchDesigner_SOP INTERFACE)
target_sources(TouchDesigner_SOP
    INTERFACE
    FILE_SET HEADERS
    TYPE HEADERS
    BASE_DIRS sop
    FILES
    sop/SOP_CPlusPlusBase.h
)
target_link_libraries(TouchDesigner_SOP INTERFACE TouchDesigner_Common)