extern crate liftof_rb;

use std::{thread, time};
use indicatif::{ProgressBar, 
                ProgressStyle};
use tof_dataclasses::events::RBEvent;
use tof_dataclasses::serialization::Serialization;

use liftof_rb::control::*;
use liftof_rb::memory::BlobBuffer;
use liftof_rb::memory::RegisterError;
use liftof_rb::memory::map_physical_mem_read;
#[macro_use] extern crate log;

///! Return the bytes located at the memory
pub fn get_bytestream(addr_space : &str, 
                  addr : u32,
                  len  : usize) -> Result<Vec::<u8>, RegisterError> {

  let blobsize = RBEvent::SIZE;
  let vec_size = blobsize*len;
  // FIXME - allocate the vector elsewhere and 
  // pass it by reference
  let mut bytestream = Vec::<u8>::with_capacity(vec_size);

  let sz = std::mem::size_of::<u8>();
  let m = match map_physical_mem_read(addr_space, addr, vec_size * sz) {
    Ok(m) => m,
    Err(err) => {
      println!("Failed to mmap! {:?}", err);
      return Err(RegisterError::MMapFail);
    }
  };
  let p = m.as_ptr() as *const u8;
  (0..vec_size).for_each(|x| unsafe {
    let value = std::ptr::read_volatile(p.offset(x as isize));
    bytestream.push(value); // push is free, since we 
                            // allocated the vector in the 
                            // beginning
  });
  Ok(bytestream)
}

///! FIXME - should become a feature
pub fn setup_progress_bar(msg : String, size : u64, format_string : String) -> ProgressBar {
  let bar = ProgressBar::new(size).with_style(
    ProgressStyle::with_template(&format_string)
    .unwrap()
    .progress_chars("##-"));
  //);
  bar.set_message(msg);
  //bar.finish_and_clear();
  ////let mut style_found = false;
  //let style_ok = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}");
  //match style_ok {
  //  Ok(_) => { 
  //    style_found = true;
  //  },
  //  Err(ref err)  => { warn!("Can not go with chosen style! Not using any! Err {err}"); }
  //}  
  //if style_found { 
  //  bar.set_style(style_ok.unwrap()
  //                .progress_chars("##-"));
  //}
  bar
}




fn main() {
  // some pre-defined time units for 
  // sleeping
  let two_seconds = time::Duration::from_millis(2000);
  let one_milli   = time::Duration::from_millis(1);

  //info!("Setting daq to idle mode");
  //match idle_drs4_daq() {
  //  Ok(_)    => info!("DRS4 set to idle:"),
  //  Err(err) => panic!("Can't set DRS4 to idle!!")
  //}
  //thread::sleep(one_milli);
  //match setup_drs4() {
  //  Ok(_)    => info!("DRS4 setup routine complete!"),
  //  Err(err) => panic!("Failed to setup DRS4!!")
  //}

  
  // get the current cache sizes
  let buf_a = BlobBuffer::A;
  let buf_b = BlobBuffer::B;
  reset_dma().unwrap();
  thread::sleep(one_milli);
  let buf_a_start = get_blob_buffer_occ(&buf_a).unwrap();
  let buf_b_start = get_blob_buffer_occ(&buf_b).unwrap();
  info!("We got start values for the blob buffers at {buf_a_start} and {buf_b_start}");
  // now we are ready to receive data 
  //info!("Starting daq!");
  //match start_drs4_daq() {
  //  Ok(_)    => info!(".. successful!"),
  //  Err(err) => panic!("DRS4 start failed!")
  //}

  // let go for a few seconds to get a 
  // rate estimate
  println!("getting rate estimate..");
  thread::sleep(two_seconds);
  let rate = get_trigger_rate().unwrap();
  println!("Running at a trigger rate of {rate} Hz");
  // the trigger rate defines at what intervals 
  // we want to print out stuff
  // let's print out something apprx every 2
  // seconds

  // event loop
  //let mut evt_cnt          : u32;
  //let mut last_evt_cnt     : u32 = 0;

  //let mut n_events         : u64 = 0;

  //let mut skipped_events   : u64 = 0;
  //let mut delta_events     : u64 = 0;

  //let mut first_iter       = true;
  
  // acquire this many events
  //let max_event : u64 = 10000;

  // sizes of the buffers

  match enable_trigger() {
    Ok(_) => (),
    Err(err) => error!("Can not enable triggers, Error {err}")
  }
  let mut dma_min = std::u32::MAX;
  let mut dma_max = 0u32;

  let mut buff_a_min_occ : u32 = 4294967295;
  let mut buff_b_min_occ : u32 = 4294967295;
  let mut buff_a_max_occ : u32 = 0;
  let mut buff_b_max_occ : u32 = 0;
  let mut buff_a_occ : u32;
  let mut buff_b_occ : u32;
  let buff_a = BlobBuffer::A;
  let buff_b = BlobBuffer::B;
  let mut n_iter = 0;
  loop {
    n_iter += 1;
    //evt_cnt = get_event_count().unwrap();
    //if first_iter {
    //  last_evt_cnt = evt_cnt;
    //  first_iter = false;
    //}
    //if evt_cnt == last_evt_cnt {
    //  thread::sleep(one_milli);
    //  continue;
    //}
    buff_a_occ  = get_blob_buffer_occ(&buff_a).unwrap();
    buff_b_occ  = get_blob_buffer_occ(&buff_b).unwrap();
    let dma_ptr = get_dma_pointer().unwrap();
    //println!("{}", dma_ptr);
    if buff_a_occ > buff_a_max_occ {
        buff_a_max_occ = buff_a_occ;
    }
    if buff_a_occ < buff_a_min_occ {
        buff_a_min_occ = buff_a_occ;
    }
    if buff_b_occ > buff_b_max_occ {
        buff_b_max_occ = buff_b_occ;
    }
    if buff_b_occ < buff_b_min_occ {
        buff_b_min_occ = buff_b_occ;
    }
    if dma_ptr > dma_max {
        dma_max = dma_ptr;
    }
    if dma_ptr < dma_min {
        dma_min = dma_ptr;
    }
    // let's do some work
    if n_iter % 100000 == 0 {
        println!("New MAX A occ {buff_a_max_occ}");
        println!("New MIN A occ {buff_a_min_occ}");
        println!("New MAX B occ {buff_b_max_occ}");
        println!("New MIN B occ {buff_b_min_occ}");
        println!("New MAX dma ptr {dma_max}");
        println!("New MIN dma ptr {dma_min}");
        println!("Estmated A buff size {}", buff_a_max_occ - buff_a_min_occ);
        println!("Estmated B buff size {}", buff_b_max_occ - buff_b_min_occ);
        println!("----- N iterations {n_iter}"); 
    }
  }
} // end main
