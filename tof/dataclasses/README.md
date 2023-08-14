# Dataclasses project for GAPS tof

## Containers for TOF data

The purpose of the tof dataclasses project is to provide containers for the 
different types of tof data. These are events from the readoutboards, monitoring
data as well as information from the master trigger board. The data are 
organizedn in "Packets" and "Events". Basically, everything which has an event id
is called an "event", a "packet" is mainly a wrapper for arbitrary data.

Packets and events can be serialized/deserialized and thus can be stored to files
on disk or transmitted over the network.

The backbone of all packets is the `TofPacket`. A `TofPacket` can wrap any other packet
or event. It has flexible size and has fields for type and size. The current structure
of a `TofPacket` is the following:

```
* HEAD    : u16 = 0xAAAA
* TYPE    : u8  = PacketType
* SIZE    : u64
* PAYLOAD : [u8;SIZE]
* TAIL    : u16 = 0x5555
```
To read TofPackets from a file (using the python API), one can do the following:

```
import gaps_tof as gt

f = '/path/to/file'
packets = gt.get_tofpackets(f)
for k in packets:
    print (k.packet_type)
```

Depending on the packet type, these packets can then be 
unpacked.

```
...
    if k.packet_type = gt.PacketType.MasterTriggerEvent:
        mte = gt.MasterTriggerEvent.from_bytestream(k.payload, 0)
        print (mte.event_id)
```

## Threefold API - rust and C++/Python

Each packet/event type is implemented twice - in rust (which will
be performing all necessary actions during flight) as well as in 
C++ (to make it easier for current analysis) with a thin wrapper
provided by pybind11 on top, so the C++ classes get exposed to 
python.

### Serialziation

The dataclasses can be serialized to bytestreams, which are 
`Vec<u8>` via methods `to_bytestream` and `from_bytestream`.
In this way, they can be (ultimatly) written to disk or 
transmitted over the network. Since the purpose of the 
rust, C++ and python APIs are different, not all APIs feature
`to_bytestream` methods. 
The C++ are meant mostly for analysis but also should allow the 
flight computer to relay back information to the TOF computer.
This means, the C++ API is mostly read-only, with a few exceptions, 
such as TofPackets and CommandPackets, for which the `to_bytestream`
methods are implemented.
The exposed python API is read-only, since it is meant only for 
analysis.

The serialization interface also facilitates the transition between
rust and C++/python. A class can be "transcribed" from rust to C++ 
by going through the (de)serialization cycle: `to_bytestream[Rust] -> from_bytestream[C++]`

## versions/branches:

Please look for the latest version branch, e.g.

_0.6 (KIHIKIHI)_

Main will only be updated as soon as the development on the latest version 
branch is finished, thus typically it will point to an older version.

Each test scenario has its own branch, e.g. SSL uses the `MANINI` branch, 
and the NTS test uses the `KIHIKIHI` branch. The branches are named after
fish which live in Hawaii.

## Typedefs

To streamline the C++ code a bit and make it more similar to the rust API, 
a few typedefs have been introduced, which can be found in `tof_typedefs.h`.

Most noteworty is:

```
std::vector<T> -> Vec<T>
```

as well as the introduction of the rust primitive types, u8, u16, etc.

## logging

To set the loglevel for the rust API, set `RUST_LOG=<loglevel>` and for C++ set `SPDLOG_LEVEL=<loglevel>`
environment variables.

## Available structures:

### Packets

* TofPacket       - general packet, variable size, type aware.
* RBMoniData      - monitoring data for individual RB
* TofCmpMoniData  - monitoring data for the Tof computer (main CPU)
* MtbMoniData     - monitoring data for the MasterTriggerBoard


### Events

* MasterTriggerEVent
* RBEventMemoryView
* RBEventHeader
* RBEvent
* RBMissingHit
* TofEvent
* MasterTriggerEvent

## Available functionality

### Parser library

To facilitate the decoding from `Vec<u8>` a parser library had been 
introduced on both the C++ as well as on the rust side. It follows
the spirit of the popular rust nom crate, which might eventually 
replase the parser library. The parser functions are of the 
following form:

```
u32 parse_u32(Vec<u8> bytestream, u64 &pos)

```

note that the position argument is mutable. It will be advanced
by the number of bytes consumed, e.g. in the case of u32 this 
is 4 bytes.


### I/O

The tof dataclasses provide some I/O functionality as well (C++/Python API only). This 
allows to read event classes/packets directly from a file or from a vector of bytes.

```

Vec<RBEventMemoryView> get_rbeventmemoryview(const String &filename);

Vec<RBEventMemoryView> get_rbeventmemoryview(const Vec<u8> &stream, u64 start_pos);

Vec<RBEventHeader> get_headers(const String &filename, bool is_header=false);

Vec<u32> get_event_ids_from_raw_stream(const Vec<u8> &bytestream, u64 &start_pos);

Vec<TofPacket> get_tofpackets(const Vec<u8> &bytestream, u64 start_pos);

Vec<TofPacket> get_tofpackets(const String filename);

```

### Legacy code

The library contains the legacy waveform analysis code, which had been formerly used in "blob2root".


