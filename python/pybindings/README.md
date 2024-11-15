# Pybindings for gaps-online-software using pyo3

Combined project for all pybindings within the project. This wraps
- tof-dataclasses
- liftof
- telemetry

If the project is build with cmake, make sure to use enable `BUILD_RUSTPYBINDINGS=ON`
This will selectively build the pybindings, if `BUILD_TELEMETRY` or `BUILD_LIFTOF` are 
set to `ON`, those bindings will be automatically added.


