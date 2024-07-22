#[macro_use] extern crate log;
extern crate clap;
extern crate colored;

extern crate crossbeam_channel;

extern crate zmq;
//extern crate openssh;
extern crate comfy_table;
extern crate tof_dataclasses;
#[cfg(features="tof-ctrl")]
extern crate tof_control;
extern crate liftof_lib;

pub mod constants;
pub mod threads;

use std::fs;
use std::path::PathBuf;

/// Prepare a new folder with the run id
///
/// This will assign a run id based on the 
/// run ids in that folder
///
/// TODO - connect it to the run database
///
/// # Arguments:
///
/// * data_path : The global path on the inflight 
///               tof computer where to store data.
pub fn prepare_run(data_path : String) -> Option<u32> {
  let stream_files_path = PathBuf::from(data_path);
  let paths = fs::read_dir(stream_files_path.clone()).unwrap();
  let mut used_runids = Vec::<u32>::new();
  for path in paths {
    // this is hideous, I am so sorry. May the rust gods have mercy on my soul...
    match format!("{}",path.as_ref().unwrap().path().iter().last().unwrap().to_str().unwrap()).parse::<u32>() {
      Ok(this_run_id) => {
        debug!("Extracted run id {}", this_run_id);
        used_runids.push(this_run_id);
      },
      Err(err)        => {
        warn!("Can not get runid from {}! {}", path.unwrap().path().display(), err);
      }
    }
  }
  let mut max_run_id = 0u32;
  match used_runids.iter().max() {
    None => (),
    Some(_r) => {
      max_run_id = *_r;
    }
  }
  println!("=> Found {} used run ids in {}. Largest run id is {}",used_runids.len(), stream_files_path.display(), max_run_id);
  if max_run_id == 0 {
    return None;
  } else {
    return Some(max_run_id + 1);
  }
}


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



