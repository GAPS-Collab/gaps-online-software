## ================================================================
## Personal analysis - C++ code.
## Supports ROOT if needed

message(STATUS "-- will build personal analysis projects!")
option(BUILD_ZWEERINK "build Jeff's tof analysis code" ON)
if (BUILD_ZWEERINK)
    if (NOT BUILD_CXX_WITH_ROOT)
    message(WARNING "-- -- unable to build Jeff's code without ROOT support. Make sure 'USE_ROOT=ON' and ROOT is found. E.g. by loading ROOT's thisroot.sh shell!")
    message(WARNING "-- -- will disable building of Jeff's code")
    option(BUILD_ZWEERINK "build Jeff's tof analysis code" OFF)
  else()
    
    #-----------------------------
    # build the different binaries
    # 
    
    set(src_datapath "zweerink")
    
    # -- displayTofpackets
    foreach(executable "displayTofpackets"  "displaySydney")
        #set(executable "displayTofpackets")
      add_executable(${executable}
        ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/${executable}.cxx
        ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/src/MainFrame.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/src/Waveform.cpp
      )
      set_property(TARGET ${executable} PROPERTY CXX_STANDARD 20)
      target_include_directories(${executable} 
                                 SYSTEM PUBLIC
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                                     $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                     $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
      )
      target_include_directories(${executable}
                                 PUBLIC
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
      )                          
      target_link_libraries(${executable}
                              PRIVATE
                                ${GONDOLA_LIBRARY_SHARED}
                                ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      )
      install(TARGETS ${executable}
          DESTINATION ${INSTALL_ANALYSIS_DIR}/zweerink
      )
    endforeach()  
    
    # -- anlayzeTofpackets
    set(executable "analyzeTofpackets")
    add_executable(${executable}
      ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/${executable}.cxx
    )
    set_property(TARGET ${executable} PROPERTY CXX_STANDARD 20)
    target_include_directories(${executable} 
                               SYSTEM PUBLIC
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                                   $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                   $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
    )
    target_include_directories(${executable}
                               PUBLIC
                                 ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
    )                          
    target_link_libraries(${executable}
                            PRIVATE
                              ${GONDOLA_LIBRARY_SHARED}
                              ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
    )
    install(TARGETS ${executable}
        DESTINATION ${INSTALL_ANALYSIS_DIR}/zweerink
    )
    
    foreach(executable "analyzeSydney")
      #set(executable "analyzeSydney")
      add_executable(${executable}
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/${executable}.cxx
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/src/EventSydney.cpp
      )
      set_property(TARGET ${executable} PROPERTY CXX_STANDARD 20)
      target_include_directories(${executable} 
                                 SYSTEM PUBLIC
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                                     $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                     $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
      )
      target_include_directories(${executable}
                                 PUBLIC
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
      )                          
      target_link_libraries(${executable}
                              PRIVATE
                                ${GONDOLA_LIBRARY_SHARED}
                                ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      )
      install(TARGETS ${executable}
          DESTINATION ${INSTALL_ANALYSIS_DIR}/zweerink
      )
    endforeach() 
    
    foreach(executable "analyzeNevis")
      #set(executable "analyzeSydney")
      add_executable(${executable}
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/${executable}.cxx
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/src/EventGAPS.cpp
      )
      set_property(TARGET ${executable} PROPERTY CXX_STANDARD 20)
      target_include_directories(${executable} 
                                 SYSTEM PUBLIC
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                                     $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                     $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
      )
      target_include_directories(${executable}
                                 PUBLIC
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
      )                          
      target_link_libraries(${executable}
                              PRIVATE
                                ${GONDOLA_LIBRARY_SHARED}
                                ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      )
      install(TARGETS ${executable}
          DESTINATION ${INSTALL_ANALYSIS_DIR}/zweerink
      )
    endforeach() 
    
    foreach(executable "analyzeRene")
      #set(executable "analyzeSydney")
      add_executable(${executable}
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/${executable}.cxx
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/src/EventRene.cpp
      )
      set_property(TARGET ${executable} PROPERTY CXX_STANDARD 20)
      target_include_directories(${executable} 
                                 SYSTEM PUBLIC
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                                     $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                     $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
      )
      target_include_directories(${executable}
                                 PUBLIC
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
      )                          
      target_link_libraries(${executable}
                              PRIVATE
                                ${GONDOLA_LIBRARY_SHARED}
                                ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      )
      install(TARGETS ${executable}
          DESTINATION ${INSTALL_ANALYSIS_DIR}/zweerink
      )
    endforeach() 
    
    foreach(executable "analyzePadrick")
      #set(executable "analyzeSydney")
      add_executable(${executable}
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/${executable}.cxx
                       ${CMAKE_CURRENT_SOURCE_DIR}/${src_datapath}/src/EventPadrick.cpp
      )
      set_property(TARGET ${executable} PROPERTY CXX_STANDARD 20)
      target_include_directories(${executable} 
                                 SYSTEM PUBLIC
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                                     ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                                     $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                                     $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
      )
      target_include_directories(${executable}
                                 PUBLIC
                                   ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
      )                          
      target_link_libraries(${executable}
                              PRIVATE
                                ${GONDOLA_LIBRARY_SHARED}
                                ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
      )
      install(TARGETS ${executable}
          DESTINATION ${INSTALL_ANALYSIS_DIR}/zweerink
      )
    endforeach() 
  endif() 
endif(BUILD_ZWEERINK)

