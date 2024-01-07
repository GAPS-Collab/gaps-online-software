# readoutboard software

## client which runs on the readoutboards

* Access the dedicated memory /dev/uio0 /dev/uio1 /dev/uio2
  to control DRS4 and readout blobs

* Send event/monitoring information per request over zmq socket

## How to compile?

The readoutboards spot a 32-bit ARM processor, so the code needs to 
be compile for that system. To ease the cross compilation, the 
[`cross`](https://github.com/cross-rs/cross) project is used. 
To use cross, a docker installation is needed with a running docker 
daemon.

With cross up and running, the code can then be compiled for the readoutboards with
`CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABI_RUSTFLAGS="-C relocation-model=dynamic-no-pic -C target-feature=+crt-static" cross build --bin rb-    soft --target=armv7-unknown-linux-musleabi --release`

[Cross FAQ page](`https://github.com/cross-rs/cross/wiki/FAQ#glibc-version-error`) is very helpful, see section also about glibc errors (which might occure)

_Comments_

* We need statically linked code. This means, no postion independent code (pic) and all the libraries need to be 
  "inside" the resulting binary.
* The `--release` flag can be omitted. This then will create a much larger binary containing the debugging symbols. 
  This might be helpful with debugging if there is a serious issue. For production, make sure the flag is enabled.
  This then will also advice the compiler to optimize the code.
* `musleabi` - The resulting binary needs to be statically linked. For some reasons, I could only get this to work 
  completely for `musleabi`. (Naturally, one would choose `gnuabi`)

## Systemd integration

The liftof-rb software will be integrated in systemd. See `liftof.service`
When activated, it will listen for run start/stop commands issued by the C&C server.
The service can be controlled on the RB's by
`sudo systemctl <start/stop/restart> liftof`

## Configuration

There is (at least) a configuration file at `/home/gaps/config`, which has to be given 
as a start parameter with the `-r` (for `--run-configuration`) paramater.

### Configuration file

The configration file is a `.json` file and currently
(Version 1.4) looks like the following. Parameters are
described below

```
{
  "nevents"                 : 0,
  "is_active"               : true,
  "nseconds"                : 0,
  "stream_any"              : true,
  "trigger_poisson_rate"    : 0,
  "trigger_fixed_rate"      : 0,
  "latch_to_mtb"            : 1,
  "data_type"               : "Physics",
  "rb_buff_size"            : 2000
}
```




### Configuration parameters
* `nevents`                 : Configure the number of events until the board stops
                              acquisition. 0 to run forever (or `nseconds`)
* `is_active`               : _internal parameter_ . This should basically always 
                              be set to true, except data taking should be explictly
                              stopped.
* `nseconds`                : Take data for `nseconds` seconds, then stop data 
                              acquisition
* `stream_any`              : There are two modes the board can operate in. Either
                              it processes all data in its buffers (this is required
                              for any standalone operation), or it gets input about
                              which events to process from a central instance, e.g.
                              the `liftof-cc` server. In the later case, set this to 0
* `trigger_poisson_rate`    : Internal Poisson trigger rate
* `trigger_fixed_rate`      : Internal periodic (fixed rate) trigger rate
* `latch_to_mtb`            : Latch RB to the MasterTriggerBoard distributed trigger
* `data_type`               : `tof_dataclasses::events::DataType` (see either there or below)
* `rb_buff_size`            : Size of the internal eventbuffers which are mapped to /dev/uio1 
                              and /dev/uio2. These buffers are maximum of about 64 MBytes.
                              Depending on the event rate, this means that the events might
                              sit quit a while in the buffers (~10s of seconds)
                              To mitigate that waiting time, we can choose a smaller buffer
                              The size of the buffer here is in <number_of_events_in_buffer>
                              [! The default value is in bytes, since per default the buffers 
                              don't hold an integer number of events]

### Data type & formats

The data format is a number code defined by `tof_dataclasses::events::DataFormat`.
For available formats check there, or here is a copy (might be outdated)

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
The representation is now independent on the internal u8 value, so one should specify
a string in the field. E.g for regular data, set `"data_type" : "Physics"`.

This might get extended with more data types & formats, so stay tuned!


### Calibration 



### Connectivity

The communication with a central C&C server (either `liftof-cc`, `liftof-tui` or technically a python script) is done
through 0MQ. We are offering:

* A 0MQ PUB socket at <local-ip>:42000. All data (raw data, monitoring), but also `TofResponse` will be published there
  Subscribers should subscribe to `RBXX` where XX is the readoutboard id.

* Listening to a 0MQ SUB socket at <cnc-server-ip>:42000. We are subscribing to any messages starting with the bytes 
  `BRCT` for "Broadcast" or `RBXX` where XX is the 2 digit readoutboard id.


### Design philosopy

### Threaded model
### Helpful resources

https://linux-kernel-labs.github.io/refs/heads/master/labs/memory_mapping.html
