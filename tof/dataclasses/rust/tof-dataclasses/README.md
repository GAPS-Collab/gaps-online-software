# tof-dataclasses

The `tof-dataclasses` is the backbone for all tof-related operations.
Included dataclasses contain containers for RB data of different 
processing levels, but also for general communications between
the subsystems.

In general, most classes come with serialization methods `to_bytestream`
and `from_bytestream` which allow them to be transcribed to a bytestream
and be send over the network.

# How to run the tests?
`cargo test --test=test --features="random"`

# How to run benchmarking?
`cargo bench --bench=bench --features="random"`

## Data structures for events


## Data structures for control

* commands

* responses

## Constants

### Result/Error codes

### Command codes

