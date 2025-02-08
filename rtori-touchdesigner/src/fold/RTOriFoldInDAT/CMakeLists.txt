#list(APPEND RTOriFoldIn_Impl
#)
#
#list(APPEND RTOriFoldIn_Headers
#)
#
#add_library(RTOriFoldIn SHARED ${RTOriFoldIn_Impl})
#
## To export the interface
#target_compile_definitions(RTOriFoldIn PRIVATE RTORI_TD_BUILD_SHARED=1)
#
#target_sources(RTOriFoldIn PUBLIC
#    FILE_SET HEADERS
#    TYPE HEADERS
#    BASE_DIRS include
#    FILES ${RTOriFoldIn_Headers}
#)
#
#target_compile_features(RTOriFoldIn PUBLIC cxx_std_20)
#target_link_libraries(
#    RTOriFoldIn
#
#    PUBLIC
#    RTOri
#    TouchDesigner_Common
#)
