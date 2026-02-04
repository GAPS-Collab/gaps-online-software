"""
Packets are containers for the different data structures which 
allow (de)serialization, so that they can be written to disk 
or sent over the network. There are 2 main types of packets 
within GAPS:
- TofPackets (which are used internally by the 
TOF and are written to the TOF disks)
- TelemetryPackets which are sent to ground

Each packet typically has methods for (de)serialization, for that 
see the respective `from_bytestream` methods
"""

from . import _gondola_core as _gc  

TelemetryPacket                    = _gc.packets.TelemetryPacket
TelemetryPacket.__module__         = __name__
TelemetryPacket.__name__           = "TelemetryPacket"

TelemetryPacketHeader              = _gc.packets.TelemetryPacketHeader 
TelemetryPacketHeader.__module__   = __name__ 
TelemetryPacketHeader.__name__     = "TelemetryPacketHeader" 

TofPacket                          = _gc.packets.TofPacket 
TofPacket.__module__               = __name__ 
TofPacket.__name__                 = "TofPacket" 

# enums 
TofPacketType                      = _gc.packets.TofPacketType 
TofPacketType.__module__           = __name__

TelemetryPacketType                = _gc.packets.TelemetryPacketType 
TelemetryPacketType.__module__     = __name__ 

__all__ = ["TelemetryPacket",\
           "TelemetryPacketHeader",\
           "TelemetryPacketType",\
           "TofPacket",\
           "TofPacketType"]
