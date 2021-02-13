cmake_minimum_required (VERSION 3.16.3)
project (dummy)

set( CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/../target/debug/deps )

add_library (dummy SHARED internal.cpp lib.cpp)
target_include_directories (dummy PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})

# Actually not necessary, done by default with SHARED targets, but adding here for reference
set_property(TARGET dummy PROPERTY POSITION_INDEPENDENT_CODE ON)
