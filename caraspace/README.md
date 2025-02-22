# CaraSpace - simple, highly efficient serializztion library for balloon project


## spiritual success of IceTray, a serialization/data accessibility framework developed for IceCube

The [IceTray framework](https://docs.icecube.aq/icetray/main/info/overview.html#what-is-icetray) is IceCube's standard software 
framework for simulation and analysis. 
The basic design concept, is a series of a sequence of Frames, which are containers able to hold any kind of "FrameObject". These 
frame objects might be a reconstruction, a hitseries or geometry information.

While not a rewrite, the goal of this project is to follow the design in the same spirit, basically providing a unified 
format for the GAPS TOF only stream as well as any Telemetry data.

Currently, this works with TofPackets and TelemetryPacktes.

The code works as a part of an "exoskeleton" for the individual data chunks.

>[!TIP]
>Carapace is the exeskeleton of crustaceens, and made out of little plates, called sclerite

## Python build-in

We love python! So we have pybindings already built in.

### Example

```
from glob import glob

# whatever the library will be called in the end
import go_pybindings as gop

# read a single TofPacket
tp_reader = gop.io.TofPacketReader('/data0/gaps/csbf/csbf-data/104/Run104_251.240723_073209UTC.tof.gaps')
for tp in tp_reader:
    print (tp)
    break

# rad a single TelemetryPacket, just for funsies make sure 
# it is a MergedEvent
tel_reader = gop.telemetry.TelemetryPacketReader('/data0/gaps/csbf/csbf-data/binaries/ethernet/RAW240715_222222.bin', filter=gop.telemetry.TelemetryPacketType.MergedEvent)
for tl in tel_reader:
    print (tl)
    break

# Write stuff to disk, arguments are directory
# and run id
writer = gop.caraspace.CRWriter('foo', 69)

# create a new frame and put stuff in it
frame = gop.caraspace.CRFrame()
frame.put_telemetrypacket(tl, "telemetry")
frame.put_tofpacket(tp, "tofstream")

# show it
print (frame)
# add it
writer.add_frame(frame)

# read our data
fname = glob('foo/Run69*')[0]
reader = gop.caraspace.CRReader(fname)
# reader reads frames
for frame in reader:
    print (frame)
    # data can be retrieved from frames by name
    tp = frame.get_tofpacket("tofstream")
    ev = gop.events.TofEvent()
    ev.from_tofpacket(tp)
    print (ev)
    
    telly_pack = frame.get_telemetrypacket("telemetry")
    # cheating - we know that this was a MergedEvent
    # however, we could have checked the packet type
    # here as well
    ev = gop.telemetry.MergedEvent()
    ev.from_telemetrypacket(telly_pack)
    print (telly_pack)
    print (ev.tof)
``` 
