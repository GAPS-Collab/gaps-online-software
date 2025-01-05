#[macro_use] extern crate log;

pub mod constants;
pub mod threads;

use std::fs;
use std::path::{
  PathBuf,
  Path
};

use std::collections::HashMap;
use std::os::unix::fs::symlink;
use std::sync::{
  Arc,
  Mutex,
};

use std::process::{
  Command,
  Child,
};

use std::thread;
use std::fs::create_dir_all;

use std::time::{
  Duration,
  Instant,
};

use crossbeam_channel::Sender;

use indicatif::{
  ProgressBar,
  ProgressStyle
};

use comfy_table::modifiers::{
  UTF8_ROUND_CORNERS,
  UTF8_SOLID_INNER_BORDERS,
};

use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

use liftof_lib::constants::{
  DEFAULT_CALIB_VOLTAGE,
  DEFAULT_RB_ID,
  DEFAULT_CALIB_EXTRA,
};

use tof_dataclasses::constants::PAD_CMD_32BIT;
use tof_dataclasses::serialization::{
  Serialization,
  Packable
};

use tof_dataclasses::errors::{
  StagingError,
  TofError
};

use tof_dataclasses::commands::{
  TofCommand,
  TofCommandV2,
  TofReturnCode,
  TofCommandCode,
};

use tof_dataclasses::status::TofDetectorStatus;
use tof_dataclasses::packets::TofPacket;
use tof_dataclasses::database::ReadoutBoard;

use tof_dataclasses::io::{
    TofPacketWriter,
    FileType,
    get_utc_timestamp
};

use liftof_lib::settings::LiftofSettings;
use liftof_lib::thread_control::ThreadControl;

/// communicaton between liftof-scheduler and 
/// liftof-cc
pub const LIFTOF_HOTWIRE : &str = "tcp://127.0.0.1:54321";

/// Produce a nicely formattable table with per RB information for scalar
/// values
pub fn rb_table(counters : &HashMap<u8, u64>, label_is_hz : bool) -> Table {
  let mut unit = "";
  if label_is_hz {
    unit = "Hz"
  }
  let mut table = Table::new();
  table
    .load_preset(UTF8_FULL)
    .apply_modifier(UTF8_ROUND_CORNERS)
    .apply_modifier(UTF8_SOLID_INNER_BORDERS)
    .set_content_arrangement(ContentArrangement::Dynamic)
    .set_width(80)
    //.set_header(vec!["Readoutboard Rates:"])
    .add_row(vec![
        Cell::new(&(format!("RB01 {:.1} {}", counters[&1], unit))),
        Cell::new(&(format!("RB02 {:.1} {}", counters[&2], unit))),
        Cell::new(&(format!("RB03 {:.1} {}", counters[&3], unit))),
        Cell::new(&(format!("RB04 {:.1} {}", counters[&4], unit))),
        Cell::new(&(format!("RB05 {:.1} {}", counters[&5], unit))),
        //Cell::new("Center aligned").set_alignment(CellAlignment::Center),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB06 {:.1} {}", counters[&6], unit))),
        Cell::new(&(format!("RB07 {:.1} {}", counters[&7], unit))),
        Cell::new(&(format!("RB08 {:.1} {}", counters[&8], unit))),
        Cell::new(&(format!("RB09 {:.1} {}", counters[&9], unit))),
        Cell::new(&(format!("RB10 {}", "N.A."))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB11 {:.1} Hz", counters[&11]))),
        Cell::new(&(format!("RB12 {}", "N.A."))),
        Cell::new(&(format!("RB13 {:.1} Hz", counters[&13]))),
        Cell::new(&(format!("RB14 {:.1} Hz", counters[&14]))),
        Cell::new(&(format!("RB15 {:.1} Hz", counters[&15]))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB16 {:.1} Hz", counters[&16]))),
        Cell::new(&(format!("RB17 {:.1} Hz", counters[&17]))),
        Cell::new(&(format!("RB18 {:.1} Hz", counters[&18]))),
        Cell::new(&(format!("RB19 {:.1} Hz", counters[&19]))),
        Cell::new(&(format!("RB20 {:.1} Hz", counters[&20]))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB21 {:.1} Hz", counters[&21]))),
        Cell::new(&(format!("RB22 {:.1} Hz", counters[&22]))),
        Cell::new(&(format!("RB23 {:.1} Hz", counters[&23]))),
        Cell::new(&(format!("RB24 {:.1} Hz", counters[&24]))),
        Cell::new(&(format!("RB25 {:.1} Hz", counters[&25]))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB26 {:.1} Hz", counters[&26]))),
        Cell::new(&(format!("RB27 {:.1} Hz", counters[&27]))),
        Cell::new(&(format!("RB28 {:.1} Hz", counters[&28]))),
        Cell::new(&(format!("RB29 {:.1} Hz", counters[&29]))),
        Cell::new(&(format!("RB30 {:.1} Hz", counters[&30]))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB31 {:.1} Hz", counters[&31]))),
        Cell::new(&(format!("RB32 {:.1} Hz", counters[&32]))),
        Cell::new(&(format!("RB33 {:.1} Hz", counters[&33]))),
        Cell::new(&(format!("RB34 {:.1} Hz", counters[&34]))),
        Cell::new(&(format!("RB35 {:.1} Hz", counters[&35]))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB36 {:.1}", counters[&36]))),
        Cell::new(&(format!("RB37 {}", "N.A."))),
        Cell::new(&(format!("RB38 {}", "N.A."))),
        Cell::new(&(format!("RB39 {:.1}", counters[&39]))),
        Cell::new(&(format!("RB40 {:.1}", counters[&40]))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB41 {:.1}", counters[&41]))),
        Cell::new(&(format!("RB43 {:.1}", counters[&42]))),
        Cell::new(&(format!("RB42 {}", "N.A."))),
        Cell::new(&(format!("RB44 {:.1}", counters[&44]))),
        Cell::new(&(format!("RB45 {}", "N.A."))),
    ])
    .add_row(vec![
        Cell::new(&(format!("RB46 {:.1} Hz", counters[&46]))),
        Cell::new(&(format!("{}", "N.A."))),
        Cell::new(&(format!("{}", "N.A."))),
        Cell::new(&(format!("{}", "N.A."))),
        Cell::new(&(format!("{}", "N.A."))),
    ]);
  table
}

/// Regular run start sequence
pub fn init_run_start(cc_pub_addr : &str) {
  let one_second   = Duration::from_secs(1);
  // deprecated way of sending commands, however as long as we might have RBS
  // with old software, we do want to send the "old style" as well
  let cmd_payload  = PAD_CMD_32BIT | (255u32) << 16 | (255u32) << 8 | (255u32);
  let cmd_depr     = TofCommand::DataRunStart(cmd_payload);
  let packet_depr  = cmd_depr.pack();
  let mut payload_depr = String::from("BRCT").into_bytes();
  payload_depr.append(&mut packet_depr.to_bytestream());
  
  let mut cmd      = TofCommandV2::new();
  cmd.command_code = TofCommandCode::DataRunStart;
  let packet       = cmd.pack();
  let mut payload  = String::from("BRCT").into_bytes();
  payload.append(&mut packet.to_bytestream());
  
  // open 0MQ socket here
  let ctx         = zmq::Context::new();
  let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  cmd_sender.bind(cc_pub_addr).expect("Unable to bind to (PUB) socket!");
  // after we opened the socket, give the RBs a chance to connect
  println!("=> Sending run start command to RBs ..");
  for _ in 0..10 {
    thread::sleep(one_second);
    print!("..");
  }
  // send old and new commands
  match cmd_sender.send(&payload_depr, 0) {
    Err(err) => {
      error!("Unable to send command! {err}");
    },
    Ok(_) => {
      debug!("We sent {:?}", payload);
    }
  }
  match cmd_sender.send(&payload, 0) {
    Err(err) => {
      error!("Unable to send command! {err}");
    },
    Ok(_) => {
      debug!("We sent {:?}", payload);
    }
  }
  print!("done!\n");
}

/// Regular run stop sequence
pub fn end_run(cc_pub_addr : &str) {
  let cmd_depr     = TofCommand::DataRunStop(DEFAULT_RB_ID as u32);
  let packet_depr  = cmd_depr.pack();
  let mut payload_depr = String::from("BRCT").into_bytes();
  payload_depr.append(&mut packet_depr.to_bytestream());

  let mut cmd      = TofCommandV2::new();
  cmd.command_code = TofCommandCode::DataRunStop;
  let packet       = cmd.pack();
  let mut payload  = String::from("BRCT").into_bytes();
  payload.append(&mut packet.to_bytestream());
  let ctx         = zmq::Context::new();
  let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  cmd_sender.bind(cc_pub_addr).expect("Unable to bind to (PUB) socket!");
  // after we opened the socket, give the RBs a chance to connect
  println!("=> Sending run stop command to all RBs...");
  println!("=> Waiting for RBs to stoop data acquisition..");
  for _ in 0..10 {
    print!("..");
  }
  match cmd_sender.send(&payload_depr, 0) {
    Err(err) => {
      error!("Unable to send command! {err}");
    },
    Ok(_) => {
      debug!("We sent {:?}", payload);
    }
  }
  match cmd_sender.send(&payload, 0) {
    Err(err) => {
      error!("Unable to send command! {err}");
    },
    Ok(_) => {
      debug!("We sent {:?}", payload);
    }
  }
  print!("..done!\n");
}

/// Get the files in the queue and sort them by number
pub fn get_queue(dir_path : &String) -> Vec<String> {
  let mut entries = fs::read_dir(dir_path)
    .expect("Directory might not exist!")
    .map(|entry| entry.unwrap().path())
    .collect::<Vec<PathBuf>>();
  entries.sort_by(|a, b| {
    let meta_a = fs::metadata(a).unwrap();
    let meta_b = fs::metadata(b).unwrap();
    meta_a.modified().unwrap().cmp(&meta_b.modified().unwrap())
  });
  entries.iter()
    .map(|path| path.to_str().unwrap().to_string())
    .collect()
}

pub fn move_file_with_name(old_path: &str, new_dir: &str) -> Result<(), std::io::Error> {
  let old_path  = Path::new(old_path);
  let file_name = old_path.file_name().unwrap().to_str().unwrap(); // Extract filename
  let new_path  = Path::new(new_dir).join(file_name); // Combine new directory with filename
  fs::rename(old_path, new_path) // Move the file
}

pub fn move_file_rename_liftof(old_path: &str, new_dir: &str) -> Result<(), std::io::Error> {
  let old_path  = Path::new(old_path);
  let new_path  = Path::new(new_dir).join("liftof-config.toml"); // Combine new directory with filename
  fs::rename(old_path, new_path) // Move the file
}

pub fn copy_file(old_path: &str, new_dir: &str) -> Result<u64, std::io::Error> {
  let old_path  = Path::new(old_path);
  let file_name = old_path.file_name().unwrap().to_str().unwrap(); // Extract filename
  let new_path  = Path::new(new_dir).join(file_name); // Combine new directory with filename
  fs::copy(old_path, new_path) 
}

pub fn copy_file_rename_liftof(old_path: &str, new_dir: &str) -> Result<u64, std::io::Error> {
  let old_path  = Path::new(old_path);
  let new_path  = Path::new(new_dir).join("liftof-config.toml"); // Combine new directory with filename
  fs::copy(old_path, new_path) 
}

pub fn delete_file(file_path: &str) -> Result<(), std::io::Error> {
  let path = Path::new(file_path);
  fs::remove_file(path) // Attempts to delete the file at the given path
}

/// Copy a config file from the queue to the current and 
/// next directories and restart liftof-cc. 
///
/// As soon as the run is started, prepare the next run
pub fn run_cycler(staging_dir : String, dry_run : bool) -> Result<(),StagingError> {
  let queue_dir   = format!("{}/queue", staging_dir);
  let next_dir    = format!("{}/next",  staging_dir);
  let current_dir = format!("{}/current", staging_dir);

  let queue   = get_queue(&queue_dir);
  let current = get_queue(&current_dir);
  let next    = get_queue(&next_dir); 
  
  if current.len() == 0 {
    // we are f***ed
    error!("We don't have a current configuration. This is BAD!");
    return Err(StagingError::NoCurrentConfig);
  }
  
  println!("= => Found {} files in run queue!", queue.len());
  if next.len() == 0 && queue.len() == 0 {
    println!("= => Nothing staged, will jusr repeat current run setting!");
    if !dry_run {
      manage_liftof_cc_service("restart");
    }
    thread::sleep(Duration::from_secs(20));
    return Ok(());
  }
  if next.len() == 0 && queue.len() != 0 {
    error!("Empty next directory, but we have files in the queue!");
    match copy_file_rename_liftof(&queue[0], &next_dir) {
      Ok(_) => (),
      Err(err) => {
        error!("Unable to copy {} to {}! {}", next[0], next_dir, err);
      }
    }
    match move_file_rename_liftof(&queue[0], &current_dir) {
      Ok(_) => (),
      Err(err) => {
        error!("Unable to copy {} to {}! {}", queue[0], current_dir, err);
      }
    }
  }
  if next.len() != 0  {
    match delete_file(&current[0]) {
      Ok(_)    => (),
      Err(err) => {
        error!("Unable to delete {}! {}", current[0], err);
      }
    }
    match move_file_rename_liftof(&next[0], &current_dir) {
      Ok(_) => (),
      Err(err) => {
        error!("Unable to copy {} to {}! {}", next[0], current_dir, err);
      }
    }
    if queue.len() != 0 {
      match move_file_with_name(&queue[0], &next_dir) {
        Ok(_) => (),
        Err(err) => {
          error!("Unable to move {} to {}! {}", queue[0], next_dir, err);
        }
      }
    }
    println!("=> Restarting liftof-cc!");
    if !dry_run {
      manage_liftof_cc_service("restart");
    }
    thread::sleep(Duration::from_secs(20));
  }
  Ok(())
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


///// Taka a data run. This can be either verification or physcis
//pub fn run(with_calibration : bool, verification : bool) {
//  //prepare_run(data_path  : String,
//  //                 config     : &LiftofSettings,
//  //                 run_id     : Option<u32>,
//  //                 create_dir : bool) -> Option<u32> {
//  if with_calibration {
//    //calibrate_tof()
//  }
//}

/// Trigger a restart of liftof-cc and start a new run
///
///
/// # Arguments
///
///   * mode     : The argument given to the systemd service 
///                - either "start", "stop", "restart", etc.
/// # Returns:
///   * success : true for succes
pub fn manage_liftof_cc_service(mode : &str) -> TofReturnCode {
  match Command::new("sudo")
    .args(["systemctl", mode, "liftof"])
    .spawn() {
    Err(err) => {
      error!("Unable to execute sudo systemctl {} liftof! {}", mode, err);
      TofReturnCode::GeneralFail
    }
    Ok(_) => {
      println!("=> Executed sudo systemctl {} liftof", mode);
      TofReturnCode::Success
    }
  }
}


/// Trigger a general command on the Readoutboards 
/// remotly through ssh.
///
/// Ssh keys and aliases (e.g. tof-rb02) must be 
/// set up for this to work
///
/// # Arguments:
///
///   * rb_list : The list of ReadoutBoard ids the commands
///               will get executed
///   * cmd     : The actual command without 'ssh <ip>'
///
/// # Returns:
///
///   * A list of rb ids where the process failed
pub fn ssh_command_rbs(rb_list : &Vec<u8>,
                       cmd     : Vec<String>) -> Result<Vec<u8>, TofError> {
  let mut rb_handles       = Vec::<thread::JoinHandle<_>>::new();
  info!("=> Executing ssh command {:?} on {} RBs!", cmd, rb_list.len());
  let mut children = Vec::<(u8,Child)>::new();
  for rb in rb_list {
    // also populate the rb thread nandles
    rb_handles.push(thread::spawn(||{}));
    let rb_address   = format!("tof-rb{:02}", rb);
    let mut ssh_args = vec![rb_address];
    let mut thisrb_cmd = cmd.clone();
    ssh_args.append(&mut thisrb_cmd);
    match Command::new("ssh")
      //.args([&rb_address, "sudo", "systemctl", "restart", "liftof"])
      .args(ssh_args)
      .spawn() {
      Err(err) => {
        error!("Unable to spawn ssh process on RB {}! {}", rb, err);
      }
      Ok(child) => {
        children.push((*rb,child));
      }
    }
  }
  let mut issues = Vec::<u8>::new();
  for rb_child in &mut children {
    // this is not optimal, since this will take as much 
    // time as the slowest child, but at the moment we 
    // have bigger fish to fry.
    let timeout = Duration::from_secs(10);
    let kill_t  = Instant::now();
    loop {
      if kill_t.elapsed() > timeout {
        error!("SSH process for board {} timed out!", rb_child.0);
        // Duuu hast aber einen schöönen Ball! [M. eine Stadt sucht einen Moerder]
        match rb_child.1.kill() {
          Err(err) => {
            error!("Unable to kill the SSH process for RB {}! {err}", rb_child.0);
          }
          Ok(_) => {
            error!("Killed SSH process for for RB {}", rb_child.0);
          }
        }
        issues.push(rb_child.0);
        // FIXME
        break
      }
      // non-blocking
      match rb_child.1.try_wait() {
        Ok(None) => {
          // the child is still busy
          thread::sleep(Duration::from_secs(1));
          continue;
        }
        Ok(Some(status)) => {
          if status.success() {
            info!("Execution of command on {} successful!", rb_child.0);
            break;
          } else {
            error!("Execution of command on {} failed with exit code {:?}!", rb_child.0, status.code());
            issues.push(rb_child.0);
            break;
          }
        }
        Err(err) => {
          error!("Unable to wait for the SSH process! {err}");
          break;
        }
      }
    }
  }
  if issues.len() == 0 {
    println!("=> Executing ssh command {:?} on {} RBs successful!", cmd, rb_list.len());
  }
  Ok(issues)
}

/// Restart liftof-rb on RBs
pub fn restart_liftof_rb(rb_list : &Vec<u8>) {
  let command = vec![String::from("sudo"),
                     String::from("systemctl"),
                     String::from("restart"),
                     String::from("liftof")];
  println!("=> Restarting liftof-rb on RBs!");
  match ssh_command_rbs(rb_list, command) {
    Err(err) => error!("Restarting liftof-rb on all RBs failed! {err}"),
    Ok(_)    => ()
  }
}

/// A "verification" run describes an any trigger/track trigger
/// run which should iluminate the entire tof so that we can do 
/// a working channel inventory. 
///
/// A verification run will not safe data to disk, but instead 
/// run it through a small analysis engine and count the active 
/// channels
pub fn verification_run(timeout        : u32,
                        tp_to_sink     : Sender<TofPacket>,
                        thread_control : Arc<Mutex<ThreadControl>>) {
  let mut write_state : bool = true; // when in doubt, write data to disk
  let mut config      = LiftofSettings::new();
  match thread_control.lock() {
    Ok(mut tc) => {
      write_state = tc.write_data_to_disk;
      tc.write_data_to_disk       = false;
      tc.verification_active      = true;
      tc.thread_master_trg_active = true;
      tc.calibration_active       = false;
      tc.thread_event_bldr_active = true;
      config = tc.liftof_settings.clone();
    }
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! {err}");
    },
  }
  let one_second  = Duration::from_millis(1000);
  let runtime     = Instant::now();
  // technically, it is run_typ, rb_id, event number
  // all to the max means run start for all
  // We don't need this - just need to make sure it gets broadcasted
  let cmd_payload: u32 =  PAD_CMD_32BIT | (255u32) << 16 | (255u32) << 8 | (255u32);
  let cmd          = TofCommand::DataRunStart(cmd_payload);
  let packet       = cmd.pack();
  let mut payload  = String::from("BRCT").into_bytes();
  payload.append(&mut packet.to_bytestream());
  
  // open 0MQ socket here
  let ctx = zmq::Context::new();
  let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");
  let cc_pub_addr = config.cmd_dispatcher_settings.cc_server_address.clone();
  cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
  // after we opened the socket, give the RBs a chance to connect
  println!("=> Give the RBs a chance to connect and wait a bit..");
  thread::sleep(10*one_second);
  match cmd_sender.send(&payload, 0) {
    Err(err) => {
      error!("Unable to send command, error{err}");
    },
    Ok(_) => {
      debug!("We sent {:?}", payload);
    }
  }
  
  println!("=> Verification run initialized!");
  // just wait until the run is finisehd
  loop {
    if runtime.elapsed().as_secs() > timeout as u64 {
      break;
    }
    thread::sleep(5*one_second);
  }
  
  println!("=> Ending verification run!");
  println!("=> Sending run termination command to the RBs");
  let cmd          = TofCommand::DataRunStop(DEFAULT_RB_ID as u32);
  let packet       = cmd.pack();
  let mut payload  = String::from("BRCT").into_bytes();
  payload.append(&mut packet.to_bytestream());
  
  warn!("=> No command socket available! Can not shut down RBs..!");
  // after we opened the socket, give the RBs a chance to connect
  println!("=> Give the RBs a chance to connect and wait a bit..");
  thread::sleep(10*one_second);
  match cmd_sender.send(&payload, 0) {
    Err(err) => {
      error!("Unable to send command! {err}");
    },
    Ok(_) => {
      debug!("We sent {:?}", payload);
    }
  }

  // move the socket out of here for further use
  let mut detector_status = TofDetectorStatus::new();
  match thread_control.lock() {
    Ok(mut tc) => {
      tc.write_data_to_disk = write_state;
      tc.verification_active = false;
      detector_status = tc.detector_status.clone();
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! {err}");
    },
  }
  println!("=> Acquired TofDetectorStatus!");
  println!("{}", detector_status);
  let pack = detector_status.pack();
  match tp_to_sink.send(pack) {
    Err(err) => error!("Unable to send TofDetectorStatus to data sink! {err}"),
    Ok(_)    => ()
  }
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
      cc_pub_addr   = tc.liftof_settings.cmd_dispatcher_settings.cc_server_address.clone();
      tc.write_data_to_disk = true;
    },
    Err(err) => {
      error!("Can't acquire lock for ThreadControl! Unable to set calibration mode! {err}");
    },
  }

  // deprecated commanding
  let voltage_level = DEFAULT_CALIB_VOLTAGE;
  let rb_id         = DEFAULT_RB_ID;
  let extra         = DEFAULT_CALIB_EXTRA;
  println!("=> Received calibration default command! Will init calibration run of all RBs...");
  let cmd_payload: u32
    = (voltage_level as u32) << 16 | (rb_id as u32) << 8 | (extra as u32);
  let default_calib_depr = TofCommand::DefaultCalibration(cmd_payload);
  let tp_depr = default_calib_depr.pack();
  let mut payload_depr = String::from("BRCT").into_bytes();
  payload_depr.append(&mut tp_depr.to_bytestream());

  let mut default_calib      = TofCommandV2::new();
  default_calib.command_code = TofCommandCode::RBCalibration;
  let tp                     = default_calib.pack();
  let mut payload            = String::from("BRCT").into_bytes();
  payload.append(&mut tp.to_bytestream());
  // open 0MQ socket here
  let ctx = zmq::Context::new();
  let cmd_sender  = ctx.socket(zmq::PUB).expect("Unable to create 0MQ PUB socket!");

  cmd_sender.bind(&cc_pub_addr).expect("Unable to bind to (PUB) socket!");
  println!("=> Give the RBs a chance to connect and wait a bit..");
  thread::sleep(10*one_second);
  match cmd_sender.send(&payload_depr, 0) { Err(err) => {
      error!("Unable to send command! {err}");
    },
    Ok(_) => {
      println!("=> Calibration  initialized!");
    }
  }
  match cmd_sender.send(&payload, 0) { Err(err) => {
      error!("Unable to send command! {err}");
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
      match thread_control.lock() {
        Ok(mut tc) => {
          tc.calibration_active = false;
        }
        Err(err) => {
          error!("Can't acquire lock for ThreadControl at this time! Unable to set calibration mode! {err}");
        }
      }
      if show_progress {
        bar.finish_with_message("Done");
      }
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
