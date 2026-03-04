cmake_minimum_required(VERSION 3.16)
project(amps_ffi LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Path to the AMPS C++ client headers
set(AMPS_CLIENT_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../amps-client/include"
    CACHE PATH "Path to AMPS C++ client include directory")

# Path to the AMPS C++ client library
set(AMPS_CLIENT_LIB_DIR "${CMAKE_SOURCE_DIR}/../amps-client/lib"
    CACHE PATH "Path to AMPS C++ client library directory")

add_library(amps_ffi STATIC
    src/amps_ffi.cpp
)

target_include_directories(amps_ffi PUBLIC
    ${CMAKE_SOURCE_DIR}/include
    ${AMPS_CLIENT_INCLUDE_DIR}
)

target_link_directories(amps_ffi PUBLIC
    ${AMPS_CLIENT_LIB_DIR}
)

# Install target
install(TARGETS amps_ffi ARCHIVE DESTINATION lib)
install(FILES include/amps_ffi.h DESTINATION include)
