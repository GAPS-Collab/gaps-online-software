# LIFTOF Command & Control server

`liftof-cc`:balloon: :rocket: is the command and control server of the TOF systen
and is supposed to be running on the TOF computer.

*Core features*

* Subscribe to RBs via zmq::SUB and receive RBEvent data
* Waveform analysis to extract timing and charge information
* Assemble RBEvents to TOFEvents
* Package TofEvent data in flight computer understandable 
  packets and publish on zmq::PUB

## The liftof eco system

The liftof system will provide monitoring and calibraton for 
the TOF system as well deal with the aspect of semi-continuous data taking.
To facilitate this, we have 3 services which need to be 
made available on the TOF main computer ('tofcpu')

### systemd

To automate our progroms, we use `systemd` [(see arch wiki for more information)](https://wiki.archlinux.org/title/Systemd).

The basic control of systemd works by configuring a `.service` file in `/etc/systemd/system` and then control the service
through 

`sudo systemctl <start|stop|restart|status> <component>`

where we are using the following components:
`liftof` (liftof-cc) `liftof-scheduler` and `liftof-moni`

### liftof configuration

The liftof system uses [TOML](https://en.wikipedia.org/wiki/TOML) for configuraton. Files with the ending `.toml` are primarily stored in the `/home/gaps/staging/` area on the tof main 
computer. Additional files are in `/home/gaps/config`. 

Additionally, the liftof system requires a local database 
(currently sqlite) which path is given in the configuration
file, currently we maintain a database in `/home/gaps/config`.
This database is a mirror of the tof channel mapping and 
coordinate spreadsheets and use for mapping RB channels to 
paddle IDs, as well as power board and LTB channels.

### liftof components

1) `liftof-cc`. This main service will take care of data taking
and calibration. It will restart itself indefinitly

2) `liftof-scheduler`. The scheduler component will listen to 
commands from the GAPS flight computer and relay them to 
liftof-cc as well as manage the `liftof-cc` service in flight.
To manage the received commands, it modifies a configuration
file, which is stored in the staging area under `next`.
After a submit command is issued, this file will then be 
copied to the `current` directory, where it will be picked up
by `liftof-cc` for the next run.


### How to run

`./liftof-cc -h` will give you a list of all available options

## Building & Deployment

In general, we build a static binary with cross. Cross uses docker, so that is required on 
the system which is used to build the code.

### A word about musl

Musl is the alternative implemementation of libc, see [wikipedia](https://en.wikipedia.org/wiki/Musl). It is optimized
for static linking, close to realtime quality and avoiding race conditions.

To deploy the code to the TOF computer, we want to generate a statically linked binary, so we don't have any dependencies
and also if everything is linked statically together, we do expect a slightly better performance compared to dynamic linking.

However, if we want to link against external libraries, they have to be build with musl as well. This might be an issue for sqlite3.
We might have to build the sqlite3 library ourselves.
This is **NOT** on the tof-computer, but the computer we are using for building & deployment.

`CC=musl-gcc ./configure --prefix=/usr/lib/musl` within the source tree of sqlite.


## tests

Test can be run with 
`cargo test --nocapture`


