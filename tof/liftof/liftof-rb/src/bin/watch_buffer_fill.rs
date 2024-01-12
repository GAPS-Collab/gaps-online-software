//! Simple binary to illustrate that the RB buffers are filling
//!
//! This is for debugging purposes mainly
extern crate liftof_rb;

use std::{thread, time};

use indicatif::{MultiProgress,
                ProgressBar, 
                ProgressStyle};

use liftof_rb::api::*;
use liftof_rb::control::*;
use liftof_rb::memory::BlobBuffer;
use liftof_rb::memory::RegisterError;


const UIO1_TRIP : u32 = 66520576;
const UIO2_TRIP : u32 = 66520576;


#[macro_use] extern crate log;

/// Non-register related constants
const TEMPLATE_BAR_A  : &str = "[{elapsed_precise}] {bar:60.blue/white} {pos:>7}/{len:7} {msg}";
const TEMPLATE_BAR_B  : &str = "[{elapsed_precise}] {bar:60.orange/white} {pos:>7}/{len:7} {msg}";
const TEMPLATE_BAR_EV : &str = "[{elapsed_precise}] {bar:60.red/white} {pos:>7}/{len:7} {msg}";



///! Get the blob buffer size from occupancy register
///
///  Read out the occupancy register and compare to 
///  a previously recoreded value. 
///  Everything is u32 (the register can't hold more)
///
///  The size of the buffer can only be defined compared
///  to a start value. If the value rools over, the 
///  size then does not make no longer sense and needs
///  to be updated.
///
///  #Arguments: 
///
fn get_buff_size(which : &BlobBuffer, buff_start : &mut u32) ->Result<u32, RegisterError> {
  let size : u32;
  let occ = get_blob_buffer_occ(&which)?;
  if *buff_start > occ {
    debug!("The occupancy counter has rolled over!");
    debug!("It reads {occ}");
    //size = occ;
    //*buff_start = occ;
    return Err(RegisterError::Unknown);
  } else {
    size  = occ - *buff_start;
  }
  Ok(size)
}

fn buff_handler(which      : &BlobBuffer,
                buff_start : u32,
                prog_bar   : Option<&ProgressBar>) -> u32 {
  
  let mut buff_start_temp = buff_start.clone();
  let mut buff_size : u32;
  let one_milli   = time::Duration::from_millis(1);

  match get_buff_size(&which, &mut buff_start_temp) {
    Ok(sz) => buff_size = sz,
    Err(_) => {
      debug!("Buffer {which:?} is full!");
      // the buffer is actually full and needs to be reset
      //switch_ram_buffer();
      match reset_ram_buffer_occ(&which) {
        Err(err) => {
          error!("Can not reset ram_buffer! {err}");
        },
        Ok(_) => ()
      }
      thread::sleep(one_milli);
      match get_buff_size(&which, &mut buff_start_temp) {
        Ok(sz) => buff_size = sz,
        Err(_) => buff_size = 0
      }
      debug!("Got NEW buffer size of {buff_size} for buff {which:?}");
    }
  }
  trace!("Got buffer size of {buff_size} for buff {which:?}");
  if buff_size > UIO1_TRIP {
    debug!("Buff {which:?} tripped");  
    // reset the buffers
    //switch_ram_buffer();
    match reset_ram_buffer_occ(&which) {
      Err(err) => {
        error!("Can not reset RAM buffers! {err}");
      },
      Ok(_) => (), 
    }
    thread::sleep(one_milli);
    // get the new size after reset
    match get_buff_size(&which, &mut buff_start_temp) {
      Ok(sz) => buff_size = sz,
      Err(_) => buff_size = 0
    }
    debug!("Got NEW buffer size of {buff_size} for buff {which:?}");
  }
  match prog_bar {
    Some(bar) => bar.set_position(buff_size as u64),
    None      => () 
  }
  buff_start_temp
}

pub fn setup_progress_bar(msg : String, size : u64, format_string : String) -> ProgressBar {
  let bar = ProgressBar::new(size).with_style(
    ProgressStyle::with_template(&format_string)
    .unwrap()
    .progress_chars("##-"));
  //);
  bar.set_message(msg);
  bar
}

fn main() {
  // some pre-defined time units for 
  // sleeping
  let two_seconds = time::Duration::from_millis(2000);
  let one_milli   = time::Duration::from_millis(1);
  info!("Setting daq to idle mode");
  match idle_drs4_daq() {
    Ok(_)    => info!("DRS4 set to idle:"),
    Err(err) => panic!("Can't set DRS4 to idle!! Err {err}")
  }
  thread::sleep(one_milli);
  match setup_drs4() {
    Ok(_)    => info!("DRS4 setup routine complete!"),
    Err(err) => panic!("Failed to setup DRS4!! Err {err}")
  }

  
  // get the current cache sizes
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  reset_dma().unwrap();
  thread::sleep(one_milli);
  let mut buf_a_start = get_blob_buffer_occ(&buf_a).unwrap();
  let mut buf_b_start = get_blob_buffer_occ(&buf_b).unwrap();
  info!("We got start values for the blob buffers at {buf_a_start} and {buf_b_start}");
  // now we are ready to receive data 
  info!("Starting daq!");
  match start_drs4_daq() {
    Ok(_)    => info!(".. successful!"),
    Err(err) => panic!("DRS4 start failed! Err {err}")
  }

  // let go for a few seconds to get a 
  // rate estimate
  println!("getting rate estimate..");
  thread::sleep(two_seconds);
  let rate = get_trigger_rate().unwrap();
  println!("Running at a trigger rate of {rate} Hz");

  // event loop
  let mut evt_cnt          : u32;
  let mut last_evt_cnt     : u32 = 0;

  let mut n_events         : u64 = 0;

  let mut skipped_events   : u64 = 0;
  let mut delta_events     : u64;

  let mut first_iter       = true;
  
  // acquire this many events
  let max_event : u64 = 10000;

  let multi_bar = MultiProgress::new();
  let bar_a  = multi_bar.add(setup_progress_bar(String::from("buff A"), UIO1_TRIP as u64, String::from(TEMPLATE_BAR_A)));  
  let bar_b  = multi_bar.insert_after(&bar_a,setup_progress_bar(String::from("buff B"), UIO2_TRIP as u64, String::from(TEMPLATE_BAR_B))); 
  let bar_ev = multi_bar.insert_after(&bar_b,setup_progress_bar(String::from("events"), max_event, String::from(TEMPLATE_BAR_EV)));         

  match enable_trigger() {
    Ok(_) => (),
    Err(err) => error!("Can not enable triggers, Error {err}")
  }

  loop {
    evt_cnt = get_event_count().unwrap();
    if first_iter {
      last_evt_cnt = evt_cnt;
      first_iter = false;
    }
    if evt_cnt == last_evt_cnt {
      thread::sleep(one_milli);
      continue;
    }
    // let's do some work
    buf_a_start = buff_handler(&buf_a, buf_a_start, Some(&bar_a)); 
    buf_b_start = buff_handler(&buf_b, buf_b_start, Some(&bar_b)); 
    delta_events = (evt_cnt - last_evt_cnt) as u64;
    if delta_events > 1 {
      skipped_events += delta_events;
    }
    
    n_events += 1;
    bar_ev.inc(delta_events);   
    // exit loop on n event basis
    if n_events > max_event {
      match idle_drs4_daq() {
        Err(err) => {
          error!("Can't set daq to idle mode! {err}");
        },
        Ok(_) => (),
      }
      println!("We skipped {skipped_events} events");
      thread::sleep(one_milli);
      bar_ev.finish();
      break;
    }
    last_evt_cnt = evt_cnt;
  }
} // end main
