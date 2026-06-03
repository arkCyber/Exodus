# Common CMake configuration for allama
# This file contains common build configuration options

# Common compiler flags
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Platform-specific settings
if(APPLE)
    # macOS specific settings
    set(CMAKE_MACOSX_RPATH ON)
elseif(UNIX)
    # Linux specific settings
endif()
