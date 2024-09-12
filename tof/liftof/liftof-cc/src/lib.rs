#[macro_use] extern crate log;

pub mod constants;
pub mod threads;

use std::fs;
use std::path::PathBuf;

use liftof_lib::settings::LiftofSettings;

/// Prepare a new folder with the run id
///
/// This will assign a run id based on the 
/// run ids in that folder. Additionally, 
/// we will copy the current settings into
/// that folder
///
/// TODO - connect it to the run database
///
/// # Arguments:
///
/// * data_path : The global path on the inflight 
///               tof computer where to store data.
/// * config    : The current configuration, to be 
///               copied into the new folder
///
pub fn prepare_run(data_path : String,
                   config    : &LiftofSettings) -> Option<u32> {



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
    let new_run_id = max_run_id + 1;
    let settings_fname = format!("{}/run{}.toml",stream_files_path.display(), new_run_id); 
    println!("=> Writing data to {}!", stream_files_path.display());
    println!("=> Writing settings to {}!", settings_fname);
    config.to_toml(settings_fname);
    return Some(new_run_id);
  }
}


