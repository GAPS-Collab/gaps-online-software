#[macro_use] extern crate log;
extern crate clap;
extern crate colored;

extern crate crossbeam_channel;

extern crate zmq;
extern crate openssh;
extern crate comfy_table;
extern crate tof_dataclasses;
#[cfg(features="tof-control")]
extern crate tof_control;
extern crate liftof_lib;

pub mod constants;
pub mod threads;

use openssh::{Session, KnownHosts};



#[deprecated(since="0.10.2", note="Unclear purpose")]
pub fn prefix_tof_cpu(input : &mut Vec<u8>) -> Vec<u8> {
  let mut bytestream : Vec::<u8>;
  let tof_cpu : String;
  tof_cpu = String::from("TOFCPU");
  //let mut response = 
  bytestream = tof_cpu.as_bytes().to_vec();
  //bytestream.append(&mut resp.to_bytestream());
  bytestream.append(input);
  bytestream
}



