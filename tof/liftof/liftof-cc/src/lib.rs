pub mod constants;
pub mod api;
pub mod event_builder;
pub mod paddle_packet_cache;
pub mod flight_comms;
pub mod threads;
// this is a list of tests
// FIXME - this should follow
// the "official" structure
// for now, let's just keep it here
mod test_blobdata;
#[macro_use] extern crate log;
extern crate clap;
extern crate json;
extern crate colored;

extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate liftof_lib;

#[cfg(feature="random")]
extern crate rand;

extern crate zmq;
extern crate tof_dataclasses;




