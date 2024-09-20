
execute_process(
  COMMAND cargo --version
  RESULT_VARIABLE CARGO_RESULT
  OUTPUT_QUIET
)

if (BUILD_RUSTTELEMETRY)
  if(CARGO_RESULT EQUAL 0)
    message(STATUS "-- found `cargo` - will be able to build rust telemetry pybindings!")
  else()
    message(WARNING "-- `cargo` NOT found! Unable to build rust telemetry pybindings.")
    message(STATUS "Please install a rust toolchain including cargo if you want to build the rust telemetry pybindings, and make sure cargo is in your $PATH!")
    option (BUILD_RUSTTELEMETRY OFF)
  endif()
endif(BUILD_RUSTTELEMETRY)

if(BUILD_RUSTTELEMETRY)
  # Define the command to build your Rust project
  message(STATUS "building Rust \\U1F980 pybindings wiut pyO3...")
  set(RUST_BUILD_COMMAND cargo build --release --all-features)
  add_custom_target(
    telemetry-pydataclasses-rust
      ALL
      COMMAND ${RUST_BUILD_COMMAND} 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/telemetry/dataclasses/rust/pybindings
  )
  add_custom_target(
    telemetry-pydataclasses-rust-doc
      COMMAND cargo doc 
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/telemetry/dataclasses/rust/pybindings
  )
  #install(TARGETS tof-pydataclasses-rust
  #    EXPORT GAPSTargets         
  #    LIBRARY DESTINATION ${INSTALL_PYTHON_DIR}
  #    #PUBLIC_HEADER DESTINATION ${INSTALL_INCLUDE_DIR}
  #    #INCLUDES DESTINATION "${GAPS_HEADER_OUTPUT_DIRECTORY}/lib/include"
  #)
  
  if(APPLE)
    install(FILES
            "${CMAKE_SOURCE_DIR}/telemetry/dataclasses/rust/pybindings/target/release/librust_telemetry.dylib"
            DESTINATION ${INSTALL_PYTHON_DIR}
            RENAME rust_telemetry.so
    )
  elseif(UNIX)
    install(FILES
            "${CMAKE_SOURCE_DIR}/telemetry/dataclasses/rust/pybindings/target/release/librust_telemetry.so"
            DESTINATION ${INSTALL_PYTHON_DIR}
            RENAME rust_telemetry.so
    )
  endif(APPLE)
endif(BUILD_RUSTTELEMETRY)

