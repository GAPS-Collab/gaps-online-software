extern crate liftof_rb;

use liftof_rb::control::*;
use liftof_rb::memory::*;

use std::time::Duration;
use std::thread;

fn main () {

  let buff_a = BlobBuffer::A; 
  let buff_b = BlobBuffer::B; 
  
  let one_sec = Duration::from_secs(1);
  println!("=> Idling DAQ...");
  let success = idle_drs4_daq();
  println!("=> DAQ Idle successfull {success:?}");
  println!("=> Resetting DAQ...");
  let mut success = reset_daq();
  println!("=> DAQ reset successfull {success:?}");

  // watch the buffers for 10 sec
  for _ in 0..10 {
    let buff_a_occ = get_blob_buffer_occ(&buff_a);
    let buff_b_occ = get_blob_buffer_occ(&buff_b);
    println!("=> Seeing occupancies of A: {buff_a_occ:?} and B: {buff_b_occ:?}");
    thread::sleep(one_sec);
  }
  println!("=> Setting DAQ MT mode...");
  success = set_master_trigger_mode();
  println!("=> DAQ MT mode set successfull {success:?}");
  println!("=> Set readout for all channels");
  success = set_readout_all_channels_and_ch9();
  println!("=> success {success:?}");
  thread::sleep(one_sec);
  println!("=> Starting DAQ...");
  success = start_drs4_daq();
  println!("=> DAQ Start successfull {success:?}");

  for _ in 0..10 {
    let buff_a_occ = get_blob_buffer_occ(&buff_a);
    let buff_b_occ = get_blob_buffer_occ(&buff_b);
    println!("=> Seeing occupancies of A: {buff_a_occ:?} and B: {buff_b_occ:?}");
    thread::sleep(one_sec);
  }
  println!("=> Clearing buffer occupancy register for Buff A");
  for _ in 0..10 {
    success = reset_ram_buffer_occ(&buff_a);
    println!("=> Resetting blob buffer A successful {success:?}");
    success = reset_ram_buffer_occ(&buff_b);
    println!("=> Resetting blob buffer B successful {success:?}");
    let buff_a_occ = get_blob_buffer_occ(&buff_a);
    let buff_b_occ = get_blob_buffer_occ(&buff_b);
    println!("=> Seeing occupancies of A: {buff_a_occ:?} and B: {buff_b_occ:?}");
    thread::sleep(one_sec);
  } 
  
  println!("=> Idling DAQ...");
  let mut success = idle_drs4_daq();
  println!("=> DAQ Idle successfull {success:?}");

  println!("=> Clearing buffer occupancy register for Buff A");
  for _ in 0..10 {
    success = reset_ram_buffer_occ(&buff_a);
    println!("=> Resetting blob buffer A successful {success:?}");
    success = reset_ram_buffer_occ(&buff_b);
    println!("=> Resetting blob buffer B successful {success:?}");
    let buff_a_occ = get_blob_buffer_occ(&buff_a);
    let buff_b_occ = get_blob_buffer_occ(&buff_b);
    println!("=> Seeing occupancies of A: {buff_a_occ:?} and B: {buff_b_occ:?}");
    thread::sleep(one_sec);
  } 


}
