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

# Access to the database

The gaps-online-software ships a project `gaps-db` containing a sqlite database
with the TOF configuration at flight. This includes the different RBs, LTBs, 
paddles, geometry and so on. To access this database (no writing possible)
we are using diesel. To get started with diesel, it is a few easy steps.

* make sure diesel.toml is available in this directory
* set `DATABASE_URL` to the gaps flight database
* `diesel setup` (this will create the migrations directory, only needed once
* `diesel print-schema > src/db_schema.rs` to generate the database schema 

# How to run the tests?

There are two types of test, _unit tests_ and _integration tests_. The former
are supposed to test the basic units of the code (e.g. functions..) and the 
latter are supposed to test the interoperability between different parts of the 
code. Both needs to be run successfully.
To see output of potential `println!` commands, add `-- --nocapture` as an arguemnt.

## Unit tests

- [ ] TODO: we might want to avoid throwing errors when building up enums/structs when we
can set their values to "Unknown." This way the functions that use these structs
can check for these Unknown and throw an error if necessary. Right now an error is
thrown in some enum encoding doesn't make sense, but that should be "Unknown" as result.
- [ ] TODO: convert everything to either tryfrom of from.
- [ ] TODO: implement better unit test following the above approach.

`cargo test --features=random`

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

### Commands
Every command has an identifier, using enum `TofCommandCodes`, and a payload. The command seems to be u16 but it seems too much, is that by definition? The payload is not necessarily fixed but can vary based on the commands it is referring to. The necessary commands to be implemented are the following:

- [ ] Power On/Off (u16 which PDU)
  - [ ] Power On/Off PDU (is this PB + RB + LTB + preamps?)
  - ? Power On/Off MTB (This I think belongs to the first point)
  - ? Power On/Off LTB (from the list this seemed possible, is that so?)
  - ? Power On/Off preamps (from the list this seemed possible, is that so?)
  - It appears to me that just the PDU are controllable, so one can not discern the specific LTB/preamps, but just the PB, which in turn power the RB, and the MTB.
- [ ] Setup RB, for a specific RB or all (TOF -> RB) (u16 which RB) (there should be more data send with this command?)
- [ ] Set thresholds (TOF -> RB) (u16 which threshold, u32 threshold level)
  - [ ] For all LTBs
  - ? For a specific LTB
  - ? For a group of LTB
- [ ] Set MTB configuration (TOF -> MTB) (u16 trigger config ID)
- [ ] Start validation run on an RB or all (TOF), small data take (u16 # events, u16 which RB)
- [ ] Start/stop data taking run (TOF -> RB), regular data take (u16 type of run, u16 # events, u16 time)
- [ ] Voltage calibration run (TOF -> RB) (u32 second voltage level (???), u16 which RB, u32 extra (???))
- [ ] Timing calibration run (TOF -> RB) (u16 which RB, u32 extra (???))
- [ ] Create calibration file (TOF) for RB or all, from three calibration runs, which runs? V/timing and? (u32 second voltage level (???), u16 which RB)
  - [ ] What one wants here still has to be defined.

### Responses

## Constants

### Result/Error codes

### Command codes

