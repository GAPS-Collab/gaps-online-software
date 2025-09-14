## ================================================================
## The core C++ library. This is a C++ re-implementation of 
## gondola-core written in RUST

if(BUILD_CXX_WITH_ROOT)
  set(GONDOLA_DICTIONARY G__GapsTof)
endif(BUILD_CXX_WITH_ROOT)

##============================================

## Locate the ROOT package and defines a number of variables (e.g. ROOT_INCLUDE_DIRS)
if(BUILD_CXX_WITH_ROOT AND BUILD_CXX_BINDINGS)
  message(STATUS "-- Using root for gondola-core C++ library. Will build SimpleDet compatibilty layer!")
  set(GONDOLA_DICTIONARY G__GondolaCore CACHE INTERNAL "${LIBRARY_OUTPUT_DIRECTORY}/G__GondolaCore" FORCE)
                   
  # Build ROOT dictionary
  ROOT_GENERATE_DICTIONARY(${GONDOLA_DICTIONARY}
                           ${CMAKE_CURRENT_SOURCE_DIR}/C++/include/sd_legacy.hpp 
                           LINKDEF ${CMAKE_CURRENT_SOURCE_DIR}/C++/include/sd_legacy_linkdef.hpp
  )
endif()  

if (BUILD_CXX_BINDINGS)
  message(STATUS "-- Building gondola-core C++ library.")
  set(GONDOLA_LIBRARY_SHARED GondolaCore CACHE INTERNAL "gondola-core C++ shared library" FORCE)
  add_library(${GONDOLA_LIBRARY_SHARED} SHARED
              C++/src/tof_packet.cxx
              C++/src/tof_event_header.cxx
              C++/src/calibration.cxx
              C++/src/serialization.cxx
              C++/src/parsers.cxx
              C++/src/events.cxx
              C++/src/io.cxx
              C++/src/monitoring.cxx
              C++/src/legacy.cxx
              C++/src/logging.cxx
              C++/src/version.cxx
              C++/src/telemetry_dataclasses.cxx
              C++/src/caraspace.cxx
              $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:C++/src/sd_legacy.cxx>
              $<$<BOOL:${BUILD_CXX_DB}>:C++/src/database.cxx>
              $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${GONDOLA_DICTIONARY}.cxx>
  )
  set_property(TARGET ${GONDOLA_LIBRARY_SHARED} PROPERTY CXX_STANDARD 20)
  target_compile_options(${GONDOLA_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
  target_include_directories(${GONDOLA_LIBRARY_SHARED}
                               PUBLIC
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include>
                                 $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/C++/include>
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result>
                                 $<INSTALL_INTERFACE:include> 
                                 $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
  )
  target_link_libraries(${GONDOLA_LIBRARY_SHARED}
                          PUBLIC
                            $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_LIBRARY_SHARED}>
                            $<$<BOOL:${BUILD_CXX_DB}>:${SQLite3_LIBRARIES}>
                            ${ROOT_LIBRARIES}
  )
  # install the library in the lib directory
  install(TARGETS ${GONDOLA_LIBRARY_SHARED}
      EXPORT GAPSTargets         
      LIBRARY DESTINATION ${INSTALL_LIBRARY_DIR}
      #PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
      #INCLUDES DESTINATION "${GAPS_HEADER_OUTPUT_DIRECTORY}/lib/include"
  )
  install(EXPORT GAPSTargets
      FILE GapsOnlineSoftware.cmake
      NAMESPACE GapsOnlineSoftware::
      DESTINATION ${CMAKE_INSTALL_PREFIX}
  )
endif(BUILD_CXX_BINDINGS)

