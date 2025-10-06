# this is only to be used with scikit
cmake_minimum_required(VERSION 3.16 FATAL_ERROR)
project(gondola_cxx VERSION 0.11 LANGUAGES C CXX)


find_package(Python 3.8 COMPONENTS Interpreter Development.Module REQUIRED)

find_package(GapsOnlineSoftware 0.11 EXACT REQUIRED)

set (ABS_PATH /srv/gaps/gaps-online-software) 
# this is stupid!
add_subdirectory(${ABS_PATH}/resources/extern/nanobind
    #add_subdirectory(${CMAKE_CURRENT_SOURCE_DIR}/../../resources/extern/nanobind
                 ${CMAKE_BINARY_DIR}/nanobind_build)


set(CXX_PYBINDINGS gondola_cxx)
nanobind_add_module(${CXX_PYBINDINGS} STABLE_ABI NOMINSIZE ${ABS_PATH}/gondola-core/C++/python/gondola_core.cxx)
set_property(TARGET ${CXX_PYBINDINGS} PROPERTY CXX_STANDARD 20)
#target_compile_definitions(${CXX_PYBINDINGS} PUBLIC BUILD_CXX_DB=1)

target_include_directories(${CXX_PYBINDINGS} 
                             SYSTEM PUBLIC
                             $<BUILD_INTERFACE:${ABS_PATH}//resources/extern/spdlog/include>
                             $<$<BOOL:${BUILD_CXX_DB}>:${ABS_PATH}/resources/extern/sqlite_orm/include/sqlite_orm>
)
target_include_directories(${CXX_PYBINDINGS}
                             PUBLIC
                             $<BUILD_INTERFACE:${ABS_PATH}/gondola-core/C++/include>
                             $<BUILD_INTERFACE:${ABS_PATH}//resources/extern/result>
                               $<INSTALL_INTERFACE:include> 
                               $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_INCLUDE_DIRS}>
)
target_link_libraries(${CXX_PYBINDINGS}
                        PUBLIC
                          $<$<BOOL:${BUILD_CXX_WITH_ROOT}>:${ROOT_LIBRARY_SHARED}>
                          $<$<BOOL:${BUILD_CXX_DB}>:${SQLite3_LIBRARIES}>
                          GapsOnlineSoftware::GondolaCore
                          ${ROOT_LIBRARIES}
)
install(TARGETS ${CXX_PYBINDINGS} LIBRARY DESTINATION gondola_cxx)
