##=========================================
# GAPS ONLINE SOFTWARE/GONDOLA
# CMAKE BUILD SYSTEM
#
# - use to build C++ library for private 
#   builds
#==========================================

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(GAPS_ONLINE_SOFTWARE VERSION 0.11 LANGUAGES C CXX)

message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE_VERSION}")
message(STATUS "-- Current release PAKI'I (Flounder)")
message(STATUS "-- Building software for ${CMAKE_SYSTEM_NAME}")
if (APPLE)
  message(WARNING "-- -- will attempt build for MacOSX system. This is experimental and might not work!")
  message(WARNING "-- -- the liftof dependency tof-control depends on LINUX and can not work on MacOSX due to the lack of Mac's I2C support. However, tof-control is only needed for the flight software, which runs linux and thus would need to be cross-compiled on Mac anyway.")
endif(APPLE)

message(STATUS "Using build directory: ${PROJECT_BINARY_DIR}")
set(CMAKE_WARN_DEPRECATED ON)

# suppress the Up-to-date output when installing
set(CMAKE_INSTALL_MESSAGE LAZY)

# we really, really want CXX20. Sometimes, cmake is a little bit
# difficult to convince
set(CMAKE_CXX_STANDARD 20 CACHE STRING "C++ standard has to be >= 20" FORCE)
set(CXX_STANDARD 20 CACHE STRING "C++ standard has to be >= 20" FORCE)
set(CMAKE_CXX_STANDARD_REQUIRED YES)

## do a release build by default
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE "Release" CACHE STRING
      "Choose the type of build, options are: Debug Release."
      FORCE)
endif(NOT CMAKE_BUILD_TYPE)

## in case of Release build, we set the NDEBUG preprocessor directive
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -DNDEBUG=1")

## also, set the default install path to be in user space
if(CMAKE_INSTALL_PREFIX_INITIALIZED_TO_DEFAULT)
    set(CMAKE_INSTALL_PREFIX "${CMAKE_BINARY_DIR}/install/gaps-online-sw-v${GAPS_ONLINE_SOFTWARE_VERSION}" CACHE PATH "default install path" FORCE)
endif()

message(STATUS "Installing into ${CMAKE_INSTALL_PREFIX}")
option(BUILD_CXX_BINDINGS "build C++ implementation of gondola-core" OFF)
option(BUILD_CXX_PYBINDINGS "build C++ pybindings with pybind11 (deprecated)" OFF)
option(BUILD_CXX_DB "Build C++ bindings with SQLITE3 to access GAPS DB. Requires SQlite3 installed" OFF) 
option(BUILD_CXX_EXAMPLES "build C++ examples to illustrate usage of the GondolaCore C++ library" OFF)
option(BUILD_CXX_PERSONAL_ANALYSIS_CODE "builds personal analysis code in the analysis/ directory. This includes e.g. Jeff's TOF code. This might require the (CERN) ROOT library, so USE_ROOT needs to be switched on" OFF)
# ROOT can be optionally used for the C++ bindings. This will allow to build the 
# SimpleDet compatibility layer as well as personal analysis projects
option(BUILD_CXX_WITH_ROOT "leverage the usage of CERN ROOT for the build. This will allow to build the SimpleDet compatibilitly layer as well as personal analysis projects as needed." OFF)
option(BUILD_SIM "build simulation library to simulate events" OFF)

# set a few paths for the installation, e.g. for exanokes
set(INSTALL_EXAMPLES_DIR "${CMAKE_INSTALL_PREFIX}/examples" CACHE PATH "installation directory for examples" FORCE)
set(INSTALL_ANALYSIS_DIR "${CMAKE_INSTALL_PREFIX}/analysis" CACHE PATH "installation direcotry for personal analysis code" FORCE)

if(BUILD_CXX_WITH_ROOT)
  list(APPEND CMAKE_PREFIX_PATH $ENV{ROOTSYS})
#  find_package(ROOT 6.26.00)
  find_package(ROOT 6.30.00)
  message(STATUS "Requested build using CERN's ROOT library") 
  if(ROOT_FOUND)
      include(${ROOT_USE_FILE})
      message(STATUS "-- Found a suitable ROOT version ${ROOT_VERSION}!")
      message(STATUS "-- Will use root libraries found in ${ROOT_LIBRARY_DIR}")
      add_definitions( -DBUILD_WITH_ROOT)
  else(NOT ROOT_FOUND)
      message(WARNING "ROOT has not been found! Disabling ROOT for build. This will disable the SimpleDet compatibility layer as well as it might disable some personal analysis projects!")
      set(BUILD_CXX_WITH_ROOT OFF)
  endif(ROOT_FOUND)
endif(BUILD_CXX_WITH_ROOT)

# check sqlite3 availability which is needed 
# for C++ database support
if(BUILD_CXX_DB)
  find_package(SQLite3 REQUIRED)
    if(SQLite3_FOUND)
      message(STATUS "-- Found SQLite3 libary")
      message(STATUS "-- -- Will use sqlite library ${SQLite3_LIBRARIES}")
      add_definitions( -DBUILD_CXXDB)
    else()
      message(WARNING "Can not find sqlite3 installation! Disabling DB feature for C++ library")
      set(BUILD_CXX_DB CACHE "Build C++ bindings with SQLITE3 to access GAPS DB. Requires SQlite3 installed" OFF FORCE) 
  endif()
endif()

# -- pybindings with pybind11

#if (BUILD_CXXPYBINDINGS)
if (BUILD_CXX_PYBINDINGS) 
  find_package(Python 3.8 COMPONENTS Interpreter Development.Module REQUIRED)
  add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/nanobind)
endif() 

set(INSTALL_LIBRARY_DIR  "${CMAKE_INSTALL_PREFIX}/lib" CACHE PATH      "installation directory for libraries" FORCE)
set(INSTALL_INCLUDE_DIR  "${CMAKE_INSTALL_PREFIX}/include" CACHE PATH  "installation directory for header files" FORCE)
set(INSTALL_BINARY_DIR   "${CMAKE_INSTALL_PREFIX}/bin" CACHE PATH      "installation directory for executables" FORCE)
set(INSTALL_PYTHON_DIR   "${CMAKE_INSTALL_PREFIX}/python" CACHE PATH   "installation directory for python code" FORCE)

add_subdirectory(gondola-core)

if (BUILD_CXX_EXAMPLES)
  add_subdirectory(examples)
endif(BUILD_CXX_EXAMPLES)

## Make relative paths absolute (needed later on)
#foreach(p LIBRARY BINARY INCLUDE PYTHON EXAMPLES)
#  set(var INSTALL_${p}_DIR)
#  if(NOT IS_ABSOLUTE "${${var}}")
#    set(${var} "${CMAKE_INSTALL_PREFIX}/${${var}}")
#  endif()
#endforeach()

if (BUILD_SIM)
  add_subdirectory(simulation)
endif (BUILD_SIM)

if (BUILD_CXX_PERSONAL_ANALYSIS_CODE)
  add_subdirectory(analysis)
endif(BUILD_CXX_PERSONAL_ANALYSIS_CODE)

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
  set(DOXYGEN_INPUT "${CMAKE_SOURCE_DIR}/gondola-core/C++")
  set(DOXYGEN_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/doc/C++")
  set(DOXYGEN_GENERATE_HTML YES)
  set(DOXYGEN_GENERATE_MAN NO) # You can enable this if you want man pages
  configure_file(${CMAKE_SOURCE_DIR}/doc/C++/Doxyfile.in ${CMAKE_BINARY_DIR}/doc/C++/Doxyfile @ONLY)
  add_custom_target(doc
    ALL
    COMMAND ${DOXYGEN_EXECUTABLE} ${CMAKE_BINARY_DIR}/doc/C++/Doxyfile
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}/doc/C++ 
    COMMENT "Generating C++ API documentation with Doxygen"
    VERBATIM
  )
  install(DIRECTORY ${CMAKE_BINARY_DIR}/doc/C++/html DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/C++API/)
endif()

if (BUILD_LIFTOF)
  if (DOXYGEN_FOUND)
    add_dependencies(doc liftof-doc)
  else()
    add_dependencies(liftof-cc liftof-doc)
  endif()
  install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tof/liftof/target/doc/ DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/RustAPI/)
endif()

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

## Install the python wrapper for the API
install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/python/gaps-online/gaps_online
        DESTINATION ${INSTALL_PYTHON_DIR}
)

## ----------------------------------=---
## External dependencies (header only)
## ----------------------------------=---

## create directories first
install(DIRECTORY DESTINATION ${CMAKE_INSTALL_PREFIX}/include/cxxopts)
install(DIRECTORY DESTINATION ${CMAKE_INSTALL_PREFIX}/include/result)
install(DIRECTORY DESTINATION ${CMAKE_INSTALL_PREFIX}/include/sqlite_orm)

## install all the header-only libraries into the include directory
install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/cxxopts/include/cxxopts.hpp
        DESTINATION ${CMAKE_INSTALL_PREFIX}/include/cxxopts/
)

install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/result/result/result.h
        DESTINATION ${CMAKE_INSTALL_PREFIX}/include/result
)

install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/sqlite_orm/include/sqlite_orm/sqlite_orm.h
        DESTINATION ${CMAKE_INSTALL_PREFIX}/include/sqlite_orm
)

## For spdlog, we just copy the whole thing
install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/spdlog/include/spdlog DESTINATION ${CMAKE_INSTALL_PREFIX}/include/spdlog
)

