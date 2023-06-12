# dataclasses project for GAPS tof

_verions 0.6 (KIHIKIHI)_

The dataclasses should provide a cross-language/cross-systm interace to data
obtained by the GAPS time-of-flight system.

This includes:

* analysis (original author J.Zweerink) : Peak-finding, calibration, leading-edge finding, 
  charge calculation, etc. on the full waveform data. 

* de/serializtion : transport dataclasses over the wire. The project introduces a system to 
  pack all information and send it indepndently of the hardware/used programming language 
  (so far, we have rust/C++/Python bindings) accross consumers.

## Available structures:



### Events

We have several event classes for different purposes:

