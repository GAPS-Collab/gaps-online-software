# dataclasses - the backbone of gaps-online-software

The dataclasses project compiles everything which is needed to interface with GAPS data on a 
low level. There are implementations in C++ and rust. For rust, the dataclasses are exposed
through pyO3 to python and incorporated into the python package `gaps_online`.

Dataclasses include structures for:

* Events (TOF/Tracker, Telemetry, combined data (L0)

* Monitoring 

* I/O - reader/writer classes for the different file types


