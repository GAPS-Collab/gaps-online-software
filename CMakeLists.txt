##=========================================
# GAPS ONLINE SOFTWARE
# CMAKE BUILD SYSTEM
#
#  - dataclasses
#  - UCLA waveform tools
#
#==========================================

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(GAPS_ONLINE_SOFTWARE VERSION 0.6.0 LANGUAGES C CXX)

message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE_VERSION}")
message(STATUS "-- Current release KIHIKIHI (Moorish idol)")
message(STATUS "Using build directory: ${PROJECT_BINARY_DIR}")
set(CMAKE_WARN_DEPRECATED ON)

# suppress the Up-to-date output when installing
set(CMAKE_INSTALL_MESSAGE LAZY)

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

## in case of Release build, we set the NDEBUG preprocessor directive
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -DNDEBUG=1")

add_subdirectory(tof)


if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install/gaps-online-sw-v${GAPS_ONLINE_SOFTWARE_VERSION}" CACHE PATH "default install path" FORCE)
endif()

message(STATUS "Installing into ${CMAKE_INSTALL_PREFIX}")

set(INSTALL_LIBRARY_DIR  "${CMAKE_INSTALL_PREFIX}/lib" CACHE PATH      "installation directory for libraries" FORCE)
set(INSTALL_INCLUDE_DIR  "${CMAKE_INSTALL_PREFIX}/include" CACHE PATH  "installation directory for header files" FORCE)
set(INSTALL_BINARY_DIR   "${CMAKE_INSTALL_PREFIX}/bin" CACHE PATH      "installation directory for executables" FORCE)
set(INSTALL_PYTHON_DIR   "${CMAKE_INSTALL_PREFIX}/python" CACHE PATH   "installation directory for python code" FORCE)

# Make relative paths absolute (needed later on)
foreach(p LIBRARY BINARY INCLUDE PYTHON EXAMPLES)
  set(var INSTALL_${p}_DIR)
  if(NOT IS_ABSOLUTE "${${var}}")
    set(${var} "${CMAKE_INSTALL_PREFIX}/${${var}}")
  endif()
endforeach()

# install README
install(FILES
  "${CMAKE_CURRENT_SOURCE_DIR}/README.md"
  DESTINATION "${CMAKE_INSTALL_PREFIX}"
)

# write setup shell

file(WRITE  ${CMAKE_INSTALL_PREFIX}/setup-env.sh "#! /bin/sh\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "export PATH=$PATH:${CMAKE_INSTALL_PREFIX}/bin\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "export PYTHONPATH=$PYTHONPATH:${CMAKE_INSTALL_PREFIX}/python\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${CMAKE_INSTALL_PREFIX}/lib\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"********************************************\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"* WELCOME TO GAPS-ONLINE-SOFTWARE          *\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"********************************************\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" -- We have set the following variables:\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" -- PYTHONPATH=$PYTHONPATH\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" -- PATH=$PATH\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" -- LD_LIBRARY_PATH=$LD_LIBRARY_PATH\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"********************************************\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" => software repositories:\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"    - https://uhhepvcs.phys.hawaii.edu/Achim/gaps-online-software\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"    - https://github.com/GAPS-Collab/gaps-online-software\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" => Where to get help?:\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"    - README.md\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \" => Maintainer?:\"\n")
file(APPEND ${CMAKE_INSTALL_PREFIX}/setup-env.sh "echo -e \t  \"    - stoessl@hawaii.edu\"\n")


