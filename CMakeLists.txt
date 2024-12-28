##=========================================
# GAPS ONLINE SOFTWARE
# CMAKE BUILD SYSTEM
#
#  - dataclasses (rust/C++)
#  - flight software (liftof)
#
#==========================================

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(GAPS_ONLINE_SOFTWARE VERSION 0.10 LANGUAGES C CXX)

message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE_VERSION}")
message(STATUS "-- Current release LELEWAA (Whale shark)")
message(STATUS "-- Building software for ${CMAKE_SYSTEM_NAME}")
if (APPLE)
  message(WARNING "-- -- will attempt build for MacOSX system. This is experimental and might not work!")
  message(WARNING "-- -- tof-control depends on LINUX and can not work on MacOSX due to the lack of Mac's I2C support. However, tof-control is only needed for the flight software, which runs linux and thus would need to be cross-compiled on Mac anyway.")
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

# -- pybindings with pybind11
option(BUILD_CXXPYBINDINGS "build C++ pybindings with pybind11 (deprecated)" OFF)
option(BUILD_RUSTTELEMETRY "builds Rust telemetry library and pybindings" OFF)
option(BUILD_RUSTPYBINDINGS "build pybindings for the RUST code with py03 \\U+1F980 (better). Includes telemetry and liftof pybindings, if the respective options are selected." OFF)
option(BUILD_CARASPACE "build the caraspace serialization library (useful)" OFF)

if (BUILD_CXXPYBINDINGS)
  message(STATUS "Checking for pyoind11....")
  # populates Python_INCLUDE_DIRS
  find_package (Python 3.10 COMPONENTS Interpreter Development)
  find_package(pybind11 )
  if(NOT ${pybind11_FOUND})
    message(WARNING "-- Can not find pybind11! Please install with sudo apt install pybind11-dev on a linux system!")
    set(BUILD_PYBINDINGS OFF CACHE BOOL "build pybindings with pybind11" FORCE)
  else()
    message(STATUS "-- found pybind11, building pybindings.")
  endif()
endif(BUILD_CXXPYBINDINGS)

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
#if (BUILD_RUSTTELEMETRY)
#  add_subdirectory(telemetry)
#endif()

# finally all the rust pybindings
if (BUILD_RUSTPYBINDINGS)
  add_subdirectory(python)
endif (BUILD_RUSTPYBINDINGS)
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
    ALL
    COMMAND ${DOXYGEN_EXECUTABLE} ${CMAKE_BINARY_DIR}/Doxyfile
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Generating C++ API documentation with Doxygen"
    VERBATIM
  )
  install(DIRECTORY ${CMAKE_BINARY_DIR}/doc/html DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/C++API/)
endif()

if (BUILD_LIFTOF)
  if (DOXYGEN_FOUND)
    add_dependencies(doc liftof-doc)
  else()
    add_dependencies(liftof-cc liftof-doc)
  endif()
  install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/tof/liftof/target/doc/ DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/RustAPI/)
endif()

# this is done by our github runner
#if (BUILD_RUSTPYBINDINGS)
#  if (DOXYGEN_FOUND)
#    add_dependencies(doc go-sphinx-doc)
#  endif()
#  install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/python/doc/ DESTINATION ${CMAKE_INSTALL_PREFIX}/doc/PythonAPI/)
#endif()

# don't install it. Django get's confused when copied/symlinked. Let's just use it from source 
# and set the PYTHONPATH in the setup-env accordingly
#install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/gaps-db/gaps_db/ DESTINATION ${INSTALL_PYTHON_DIR})

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

