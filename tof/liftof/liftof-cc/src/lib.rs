pub mod readoutboard_comm;
//mod reduced_tofevent;
pub mod constants;
pub mod waveform;
pub mod errors;
pub mod api;
pub mod event_builder;
pub mod paddle_packet_cache;
pub mod flight_comms;
// this is a list of tests
// FIXME - this should follow
// the "official" structure
// for now, let's just keep it here
mod test_blobdata;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate sensors;
extern crate clap;
extern crate json;
#[cfg(feature = "diagnostics")]
extern crate hdf5;
#[cfg(feature = "diagnostics")]
extern crate ndarray;

extern crate local_ip_address;
extern crate crossbeam_channel;
extern crate liftof_lib;
use liftof_lib::{ReadoutBoard, 
                 //rb_manifest_from_json,
                 get_rb_manifest};

#[cfg(feature="random")]
extern crate rand;

extern crate zmq;

extern crate tof_dataclasses;



