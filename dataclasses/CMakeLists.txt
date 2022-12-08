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
    add_custom_target(protobuf-dataclasses DEPENDS protobuf-dataclasses-build
        COMMAND mv TofEvent.pb.cc TofEvent.pb.h TofEvent_pb2.py TrackerEvent.pb.h TrackerEvent.pb.cc TrackerEvent_pb2.py GapsEvent.pb.h GapsEvent.pb.cc GapsEvent_pb2.py ${CMAKE_CURRENT_SOURCE_DIR})
  else()
    message(WARN "Did NOT find google protobuf libraries, can not build dataclasses!")
    set(BUILD_PROTOBUF_DATACLASSES OFF)
  endif(Protobuf_FOUND)
endif(BUILD_PROTOBUF_DATACLASSES)

