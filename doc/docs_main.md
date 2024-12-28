# gaps-online-software documentation

WIP - this is work in progress

## Rust API

* [tof-dataclasses](tof_dataclasses/index.html)
_tof-dataclasses provides the Rust side of the general TOF API. This comprises
classes for events, calibration and (de)serialization methods_

* [tof-control](tof_control/index.html)
_tof-control is Takeru's code for changing voltages/thresholds etc and monitor
Tof environmental sensors_

## CXX API

The CXX API is an independent implementation of the rust `tof-dataclasses` project.

* [CXX-API](index.html)

## Python API

The python API is exposed through `gaps_online` which can be imported
if the `setup-env.sh` shell has been sourced and the thus PYTHONPATH
has been modified.
If `BUILD_RUSTPYBINDINGS` are enabled, the project exposes the rust 
code throug pyo3 crafted pybindings. The CXX interface is wrapped 
through pybind11, these pybindings can be enable with `BUILD_CXXPYBINDINGS`
_as of version 0.10 (LELEWAA) we discourage to use the cxx pybindings, sinde the 
rust pybindings have matured enough and are faster as well as more stable_

The rust side of the rust pybdingings is documented through the `rust doc` system and can be found here:

* [go-pybidings](go_pybindings/index.html). Pyo3 wrapper for a part of the rust code. If `BUILD_TELEMETRY` and/or `BUILD_LIFTOF` are enabled, these parts of the code will be exposed through python as well.

The python API exposed through python (which is what a user will encounter in their ipython shell or similar)
are doucmented through sphinx and can be found here:

* [gaps-online](gaps_online/index.html) The 'meta' package for access through python. 

## Executable programs and higher level libraries [liftof]

Liftof ("liftof is for TOF") is a collection of binaries and
a library which are used for in-flight data taking.
Liftof is written in [Rust](https://www.rust-lang.org/)
It also ships with binaries for debugging and analysis.

* [liftof-lib](liftof_lib/index.html)
_high level library for applications_

* [liftof-cc](liftof_cc/index.html)
_command and control sever_

liftof-cc contains 2 executable programs:
    - liftof-cc : Command and control server
    - liftof-mt : Readout/Diagnose MTB, show event stream and
                  can send RBEventRequests to RBs

* [liftof-rb](liftof_rb/index.html)
_readoutboard code_

* [liftof-tui](liftof_tui/index.html)
_monitoring program for terminal_

