## Build options
option(BUILD_RBEMULATOR "build the readoutboard emulator for debugging purposes" OFF)
set(RBEMULATOR_DIR ${CMAKE_CURRENT_SOURCE_DIR}/readoutboard-emulator)

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
                                ${spdlog_SOURCE_DIR}/include
                                ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                                ${RBEMULATOR_DIR}/include
                              )
    target_link_libraries(${RBEMULATOR_BIN}
                         PUBLIC
                            $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
                            ${GAPSTOF_LIBRARY_SHARED}
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
    dataclasses/C++/src/tof_packet.cxx
    dataclasses/C++/src/REventPacket.cxx
    dataclasses/C++/src/RPaddlePacket.cxx
    dataclasses/C++/src/WfPaddlePacket.cxx
    dataclasses/C++/src/calibration.cxx
    dataclasses/C++/src/serialization.cxx
    dataclasses/C++/src/parsers.cxx
    dataclasses/C++/src/events.cxx
    dataclasses/C++/src/io.cxx
    dataclasses/C++/src/monitoring.cxx
    dataclasses/C++/src/WaveGAPS.cpp
    dataclasses/C++/src/CommandPacket.cxx
    dataclasses/C++/src/MasterTriggerPacket.cxx
    dataclasses/C++/src/tofpacket_reader.cxx
)

target_compile_options(${GAPSTOF_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
target_include_directories(${GAPSTOF_LIBRARY_SHARED}
                            PUBLIC
                            ${spdlog_SOURCE_DIR}/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
                              ${Boost_INCLUDE_DIR}
                          )
target_link_libraries(${GAPSTOF_LIBRARY_SHARED}
                      PUBLIC
                         $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
                         #$<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
                    )

if (BUILD_PYBINDINGS)
  set(pybindings "gaps_tof")

  pybind11_add_module(${pybindings} MODULE pybindings/module.cxx pybindings/helpers.cxx)
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
