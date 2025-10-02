.. gaps_online documentation master file, created by
   sphinx-quickstart on Mon Sep 30 16:32:04 2024.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

GAPS Online Software documentation
======================================

GAPS Online Software ("gondola") compriseses several components.

* liftof system - flight software for the TOF computer, data acquisition
                  GAPS run scheduling, RB communication, calibration,
                  MTB interaction.
                  It contains several binaries to be run on various 
                  components of the TOF system. 

* gondola-core  - core library with dataclasses to read data structures. 
                  This included TOF only structures, data sent over telemetry,
                  TOF data from disk, etc. 
                  Gondola core allows to calibrate the event data as well 

* gander        - a streamlit app which allows to have a quick look into 
                  recorded data on disk or hook up to a telemetry stream

The code is mainly written in RUST and exposed to python through pyo3. There is 
additional C++ code which implements the same API, and can be used for compatiblity
with C++ applications.

The python API of gaps-online-software wraps the gondoala-core rust API with pyO3. 
The python module is called `gondola`.

.. toctree::
   :maxdepth: 2
   :caption: Contents:

   guided_tour
   gondola 

PYTHON API Software documentation
=================================

Installation
------------

The oython code can be install through `pip`/`uv` e.g. 

| `pip install gondola`

or when using rye

| `rye add gondola` 

API docs
--------
.. autosummary::
   :toctree: _autosummary
   :recursive:
   
   gondola

CXX API Software documentation
=================================

`Doxygen API docs <C++/index.html>`_



