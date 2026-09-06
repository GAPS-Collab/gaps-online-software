## ================================================================
## The core C++ library. This is a C++ re-implementation of 
## gondola-core written in RUST

if(BUILD_CXX_WITH_ROOT)
  set(GONDOLA_DICTIONARY G__GondolaCore)
endif(BUILD_CXX_WITH_ROOT)

##============================================

## Locate the ROOT package and defines a number of variables (e.g. ROOT_INCLUDE_DIRS)
if(BUILD_CXX_WITH_ROOT AND BUILD_CXX_BINDINGS)
  message(STATUS "-- Using root for gondola-core C++ library. Will build SimpleDet compatibilty layer!")
  message(STATUS "-- This will requir the DATABASE feature to be turned on!")
  include_directories(
                      ${CMAKE_CURRENT_SOURCE_DIR}/C++/include/ 
                      ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result
                      ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm
  )
  


  set(GONDOLA_DICTIONARY G__GondolaCore CACHE INTERNAL "${LIBRARY_OUTPUT_DIRECTORY}/G__GondolaCore" FORCE)
                   
  # Build ROOT dictionary
  ROOT_GENERATE_DICTIONARY(${GONDOLA_DICTIONARY}
                           ${CMAKE_CURRENT_SOURCE_DIR}/C++/include/sd_legacy.hpp 
                           LINKDEF ${CMAKE_CURRENT_SOURCE_DIR}/C++/include/sd_legacy_linkdef.hpp
  )
  set_property(DIRECTORY . PROPERTY INCLUDE_DIRECTORIES "")
endif()  

##============================================

if (BUILD_CXX_BINDINGS)
  message(STATUS "-- Building gondola-core C++ library.")
  set(GONDOLA_LIBRARY_SHARED GondolaCore CACHE INTERNAL "gondola-core C++ shared library" FORCE)
  add_library(${GONDOLA_LIBRARY_SHARED} SHARED
              C++/src/tof_packet.cxx
              C++/src/calibration.cxx
              C++/src/serialization.cxx
              C++/src/parsers.cxx
              C++/src/rb_event.cxx
              C++/src/rb_event_header.cxx
              C++/src/events.cxx
              C++/src/io.cxx
              C++/src/telemetry_reader.cxx
              C++/src/monitoring.cxx
              C++/src/legacy.cxx
              C++/src/version.cxx
              C++/src/telemetry_dataclasses.cxx
              C++/src/tracker_hit.cxx
              C++/src/caraspace.cxx
              C++/src/tracklet.cxx
              C++/src/cooling.cxx
              C++/src/telemetry_packet.cxx
              $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:C++/src/sd_legacy.cxx>
              $<$<BOOL:${BUILD_CXX_DB}>:C++/src/database.cxx>
              $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${GONDOLA_DICTIONARY}.cxx>
  )
  set_property(TARGET ${GONDOLA_LIBRARY_SHARED} PROPERTY CXX_STANDARD 20)
  target_compile_options(${GONDOLA_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
  if(BUILD_CXX_DB)
    target_compile_definitions(${GONDOLA_LIBRARY_SHARED} PUBLIC BUILD_CXX_DB)
  endif(BUILD_CXX_DB)
  
  target_include_directories(${GONDOLA_LIBRARY_SHARED} 
                               SYSTEM PUBLIC
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include>
                                 $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
  )
  target_include_directories(${GONDOLA_LIBRARY_SHARED}
                               PUBLIC
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/C++/include>
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result>
                                 $<INSTALL_INTERFACE:include> 
                                 $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
  )
  target_link_libraries(${GONDOLA_LIBRARY_SHARED}
                          PUBLIC
                            "$<$<BOOL:${BUILD_CXX_DB}>:${SQLite3_LIBRARIES}>"
                            "$<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_LIBRARIES}>"
  )
  # install the library in the lib directory
  install(TARGETS ${GONDOLA_LIBRARY_SHARED}
      EXPORT GAPSTargets         
      LIBRARY DESTINATION ${INSTALL_LIBRARY_DIR}
      PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
      INCLUDES DESTINATION ${INSTALL_INCLUDE_DIR}
  )
  

  # -- general installation 
  
  if (BUILD_CXX_WITH_ROOT)
    # install the additional ROOT libraries
    install(FILES 
        ${CMAKE_CURRENT_BINARY_DIR}/lib${GONDOLA_LIBRARY_SHARED}_rdict.pcm 
        ${CMAKE_CURRENT_BINARY_DIR}/lib${GONDOLA_LIBRARY_SHARED}.rootmap
        DESTINATION lib
    )
  endif (BUILD_CXX_WITH_ROOT)

  include(CMakePackageConfigHelpers)
  write_basic_package_version_file(
      ${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfigVersion.cmake
      VERSION ${GAPS_ONLINE_SOFTWARE_VERSION}
      COMPATIBILITY AnyNewerVersion
  )

  configure_package_config_file(${CMAKE_CURRENT_SOURCE_DIR}/../GapsOnlineSoftwareConfig.cmake.in
    "${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfig.cmake"
    INSTALL_DESTINATION ${CMAKE_INSTALL_PREFIX} 
  )

  install(FILES
        "${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfig.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfigVersion.cmake"
        DESTINATION ${CMAKE_INSTALL_PREFIX})

  install(EXPORT GAPSTargets
      FILE GapsOnlineSoftware.cmake
      NAMESPACE GapsOnlineSoftware::
      DESTINATION ${CMAKE_INSTALL_PREFIX}
  )
  install(DIRECTORY C++/include/
    DESTINATION ${INSTALL_INCLUDE_DIR}
    FILES_MATCHING PATTERN "*.hxx" PATTERN "*.hpp" PATTERN "*.h"
  )
endif(BUILD_CXX_BINDINGS)
  
# -- static library for pybindings 
if (BUILD_CXX_STATIC_LIB)
  message(STATUS "-- Building gondola-core C++ STATIC library (for the use with nanobind).")
  set(GONDOLA_LIBRARY_STATIC GondolaCoreStatic CACHE INTERNAL "gondola-core C++ static library" FORCE)
  add_library(${GONDOLA_LIBRARY_STATIC} STATIC 
              C++/src/tof_packet.cxx
              C++/src/calibration.cxx
              C++/src/serialization.cxx
              C++/src/parsers.cxx
              C++/src/events.cxx
              C++/src/io.cxx
              C++/src/telemetry_reader.cxx
              C++/src/monitoring.cxx
              C++/src/legacy.cxx
              C++/src/version.cxx
              C++/src/telemetry_dataclasses.cxx
              C++/src/caraspace.cxx
              C++/src/tracklet.cxx
              $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:C++/src/sd_legacy.cxx>
              $<$<BOOL:${BUILD_CXX_DB}>:C++/src/database.cxx>
              $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${GONDOLA_DICTIONARY}.cxx>
  )
  set_target_properties(${GONDOLA_LIBRARY_STATIC} PROPERTIES 
    POSITION_INDEPENDENT_CODE ON
  )
  set_target_properties(${GONDOLA_LIBRARY_STATIC} PROPERTIES 
    EXCLUDE_FROM_ALL FALSE
  )
  
  set_property(TARGET ${GONDOLA_LIBRARY_STATIC} PROPERTY CXX_STANDARD 20)
  if(BUILD_CXX_DB)
      target_compile_definitions(${GONDOLA_LIBRARY_STATIC} PUBLIC BUILD_CXX_DB)
  endif(BUILD_CXX_DB)
  target_compile_options(${GONDOLA_LIBRARY_STATIC} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
  
  target_include_directories(${GONDOLA_LIBRARY_STATIC} 
                               SYSTEM PUBLIC
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include>
                                 $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
  )
  target_include_directories(${GONDOLA_LIBRARY_STATIC}
                               PUBLIC
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/C++/include>
                                 $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result>
                                 $<INSTALL_INTERFACE:include> 
                                 $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
  )
  target_link_libraries(${GONDOLA_LIBRARY_STATIC}
                          PUBLIC
                            "$<$<BOOL:${BUILD_CXX_DB}>:${SQLite3_LIBRARIES}>"
                            "$<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_LIBRARIES}>"
  )
  install(TARGETS ${GONDOLA_LIBRARY_STATIC}
    EXPORT GAPSTargets         
    ARCHIVE DESTINATION lib     # Where the single static .a goes
  )
endif(BUILD_CXX_STATIC_LIB)

