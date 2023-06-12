##=========================================
#
# GAPS - GFP analysis metapacakge
# 
#  - dataclasses
#  - UCLA waveform tools
#
#==========================================

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(GAPS_ONLINE_SOFTWARE VERSION 0.6.0 LANGUAGES C CXX)

message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE_VERSION}")
message(STATUS "-- Current release KIHIKIHI (Moorish idol)")
message(STATUS "Running cmake for GAPS softwere version ${GAPS_WAVEFORM_VERSION}")
message(STATUS "Using build directory: ${PROJECT_BINARY_DIR}")
set(CMAKE_WARN_DEPRECATED ON)

set(CMAKE_CXX_STANDARD 17 CACHE STRING "C++ standard has to be >= 17")

include(FetchContent)

FetchContent_Declare(
  spdlog
  GIT_REPOSITORY https://github.com/gabime/spdlog.git 
  GIT_TAG        ad0e89cbfb4d0c1ce4d097e134eb7be67baebb36 # release-1.10.0
)
FetchContent_MakeAvailable(spdlog)
FetchContent_GetProperties(spdlog)
if(NOT spdlog_POPULATED)
  # Fetch the content using previously declared details
  FetchContent_Populate(spdlog)

  # Set custom variables, policies, etc.
  # ...

  # Bring the populated content into the build
  add_subdirectory(${spdlog_SOURCE_DIR} ${spdlog_BINARY_DIR})
endif()

# == ALEX bfsw repository. 
# == currently not used, but maybe in the future?

#
#FetchContent_Declare(
#  bfsw
#  GIT_REPOSITORY https://git.ssl.berkeley.edu/bfsw/bfsw.git 
#  GIT_TAG origin/main
#)
#
#FetchContent_MakeAvailable(bfsw)
#FetchContent_GetProperties(bfsw)
#if(NOT bfsw_POPULATED)
#  # Fetch the content using previously declared details
#  FetchContent_Populate(bfsw)
#
#  # Set custom variables, policies, etc.
#  # ...
#
#  # Bring the populated content into the build
#  add_subdirectory(${bfsw_SOURCE_DIR} ${bfsw_BINARY_DIR})
#endif()

# -- pybindings with pybind11
option(BUILD_PYBINDINGS "build pybindings with pybind11" OFF)
if (BUILD_PYBINDINGS)
message(STATUS "Checking for pyoind11....")
find_package(pybind11 )
  if(NOT ${pybind11_FOUND})
    message(WARNING "-- Can not find pybind11! Please install with sudo apt install pybind11-dev on a linux system!")
    set(BUILD_PYBINDINGS OFF CACHE BOOL "build pybindings with pybind11" FORCE)
  else()
    message(STATUS "-- found pybind11, building pybindings.")
  endif()
endif(BUILD_PYBINDINGS)

## do a release build by default
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE "Release" CACHE STRING
      "Choose the type of build, options are: Debug Release."
      FORCE)
endif(NOT CMAKE_BUILD_TYPE)

add_subdirectory(tof)

