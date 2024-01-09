## ================================================================
## cmake build type
## can be either "Release" or "Debug"

## in case of Release build, we set the NDEBUG preprocessor directive
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -DNDEBUG=1")

################

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

set(GAPSTOF_LIBRARY_SHARED GapsTof CACHE INTERNAL "GapsTof shared library" FORCE)
add_library(${GAPSTOF_LIBRARY_SHARED} SHARED
    dataclasses/C++/src/tof_packet.cxx
    dataclasses/C++/src/tof_event_header.cxx
    dataclasses/C++/src/calibration.cxx
    dataclasses/C++/src/serialization.cxx
    dataclasses/C++/src/parsers.cxx
    dataclasses/C++/src/events.cxx
    dataclasses/C++/src/io.cxx
    dataclasses/C++/src/monitoring.cxx
    dataclasses/C++/src/CommandPacket.cxx
    dataclasses/C++/src/legacy.cxx
)
target_compile_options(${GAPSTOF_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
target_include_directories(${GAPSTOF_LIBRARY_SHARED}
                            PUBLIC
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
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

  pybind11_add_module(${pybindings} MODULE dataclasses/C++/pybindings/module.cxx 
                                           dataclasses/C++/pybindings/helpers.cxx)
  target_include_directories(${pybindings}
                            PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
            			      ${Python_INCLUDE_DIRS}
			                )
  target_link_libraries(${pybindings}
                        PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED})

endif(BUILD_PYBINDINGS)

########################################
# LIFTOF - GAPS TOF flight software
########################################

# rust part. Check if cargo is available
option(BUILD_LIFTOF "build liftof tof flight software suite" OFF)
execute_process(
  COMMAND cargo --version
  RESULT_VARIABLE CARGO_RESULT
  OUTPUT_QUIET
)

if (BUILD_LIFTOF)
  if(CARGO_RESULT EQUAL 0)
    message(STATUS "-- found `cargo` - will be able to build liftof!")
  else()
    message(WARNING "-- `cargo` NOT found! Unable to build liftof.")
    message(STATUS "Please install a rust toolchain including cargo if you want to build liftof, and make sure cargo is in your $PATH!")
    option (BUILD_LIFTOF OFF)
  endif()
endif()

if (BUILD_LIFTOF)
  
  # Define the command to build your Rust project
  set(RUST_BUILD_COMMAND cargo build --release --all-features)
  add_custom_target(
    liftof-doc
      COMMAND cargo doc 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof
  )
  add_custom_target(
    rust-pybindings
      ALL
      COMMAND ${RUST_BUILD_COMMAND} 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/dataclasses/rust/pybindings 
  )
  add_custom_target(
    liftof-cc
      ALL
      COMMAND ${RUST_BUILD_COMMAND}
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof/liftof-cc
  )
  add_custom_target(
    liftof-mt
      ALL
      COMMAND cargo build --release --bin=liftof-mt
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof/liftof-cc
  )
  add_custom_target(
    liftof-analysis
      ALL
      COMMAND cargo build --release --bin=liftof-robin-reader
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof/liftof-analysis
  )
  add_custom_target(
    liftof-tui
      ALL
      COMMAND ${RUST_BUILD_COMMAND}
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof/liftof-tui
  )
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-cc
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-mt
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-robin-reader
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-tui
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(FILES ${CMAKE_SOURCE_DIR}/tof/dataclasses/rust/pybindings/target/release/librust_dataclasses.so
      DESTINATION ${INSTALL_PYTHON_DIR}/
      RENAME rust_dataclasses.so
  )

endif(BUILD_LIFTOF)

#################################################
# installation
#################################################
set(INSTALL_EXAMPLES_DIR "${CMAKE_INSTALL_PREFIX}/examples" CACHE PATH "installation directory for exanokes" FORCE)

# Make relative paths absolute (needed later on)
foreach(p EXAMPLES)
  set(var INSTALL_${p}_DIR)
  if(NOT IS_ABSOLUTE "${${var}}")
    set(${var} "${CMAKE_INSTALL_PREFIX}/${${var}}")
  endif()
endforeach()

file(GLOB python_example_scripts "${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/python/*.py")
install(FILES ${python_example_scripts}
    DESTINATION ${INSTALL_EXAMPLES_DIR}
    PERMISSIONS OWNER_EXECUTE
)

# install the library in the lib directory
install(TARGETS ${GAPSTOF_LIBRARY_SHARED}
    EXPORT GAPSTargets         
    LIBRARY DESTINATION ${INSTALL_LIBRARY_DIR}
    PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
    #INCLUDES DESTINATION "${GAPS_HEADER_OUTPUT_DIRECTORY}/lib/include"
)

install(TARGETS ${pybindings}
    EXPORT GAPSTargets         
    LIBRARY DESTINATION ${INSTALL_PYTHON_DIR}
    #PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
    #INCLUDES DESTINATION "${GAPS_HEADER_OUTPUT_DIRECTORY}/lib/include"
)

#######################################
# Examples
#######################################

option(BUILD_EXAMPLES "build C++ example for data i/o" OFF)

if (BUILD_EXAMPLES)
  set(unpack-example "unpack-tofpackets")
  add_executable(${unpack-example}
      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/unpack-tofpackets.cxx
  )
  target_include_directories(${unpack-example} 
                            PUBLIC
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                            )
  target_link_libraries(${unpack-example}
                        PRIVATE
                        ${GAPSTOF_LIBRARY_SHARED})
  install(TARGETS ${unpack-example}
          DESTINATION ${INSTALL_EXAMPLES_DIR}

  )
  if (BUILD_ROOTCOMPONENTS)
    set(display-example "displayTofpackets")
    add_executable(${display-example}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/displayTofpackets.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/MainFrame.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/Waveform.cpp
    )
    target_include_directories(${display-example} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
      		    )
    target_link_libraries(${display-example}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED}
      		$<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
      		ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      		)
    install(TARGETS ${display-example}
            DESTINATION ${INSTALL_EXAMPLES_DIR}

        )
      
    set(analyze-example "analyzeTofpackets")
    add_executable(${analyze-example}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeTofpackets.cxx
    )
    target_include_directories(${analyze-example} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              )
    target_link_libraries(${analyze-example}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED})
    install(TARGETS ${analyze-example}
            DESTINATION ${INSTALL_EXAMPLES_DIR}
    )
  endif(BUILD_ROOTCOMPONENTS)
endif()




