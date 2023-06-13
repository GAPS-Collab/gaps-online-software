##=========================================
#
# GAPS - GFP analysis metapacakge
# 
#  - dataclasses
#  - UCLA waveform tools
#
#==========================================

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(GAPS_WAVEFORM VERSION 0.1.0 LANGUAGES C CXX)


message(STATUS "Running cmake for ${CMAKE_PROJECT_NAME} softwere version ${GAPS_ONLINE_SOFTWARE_VERSION}")
message(STATUS "Running cmake for GAPS softwere version ${GAPS_WAVEFORM_VERSION}")
message(STATUS "Using build directory: ${PROJECT_BINARY_DIR}")
set(CMAKE_WARN_DEPRECATED ON)

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

FetchContent_Declare(
  bfsw
  GIT_REPOSITORY https://git.ssl.berkeley.edu/bfsw/bfsw.git 
  GIT_TAG origin/main
)
FetchContent_MakeAvailable(bfsw)
FetchContent_GetProperties(bfsw)
if(NOT bfsw_POPULATED)
  # Fetch the content using previously declared details
  FetchContent_Populate(bfsw)

  # Set custom variables, policies, etc.
  # ...

  # Bring the populated content into the build
  add_subdirectory(${bfsw_SOURCE_DIR} ${bfsw_BINARY_DIR})
endif()

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

#add_subdirectory(dataclasses)
add_subdirectory(tof)


#
## - ON/OFF switches for different components of the software
#option(BUILD_UCLA_WFLIB "Waveform processing library written by the UCLA tof group" ON )
#option(BUILD_PROTOBUF_DATACLASSES "Lightweight dataclasses for de/serialization and debugging, using google protocol buffers" ON)
#option(BUILD_PYBINDINGS "UCLA Waveform library pybindings. Required for Tof analysis" ON)
#
## this is important for the pybindings
#set(CMAKE_POSITION_INDEPENDENT_CODE TRUE)
#
#set(CMAKE_CXX_STANDARD 17 CACHE STRING "C++ standard has to be >= 17")
#message(STATUS "Building with C++ standard ${CMAKE_CXX_STANDARD}")
#
#if (BUILD_UCLA_WFLIB)
#  # ROOT - Analysis framework - needed for UCLA waveform library
#  
#  list(APPEND CMAKE_PREFIX_PATH $ENV{ROOTSYS})
#  find_package(ROOT 6.26.00 )
#  
#  # Define useful ROOT functions and macros (e.g. ROOT_GENERATE_DICTIONARY)
#  if(ROOT_FOUND)
#      include(${ROOT_USE_FILE})
#  else(NOT ROOT_FOUND)
#      message(FATAL_ERROR "You need CERN ROOT for this program to work!")
#  endif(ROOT_FOUND)
#
#  set(waveform_LIBRARY_SHARED TofWaveform CACHE INTERNAL "Name for the UCLA waveform library" FORCE)
#  add_library(${waveform_LIBRARY_SHARED} SHARED ${CMAKE_CURRENT_SOURCE_DIR}/TOFsoftware/analysis/src/Waveform.cpp)
#  target_include_directories(${waveform_LIBRARY_SHARED} 
#      PUBLIC
#      "${CMAKE_CURRENT_SOURCE_DIR}/TOFsoftware/analysis/include"
#            )
#  target_link_libraries(${waveform_LIBRARY_SHARED}
#                ${ROOT_LIBRARIES}
#            )      
#endif (BUILD_UCLA_WFLIB)
#
#if (BUILD_PROTOBUF_DATACLASSES)
#    find_package(Protobuf REQUIRED)    
#    if (Protobuf_FOUND)
#        message(STATUS "Building TofEvent CXX...")
#        protobuf_generate_cpp(TOF_EVENT_PB_SRC
#                              TOF_EVENT_PB_HDR 
#                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/TofEvent.proto)
#        protobuf_generate_cpp(TRK_EVENT_PB_SRC
#                              TRK_EVENT_PB_HDR 
#                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/TrackerEvent.proto)
#        protobuf_generate_cpp(GAPS_EVENT_PB_SRC
#                              GAPS_EVENT_PB_HDR 
#                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/GapsEvent.proto)
#        protobuf_generate_python(TOF_EVENT_PB_PY
#                                 ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/TofEvent.proto)
#        protobuf_generate_python(TRK_EVENT_PB_PY
#                                 ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/TrackerEvent.proto)
#        protobuf_generate_python(GAPS_EVENT_PB_PY
#                                 ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/GapsEvent.proto)
#
#        add_custom_target(protobuf-dataclasses-build DEPENDS ${TOF_EVENT_PB_HDR}
#                                                      ${TOF_EVENT_PB_SRC}
#                                                      ${TRK_EVENT_PB_HDR}
#                                                      ${TRK_EVENT_PB_SRC}
#                                                      ${GAPS_EVENT_PB_HDR}
#                                                      ${GAPS_EVENT_PB_SRC}
#                                                      ${TOF_EVENT_PB_PY}
#                                                      ${TRK_EVENT_PB_PY}
#                                                      ${GAPS_EVENT_PB_PY}
#                            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses
#                                                  )
#        add_custom_target(protobuf-dataclasses DEPENDS protobuf-dataclasses-build
#            COMMAND mv TofEvent.pb.cc TofEvent.pb.h TofEvent_pb2.py TrackerEvent.pb.h TrackerEvent.pb.cc TrackerEvent_pb2.py GapsEvent.pb.h GapsEvent.pb.cc GapsEvent_pb2.py ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses)
#    else()
#        message(WARN "Did NOT find google protobuf libraries, can not build dataclasses!")
#        set(BUILD_PROTOBUF_DATACLASSES OFF)
#    endif(Protobuf_FOUND)
#endif(BUILD_PROTOBUF_DATACLASSES)
#
#if (BUILD_PYBINDINGS)
#message(STATUS "Checking for pyoind11....")
#find_package(pybind11 )
#  if(NOT ${pybind11_FOUND})
#    message(WARNING "-- Can not find pybind11! Please install with sudo apt install pybind11-dev on a linux system!")
#    set(BUILD_PYBINDINGS OFF) 
#  else()
#    message(STATUS "-- found pybind11, building pybindings.")
#
#    # now it gets a bit complicated, since we want to have that
#    # nice pybind11 protobuf going
#    add_subdirectory(external/abseil-cpp)
#    file(GLOB pybind11_protobuf_SRC ${CMAKE_CURRENT_SOURCE_DIR}/external/pybind11_protobuf/pybind11_protobuf/*.cc)
#    pybind11_add_module( _pyWaveform MODULE ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/TofEvent.pb.cc
#                                            ${pybind11_protobuf_SRC}
#                                            ${CMAKE_CURRENT_SOURCE_DIR}/gaps_gfp/src/_pyWaveform.cxx 
#                         )
#
#    add_dependencies(_pyWaveform protobuf-dataclasses)
#    target_include_directories(_pyWaveform  
#      PUBLIC
#      "${CMAKE_CURRENT_SOURCE_DIR}/external/pybind11_protobuf"
#      "${CMAKE_CURRENT_SOURCE_DIR}/external/abseil-cpp"
#      "${CMAKE_CURRENT_SOURCE_DIR}/external/protobuf"
#      "${CMAKE_CURRENT_SOURCE_DIR}/TOFsoftware/analysis/include"
#      "${CMAKE_CURRENT_SOURCE_DIR}/dataclasses"
#    )
#    target_link_libraries(_pyWaveform 
#      PUBLIC
#        ${Protobuf_LIBRARIES}
#        ${waveform_LIBRARY_SHARED}
#        #${Protobuf_PROTOC_LIBRARIES}
#        absl::strings
#        absl::hash
#        absl::low_level_hash
#        ${ROOT_LIBRARIES}
#        # I am sure I don't need all of these
#        #absl::spy_hash_state
#        #absl::hash_testing
#        absl::core_headers
#        absl::btree
#        absl::flat_hash_map
#        absl::flat_hash_set
#        absl::node_hash_map
#        absl::node_hash_set
#    )
#  endif()
#endif()
######## set paths
#######set(common_SOURCE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/src)
#######set(common_INCLUDE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/include)
######## make the common include dir also accessible by other subprojects
#######set(common_INCLUDE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/include PARENT_SCOPE)
#######
######## set library names
######option(BUILD_PYBINDINGS ON)
######file(GLOB pbpb_SOURCE_FILES pybind_protobuf/*.cc)
######add_library(pybind_protobuf SHARED ${pbpb_SOURCE_FILES})
######target_link_libraries(pybind_protobuf
######    PRIVATE 
######            "/usr/lib/libprotobuf.so"
######            absl::strings
######            absl::hash
######            absl::low_level_hash
######            absl::spy_hash_state
######            absl::hash_testing
######            absl::core_headers
######            absl::btree
######            absl::flat_hash_map
######            absl::flat_hash_set
######            absl::node_hash_map
######            absl::node_hash_set
######
######
######        )
######target_include_directories(pybind_protobuf
######        PUBLIC
######            "/usr/include/python3.10/"
######            "/srv/software/pybind11_protobuf/"
######            "${CMAKE_CURRENT_BINARY_DIR}/abseil-cpp"
######            #"/home/achim/.cache/bazel/_bazel_achim/4aab7c80083dc5d2369789eab728aece/external/com_google_absl/" #absl/strings/string_view.h
######            "/home/achim/.cache/bazel/_bazel_achim/4aab7c80083dc5d2369789eab728aece/external/com_google_protobuf/"
######        )
#######
#######build shared library
######## FIXME: fix the install interface
#######target_include_directories(${common_LIBRARY_SHARED}
#######                           PUBLIC
#######                               $<INSTALL_INTERFACE:include>
#######                               $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
#######                               ${ROOT_INCLUDE_DIRS}
#######                               $<$<BOOL:${USE_BOOST_ANY}>:${Boost_INCLUDE_DIR}>
#######                               $<$<BOOL:${BUILD_SIMULATION}>:${Geant4_INCLUDE_DIRS}>
#######                          )
#######
#######
######target_link_libraries(${waveform_LIBRARY_SHARED}
######                      PRIVATE
#######                        $<$<BOOL:${USE_BOOST_PROGRAM_OPTIONS}>:Boost::program_options>
#######                        $<$<BOOL:${USE_BOOST_TIMER}>:Boost::timer>
#######                        $<$<BOOL:${USE_BOOST_LOG}>:Boost::log>
#######                        $<$<BOOL:${USE_BOOST_LOG}>:Boost::log_setup>
#######                        $<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
#######                      PUBLIC
######                        ${ROOT_LIBRARIES})
######
######                    
######if (BUILD_PYBINDINGS)
######    set(pybindings_waveform "_pyWaveform")
######
######
######    pybind11_add_module(${pybindings_waveform} MODULE ${pbpb_SOURCE_FILES} TofEvent.pb.cc module.cxx)
######    target_include_directories(${pybindings_waveform}
######        PUBLIC
######        "/srv/software/pybind11_protobuf/"
######        #"/home/achim/.cache/bazel/_bazel_achim/4aab7c80083dc5d2369789eab728aece/external/com_google_absl/" #absl/strings/string_view.h
######        ) 
#######target_include_directories(${pybindings_waveform}
#######                           PRIVATE
#######                           #$<INSTALL_INTERFACE:include>
#######                             $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
#######                             $<BUILD_INTERFACE:${common_INCLUDE_DIR}>
#######                             ${ROOT_INCLUDE_DIRS}
#######                             ${Geant4_INCLUDE_DIRS}
#######                          )
#######
######target_link_libraries(${pybindings_waveform}
######                      PRIVATE
######                        absl::strings
######                        absl::hash
######                        pybind_protobuf
######                        ${ROOT_LIBRARIES}
######                        ${waveform_LIBRARY_SHARED}
######                        "/usr/lib/libprotobuf.so"
######                        #                        ${Geant4_LIBRARIES}
######                        #                        ${common_LIBRARY_SHARED}
######                        #                        $<$<BOOL:${BUILD_ANALYSIS}>:${analysis_LIBRARY_SHARED}>
######                        #                        $<$<BOOL:${USE_BOOST_PROGRAM_OPTIONS}>:Boost::program_options>
######                        #                        $<$<BOOL:${USE_BOOST_TIMER}>:Boost::timer>
######                        #                        $<$<BOOL:${USE_BOOST_LOG}>:Boost::log_setup>
######                        #                        $<$<BOOL:${USE_BOOST_LOG}>:Boost::log>
######                        #                        $<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
######                        #                        ${simulation_LIBRARY_SHARED}
######                      )
######
######endif(BUILD_PYBINDINGS) 
######                    
######                    
######                    
######                    #if(BUILD_SIMULATION)
#######  foreach(lib ${Geant4_LIBRARIES})
#######  target_link_libraries(${common_LIBRARY_SHARED}
#######                        PUBLIC
#######                          ${lib}
#######                       )
#######  endforeach()
#######endif()
######## tests
######## if enabled, build tests
######## FIXME: The catch header might be downloaded several times
######## however, this has also the advantage to keep all the projects
######## separate and avoid dependency issues in case not all of them
######## are build
#######
#######if(BUILD_TESTS)
#######  set(TEST_EXE
#######    TestCraneLogging
#######    TestCEventMc
#######    TestCEventBase
#######    TestCEventRec
#######    # tracks
#######    TestCTrack
#######    TestGDigitizer
#######    TestInterpolation
#######    TestGAnalysisIdentification
#######    TestCombineEnergyDepositions
#######    )
#######  foreach(source ${TEST_EXE})
#######      add_executable(${source} tests/${source}.cc)
#######      target_include_directories(${source}
#######                                  PRIVATE
#######                                    $<BUILD_INTERFACE:${external_INCLUDE_DIRS}>
#######                                    $<$<BOOL:${BUILD_TESTS}>:${catch_INCLUDE_DIR}>
#######                                )
#######      target_link_libraries(${source}
#######                            PRIVATE
#######                               ${ROOT_LIBRARIES}
#######                               ${common_LIBRARY_SHARED}
#######                               $<$<BOOL:${USE_BOOST_PROGRAM_OPTIONS}>:Boost::program_options>
#######                               $<$<BOOL:${USE_BOOST_TIMER}>:Boost::timer>
#######                               $<$<BOOL:${USE_BOOST_LOG}>:Boost::log_setup>
#######                               $<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
#######                               $<$<BOOL:${USE_BOOST_LOG}>:Boost::log>
#######                           )
#######  endforeach()
#######  if(BUILD_SIMULATION)
#######    foreach(lib ${Geant4_LIBRARIES})
#######    target_link_libraries(TestCombineEnergyDepositions
#######                          PUBLIC
#######                            ${lib}
#######                         )
#######    endforeach()
#######  endif()
#######
#######
#######endif()
#######
