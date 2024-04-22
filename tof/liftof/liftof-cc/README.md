# LIFTOF Command & Control server

`liftof-cc`:balloon: :rocket: is the command and control server of the TOF systen
and is supposed to be running on the TOF computer.

*Core features*

* Subscribe to RBs via zmq::SUB and receive RBEvent data
* Waveform analysis to extract timing and charge information
* Assemble RBEvents to TOFEvents
* Package TofEvent data in flight computer understandable 
  packets and publish on zmq::PUB

## TODO List
- [ ] Implement commands

## Deployment

In general, we build a static binary with cross. Cross uses docker, so that is required on 
the system which is used to build the code.

### A word about musl

Musl is the alternative implemementation of libc, see [wikipedia](https://en.wikipedia.org/wiki/Musl). It is optimized
for static linking, close to realtime quality and avoiding race conditions.

To deploy the code to the TOF computer, we wnat to generate a statically linked binary, so we don't have any dependencies
and also if everything is linked statically together, we do expect a slightly better performance compared to dynamic linking.

However, if we want to link against external libraries, they have to be build with musl as well. This might be an issue for sqlite3.
We might have to build the sqlite3 library ourselves.
This is **NOT** on the tof-computer, but the computer we are using for building & deployment.

`CC=musl-gcc ./configure --prefix=/usr/lib/musl` within the source tree of sqlite.

**FIXME**

* *make sqlite3 a git submodule and provide a build script with above instruction*

### How to run

`cargo run --release --bin=liftof-cc` 

### tests

Test can be run with 
`cargo test --nocapture`


