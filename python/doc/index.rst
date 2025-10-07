.. gaps_online documentation master file, created by
   sphinx-quickstart on Mon Sep 30 16:32:04 2024.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

GAPS Online Software documentation
======================================

The python API of gaps-online-software wraps the rust API with pyO3. 
It is based mainly on `tof-dataclasses` as well as `telemetry-dataclasses`. 

.. autosummary::
   :toctree: _autosummary
   :recursive:

   gaps_online

.. toctree::
   :maxdepth: 2
   :caption: Contents:

RUST API Software documentation
===============================

The "heart" of the project - implementes necessarry dataclasses
and i/o operations

`gondola-core (common library) <gondola_core/index.html>`_

TOF flight computer code for data acquisition and control in-flight 

`liftof-cc <liftof_cc/index.html>`_


TOF RB data acquisition/control code to be controled by liftof-cc 

`liftof-rb <liftof_rb/index.html>`_

Run scheduler to manage settings and do bookkeeping of them

`liftof-scheduler <liftof_scheduler/index.html>`_

Terminal application to monitor TOF activity

`liftof-tui <liftof_tui/index.html>`_

Common used functions across liftof 

`liftof-lib <liftof_lib/index.html>`_

The rust part of `gaps_online`, which is also accessible through `gaps_online`.

`go_pybindings <go_pybindings/index.html>`_

CXX API Software documentation
==============================

`Doxygen API docs <C++/html/index.html>`_



