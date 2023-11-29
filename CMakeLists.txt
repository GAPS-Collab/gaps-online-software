##=========================================
# GAPS ONLINE SOFTWARE
# CMAKE BUILD SYSTEM
#
#  - dataclasses
#  - UCLA waveform tools
#
#==========================================

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(GAPS_ONLINE_SOFTWARE VERSION 0.7.0 LANGUAGES C CXX)

message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE_VERSION}")
message(STATUS "-- Current release OMILU (Bluefin Trevali)")
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


if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install/gaps-online-sw-v${GAPS_ONLINE_SOFTWARE_VERSION}" CACHE PATH "default install path" FORCE)
endif()

message(STATUS "Installing into ${CMAKE_INSTALL_PREFIX}")

set(INSTALL_LIBRARY_DIR  "${CMAKE_INSTALL_PREFIX}/lib" CACHE PATH      "installation directory for libraries" FORCE)
set(INSTALL_INCLUDE_DIR  "${CMAKE_INSTALL_PREFIX}/include" CACHE PATH  "installation directory for header files" FORCE)
set(INSTALL_BINARY_DIR   "${CMAKE_INSTALL_PREFIX}/bin" CACHE PATH      "installation directory for executables" FORCE)
set(INSTALL_PYTHON_DIR   "${CMAKE_INSTALL_PREFIX}/python" CACHE PATH   "installation directory for python code" FORCE)

add_subdirectory(tof)

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

#################################3
# C++ documentation with DOXYGEN
###################################

find_package(Doxygen)
if(DOXYGEN_FOUND)
  set(DOXYGEN_INPUT "${CMAKE_SOURCE_DIR}/tof/dataclasses/C++/include")
  set(DOXYGEN_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/doc")
  set(DOXYGEN_GENERATE_HTML YES)
  set(DOXYGEN_GENERATE_MAN NO) # You can enable this if you want man pages
  configure_file(${CMAKE_SOURCE_DIR}/Doxyfile.in ${CMAKE_BINARY_DIR}/Doxyfile @ONLY)
  add_custom_target(doc
    COMMAND ${DOXYGEN_EXECUTABLE} ${CMAKE_BINARY_DIR}/Doxyfile
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating C++ API documentation with Doxygen"
    VERBATIM
  )
  install(DIRECTORY ${CMAKE_BINARY_DIR}/doc/html DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/C++API/)
endif()

if (BUILD_LIFTOF)
  add_dependencies(doc liftof-doc)
  install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tof/liftof/target/doc/ DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/RustAPI/)
endif()

install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/gaps-db/gaps_db/ DESTINATION ${INSTALL_PYTHON_DIR})

###################################
# Write setup-env.sh shell script
####################################

## Define the paths
set(SETUP_SCRIPT_SOURCE ${CMAKE_SOURCE_DIR}/setup-env.sh.in)
set(SETUP_SCRIPT_DEST ${CMAKE_BINARY_DIR}/setup-env.sh)

# Configure the setup script during configuration
configure_file(${SETUP_SCRIPT_SOURCE} ${SETUP_SCRIPT_DEST} @ONLY)
#
## Install the generated setup script during installation
install(FILES ${SETUP_SCRIPT_DEST} DESTINATION ${CMAKE_INSTALL_PREFIX})

