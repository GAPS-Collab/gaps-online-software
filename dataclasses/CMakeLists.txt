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


  else()
    message(WARN "Did NOT find google protobuf libraries, can not build dataclasses!")
    set(BUILD_PROTOBUF_DATACLASSES OFF)
  endif(Protobuf_FOUND)
endif(BUILD_PROTOBUF_DATACLASSES)

