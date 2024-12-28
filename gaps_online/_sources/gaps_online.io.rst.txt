gaps_online.io 
===========================

input/output for TOF only data as well as telemetry files (".bin").

.. note
   Gaps-online-software needs to be built with `BUILD_RUSTTELEMETRY=ON`
   for the telemetry features to become available

Readers/Writers
------------------

.. autoclass:: gaps_online.io.TelemetryPacketReader
   :members:       
.. autoclass:: gaps_online.io.TofPacketReader
   :members:       


Packets
------------------

.. autoclass:: gaps_online.io.TelemetryPacket
   :members:       
.. autoclass:: gaps_online.io.TelemetryPacketType
   :members:       
.. autoclass:: gaps_online.io.TofPacket
   :members:       
.. autoclass:: gaps_online.io.TofPacketType
   :members:       


Functions
------------

.. autosummary::
   :toctree: _autosummary
   :recursive:

   gaps_online.io
