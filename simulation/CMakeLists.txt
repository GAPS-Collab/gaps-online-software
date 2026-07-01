##=========================================
# GAPS ONLINE SOFTWARE/GONDOLA
# CMAKE BUILD SYSTEM
#
# - simulation part 
#==========================================

if(NOT BUILD_CXX_DB)
  message(WARNING "Simulation part requires CXX DB!")
  set(BUILD_SIM OFF) 
endif(NOT BUILD_CXX_DB)

if(NOT BUILD_CXX_BINDINGS)
  message(WARNING "Simulation part requires CXX Bindings!")
  set(BUILD_SIM OFF) 
endif(NOT BUILD_CXX_BINDINGS)
  
find_package(Geant4 11 COMPONENTS gdml)
if(${Geant4_gdml_FOUND})
  message(STATUS "Found Geant4 ${Geant4_VERSION} with gdml support!")
  include(${Geant4_USE_FILE})
  get_filename_component(G4LIBDIR ${Geant4_DIR} DIRECTORY)
  message(STATUS "-- Found Geant4 libraries at ${G4LIBDIR}")
  message(STATUS "-- Found Geant4 include dir at ${Geant4_INCLUDE_DIRS}")
  message(STATUS "-- Will build geometry-builder")
else()
  message(WARNING "Can not find Geant4 C++ package! Simulation part requires Geant4!")
  set(BUILD_SIM OFF)
endif(${Geant4_gdml_FOUND})

if(BUILD_SIM)
  set(GAPSSIM_LIBRARY_SHARED GapsSim CACHE INTERNAL "GapsSim shared library" FORCE)
  add_library(${GAPSSIM_LIBRARY_SHARED} SHARED
    src/volume_store.cxx
    src/materials.cxx
    src/detector.cxx
    src/gun.cxx
    src/event_action.cxx
    src/sensitive_detectors.cxx
    src/mc_hit.cxx 
    src/mc_event.cxx 
    src/sim_config.cxx 
  ) 
    
  set_property(TARGET ${GAPSSIM_LIBRARY_SHARED} PROPERTY CXX_STANDARD 20)
  target_compile_options(${GAPSSIM_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
  target_include_directories(${GAPSSIM_LIBRARY_SHARED}
                             SYSTEM PUBLIC  
                             $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                             $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/tomlplusplus/include>
                          )
  target_include_directories(${GAPSSIM_LIBRARY_SHARED}
                              PUBLIC
                              $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include>
                              #$<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                              $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                              $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result>
                              $<INSTALL_INTERFACE:include> 
                              $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include>
                              ${Geant4_INCLUDE_DIRS}
                          )
  
  target_link_libraries(${GAPSSIM_LIBRARY_SHARED}
                        PUBLIC
                           $<$<BOOL:${BUILD_CXXDB}>:${SQLite3_LIBRARIES}>
                           ${Geant4_LIBRARIES}
  )
  add_executable(geometry-builder bin/geometry-builder.cxx)

  target_include_directories(geometry-builder
                           PUBLIC
                             $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include>
                             $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                             $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include>
                             $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result>
                             $<INSTALL_INTERFACE:include> 
                             $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                             ${Geant4_INCLUDE_DIRS}
                           PRIVATE
                             ${spdlog_SOURCE_DIR}/include
                           )

  target_link_libraries(geometry-builder
                        PRIVATE
                          ${Geant4_LIBRARIES}
                          ${GAPSSIM_LIBRARY_SHARED}
                          ${GONDOLA_LIBRARY_SHARED}
                       )
  install(TARGETS geometry-builder
      DESTINATION bin
  )
  install(TARGETS ${GAPSSIM_LIBRARY_SHARED}
      EXPORT GAPSTargets         
      LIBRARY DESTINATION ${INSTALL_LIBRARY_DIR}
  )
  #install(EXPORT GAPSTargets
  #    FILE GapsOnlineSoftware.cmake
  #    NAMESPACE GapsOnlineSoftware::
  #    DESTINATION ${CMAKE_INSTALL_PREFIX}
  #)
endif(BUILD_SIM)

