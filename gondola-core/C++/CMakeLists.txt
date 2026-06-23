# -- -- -- -- -- -- -- -- -- -- -- -- -- -- 
# CMakeLists.txt for nanobind pybindings 
# -- intended to be used with scikit 
# -- see build.sh script in this directory
# -- -- -- -- -- -- -- -- -- -- -- -- -- --

cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(gondola_cxx VERSION 0.12 LANGUAGES C CXX)

find_package(Python 3.11 COMPONENTS Interpreter Development.Module REQUIRED)

find_package(GapsOnlineSoftware 0.12 EXACT REQUIRED)

find_package(ROOT 6.36.00)
message(STATUS "Requested build using CERN's ROOT library") 
if(ROOT_FOUND)
   include(${ROOT_USE_FILE})
endif(ROOT_FOUND)

# there is a gotcha when building with scikit - it sets up 
# a temporary CMAKE_CURRENT_SOURCE_DIR, and does not copy 
# the whole repository there, so we can't locate the externals 
# and headers the usual way
set (GONDOLA_PATH $ENV{GapsOnlineSoftware_SRC}) 

if (NOT EXISTS "${GONDOLA_PATH}") 
  message(FATAL_ERROR "Need to load gaps-online-software setup-env.sh shell for build!")
else() 
  message(STATUS "Found gaps-online-software src directory at ${GONDOLA_PATH}")
endif()

add_subdirectory(${GONDOLA_PATH}/resources/extern/nanobind
                 ${CMAKE_BINARY_DIR}/nanobind_build)

set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CXX_PYBINDINGS gondola_cxx)
nanobind_add_module(${CXX_PYBINDINGS} STABLE_ABI NOMINSIZE ${GONDOLA_PATH}/gondola-core/C++/python/gondola_core.cxx)
set_property(TARGET ${CXX_PYBINDINGS} PROPERTY CXX_STANDARD 20)
target_compile_definitions(${CXX_PYBINDINGS} PUBLIC BUILD_CXX_WITH_ROOT=1)

target_include_directories(${CXX_PYBINDINGS} 
                             SYSTEM PUBLIC
                             $<BUILD_INTERFACE:${GONDOLA_PATH}//resources/extern/spdlog/include>
                             $<$<BOOL:${BUILD_CXX_DB}>:${GONDOLA_PATH}/resources/extern/sqlite_orm/include/sqlite_orm>
)
target_include_directories(${CXX_PYBINDINGS}
                             PUBLIC
                             $<BUILD_INTERFACE:${GONDOLA_PATH}/gondola-core/C++/include>
                             $<BUILD_INTERFACE:${GONDOLA_PATH}/resources/extern/result>
                             $<INSTALL_INTERFACE:include> 
                             $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
)
target_link_libraries(${CXX_PYBINDINGS}
                        PUBLIC
                          $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_LIBRARY_SHARED}>
                          $<$<BOOL:${BUILD_CXX_DB}>:${SQLite3_LIBRARIES}>
                          GapsOnlineSoftware::GondolaCoreStatic
                          ${ROOT_LIBRARIES}
)
install(TARGETS ${CXX_PYBINDINGS} LIBRARY DESTINATION gondola_cxx)
