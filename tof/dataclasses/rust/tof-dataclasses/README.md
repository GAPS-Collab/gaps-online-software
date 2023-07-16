# tof-dataclasses

The `tof-dataclasses` is the backbone for all tof-related operations.
Included dataclasses contain containers for RB data of different 
processing levels, but also for general communications between
the subsystems.

# Serialization

The trait `Serialization` implements `to_bytestream` and `from_bytestream`, 
which are implemented for most of the classes. They will (de)serialize the 
respective class to a vector of bytes, which also allows transportation 
over the network or writing to disk.


# How to run the tests?

There are two types of test, _unit tests_ and _integration tests_. The former
are supposed to test the basic units of the code (e.g. functions..) and the 
latter are supposed to test the interoperability between different parts of the 
code. Both needs to be run successfully.
To see output of potential `println!` commands, add `-- --nocapture` as an arguemnt.

## Unit tests

`cargo test`

## Integration tests

`cargo test --test=test --features="random"`

# How to run benchmarking?

Benchmarking is explicitly important since we are 
running on a low power consuming CPU.
Luckily cargo provides benchmarking features. To 
run the benches, run

`cargo bench --bench=bench --features="random"`

# Overview over the data structures

## Data structures for events


## Data structures for control

* commands

* responses

## Constants

### Result/Error codes

### Command codes

