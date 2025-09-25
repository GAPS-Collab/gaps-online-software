# Liftof - GAPS ToF operation code suite

Liftof (_Liftof is for Tof_) :rocket: :balloon: is a software suite
for control/data acquisition tasks for the
for the Time-of-Flight instrument of the GAPS
science experiment.
The code is writen in Rust.

Ths software has two main components to control the
tof system. The desing philosophy is a bit like that 
of a kraken :octopus: - a central brain, but pretty 
intelligent arms.

1) *Command & Control server* - a central instance 
   running on the tof-computer. 
2) *RBClient* - code running as a service on the 
   individual readout boards, answering to the 
   C&C

## Cmponents:

The code has several individual, independent 
programs, each organized in its own crate

* liftof-lib : common routines to interact
  with different parts of the tof/gaps 
  subsystems, e.g. the MasterTrigger.
  This does *NOT* include the `tof-dataclasses`, 
  which is yet a separate project. 

* liftof-cc: (formerly `crusty_kraken` :octopus:) 
  The central C&C instance running on the tof 
  computer. This will have command/data connections
  to all the readout boards and takes care of 
  data readout, storage and run control as well 
  as monitoring. liftof-cc will be connecting to 
  the flight computer and sent datapackets defined
  in the `tof-dataclasses` project over a 0MQ socket.

* liftof-rb : Readoutboard instances for the software.
  These instances will interact control the DRS4 chips
  directly, read out data and pass it down the 
  data-pipeline. They have 2 'wires' which are 2 
  independent 0MQ connections. Each instance opens a 
  REP as well as a PUB socket for independent control 
  as well as data publishing.
  liftof-rb can be run independently of liftof-cc.
  A pecularity of liftof-rb is that it needs to be 
  cross-compiled for the ARM7 32-bit architecture, 
  for that, see the README of liftof-rb.

* liftof-tui : TUI (_termnial user interface_) to connect and 
  debug Readout boards via graphical interface in the 
  terminal. Connects to littof-rb instances and
  displays information. Limited control abilities.

## depenencies

* tof-dataclasses: The various data/packets and 
  serialization routines to be used to transport
  information over the wire. This is yet another, 
  separate crate and has a sister project which 
  is written in C++/pybind11. 

* 0MQ/ZMQ ("ZeroMQ") : Implementation of various 
  socket types, to transport data over the network
  or unix sockets. Various "patterns" have been 
  implemented by the designers of 0MQ. The library
  is quite complex, but one of the benefits is 
  that it is agnostic to the programming language
  and allows for _some_ security by adding the 
  size automatically to each message sent.
  [Reference for 0MQ](https://zeromq.org)

* tof-dataclasses: A comprehensive project containing
  containers to transport data over 0MQ sockets.
  Basically each container can be serialized to 
  a bytestream for easy network transport. 
  See the documentation for further information.

* _a number of rust crates_ : The software utilizes 
  several third-party libraries. Thus for compilation
  (at least for the first time) a connection to the 
  internet might be necessary. 
  Some noticable mentions are:
  - logging
  - crossbeam (improved inter-thread communications)

## Wiring scheme

Each RB opens a 0MQ REP and PUB socket. Commands will 
be issued over the REP channel, while the PUB channel
is for the data.

## Conrerstones of design philosophy

* *Command - Response pattern* - Each Commnad triggers a 
  response. No sent command will be silent. This is 
  implemented by a 0MQ REP/REQ socket connection, which 
  in its core can only operate in alternate send/recv mode.
  The two central classes to send commands and receive responses
  `TofCommand` as well as `TofResponse` are part of the `tof-dataclasses` 
  project and are documented there. 
  A list of availabel commands/responses together with their numeric codes
  can be found [in the documentation]()

* *Separation of command and data wires* - To ensure that the software is 
  responsive independent of the tasks, commands/responses shall go through 
  a seperate wire. Each command will be acknowledged (REQ/REP pattern).
  Data will be distributed separatly through PUB/SUB. 
  _Note: It would be a nice feature if each readoutboard could prefix it's 
  datastream for the PUB/SUB pattern with it's own id, so the C&C could 
  subscribe to a subset of RB's if necessary._
  The C&C is responsible for checking the PUB/SUB stream for the data it requested, 
  and if not happy, request it again. 

* *Don't fail silently* - this is achieved by strictly following rust's own 
  `Result<>` and `Option<>` philosophy. E.g. a call to a `send` on a data 
  socket will typically look like this (rust pseudo-code)
  ```
  match socket.send(payload) {
    Ok =>  info!("Perfect! Let's move on"),
    Err(err) => {
      warn!("There is a problem!  {err}");
      notify_somebody();
      total_errs += 1;
    }
  }
  ```
  We will work hard on following this pattern consequently. 
  Not doing so, should be reported on the issue tracker.

* Don't panic. The main thread should not panic, except when 
  starting up and the startup conditions are wrong and it can 
  not work (e.g. no network interface available.)
  All panics should be catched, and instead errors should be 
  propagated through `TofResponse` with a specific error code.

* Immortality/Revivability. Threads shall basically have infite 
  loops at its core, and have a *heartbeat* mechanism, which 
  restarts them when they fail.

* Hearbeat - a dedicated thread shall provide the heartbeat,
  _even to itself_ to monitor the different threads. Not all 
  threads need to be heartbeated. (e.g. a "runner" thread which 
  is starting a run can die, since this does not affect the 
  DRS4 state machine.) 

* Monitoring - independent of everything else, monitoring threads
  shall provide monitoring data in a heartbeat pattern through the 
  PUB/SUB socket system. _Note: It would be nice, if that could be 
  prefixed as well, so that the C&C can decide if it wants ot 
  sobscribe to Moni data_

## Features (_currently not everything is implemented, and the list is 
not complete_)

* Data readout + central storage of binary data 
  of all the readout boards. "Save to disk".

* Waveform analysis + event compression

* Interaction/Receiving of commands from the flight computer/ground

* Comprehensive monitoring.

## Tof data flow

![Flowchart of the tof dataflow](doc/assets/dataflow.pdf "Iof dataflow")


