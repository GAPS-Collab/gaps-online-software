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
#  find_package(ROOT 6.26.00)
  find_package(ROOT 6.30.00)
  
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
    dataclasses/C++/src/legacy.cxx
    dataclasses/C++/src/logging.cxx
)

target_compile_options(${GAPSTOF_LIBRARY_SHARED} PRIVATE -Wall -Wextra -Wpedantic)# -Werror)
target_include_directories(${GAPSTOF_LIBRARY_SHARED}
                            PUBLIC
                            $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include>
                            $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include>
                            $<INSTALL_INTERFACE:include> 
                            #${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                            #${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
                              #  ${Boost_INCLUDE_DIR}
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
                              $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include>
                              $<INSTALL_INTERFACE:include> 
                              #${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
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

if (BUILD_LIFTOF OR BUILD_RUSTPYBIDINGS)
  if(CARGO_RESULT EQUAL 0)
    message(STATUS "-- found `cargo` - will be able to build liftof!")
  else()
    message(WARNING "-- `cargo` NOT found! Unable to build liftof.")
    message(STATUS "Please install a rust toolchain including cargo if you want to build liftof, and make sure cargo is in your $PATH!")
    option (BUILD_LIFTOF OFF)
    option (BUILD_RUSTPYBIDINGS OFF)
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
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof/liftof-python 
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
    liftof-tui
      ALL
      COMMAND ${RUST_BUILD_COMMAND}
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/liftof/liftof-tui
  )
  
  # installation directives for liftof components
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-cc
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-mt
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(PROGRAMS ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/liftof-tui
      DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
  )
  install(FILES ${CMAKE_SOURCE_DIR}/tof/liftof/target/release/librust_dataclasses.so
      DESTINATION ${INSTALL_PYTHON_DIR}/
      RENAME rust_dataclasses.so
  )
endif(BUILD_LIFTOF)

if(BUILD_RUSTPYBIDINGS)
  # Define the command to build your Rust project
  message(STATUS "building Rust \\U+1F980 pybindings wiut pyO3...")
  set(RUST_BUILD_COMMAND cargo build --release --all-features)
  add_custom_target(
    tof-pydataclasses-rust
      ALL
      COMMAND ${RUST_BUILD_COMMAND} 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/dataclasses/rust/pybindings
  )
  add_custom_target(
    tof-pydataclasses-rust-doc
      COMMAND cargo doc 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/tof/dataclasses/rust/pybindings
  )
  #install(TARGETS tof-pydataclasses-rust
  #    EXPORT GAPSTargets         
  #    LIBRARY DESTINATION ${INSTALL_PYTHON_DIR}
  #    #PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
  #    #INCLUDES DESTINATION "${GAPS_HEADER_OUTPUT_DIRECTORY}/lib/include"
  #)
  install(FILES
          "${CMAKE_SOURCE_DIR}/tof/dataclasses/rust/pybindings/target/release/librpy_tof_dataclasses.so"
          DESTINATION ${INSTALL_PYTHON_DIR}
          RENAME rpy_tof_dataclasses.so
  )

endif(BUILD_RUSTPYBIDINGS)

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
    #PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
    #INCLUDES DESTINATION "${GAPS_HEADER_OUTPUT_DIRECTORY}/lib/include"
)

install(EXPORT GAPSTargets
    FILE GapsOnlineSoftware.cmake
    NAMESPACE GapsOnlineSoftware::
    DESTINATION ${CMAKE_INSTALL_PREFIX}
)

# not sure about this
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    "GapsOnlineSoftwareConfigVersion.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY AnyNewerVersion)

configure_package_config_file(${CMAKE_CURRENT_SOURCE_DIR}/../GapsOnlineSoftwareConfig.cmake.in
    "${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfig.cmake"
    INSTALL_DESTINATION ${CMAKE_INSTALL_PREFIX})

install(FILES
        "${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfig.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/GapsOnlineSoftwareConfigVersion.cmake"
        DESTINATION ${CMAKE_INSTALL_PREFIX})

install(DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
    DESTINATION ${CMAKE_INSTALL_PREFIX})

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

    set(display-sydney "displaySydney")
    add_executable(${display-sydney}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/displaySydney.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/MainFrame.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/Waveform.cpp
    )
    target_include_directories(${display-sydney} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
      		    )
    target_link_libraries(${display-sydney}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED}
      		$<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
      		ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      		)
    install(TARGETS ${display-sydney}
            DESTINATION ${INSTALL_EXAMPLES_DIR}

        )
      
    set(analyze-sydney "analyzeSydney")
    add_executable(${analyze-sydney}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeSydney.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventSydney.cpp
    )
    target_include_directories(${analyze-sydney} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              )
    target_link_libraries(${analyze-sydney}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED}
      			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
      			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
			  )
    install(TARGETS ${analyze-sydney}
            DESTINATION ${INSTALL_EXAMPLES_DIR}
    )

    set(analyze-nevis "analyzeNevis")
    add_executable(${analyze-nevis}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeNevis.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventGAPS.cpp
    )
    target_include_directories(${analyze-nevis} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              )
    target_link_libraries(${analyze-nevis}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED}
      			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
      			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
			  )
    install(TARGETS ${analyze-nevis}
            DESTINATION ${INSTALL_EXAMPLES_DIR}
    )

    set(analyze-rene "analyzeRene")
    add_executable(${analyze-rene}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeRene.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventRene.cpp
    )
    target_include_directories(${analyze-rene} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              )
    target_link_libraries(${analyze-rene}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED}
      			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
      			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
			  )
    install(TARGETS ${analyze-rene}
            DESTINATION ${INSTALL_EXAMPLES_DIR}
    )
    set(analyze-padrick "analyzePadrick")
    add_executable(${analyze-padrick}
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzePadrick.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventPadrick.cpp
    )
    target_include_directories(${analyze-padrick} 
                              PUBLIC
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                              ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
                              )
    target_link_libraries(${analyze-padrick}
                          PRIVATE
                          ${GAPSTOF_LIBRARY_SHARED}
      			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
      			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
			  )
    install(TARGETS ${analyze-padrick}
            DESTINATION ${INSTALL_EXAMPLES_DIR}
    )

  endif(BUILD_ROOTCOMPONENTS)
endif()




