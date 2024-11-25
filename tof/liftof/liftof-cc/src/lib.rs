#[macro_use] extern crate log;

pub mod constants;
pub mod threads;

use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::symlink;
use std::sync::{
  Arc,
  Mutex,
};

use std::thread;
use std::fs::create_dir_all;

use std::time::{
  Duration,
  Instant,
};

use indicatif::{
  ProgressBar,
  ProgressStyle
};

use liftof_lib::constants::{
  DEFAULT_CALIB_VOLTAGE,
  DEFAULT_RB_ID,
  DEFAULT_CALIB_EXTRA
};

use tof_dataclasses::serialization::{
  Serialization,
  Packable
};

use tof_dataclasses::commands::{
    TofCommand,
    //TofCommandCode,
    //TofResponse,
};

use tof_dataclasses::io::{
    TofPacketWriter,
    FileType,
    get_utc_timestamp
};
//use tof_dataclasses::calibrations::RBCalibrations;

use liftof_lib::settings::LiftofSettings;

use liftof_lib::thread_control::ThreadControl;

use tof_dataclasses::database::ReadoutBoard;

pub fn ssh_commands_rbs() {
}


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


/// Taka a data run. This can be either verification or physcis
pub fn run(with_calibration : bool, verification : bool) {
  //prepare_run(data_path  : String,
  //                 config     : &LiftofSettings,
  //                 run_id     : Option<u32>,
  //                 create_dir : bool) -> Option<u32> {
  if with_calibration {
    //calibrate_tof()
  }
}

/// A "verification" run describes an any trigger/track trigger
/// run which should iluminate the entire tof so that we can do 
/// a working channel inventory. 
///
/// A verification run will not safe data to disk, but instead 
/// run it through a small analysis engine and count the active 
/// channels
pub fn verification_run(timeout : u32) {
}


/// Run a full tof calibration - RBCalibration
/// 
/// The purpose of the RB calibration to is to create a 
/// relationship between the adc/timing bins and voltages
/// and nanoseconds
///
/// This function is blocking, until a certain (configurable)
/// timeout is expired. The timeout can be set in the configuration
/// file
///
/// # Argumeents:
///
///   * thread_control : general shared memory to hold configuration
///                      settings, program st ate
///   * rb_list        : List of active readoutboards
///   * show_progress  : if true, it will show a progressbar with 
///                      indicatif
///
pub fn calibrate_tof(thread_control : Arc<Mutex<ThreadControl>>,
                     rb_list        : &Vec<ReadoutBoard>,
                     show_progress  : bool) {

  let one_second               = Duration::from_millis(1000);
  let mut cc_pub_addr          = String::from("");
  let calibration_timeout_fail = Duration::from_secs(300); // in seconds
 
  let mut cali_dir_created = false;
  let mut cali_output_dir  = String::from("");
  let mut cali_base_dir        = String::from("");

  match thread_control.lock() {
    Ok(mut tc) => {
      for rb in rb_list {
        tc.finished_calibrations.insert(rb.rb_id,false); 
      }
      cali_base_dir = tc.liftof_settings.calibration_dir.clone();
      cc_pub_addr = tc.liftof_settings.cmd_dispatcher_settings.cc_server_address.clone();
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }

  let voltage_level = DEFAULT_CALIB_VOLTAGE;
  let rb_id         = DEFAULT_RB_ID;
  let extra         = DEFAULT_CALIB_EXTRA;
  println!("=> Received calibration default command! Will init calibration run of all RBs...");
  let cmd_payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let default_calib = TofCommand::DefaultCalibration(cmd_payload);
  let tp = default_calib.pack();
  let mut payload  = String::from("BRCT").into_bytes();
  payload.append(&mut tp.to_bytestream());
  // open 0MQ socket here
  let ctx = zmq::Context::new();
  let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");

  cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
  println!("=> Give the RBs a chance to connect and wait a bit..");
  thread::sleep(10*one_second);
  match cmd_sender.send(&payload, 0) { Err(err) => {
      error!("Unable to send command, error{err}");
    },
    Ok(_) => {
      println!("=> Calibration  initialized!");
    }
  }
  match thread_control.lock() {
    Ok(mut tc) => {
      // deactivate the master trigger thread
      tc.thread_master_trg_active =false;
      tc.calibration_active = true;
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }

  let bar_template : &str = "[{elapsed_precise}] {prefix} {msg} {spinner} {bar:60.blue/grey} {pos:>7}/{len:7}";
  let bar_style  = ProgressStyle::with_template(bar_template).expect("Unable to set progressbar style!");
  let mut bar    = ProgressBar::hidden();
  
  println!("=> .. now we need to wait until the calibration is finished!");
  if show_progress {
    bar = ProgressBar::new(rb_list.len() as u64); 
    bar.set_position(0);
    let bar_label  = String::from("Acquiring RB calibration data");
    bar.set_message (bar_label);
    bar.set_prefix  ("\u{2699}\u{1F4D0}");
    bar.set_style   (bar_style);
  }

  // now block until the calibrations are done or we time outu
  // FIXME - set timeout parameter in settings
  let timeout = Instant::now();
  let mut cali_received = 0;
  'main: loop {
    thread::sleep(10*one_second);
    if timeout.elapsed() > calibration_timeout_fail {
      error!("Calibration timeout! Calibrations might not be complete!");
      break;
    }
    //let mut rbcali = RBCalibrations::new();
    match thread_control.lock() {
      Ok(mut tc) => {
        for rbid in rb_list {
          // the global data sink sets these flags
          let mut finished_keys = Vec::<u8>::new();
          if tc.stop_flag {
            println!("Stop signal received, exiting calibration routine!");
            break 'main;
          }
          if tc.finished_calibrations[&rbid.rb_id] {
            cali_received += 1;
            let rbcali = tc.calibrations.get(&rbid.rb_id).expect("We got the signal tat this calibration is ready but it is not!");
            let pack   = rbcali.pack();
            // See RBCalibration reference
            let file_type  = FileType::CalibrationFile(rbid.rb_id);
            //println!("==> Writing stream to file with prefix {}", streamfile_name);
            //let mut cali_writer = TofPacketWriter::new(write_stream_path.clone(), file_type);
            if !cali_dir_created {
              let today           = get_utc_timestamp();
              cali_output_dir     = format!("{}/{}", cali_base_dir.clone(), today);
              match create_dir_all(cali_output_dir.clone()) {
                Ok(_)    => info!("Created {} for calibration data!", cali_output_dir),
                Err(err) => error!("Unable to create {} for calibration data! {}", cali_output_dir, err)
              }
              cali_dir_created = true;
            }
            let mut cali_writer = TofPacketWriter::new(cali_output_dir.clone(), file_type);
            cali_writer.add_tof_packet(&pack);
            drop(cali_writer);

            bar.set_position(cali_received);
            finished_keys.push(rbid.rb_id);
          }
          for rbid in &finished_keys {
            *tc.finished_calibrations.get_mut(&rbid).unwrap() = false; 
          }
        }
        // FIXME - this or a timer
        if cali_received as usize == rb_list.len() {
          // cali_received = 0;
          // if we want to redo a calibration, 
          // somebody else has to set this 
          // flag again.
          tc.calibration_active = false;
          // reset the counters
          for rbid in rb_list {
            *tc.finished_calibrations.get_mut(&rbid.rb_id).unwrap() = false; 
          }
          if show_progress {
            bar.finish_with_message("Done");
          }
          break;
        }
      }
      Err(err) => {
        error!("Can't acquire lock for ThreadControl at this time! Unable to set calibration mode! {err}");
      }
    }
  } // end loop
  // The last step is to create te symlink
  let cali_link_dir = cali_base_dir.clone() + "latest";
  match fs::remove_file(cali_link_dir.clone()) {
    Ok(_) => {
      println!("=> Symlink {} removed!", cali_link_dir);
    },
    Err(err) => {
      error!("Unable to remove symlink to latest calibrations! {err}");
    }
  }
  println!("=> Will create symlink {}", cali_link_dir);
  match symlink(cali_output_dir, cali_link_dir) {
    Err(err) => error!("Unable to create symlink for calibration data! {err}"),
    Ok(_)    => ()
  }
}
