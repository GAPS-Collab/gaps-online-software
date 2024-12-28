execute_process(
  COMMAND cargo --version
  RESULT_VARIABLE CARGO_RESULT
  OUTPUT_QUIET
)

if(CARGO_RESULT EQUAL 0)
    message(STATUS "Found `cargo` - will be able to build liftof!")
else()
  message(WARNING "-- `cargo` NOT found! Unable to build rust pybindings.")
  message(STATUS "Please install a rust toolchain including cargo if you want to build liftof, and make sure cargo is in your $PATH!")
  option (BUILD_RUSTPYBINDINGS OFF)
endif()
  
# make sure everything gets cleaned up so that cmake notices that 
# there might be a change
add_custom_target(cleanup_pybindings
  COMMAND ${CMAKE_COMMAND} -E remove "${INSTALL_PYTHON_DIR}/go_pybindings.so"
)

if(BUILD_RUSTPYBINDINGS)
  # FXIME - assemble the featrues string prgromaatically
  # this here acutally does not really work
  set(RUST_BUILD_COMMAND_PLAIN cargo build --release --all-features)
  set(RUST_BUILD_COMMAND_LIFTOF cargo build --release --features=liftof)
  set(RUST_BUILD_COMMAND_TELEMETRY cargo build --release --features=telemetry)
  set(RUST_BUILD_COMMAND_CARASPACE cargo build --release --features=caraspace-serial)
  set(RUST_BUILD_COMMAND_FEATALL cargo build --release --all-features)
  # Define the command to build your Rust project
  message(STATUS "-- -- building Rust \\U1F980 pybindings wiut pyO3...")
  if (BUILD_LIFTOF AND NOT BUILD_RUSTTELEMETRY)
    message(STATUS "-- -- -- adding liftof pybindings.")
    add_custom_target(
      go-pybindings
        ALL
        COMMAND ${RUST_BUILD_COMMAND_LIFTOF} 
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/pybindings/
    )
  
  elseif(BUILD_RUSTTELEMETRY AND NOT BUILD_LIFTOF)
    message(STATUS "-- -- -- adding telemetry pybindings.")
    add_custom_target(
      go-pybindings
        ALL
        COMMAND ${RUST_BUILD_COMMAND_TELEMETRY} 
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/pybindings/
    )
  elseif(NOT BUILD_RUSTTELEMETRY AND NOT BUILD_LIFTOF)
    add_custom_target(
      go-pybindings
        ALL
        COMMAND ${RUST_BUILD_COMMAND_PLAIN} 
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/pybindings/
    )
  elseif(BUILD_RUSTTELEMETRY AND BUILD_LIFTOF)
    message(STATUS "-- -- -- adding telemetry and liftof pybindings.")
    add_custom_target(
      go-pybindings
        ALL
        COMMAND ${RUST_BUILD_COMMAND_FEATALL} 
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/pybindings/
    )
  endif()
  # add the cleanup process before compiling
  add_dependencies(go-pybindings cleanup_pybindings)

  # documentation
  add_custom_target(
    go-pybindings-rust-doc
      COMMAND cargo doc 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/pybindings
  )

  #add_custom_target(
  #  go-sphinx-doc
  #    COMMAND ./build-sphinx.sh
  #    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/doc
  #)
  
  # install the files, taking care of APPLE's dylib ending
  if(APPLE) 
    install(FILES
            "${CMAKE_SOURCE_DIR}/python/pybindings/target/release/libgo_pybindings.dylib"
            DESTINATION ${INSTALL_PYTHON_DIR}
            RENAME go_pybindings.so
    )
   elseif(UNIX)
    install(FILES
            "${CMAKE_SOURCE_DIR}/python/pybindings/target/release/libgo_pybindings.so"
            DESTINATION ${INSTALL_PYTHON_DIR}
            RENAME go_pybindings.so
    )
  endif(APPLE)

endif(BUILD_RUSTPYBINDINGS)
