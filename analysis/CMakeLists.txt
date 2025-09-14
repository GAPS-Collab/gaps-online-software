## ================================================================
## Personal analysis - C++ code.
## Supports ROOT if needed

message(STATUS "-- will build personal analysis projects!")
option(BUILD_ZWEERINK "build Jeff's tof analysis code")
if (BUILD_ZWEERINK)
  if (NOT USE_ROOT)
    message(ERROR "-- -- unable to build Jeff's code without ROOT support. Make sure 'USE_ROOT=ON' and ROOT is found. E.g. by loading ROOT's thisroot.sh shell!")
    message(WARN "-- -- will disable building of Jeff's code")
    option(BUILD_ZWEERINK "build Jeff's tof analysis code" OFF)
  else()
    
    #-----------------------------
    # build the different binaries
    # 
    # -- displayTofpackets
    set(display-example "displayTofpackets")
    add_executable(${display-example}
      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/displayTofpackets.cxx
      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/MainFrame.cpp
      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/Waveform.cpp
    )
    set_property(TARGET ${display-example} PROPERTY CXX_STANDARD 20)
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
    )
    install(TARGETS ${display-example}
            DESTINATION ${INSTALL_EXAMPLES_DIR}
    )
      
    # analyzeTofpackets"
    set(analyze-example "analyzeTofpackets")
    add_executable(${analyze-example}
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeTofpackets.cxx
    )
    set_property(TARGET ${analyze-example} PROPERTY CXX_STANDARD 20)
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
    # display-sydney 
    add_executable(${display-sydney}
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/displaySydney.cxx
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/MainFrame.cpp
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/Waveform.cpp
    )
    set_property(TARGET ${display-sydney} PROPERTY CXX_STANDARD 20)
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
    
    # analyze Sydney  
    set(analyze-sydney "analyzeSydney")
    add_executable(${analyze-sydney}
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeSydney.cxx
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventSydney.cpp
    )
    set_property(TARGET ${analyze-sydney} PROPERTY CXX_STANDARD 20)
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

    # analyze Nevis
    set(analyze-nevis "analyzeNevis")
    add_executable(${analyze-nevis}
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeNevis.cxx
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventGAPS.cpp
    )
    set_property(TARGET ${analyze-nevis} PROPERTY CXX_STANDARD 20)
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

    # analyze Rene
    set(analyze-rene "analyzeRene")
    add_executable(${analyze-rene}
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeRene.cxx
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventRene.cpp
    )
    set_property(TARGET ${analyze-rene} PROPERTY CXX_STANDARD 20)
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
    
    # analyze Padrick
    set(analyze-padrick "analyzePadrick")
    add_executable(${analyze-padrick}
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzePadrick.cxx
                   ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventPadrick.cpp
    )
    set_property(TARGET ${analyze-padrick} PROPERTY CXX_STANDARD 20)
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
  endif() 
endif(BUILD_ZWEERINK)

