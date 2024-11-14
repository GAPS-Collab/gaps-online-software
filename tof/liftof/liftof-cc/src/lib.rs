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
/// * data_path  : The global path on the inflight 
///                tof computer where to store data.
/// * config     : The current configuration, to be 
///                copied into the new folder
/// * run_id     : Optionally define a pre-given 
///                run-id
/// * create_dir : Create the directory for runfiles
pub fn prepare_run(data_path  : String,
                   config     : &LiftofSettings,
                   run_id     : Option<u32>,
                   create_dir : bool) -> Option<u32> {
  let mut stream_files_path = PathBuf::from(data_path);
  // Unwrap is ok here, since this should only happen at run start.
  // Also, if the path is wrong, we are going to fail catastrophically
  // anyway
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
  let new_run_id : Option<u32>;
  if max_run_id == 0 {
    // we can use the run)given as the argument
    // return run_id;
    new_run_id = run_id;
  } else if run_id.is_some() {
    if used_runids.contains(&run_id.unwrap()) {
      // the assigned run id has been used already
      error!("Duplicate run id ({})!", run_id.unwrap());
      //return None;
      new_run_id = None;
    } else {
      //return run_id;
      new_run_id = run_id;
    }
  } else {
      new_run_id = Some(max_run_id + 1);
  }
  // We were not able to assign a new run id
  if new_run_id.is_none() {
    return new_run_id;
  }
  stream_files_path.push(new_run_id.unwrap().to_string().as_str());
  if create_dir {
    // Create directory if it does not exist
    // Check if the directory exists
    if let Ok(metadata) = fs::metadata(&stream_files_path) {
      if metadata.is_dir() {
        println!("=> Directory {} for run number {} already consists and may contain files!", stream_files_path.display(), new_run_id.unwrap());
        // FILXME - in flight, we can not have interactivity.
        // But the whole system with the run ids might change 
      } 
    } else {
      match fs::create_dir(&stream_files_path) {
        Ok(())   => println!("=> Created {} to save stream data", stream_files_path.display()),
        Err(err) => panic!("Failed to create directory: {}! {}", stream_files_path.display(), err),
      }
    }
  }
  let settings_fname = format!("{}/run{}.toml",
    stream_files_path.display(),
    new_run_id.unwrap()); 
  println!("=> Writing data to {}/{}!", stream_files_path.display(), new_run_id.unwrap());
  println!("=> Writing settings to {}!", settings_fname);
  config.to_toml(settings_fname);
  return new_run_id;
}


