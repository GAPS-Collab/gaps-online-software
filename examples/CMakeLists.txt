## ================================================================
## - C++ example binaries 

message(STATUS "-- will build CXX examples!")

foreach(example "unpack-tofpackets" "print-database" "read-caraspace" "calculate-tof")

   #set(unpack-example "unpack-tofpackets")
  add_executable(${example}
    ${CMAKE_CURRENT_SOURCE_DIR}/C++/${example}.cxx
  )
  set_property(TARGET ${example} PROPERTY CXX_STANDARD 20)
  target_include_directories(${example} 
                            SYSTEM PUBLIC
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                            $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
  
  )
  target_include_directories(${example}
                             PUBLIC
                               ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
  )                          
  target_link_libraries(${example}
                        PRIVATE
                        ${GONDOLA_LIBRARY_SHARED})
  install(TARGETS ${example}
          DESTINATION ${INSTALL_EXAMPLES_DIR}
  )
endforeach()

# sd-legacy reader needs ROOT
if (BUILD_CXX_WITH_ROOT)
  set(example "sd-legacy-reader")
  add_executable(${example}
    ${CMAKE_CURRENT_SOURCE_DIR}/C++/${example}.cxx
  )
  set_property(TARGET ${example} PROPERTY CXX_STANDARD 20)
  target_include_directories(${example} 
                            SYSTEM PUBLIC
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/result/
                            $<$<BOOL:${BUILD_CXX_DB}>:${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/sqlite_orm/include/sqlite_orm>
                            $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
  
  )
  target_include_directories(${example}
                             PUBLIC
                               ${CMAKE_CURRENT_SOURCE_DIR}/../gondola-core/C++/include
  )                          
  target_link_libraries(${example}
                        PRIVATE
                        ${GONDOLA_LIBRARY_SHARED}
                        $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_LIBRARY_SHARED}>
  )
  install(TARGETS ${example}
          DESTINATION ${INSTALL_EXAMPLES_DIR}
  )

    
endif()
#
#if (BUILD_CXXDB)
#  # no longer just a paddle example, but also has tracker strips now
#  set(paddle-example "print-database")
#  add_executable(${paddle-example}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/${paddle-example}.cxx
#  )
#  set_property(TARGET ${paddle-example} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${paddle-example} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${paddle-example}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED})
#  install(TARGETS ${paddle-example}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#
#  )
#  
#  # how to calculate TOF values
#  set(calculate-tof-example "calculate-tof")
#  add_executable(${calculate-tof-example}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/${calculate-tof-example}.cxx
#  )
#  set_property(TARGET ${calculate-tof-example} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${calculate-tof-example} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${calculate-tof-example}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED})
#  install(TARGETS ${calculate-tof-example}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#
#  )
#endif (BUILD_CXXDB)
#if (BUILD_CARASPACE)
#  set(caraspace-example "read-caraspace")
#  add_executable(${caraspace-example}
#    ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/read-caraspace.cxx
#  )
#  set_property(TARGET ${caraspace-example} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${caraspace-example} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${caraspace-example}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED})
#  install(TARGETS ${caraspace-example}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#
#  )
#endif (BUILD_CARASPACE)
#if (BUILD_ROOTCOMPONENTS)
#  set(display-example "displayTofpackets")
#  add_executable(${display-example}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/displayTofpackets.cxx
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/MainFrame.cpp
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/Waveform.cpp
#  )
#  set_property(TARGET ${display-example} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${display-example} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
#    		    )
#  target_link_libraries(${display-example}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED}
#    		$<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
#    		ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
#    		)
#  install(TARGETS ${display-example}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#
#      )
#    
#  set(analyze-example "analyzeTofpackets")
#  add_executable(${analyze-example}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeTofpackets.cxx
#  )
#  set_property(TARGET ${analyze-example} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${analyze-example} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${analyze-example}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED})
#  install(TARGETS ${analyze-example}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#  )
#
#  set(display-sydney "displaySydney")
#  add_executable(${display-sydney}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/displaySydney.cxx
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/MainFrame.cpp
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/Waveform.cpp
#  )
#  set_property(TARGET ${display-sydney} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${display-sydney} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_INCLUDE_DIRS}>
#    		    )
#  target_link_libraries(${display-sydney}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED}
#    		$<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
#    		ROOT::Core ROOT::Gui ROOT::RIO ROOT::Net ROOT::Hist
#    		)
#  install(TARGETS ${display-sydney}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#
#      )
#    
#  set(analyze-sydney "analyzeSydney")
#  add_executable(${analyze-sydney}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeSydney.cxx
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventSydney.cpp
#  )
#  set_property(TARGET ${analyze-sydney} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${analyze-sydney} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${analyze-sydney}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED}
#    			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
#    			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
#  		  )
#  install(TARGETS ${analyze-sydney}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#  )
#
#  set(analyze-nevis "analyzeNevis")
#  add_executable(${analyze-nevis}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeNevis.cxx
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventGAPS.cpp
#  )
#  set_property(TARGET ${analyze-nevis} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${analyze-nevis} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${analyze-nevis}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED}
#    			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
#    			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
#  		  )
#  install(TARGETS ${analyze-nevis}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#  )
#
#  set(analyze-rene "analyzeRene")
#  add_executable(${analyze-rene}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzeRene.cxx
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventRene.cpp
#  )
#  set_property(TARGET ${analyze-rene} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${analyze-rene} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${analyze-rene}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED}
#    			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
#    			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
#  		  )
#  install(TARGETS ${analyze-rene}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#  )
#  set(analyze-padrick "analyzePadrick")
#  add_executable(${analyze-padrick}
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/analyzePadrick.cxx
#      ${CMAKE_CURRENT_SOURCE_DIR}/resources/examples/C++/src/EventPadrick.cpp
#  )
#  set_property(TARGET ${analyze-padrick} PROPERTY CXX_STANDARD 20)
#  target_include_directories(${analyze-padrick} 
#                            PUBLIC
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/example/spdlog/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/../resources/extern/cxxopts/include
#                            ${CMAKE_CURRENT_SOURCE_DIR}/dataclasses/C++/include
#                            )
#  target_link_libraries(${analyze-padrick}
#                        PRIVATE
#                        ${GAPSTOF_LIBRARY_SHARED}
#    			  $<$<BOOL:${BUILD_ROOTCOMPONENTS}>:${ROOT_LIBRARY_SHARED}>
#    			  ROOT::Gui ROOT::Core ROOT::RIO ROOT::Net ROOT::Hist ROOT::Tree ROOT::Rint
#  		  )
#  install(TARGETS ${analyze-padrick}
#          DESTINATION ${INSTALL_EXAMPLES_DIR}
#  )
