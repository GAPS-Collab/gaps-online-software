# Readoutboard software - liftof-rb
_Current version 0.8.x - NIUHI_

TOF data acquisition/slow control for the TOF Readoutboards (RB)

## Liftof-rb has the following tasks

* Access the dedicated memory /dev/uio0 /dev/uio1 /dev/uio2
  to control DRS4 and readout blobs

* Stream monitoring information over zmq socket

* Stream RB data in form of RBEvent or RBEventMemoryView 
  over zmq socket

* Control attached components (LocalTriggerBoard, PowerBoard)

## How to compile?

The readoutboards sport a 32-bit ARM processor, so the code needs to 
be compile for that system. To ease the cross compilation, the 
[`cross`](https://github.com/cross-rs/cross) project is used. 
To use cross, a docker installation is needed with a running docker 
daemon.
There are issues with libc on the arm systems, so the musle implementation of 
libc is selected, which seems to be more popular for embedded systems.

With cross up and running, the code can then be compiled for the readoutboards with
`CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin liftof-rb --target=armv7-unknown-linux-musleabi --release`

[Cross FAQ page](`https://github.com/cross-rs/cross/wiki/FAQ#glibc-version-error`) is very helpful, see section also about glibc errors (which might occure)

_Comments_

* We need statically linked code. This means, no postion independent code (pic) and all the libraries need to be 
  "inside" the resulting binary.
* The `--release` flag can be omitted. This then will create a much larger binary containing the debugging symbols. 
  This might be helpful with debugging if there is a serious issue. For production, make sure the flag is enabled.
  This then will also advice the compiler to optimize the code. 
  WITHOUT THIS FLAG, THE CODE DOES NOT ACHIEVE PRODUCTION LEVEL NEEDED PERFORMANCE AND IS TOO SLOW!
* See the comment above for `musleabi`. This is an advanced topic.

## Operations

### Commandline parameters

`./liftof-rb --help` will show the available parameters. Most notable parameters are

* `-r <runconfig.json>` - file with configuration parameters
* `--listen`            - listen to commands from a supervisor. (E.g. `liftof-cc`)

### Logging & verbosity

The log level can be set by prefixing the liftof command with `RUST_LOG` like so:

```RUST_LOG=info ./liftof-rb <args>```

The available levels in increasing verbosity are

* error - only error messages
* warn  - warnings & errors
* info  - general information
* debug - general infomration can be as frequent as 
          events.
* trace - typically unwieldy amount of information, 
          cna be with higher frequency than events 
          

### Systemd integration

The liftof-rb can be managed through ![systemd](https://en.wikipedia.org/wiki/Systemd).i
The shipped ![`liftof.service`](liftof.service) file needs to be installed in `/etc/systemd/system`
The service can be controlled on the RB's by
`sudo systemctl <start/stop/restart> liftof`.  

>[!NOTE] 
> Only a single instance of liftof-rb, either systemd or manually started should be active at any time!

>[!NOTE] 
> If the service gets started through systemd, this doesn't mean it will start taking data. It will ONLY
> take data if instructed from the central instance, e.g. `liftof-cc` on the tof-computer.

### Configuration

There is (at least) a configuration file at `/home/gaps/config`, which has to be given 
as a start parameter with the `-r` (for `--run-configuration`) paramater.

#### Configuration file

The configration file is a `.json` file and currently
(>= liftof-rb 0.8.6) looks like the following. Parameters are
described below

```
{
  "runid"                   : 0,
  "nevents"                 : 0,
  "is_active"               : true,
  "nseconds"                : 0,
  "tof_op_mode"             : "StreamAny",
  "trigger_poisson_rate"    : 0,
  "trigger_fixed_rate"      : 0,
  "data_type"               : "Physics",
  "rb_buff_size"            : 2000
}
```


#### Configuration parameters
* `runid`                   : Generic identifier for runs
* `nevents`                 : Configure the number of events until the board stops
                              acquisition. 0 to run forever (or `nseconds`)
* `is_active`               : _internal parameter_ . This should basically always 
                              be set to true, except data taking should be explictly
                              stopped.
* `tof_op_mode`             : "TOF Operation mode" -> The workload within the TOF system
                              can be distributed differently. E.g. should the individual 
                              RBs already perform waveform analysis?
* `nseconds`                : Take data for `nseconds` seconds, then stop data 
                              acquisition
* `trigger_poisson_rate`    : Internal Poisson trigger rate
* `trigger_fixed_rate`      : Internal periodic (fixed rate) trigger rate
* `data_type`               : `tof_dataclasses::events::DataType` (see either there or below)
* `rb_buff_size`            : Size of the internal eventbuffers which are mapped to /dev/uio1 
                              and /dev/uio2. These buffers are maximum of about 64 MBytes.
                              Depending on the event rate, this means that the events might
                              sit quit a while in the buffers (~10s of seconds)
                              To mitigate that waiting time, we can choose a smaller buffer
                              The size of the buffer here is in <number_of_events_in_buffer>
                              [! The default value is in bytes, since per default the buffers 
                              don't hold an integer number of events]

#### Data types

Typically, the DataType should be "Physics", however, there are a few other
datatyps mainly reserved for calibration purposes.

```
pub enum DataType {
  VoltageCalibration = 0u8,
  TimingCalibration  = 10u8,
  Noi                = 20u8,
  Physics            = 30u8,
  RBTriggerPeriodic  = 40u8,
  RBTriggerPoisson   = 50u8,
  MTBTriggerPoisson  = 60u8,
  Unknown            = 70u8,
}
```

Remember, DataType in the configuration file has to be a string

>[!IMPORTANT] 
>The fields "data_type" and "tof_op_mode" needs to be strings,
>in the configuration file need to be strings, not numbers! 

### Calibration 

The full calibration routine can be performed by supplying a single
command line flag, `--calibration` like so:
`./liftof-rb --calibration`

This will start the routine and save the resulting file in 
`/home/gaps/calib` directly on the RB

## Connectivity

RB communications goes through 2 sockets, adhering to the protocolls as defined in the 0MQ library.

* A 0MQ PUB socket at <local-ip>:42000. All data (raw data, monitoring),
  but also `TofResponse` will be published at this socket.
  Subscribers should subscribe to `RBXX` where XX is the readoutboard id.

* Listening to a 0MQ SUB socket at <cnc-server-ip>:42000. We are subscribing to any messages starting with the bytes 
  `BRCT` for "Broadcast" or `RBXX` where XX is the 2 digit readoutboard id.

### Helpful resources
This might be helpful in understanding how them memory is read out from the /dev/uioX 
buffers.
https://linux-kernel-labs.github.io/refs/heads/master/labs/memory_mapping.html
