# gaps-online-software documentation

WIP - this is work in progress

## Rust API

* [tof-dataclasses](tof_dataclasses/index.html)
_tof-dataclasses provides the Rust side of the general TOF API. This comprises
classes for events, calibration and (de)serialization methods_

* [tof-control](tof_control/index.html)
_tof-control is Takeru's code for changing voltages/thresholds etc and monitor
Tof environmental sensors_

## Rust/python API

* [rpy-tof-dataclasses](rpy_tof_dataclasses/index.html). The Rust `tof-dataclasses` project wrapped in pybindings.

## CXX/python API

The python API follows the CXX API very closely. Right now, there is now dedicated 
API documentation (which we are working on). 
Examples can be found in `tof/resources/examples/python`
The python API will undergo some easy-of-use improvements soon.

_as of version 0.10 (LELEWAA) we discourage to use the cxx pybindings, sinde the 
rust pybindings have matured enough and are faster as well as more stable_

* [CXX-API](index.html)

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

