option(BUILD_PROTOBUF_DATACLASSES "Lightweight dataclasses for de/serialization and debugging, using google protocol buffers" ON)

if (BUILD_PROTOBUF_DATACLASSES)
  message(STATUS "Building google-protobuf dataclasses..")
  find_package(Protobuf REQUIRED)    
  if (Protobuf_FOUND)
    set(PROTO_SRC_DIR ${CMAKE_CURRENT_SOURCE_DIR}/google-protobuf)
    message(STATUS "Building CXX headers in ${PROTO_SRC_DIR}...")
    protobuf_generate_cpp(TOF_EVENT_PB_SRC
                          TOF_EVENT_PB_HDR 
                          ${PROTO_SRC_DIR}/TofEvent.proto)
    protobuf_generate_cpp(TRK_EVENT_PB_SRC
                          TRK_EVENT_PB_HDR 
                          ${PROTO_SRC_DIR}/TrackerEvent.proto)
    protobuf_generate_cpp(GAPS_EVENT_PB_SRC
                          GAPS_EVENT_PB_HDR 
                          ${PROTO_SRC_DIR}/GapsEvent.proto)

    message(STATUS "Building PY headers...")
    protobuf_generate_python(TOF_EVENT_PB_PY
                             ${PROTO_SRC_DIR}/TofEvent.proto)
    protobuf_generate_python(TRK_EVENT_PB_PY
                             ${PROTO_SRC_DIR}/TrackerEvent.proto)
    protobuf_generate_python(GAPS_EVENT_PB_PY
                             ${PROTO_SRC_DIR}/GapsEvent.proto)

    file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include) 
    file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/src) 
    file(MAKE_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/python) 


    add_custom_target(protobuf-dataclasses-build DEPENDS ${TOF_EVENT_PB_HDR}
                                                  ${TOF_EVENT_PB_SRC}
                                                  ${TRK_EVENT_PB_HDR}
                                                  ${TRK_EVENT_PB_SRC}
                                                  ${GAPS_EVENT_PB_HDR}
                                                  ${GAPS_EVENT_PB_SRC}
                                                  ${TOF_EVENT_PB_PY}
                                                  ${TRK_EVENT_PB_PY}
                                                  ${GAPS_EVENT_PB_PY}
                                                  WORKING_DIRECTORY ${PROTO_SRC_DIR}/
                                              )
    # for some very strange reason, this doesn't work with the defined variables
    # for the files FIXME
    add_custom_target(protobuf-dataclasses DEPENDS protobuf-dataclasses-build
        COMMAND mv TofEvent.pb.cc TrackerEvent.pb.cc GapsEvent.pb.cc ${CMAKE_CURRENT_SOURCE_DIR}/src
        COMMAND mv TofEvent.pb.h TrackerEvent.pb.h GapsEvent.pb.h ${CMAKE_CURRENT_SOURCE_DIR}/include 
        COMMAND mv TofEvent_pb2.py TrackerEvent_pb2.py GapsEvent_pb2.py ${CMAKE_CURRENT_SOURCE_DIR}/python
        COMMENT "Copying generated protobuf classes.."
        BYPRODUCTS dataclasses/include/TofEvent.pb.h
                   dataclasses/src/TofEvent.pb.cc
                   dataclasses/python/TofEvent_pb2.py
                   dataclasses/include/TrackerEvent.pb.h
                   dataclasses/src/TrackerEvent.pb.cc
                   dataclasses/python/TrackerEvent_pb2.py
                   dataclasses/include/GapsEvent.pb.h
                   dataclasses/src/GapsEvent.pb.cc
                   dataclasses/python/GapsEvent_pb2.py
    )

    set(GAPS_GP_DATACLASSES GapsPBDataclasses CACHE INTERNAL "Gaps protocoll buffer dawtaclasses shared library" FORCE)
    add_library(${GAPS_GP_DATACLASSES} SHARED 
        ${CMAKE_CURRENT_SOURCE_DIR}/src/TofEvent.pb.cc
        ${CMAKE_CURRENT_SOURCE_DIR}/src/TrackerEvent.pb.cc
        ${CMAKE_CURRENT_SOURCE_DIR}/src/GapsEvent.pb.cc
    )
    add_dependencies(${GAPS_GP_DATACLASSES} protobuf-dataclasses)
    target_compile_options(${GAPS_GP_DATACLASSES} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
    target_include_directories(${GAPS_GP_DATACLASSES}
                            PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/include
                              #$<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
                              #${Boost_INCLUDE_DIR}
                          )
    target_link_libraries(${GAPS_GP_DATACLASSES}
                      PUBLIC
                        ${Protobuf_LIBRARIES}
    #                     $(<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
    #                     $(<$<BOOL:${USE_BOOST_LOG}>:Boost::log_setup>
    #                     #($<$<BOOL:${USE_GOOGLEPERFTOOLS}>:${GOOGLEPERFTOOLS_LIB}>
    #                     $(<$<BOOL:${USE_BOOST_LOG}>:Boost::log>
                    )
#

    if(BUILD_PYBINDINGS)
      message(STATUS "Building pybindings for protobuf dataclasses...")
      message(STATUS " ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/pybind11_protobuf")

      set(CMAKE_POSITION_INDEPENDENT_CODE TRUE)
      set(pybindings "pb_dataclasses")
      # now it gets a bit complicated, since we want to have that
      # nice pybind11 protobuf going
      add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/abseil-cpp)

      file(GLOB pybind11_protobuf_SRC ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/pybind11_protobuf/pybind11_protobuf/*.cc)
      pybind11_add_module(${pybindings} MODULE
                            ${pybind11_protobuf_SRC}
                            ${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/abseil-cpp
                            pybindings/WaveGAPS.cpp
                            pybindings/waveform.cxx)
                         #110     target_include_directories(_pyWaveform
                         #111       PUBLIC
                         #112       "${CMAKE_CURRENT_SOURCE_DIR}/external/pybind11_protobuf"
                         #113       "${CMAKE_CURRENT_SOURCE_DIR}/external/abseil-cpp"
                         #114       "${CMAKE_CURRENT_SOURCE_DIR}/external/protobuf"
                         #115       "${CMAKE_CURRENT_SOURCE_DIR}/TOFsoftware/analysis/include"
                         #116       "${CMAKE_CURRENT_SOURCE_DIR}/dataclasses"
                         #117     )
      target_include_directories(${pybindings}
                            PUBLIC
                              "${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/abseil-cpp" 
                              "${CMAKE_CURRENT_SOURCE_DIR}/resources/extern/protobuf"
                              "${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/pybind11_protobuf/"
                              "${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/pybind11_protobuf/pybind11_protobuf/"
                              "${CMAKE_CURRENT_SOURCE_DIR}/include"
                            )
                            #get_property(the_include_dirs TARGET ${pybindings} PROPERTY INCLUDE_DIRECTORIES)
                            #foreach(dir ${the_include_dirs})
                            #message(STATUS "dir='${dir}'")
                            #endforeach()
                            #list(REMOVE_ITEM ${the_include_dirs} "/usr/include/python2.7")
                            #list(APPEND ${the_include_dirs} "/usr/include/python3.8")
                            #string(REPLACE "/usr/include/python2.7" ";;"  new_include_dirs ${the_include_dirs})
                            #list(APPEND ${new_include_dirs} "/usr/include/python3.8")
                            ##set(pybind_include_dirs "/usr/include/python3.8"  "${CMAKE_CURRENT_SOURCE_DIR}/include") 
                            #set(pybind_include_dirs ${new_include_dirs})
                            #foreach(dir ${pybind_include_dirs})
                            #message(STATUS "dir='${dir}'")
                            #endforeach()
                            #set_property(TARGET ${pybindings} PROPERTY INCLUDE_DIRECTORIES ${pybind_include_dirs})
      target_link_libraries(${pybindings}
                          PRIVATE
                          ${Protobuf_LIBRARIES}
                          absl::strings
                          absl::hash
                          absl::low_level_hash
                          absl::core_headers
                          absl::btree
                          absl::flat_hash_map
                          absl::flat_hash_set
                          absl::node_hash_map
                          absl::node_hash_set
                          #${GAPSSERVER_LIBRARY_SHARED})
                          )
      add_dependencies(${pybindings} protobuf-dataclasses)

    endif(BUILD_PYBINDINGS)


  else()
    message(WARN "Did NOT find google protobuf libraries, can not build dataclasses!")
    set(BUILD_PROTOBUF_DATACLASSES OFF)
  endif(Protobuf_FOUND)
endif(BUILD_PROTOBUF_DATACLASSES)

