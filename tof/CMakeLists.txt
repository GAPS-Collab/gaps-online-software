#cmake_minimum_required(VERSION 3.16 FATAL_ERROR)

#project(GAPS_ONLINE_SOFTWARE VERSION 0.0.1 LANGUAGES C CXX)
#message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE__VERSION}")
#message(STATUS "Using build directory: ${PROJECT_BINARY_DIR}")
#set(CMAKE_WARN_DEPRECATED ON)
#set(CMAKE_CXX_STANDARD 17 CACHE STRING "C++ standard has to be >= 17")

# rust part
#set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/corrosion/cmake/)
#find_package(Corrosion REQUIRED)

#set(CMAKE_Rust_COMPILER "/usr/bin/rustc" CACHE STRING "The location of the rust compiler")
#enable_language(Rust)
#include(CMakeCargo)

#### Build options
option(BUILD_CRUSTYKRAKEN "build the rust tof-server code" OFF)
option(BUILD_RBEMULATOR "build the readoutboard emulator for debugging purposes" OFF)
set(RBEMULATOR_DIR ${CMAKE_CURRENT_SOURCE_DIR}/readoutboard-emulator)

option(USE_BOOST_LOG "use boost logging library" OFF)

## ================================================================
## cmake build type
## can be either "Release" or "Debug"

## in case of Release build, we set the NDEBUG preprocessor directive
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -DNDEBUG=1")



################
# Configure number of readoutboards, nchannels
set(N_READOUT_BOARDS 4 CACHE STRING "Compile for a specific number of readout boards" )
set(USE_CHANNEL_9  ON CACHE STRING "Compile for all channels (including channel 9 which holds the sine wave for synchornisation" )
set(RB_EMULATOR_DATADIR  "../test/readoutboard-emulator/resources/example-data/" CACHE PATH "Set data directory with blobfiles for readoutboard emulator replay" )
set(RB_EMULATOR_BASEFILE_PATTERN  "" CACHE PATH "Set pattern for blobfile naming convention" )
set(NCHANNELS 8 CACHE INTERNAL "Number of readoutboard channels" )
set(MEMEVENTS 10000 CACHE STRING "Size of internam event cache (in events)" )
set(DATAFILES "${CMAKE_CURRENT_SOURCE_DIR}/datafiles" CACHE PATH "Location of the datafiles directory which contains calibration and paddle info files.")

message(STATUS "Configuring...")
message(STATUS " -- Will compile software for ${N_READOUT_BOARDS} readout boards!")
if(USE_CHANNEL_9)
  message(STATUS " -- Will compile software for the use of channel 9 (sine-wave)")
  set(NCHANNELS 9 CACHE INTERNAL "Number of readoutboard channels" FORCE)
else()
  message(WARNING " -- Will compile software for the use with channel 9! No synchronisation across boards possible!")
  set(NCHANNELS 8 CACHE INTERNAL "Number of readoutboard channels" FORCE)
endif(USE_CHANNEL_9) 
configure_file(${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include/TOFCommon.h.in ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include/TOFCommon.h @ONLY)
#configure_file(${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/threads/TOFParam.cpp.in ${CMAKE_CURRENT_SOURCE_DIR}/threads/TOFParam.cpp @ONLY)
configure_file(${RBEMULATOR_DIR}/bin/RBEmulator.cxx.in ${RBEMULATOR_DIR}/bin/RBEmulator.cxx @ONLY)




# ROOT part is still needed for the blobroutines!
# Locate the ROOT package and defines a number of variables (e.g. ROOT_INCLUDE_DIRS)
option(BUILD_ROOTCOMPONENTS "build components of the software which require CERN ROOT" ON)

if(BUILD_ROOTCOMPONENTS)

  list(APPEND CMAKE_PREFIX_PATH $ENV{ROOTSYS})
  find_package(ROOT 6.26.00)
  
  if(ROOT_FOUND)
      include(${ROOT_USE_FILE})
      message(STATUS "Found ROOT!")
      message(STATUS "-- Will use root libraries found in ${ROOT_LIBRARY_DIR}")
      add_definitions( -DBUILD_ROOTCOMPONENTS)
  else(NOT ROOT_FOUND)
      message(WARNING "ROOT has not been found! Disabling components which require ROOT")
      set(BUILD_ROOTCOMPONENTS OFF)
  endif(ROOT_FOUND)
endif(BUILD_ROOTCOMPONENTS)

# boost
message(STATUS "Searching for BOOST libraries....")
find_package(Boost 1.53 COMPONENTS thread log log_setup)

# see comment about boost log and ROOT above
if (USE_BOOST_LOG)
  if (${Boost_LOG_FOUND})
    set(USE_BOOST_ANY ON) 
    message(WARNING "**You are building the TOF server software with USE_BOOST_LOG=ON. While the advanced logging system is very helpful, be aware that due to a bug in current ROOT versions, ROOT will NOT BUILD ITS DICTIONARIES. This means that the ROOT cppyy generated PYBINDINGS MOST LIKELY WONT WORK!! If you want to use these features, please use the simplified logging system and set USE_BOOST_LOG=OFF**")
  else()
    message(WARNING "Could not find Boost_log. Will disable it for now.")
    set(USE_BOOST_LOG OFF)
  endif()
else()
  message(WARNING "-- -- USE_BOOST_LOG is set to OFF, building without boost log!")
endif(USE_BOOST_LOG)

if(USE_BOOST_LOG)
  add_definitions( -DUSE_BOOST_LOG)
  add_definitions( -DBOOST_LOG_DYN_LINK)
endif()

## environmental system
#
## invoke the go compiler to create a c++ style shared library
## we set the working directory to the include dir, so 
## the header gets created in the right place
## the shared library can live in our build directory
#set(DBQUERY_LIBRARY libDBQuery)
#add_custom_target(${DBQUERY_LIBRARY} DEPENDS ${DBQUERY_LIBRARY}.so )
#add_custom_command(OUTPUT ${DBQUERY_LIBRARY}.so 
#    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include
#	COMMAND env CGO_CFLAGS="-I${CMAKE_CURRENT_SOURCE_DIR}/include" GOPATH=${GOPATH} go build -buildmode=c-shared -o ${CMAKE_CURRENT_BINARY_DIR}/libDBQuery.so  ${CMAKE_GO_FLAGS} ${CMAKE_CURRENT_SOURCE_DIR}/library/go/libDBQuery.go && mv ${CMAKE_CURRENT_BINARY_DIR}/libDBQuery.h ${CMAKE_CURRENT_SOURCE_DIR}/include
#    COMMENT "Building C/CXX shared object libDBQuery.so from go source. ")
##FIXME - this is currently just to test
## if the cgo environment is working as it should
## this can go away in the future
#add_executable(QueryEnvData test/QueryEnvData.cxx)
#add_dependencies(QueryEnvData ${DBQUERY_LIBRARY})
#target_include_directories(QueryEnvData
#                            PUBLIC
#                              ${CMAKE_CURRENT_SOURCE_DIR}/include
#                          )
#target_link_libraries(QueryEnvData
#                      PUBLIC
#		                ${CMAKE_CURRENT_BINARY_DIR}/${DBQUERY_LIBRARY}.so
#		                #${DBQUERY_LIBRARY}
#                      )

# tests/reeadoutboard emulator
if (BUILD_RBEMULATOR)
    message(STATUS "Building readoutboard-emulator")
    message(STATUS "-- found readout-builder code in ${RBEMULATOR_DIR}") 
    # RBEMULATOR binary
    set(RBEMULATOR_BIN RBEmulator) 
    add_executable(${RBEMULATOR_BIN} ${RBEMULATOR_DIR}/bin/RBEmulator.cxx
                   ${RBEMULATOR_DIR}/src/BlobFileServer.cxx
                  )
    target_include_directories(${RBEMULATOR_BIN}
                                PUBLIC
                                ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                                ${RBEMULATOR_DIR}/include
                              )
    target_link_libraries(${RBEMULATOR_BIN}
                         PUBLIC
                            $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
                            #$<$<BOOL:${USE_BOOST_LOG}>:Boost::log_setup>
                            #$<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
                            #$<$<BOOL:${USE_BOOST_LOG}>:Boost::log>
                            ${GAPSTOF_LIBRARY_SHARED}
                            #${Boost_THREAD_LIBRARY_RELEASE}
                            "libczmq.so"
                            "libzmq.so"
                        )
endif(BUILD_RBEMULATOR)



# -- the readoutboard emulator is something which 
# -- can distribute "old" blob data for 5 readoutboards
# -- through the ports which they would usually use 
# -- so this can be used for debugging the server program



###############################
# Server shared libary 
# - packets
# - blobroutines 
#
################################3

set(GAPSTOF_LIBRARY_SHARED GapsTof CACHE INTERNAL "GapsTof shared library" FORCE)
add_library(${GAPSTOF_LIBRARY_SHARED}
    SHARED dataclasses/C++/src/blobroutines.cpp 
    dataclasses/C++/src/TofPacket.cxx
    dataclasses/C++/src/REventPacket.cxx
    dataclasses/C++/src/RBEnvPacket.cxx
    dataclasses/C++/src/RPaddlePacket.cxx
    dataclasses/C++/src/WfPaddlePacket.cxx
    dataclasses/C++/src/serialization.cxx
    dataclasses/C++/src/parsers.cxx
    dataclasses/C++/src/CraneLogging.cc
    dataclasses/C++/src/WaveGAPS.cpp
    dataclasses/C++/src/CommandPacket.cxx
    dataclasses/C++/src/RBMoniPacket.cxx
    dataclasses/C++/src/MasterTriggerPacket.cxx
)

target_compile_options(${GAPSTOF_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
target_include_directories(${GAPSTOF_LIBRARY_SHARED}
                            PUBLIC
                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
                              ${Boost_INCLUDE_DIR}
                          )
target_link_libraries(${GAPSTOF_LIBRARY_SHARED}
                      PUBLIC
                         $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
                         $<$<BOOL:${USE_BOOST_LOG}>:Boost::log_setup>
                         #$<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
                         $<$<BOOL:${USE_BOOST_LOG}>:Boost::log>
                    )
#
## GAPSSERVER binary
##set(SERVER_BIN GAPS_server)
#set(SERVER_BIN DevGfpServer) 
#add_executable(${SERVER_BIN} bin/GAPS_server.cpp
#                threads/AnalyzeEvts.cpp
#                threads/RBCommunication.cpp
#                threads/SendToFlight.cpp
#                threads/DataManage.cpp
#                threads/TOFParam.cpp
#              )
#target_include_directories(${SERVER_BIN}
#                            PUBLIC
#                              ${CMAKE_CURRENT_SOURCE_DIR}/include
#                              ${Boost_INCLUDE_DIR}
#                              #${ROOT_INCLUDE_DIRS}
#                          )
#target_link_libraries(${SERVER_BIN}
#                     PUBLIC
#                        ${GAPSSERVER_LIBRARY_SHARED}
#                        ${CMAKE_CURRENT_BINARY_DIR}/${DBQUERY_LIBRARY}.so
#                        ${Boost_THREAD_LIBRARY_RELEASE}
#                        "libczmq.so"
#                        "libzmq.so"
#                    )
#
#
if (BUILD_PYBINDINGS)
  set(pybindings "gaps_tof")

  pybind11_add_module(${pybindings} MODULE pybindings/module.cxx)
  target_include_directories(${pybindings}
                            PUBLIC
                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                            )
  get_property(the_include_dirs TARGET ${pybindings} PROPERTY INCLUDE_DIRECTORIES)
  foreach(dir ${the_include_dirs})
  message(STATUS "dir='${dir}'")
  endforeach()
  list(REMOVE_ITEM ${the_include_dirs} "/usr/include/python2.7")
  list(APPEND ${the_include_dirs} "/usr/include/python3.8")
  string(REPLACE "/usr/include/python2.7" ";;"  new_include_dirs ${the_include_dirs})
  list(APPEND ${new_include_dirs} "/usr/include/python3.8")
  set(pybind_include_dirs "/usr/include/python3.8"  "${CMAKE_CURRENT_SOURCE_DIR}/include") 
  foreach(dir ${pybind_include_dirs})
  message(STATUS "dir='${dir}'")
  endforeach()
  set_property(TARGET ${pybindings} PROPERTY INCLUDE_DIRECTORIES ${pybind_include_dirs})
  target_link_libraries(${pybindings}
                        PRIVATE
                        ${GAPSTOF_LIBRARY_SHARED})

endif(BUILD_PYBINDINGS)
#
